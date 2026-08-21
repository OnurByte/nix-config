{
  coreutils,
  git,
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
    gnugrep
    hyprland
    jq
    procps
  ];

  text = ''
    set -uo pipefail

    state_dir="''${VESPER_AGENT_STATE_DIR:-$HOME/.local/state/vesper/agents}"
    cache_ttl="''${VESPER_AGENT_CACHE_TTL:-10}"
    [[ "$cache_ttl" =~ ^[0-9]+$ ]] || cache_ttl=10
    mkdir -p "$state_dir"

    cleanup_state() {
      local file pid
      shopt -s nullglob
      for file in "$state_dir"/*.json; do
        [[ "$file" == "$state_dir/status.json" ]] && continue
        pid="$(jq -r '.pid // 0' "$file" 2>/dev/null || echo 0)"
        if ! [[ "$pid" =~ ^[0-9]+$ ]] || ! kill -0 "$pid" 2>/dev/null; then
          rm -f "$file"
        fi
      done
      shopt -u nullglob
    }

    status_json() {
      local items='[]'
      local now now_epoch snapshot_file snapshot_mtime
      now="$(date --iso-8601=seconds)"
      cleanup_state

      # QML may ask every five seconds. Reuse a bounded diagnostic snapshot so
      # the visual refresh rate does not become a full process/Git scan rate.
      # ponytail: one TTL for the whole snapshot; split liveness/Git caches if
      # a ten-second stale window becomes operationally significant.
      snapshot_file="$state_dir/status.json"
      now_epoch="$(date +%s)"
      snapshot_mtime="$(stat -c %Y "$snapshot_file" 2>/dev/null || echo 0)"
      if [[ "$snapshot_mtime" =~ ^[0-9]+$ ]] \
        && (( now_epoch >= snapshot_mtime )) \
        && (( now_epoch - snapshot_mtime < cache_ttl )) \
        && jq -e '.schemaVersion == 2 and (.agents | type == "array")' "$snapshot_file" >/dev/null 2>&1; then
        cat "$snapshot_file"
        return
      fi

      while IFS='|' read -r agent pattern; do
        [[ -n "$agent" ]] || continue

        while IFS= read -r pid; do
          [[ -n "$pid" ]] || continue

          local cwd repo_root project branch dirty process_name item slug state_file first_seen elapsed_seconds
          cwd="$(readlink -f "/proc/$pid/cwd" 2>/dev/null || true)"
          # Keep process identity, never its argv. Prompts, tokens and paths
          # passed to an agent can otherwise become durable state.
          process_name="$(ps -p "$pid" -o comm= 2>/dev/null | sed 's/[[:space:]]*$//' || true)"
          [[ -n "$process_name" ]] || process_name="unknown"
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
            --arg command "$process_name" \
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

      local snapshot
      snapshot="$(jq -cn \
        --argjson schemaVersion 2 \
        --argjson count "$count" \
        --arg state "$state" \
        --arg tooltip "$tooltip" \
        --arg stateDir "$state_dir" \
        --argjson agents "$items" \
        '{schemaVersion:$schemaVersion,count:$count,class:$state,tooltip:$tooltip,stateDir:$stateDir,agents:$agents}')"
      printf '%s\n' "$snapshot" > "$snapshot_file"
      printf '%s\n' "$snapshot"
    }

    focus_pid() {
      local pid="''${1:-}"
      if ! [[ "$pid" =~ ^[0-9]+$ ]]; then
        echo "usage: vesper-agent-cockpit focus <pid>" >&2
        exit 2
      fi
      hyprctl dispatch focuswindow "pid:$pid" >/dev/null
    }

    case "''${1:-status}" in
      status|--json)
        status_json
        ;;
      focus)
        focus_pid "''${2:-}"
        ;;
      *)
        echo "usage: vesper-agent-cockpit [status|focus <pid>]" >&2
        exit 2
        ;;
    esac
  '';
}
