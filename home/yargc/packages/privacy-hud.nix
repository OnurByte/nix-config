{
  coreutils,
  gnugrep,
  jq,
  pipewire,
  procps,
  psmisc,
  systemd,
  util-linux,
  wireplumber,
  writeShellApplication,
}:
writeShellApplication {
  name = "vesper-privacy-hud";
  runtimeInputs = [
    coreutils
    gnugrep
    jq
    pipewire
    procps
    psmisc
    systemd
    util-linux
    wireplumber
  ];

  text = ''
    set -uo pipefail
    umask 077

    cache_is_fresh() {
      local cache_file="$1"
      local now="$2"
      local ttl="$3"

      [[ -r "$cache_file" ]] && jq -e \
        --argjson now "$now" \
        --argjson ttl "$ttl" \
        '(.generatedAt | type == "number") and (($now - .generatedAt) >= 0) and (($now - .generatedAt) <= $ttl)' \
        "$cache_file" >/dev/null 2>&1
    }

    refresh_cache() {
      local cache_file="$1"
      local temporary="$cache_file.$$"
      if status_json >"$temporary" && jq -e 'type == "object"' "$temporary" >/dev/null 2>&1; then
        chmod 600 -- "$temporary"
        mv -f -- "$temporary" "$cache_file"
        cat -- "$cache_file"
        return 0
      fi
      rm -f -- "$temporary"
      return 1
    }

    status_json() {
      local tor="off"
      local mic="unknown"
      local camera="none"
      local clipboard="off"
      local node="off"

      # Tor Browser owns its bundled tor process. Only the system unit proves
      # that Vesper's system Tor client is active.
      if systemctl is-active --quiet tor.service 2>/dev/null; then
        tor="on"
      fi

      # A default-source volume/mute value is only device readiness. A
      # running PipeWire input stream is the ownership-specific capture
      # signal we can prove without guessing from application names.
      local pipewire_dump
      if pipewire_dump="$(pw-dump -N 2>/dev/null)"; then
        if jq -e 'any(.[]?; .type == "PipeWire:Interface:Node" and .info.state == "running" and .info.props["media.class"] == "Stream/Input/Audio")' <<<"$pipewire_dump" >/dev/null 2>&1; then
          mic="active"
        else
          mic="inactive"
        fi
      fi

      if compgen -G '/dev/video*' >/dev/null; then
        camera="idle"
        if fuser /dev/video* >/dev/null 2>&1; then
          camera="active"
        fi
      fi

      local text_clipboard="off"
      local image_clipboard="off"
      if systemctl --user is-active --quiet vesper-cliphist-text.service 2>/dev/null; then
        text_clipboard="on"
      fi
      if systemctl --user is-active --quiet vesper-cliphist-image.service 2>/dev/null; then
        image_clipboard="on"
      fi
      if [[ "$text_clipboard" == "on" && "$image_clipboard" == "on" ]]; then
        clipboard="ready"
      elif [[ "$text_clipboard" == "on" || "$image_clipboard" == "on" ]]; then
        clipboard="degraded"
      fi

      if pgrep -x cuprated >/dev/null 2>&1; then
        node="cuprated"
      elif pgrep -x monerod >/dev/null 2>&1; then
        node="monerod"
      fi

      local state label
      if [[ "$camera" == "active" ]]; then
        state="alert"
        label="CAM"
      elif [[ "$mic" == "active" ]]; then
        state="attention"
        label="MIC"
      elif [[ "$clipboard" == "degraded" ]]; then
        state="attention"
        label="CLIP"
      elif [[ "$tor" == "on" ]]; then
        state="private"
        label="TOR"
      else
        state="idle"
        label="LOC"
      fi

      local tooltip
      tooltip="Tor: $tor\nMic: $mic\nCamera: $camera\nClipboard history: $clipboard\nMonero node: $node"

      jq -cn \
        --arg tor "$tor" \
        --arg mic "$mic" \
        --arg camera "$camera" \
        --arg clipboard "$clipboard" \
        --arg node "$node" \
        --arg state "$state" \
        --arg label "$label" \
        --arg tooltip "$tooltip" \
        --arg textClipboard "$text_clipboard" \
        --arg imageClipboard "$image_clipboard" \
        --argjson generatedAt "$(date +%s 2>/dev/null || echo 0)" \
        '{tor:$tor,mic:$mic,camera:$camera,clipboard:$clipboard,clipboardText:$textClipboard,clipboardImage:$imageClipboard,node:$node,class:$state,label:$label,tooltip:$tooltip,generatedAt:$generatedAt}'
    }

    cached_status() {
      local cache_file="$1"
      local now="$2"
      local ttl="$3"
      if cache_is_fresh "$cache_file" "$now" "$ttl"; then
        cat -- "$cache_file"
        return 0
      fi
      return 1
    }

    status_cached() {
      local cache_root="''${XDG_RUNTIME_DIR:-''${XDG_CACHE_HOME:-''${HOME:-/tmp}}}/vesper"
      local cache_file="$cache_root/privacy-hud.json"
      local lock_file="$cache_root/privacy-hud.lock"
      local ttl=10
      local now="$(date +%s 2>/dev/null || echo 0)"

      if ! mkdir -p -- "$cache_root" || ! chmod 700 -- "$cache_root"; then
        status_json
        return
      fi
      if cached_status "$cache_file" "$now" "$ttl"; then
        return 0
      fi

      local lock_fd
      if ! exec {lock_fd}>"$lock_file"; then
        status_json
        return
      fi
      if flock -n "$lock_fd"; then
        now="$(date +%s 2>/dev/null || echo 0)"
        if cached_status "$cache_file" "$now" "$ttl"; then
          flock -u "$lock_fd"
          exec {lock_fd}>&-
          return 0
        fi
        refresh_cache "$cache_file"
        local result=$?
        flock -u "$lock_fd"
        exec {lock_fd}>&-
        if [[ "$result" -eq 0 ]]; then
          return 0
        fi
        status_json
        return
      fi
      exec {lock_fd}>&-

      # ponytail: one user-session lock, with a short wait; split locks only
      # if concurrent HUD callers become a measured bottleneck.
      for attempt in 1 2 3 4; do
        sleep 0.05
        now="$(date +%s 2>/dev/null || echo 0)"
        if cached_status "$cache_file" "$now" "$ttl"; then
          return 0
        fi
      done
      status_json
    }

    case "''${1:-status}" in
      status|--json)
        status_cached
        ;;
      *)
        echo "usage: vesper-privacy-hud [status]" >&2
        exit 2
        ;;
    esac
  '';
}
