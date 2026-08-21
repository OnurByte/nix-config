{
  coreutils,
  gnugrep,
  jq,
  pipewire,
  procps,
  psmisc,
  systemd,
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
    wireplumber
  ];

  text = ''
    set -uo pipefail

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
        '{tor:$tor,mic:$mic,camera:$camera,clipboard:$clipboard,clipboardText:$textClipboard,clipboardImage:$imageClipboard,node:$node,class:$state,label:$label,tooltip:$tooltip}'
    }

    case "''${1:-status}" in
      status|--json)
        status_json
        ;;
      *)
        echo "usage: vesper-privacy-hud [status]" >&2
        exit 2
        ;;
    esac
  '';
}
