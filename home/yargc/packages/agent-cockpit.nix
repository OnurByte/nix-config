{ coreutils, git, ghostty, gnugrep, jq, procps, writeShellApplication }:
writeShellApplication {
  name = "vesper-agent-cockpit";
  runtimeInputs = [
    coreutils
    git
    ghostty
    gnugrep
    jq
    procps
  ];

  text = ''
    set -uo pipefail

    status_json() {
      local items='[]'

      while IFS='|' read -r agent pattern; do
        [[ -n "$agent" ]] || continue

        while IFS= read -r pid; do
          [[ -n "$pid" ]] || continue

          local cwd repo_root project branch dirty command item
          cwd="$(readlink -f "/proc/$pid/cwd" 2>/dev/null || true)"
          command="$(ps -p "$pid" -o args= 2>/dev/null || true)"
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

          item="$(jq -cn \
            --arg agent "$agent" \
            --argjson pid "$pid" \
            --arg project "$project" \
            --arg cwd "$cwd" \
            --arg branch "$branch" \
            --arg command "$command" \
            --argjson dirty "$dirty" \
            '{agent:$agent,pid:$pid,project:$project,cwd:$cwd,branch:$branch,command:$command,dirty:$dirty}')"
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
        --argjson agents "$items" \
        '{count:$count,class:$state,tooltip:$tooltip,agents:$agents}'
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
            "  cwd      " + (if .cwd == "" then "-" else .cwd end)
          ) | join("\n\n"))
        end) +
        "\n\nrefreshes every 2s · Ctrl+C closes"
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
      *)
        echo "usage: vesper-agent-cockpit [popup|tui|status|render]" >&2
        exit 2
        ;;
    esac
  '';
}
