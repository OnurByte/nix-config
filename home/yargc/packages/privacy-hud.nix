{
  coreutils,
  gnugrep,
  jq,
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

      if systemctl is-active --quiet tor.service 2>/dev/null || pgrep -x tor >/dev/null 2>&1; then
        tor="on"
      fi

      local mic_line
      mic_line="$(wpctl get-volume @DEFAULT_AUDIO_SOURCE@ 2>/dev/null || true)"
      if [[ -n "$mic_line" ]]; then
        if grep -q '\[MUTED\]' <<<"$mic_line"; then
          mic="muted"
        else
          mic="unmuted"
        fi
      fi

      if compgen -G '/dev/video*' >/dev/null; then
        camera="idle"
        if fuser /dev/video* >/dev/null 2>&1; then
          camera="active"
        fi
      fi

      if pgrep -u "$UID" -f 'caelestia|quickshell' >/dev/null 2>&1; then
        clipboard="shell"
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
      elif [[ "$mic" == "unmuted" ]]; then
        state="attention"
        label="MIC"
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
        '{tor:$tor,mic:$mic,camera:$camera,clipboard:$clipboard,node:$node,class:$state,label:$label,tooltip:$tooltip}'
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
