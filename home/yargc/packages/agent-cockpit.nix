{
  coreutils,
  git,
  ghostty,
  gnugrep,
  hyprland,
  jq,
  procps,
  writeShellApplication,
}:
writeShellApplication {
  name = "vesper-agent-cockpit";
  runtimeInputs = [
    coreutils
    git
    ghostty
    gnugrep
    hyprland
    jq
    procps
  ];

  text = ''
    set -uo pipefail

    state_dir="''${VESPER_AGENT_STATE_DIR:-$HOME/.local/state/vesper/agents}"
    mkdir -p "$state_dir"

    cleanup_state() {
      local file pid
      shopt -s nullglob
      for file in "$state_dir"/*.json; do
        pid="$(jq -r '.pid // 0' "$file" 2>/dev/null || echo 0)"
        if ! [[ "$pid" =~ ^[0-9]+$ ]] || ! kill -0 "$pid" 2>/dev/null; then
          rm -f "$file"
        fi
      done
      shopt -u nullglob
    }

    status_json() {
      local items='[]'
      local now
      now="$(date --iso-8601=seconds)"
      cleanup_state

      while IFS='|' read -r agent pattern; do
        [[ -n "$agent" ]] || continue

        while IFS= read -r pid; do
          [[ -n "$pid" ]] || continue

          local cwd repo_root project branch dirty command item slug state_file first_seen elapsed_seconds
          cwd="$(readlink -f "/proc/$pid/cwd" 2>/dev/null || true)"
          command="$(ps -p "$pid" -o args= 2>/dev/null || true)"
          elapsed_seconds="$(ps -p "$pid" -o etimes= 2>/dev/null | tr -d ' ' || true)"
          [[ "$elapsed_seconds" =~ ^[0-9]+$ ]] || elapsed_seconds=0
          repo_root=""
          project="unknown"
          branch=""
          dirty=false

          if [[ -n "$cwd" && -d "$cwd" ]]; then
            repo_root="$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null || true)"
            if [[ -n "$repo_root" ]]; then
              project="$(basename "$repo_root")"
              branch="$(git -C "$repo_root" branch --show-current 2>/dev/null || true)"
              if [[ -z "$branch" ]]; then
                branch="$(git -C "$repo_root" rev-parse --short HEAD 2>/dev/null || true)"
              fi
              if [[ -n "$(git -C "$repo_root" status --porcelain 2>/dev/null || true)" ]]; then
                dirty=true
              fi
            else
              project="$(basename "$cwd")"
            fi
          fi

          slug="$(printf '%s' "$agent" | tr '[:upper:]' '[:lower:]')-$pid"
          state_file="$state_dir/$slug.json"
          first_seen=""
          if [[ -r "$state_file" ]]; then
            first_seen="$(jq -r '.firstSeen // empty' "$state_file" 2>/dev/null || true)"
          fi
          [[ -n "$first_seen" ]] || first_seen="$now"

          item="$(jq -cn \
            --arg agent "$agent" \
            --argjson pid "$pid" \
            --arg project "$project" \
            --arg cwd "$cwd" \
            --arg branch "$branch" \
            --arg command "$command" \
            --argjson dirty "$dirty" \
            --arg firstSeen "$first_seen" \
            --arg lastSeen "$now" \
            --arg stateFile "$state_file" \
            --argjson elapsedSeconds "$elapsed_seconds" \
            '{agent:$agent,pid:$pid,project:$project,cwd:$cwd,branch:$branch,command:$command,dirty:$dirty,firstSeen:$firstSeen,lastSeen:$lastSeen,elapsedSeconds:$elapsedSeconds,stateFile:$stateFile}')"

          printf '%s\n' "$item" > "$state_file"
          items="$(jq -cn --argjson items "$items" --argjson item "$item" '$items + [$item]')"
        done < <(pgrep -u "$UID" -f "$pattern" 2>/dev/null || true)
      done <<'AGENTS'
Codex|(^|/)codex([[:space:]]|$)
Claude|(^|/)claude([[:space:]]|$)
OpenCode|(^|/)opencode([[:space:]]|$)
Hermes|(^|/)hermes([[:space:]]|$)
Grok|(^|/)grok([[:space:]]|$)
bb|(^|/)bb-app([[:space:]]|$)
AGENTS

      local count state tooltip
      count="$(jq 'length' <<<"$items")"
      if (( count > 0 )); then
        state="active"
      else
        state="idle"
      fi

      tooltip="$(jq -r '
        if length == 0 then
          "No active coding agents"
        else
          map(
            "\(.agent) · \(.project)" +
            (if .branch == "" then "" else " · \(.branch)" end) +
            (if .dirty then " · dirty" else "" end)
          ) | join("\n")
        end
      ' <<<"$items")"

      jq -cn \
        --argjson count "$count" \
        --arg state "$state" \
        --arg tooltip "$tooltip" \
        --arg stateDir "$state_dir" \
        --argjson agents "$items" \
        '{count:$count,class:$state,tooltip:$tooltip,stateDir:$stateDir,agents:$agents}'
    }

    render() {
      local payload
      payload="$(status_json)"
      clear
      jq -r '
        "VESPER AGENT COCKPIT\n" +
        "active sessions  \(.count)\n" +
        "────────────────────────────────────────────────────────\n" +
        (if .agents | length == 0 then
          "no active coding agents"
        else
          (.agents | map(
            "\(.agent)\n" +
            "  project  \(.project)\n" +
            "  branch   " + (if .branch == "" then "-" else .branch end) +
            (if .dirty then "  (dirty)" else "  (clean)" end) + "\n" +
            "  pid      \(.pid)\n" +
            "  age      \(.elapsedSeconds)s\n" +
            "  cwd      " + (if .cwd == "" then "-" else .cwd end)
          ) | join("\n\n"))
        end) +
        "\n\nstate  \(.stateDir)" +
        "\nrefreshes every 2s · Ctrl+C closes"
      ' <<<"$payload"
    }

    tui() {
      trap 'exit 0' INT TERM
      while true; do
        render
        sleep 2
      done
    }

    popup() {
      exec ghostty --class=vesper-agent-cockpit -e vesper-agent-cockpit tui
    }

    focus_pid() {
      local pid="''${1:-}"
      if ! [[ "$pid" =~ ^[0-9]+$ ]]; then
        echo "usage: vesper-agent-cockpit focus <pid>" >&2
        exit 2
      fi
      hyprctl dispatch focuswindow "pid:$pid" >/dev/null
    }

    case "''${1:-popup}" in
      status|--json)
        status_json
        ;;
      render)
        render
        ;;
      tui)
        tui
        ;;
      popup)
        popup
        ;;
      focus)
        focus_pid "''${2:-}"
        ;;
      *)
        echo "usage: vesper-agent-cockpit [popup|tui|status|render|focus <pid>]" >&2
        exit 2
        ;;
    esac
  '';
}
