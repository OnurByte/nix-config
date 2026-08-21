{ pkgs, ... }:
let
  vesperDoctor = pkgs.writeShellApplication {
    name = "vesper-doctor";
    runtimeInputs = with pkgs; [
      btrfs-progs
      coreutils
      gnugrep
      hyprland
      jq
      pciutils
      systemd
      util-linux
    ];
    text = ''
      set -uo pipefail

      json_mode=false
      case "''${1:-}" in
        "") ;;
        --json) json_mode=true ;;
        *)
          echo "usage: vesper-doctor [--json]" >&2
          exit 2
          ;;
      esac

      checks='[]'

      record() {
        local level="$1"
        local key="$2"
        local message="$3"

        checks="$(jq -cn \
          --argjson checks "$checks" \
          --arg level "$level" \
          --arg key "$key" \
          --arg message "$message" \
          '$checks + [{level:$level,key:$key,message:$message}]')"

        if [ "$json_mode" = false ]; then
          case "$level" in
            ok) printf '[ok] %s\n' "$message" ;;
            warn) printf '[!!] %s\n' "$message" ;;
            info) printf '[--] %s\n' "$message" ;;
          esac
        fi
      }

      if [ "$json_mode" = false ]; then
        printf 'Vesper workstation doctor\n\n'
      fi

      root_fs="$(findmnt -n -o FSTYPE / 2>/dev/null || true)"
      if [ "$root_fs" = "btrfs" ]; then
        record ok root_fs "root filesystem: Btrfs"
        if systemctl list-timers --all --no-legend 2>/dev/null | grep -q 'btrfs-scrub'; then
          record ok btrfs_scrub "Btrfs scrub timer is present"
        else
          record warn btrfs_scrub "Btrfs scrub timer is not visible"
        fi
      elif [ -n "$root_fs" ]; then
        record warn root_fs "root filesystem is $root_fs, Vesper expects Btrfs"
      else
        record warn root_fs "could not determine root filesystem"
      fi

      if [ -r /sys/devices/system/cpu/amd_pstate/status ]; then
        amd_state="$(cat /sys/devices/system/cpu/amd_pstate/status)"
        if [ "$amd_state" = "active" ]; then
          record ok amd_pstate "amd_pstate: active"
        else
          record warn amd_pstate "amd_pstate: $amd_state"
        fi
      else
        record warn amd_pstate "amd_pstate status is unavailable"
      fi

      if systemctl is-active --quiet power-profiles-daemon.service; then
        record ok power_profiles "power-profiles-daemon is active"
      else
        record warn power_profiles "power-profiles-daemon is not active"
      fi

      if command -v nvidia-smi >/dev/null 2>&1; then
        gpu="$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -n1 || true)"
        if [ -n "$gpu" ]; then
          record ok nvidia "NVIDIA driver: $gpu"
        else
          record warn nvidia "nvidia-smi exists but could not query the GPU"
        fi
      else
        record warn nvidia "nvidia-smi is unavailable"
      fi

      if command -v nvidia-offload >/dev/null 2>&1; then
        record ok prime "PRIME offload command is installed"
      else
        record warn prime "nvidia-offload command is unavailable"
      fi

      if [ -n "''${HYPRLAND_INSTANCE_SIGNATURE:-}" ]; then
        monitor_summary="$(hyprctl monitors -j 2>/dev/null | jq -r '.[] | "\(.name): \(.width)x\(.height) @ \(.refreshRate) Hz scale=\(.scale)"' || true)"
        if [ -n "$monitor_summary" ]; then
          record info monitors "Hyprland monitors: $monitor_summary"
          if printf '%s\n' "$monitor_summary" | grep -Eq '@ 16[45]\.'; then
            record ok refresh_rate "internal panel appears to be running near 165 Hz"
          else
            record warn refresh_rate "no monitor currently reports ~165 Hz; verify the internal panel mode"
          fi
        else
          record warn monitors "could not read Hyprland monitor state"
        fi
      else
        record info monitors "Hyprland is not the current session; skipping live monitor check"
      fi

      if systemctl is-active --quiet tor.service; then
        record ok tor "system Tor client is active"
      else
        record warn tor "system Tor client is not active"
      fi

      if systemctl is-active --quiet vesper-web.target; then
        record info web "local web stack: active"
      else
        record ok web "local web stack: stopped"
      fi

      restic_env="''${VESPER_RESTIC_ENV_FILE:-/etc/vesper/restic.env}"
      if [ -f "$restic_env" ]; then
        # The normal user must not read this root-owned file. Existence is a
        # configuration signal; repository reachability belongs to backup code.
        record ok restic "Restic backup configuration exists (contents remain root-only)"
        if systemctl list-timers --all --no-legend 2>/dev/null | grep -q 'vesper-backup'; then
          record ok backup_timers "Vesper backup timers are present"
        else
          record warn backup_timers "Vesper backup timers are not visible"
        fi
      else
        record warn restic "Restic is not configured yet: $restic_env is missing"
      fi

      failed="$(systemctl --failed --no-legend --plain 2>/dev/null || true)"
      if [ -z "$failed" ]; then
        record ok failed_units "no failed systemd units"
      else
        record warn failed_units "failed systemd units: $failed"
      fi

      hermes_registry="''${VESPER_HERMES_JOB_REGISTRY:-$HOME/.config/vesper/hermes-jobs.json}"
      hermes_state="''${VESPER_RESEARCH_STATE_DIR:-$HOME/.local/state/vesper/research}"
      if [ -r "$hermes_registry" ]; then
        freshness_problems=0
        while IFS=$'\t' read -r job_name task freshness_minutes; do
          [ -n "$job_name" ] || continue
          latest="$hermes_state/runs/$task/latest.json"
          key="hermes_run_''${job_name//[^A-Za-z0-9_]/_}"

          if [ ! -r "$latest" ]; then
            record warn "$key" "Hermes $job_name has no durable run record yet"
            freshness_problems=$((freshness_problems + 1))
            continue
          fi

          run_status_value="$(jq -r '.status // "unknown"' "$latest" 2>/dev/null || printf 'invalid')"
          if [ "$run_status_value" != "ok" ]; then
            run_error="$(jq -r '.error // "no error detail"' "$latest" 2>/dev/null || printf 'invalid run record')"
            record warn "$key" "Hermes $job_name latest run status=$run_status_value: $run_error"
            freshness_problems=$((freshness_problems + 1))
            continue
          fi

          mtime="$(stat -c %Y "$latest" 2>/dev/null || printf '0')"
          now="$(date +%s)"
          if [ "$mtime" -le 0 ] 2>/dev/null; then
            record warn "$key" "Hermes $job_name run record timestamp is unreadable"
            freshness_problems=$((freshness_problems + 1))
            continue
          fi

          age_minutes=$(( (now - mtime) / 60 ))
          if [ "$age_minutes" -gt "$freshness_minutes" ]; then
            record warn "$key" "Hermes $job_name is stale: last successful run is $age_minutes min old (limit $freshness_minutes)"
            freshness_problems=$((freshness_problems + 1))
          fi
        done < <(
          jq -r '
            to_entries[]
            | select((.value.enabled // true) == true)
            | select((.value.mode // "dispatch") == "dispatch")
            | select((.value.freshnessMinutes // 0) > 0)
            | [.key, (.value.task // .key), ((.value.freshnessMinutes // 0) | tostring)]
            | @tsv
          ' "$hermes_registry" 2>/dev/null || true
        )

        if [ "$freshness_problems" -eq 0 ]; then
          record ok hermes_run_freshness "Hermes scheduled run records are within declared freshness windows"
        fi
      else
        record warn hermes_registry "Hermes job registry is unavailable: $hermes_registry"
      fi

      if command -v vesper-agent-messenger-auth >/dev/null 2>&1; then
        record ok communications_auth_boundary "communications setup uses the Vesper auth-only Agent Messenger wrapper"
      else
        record warn communications_auth_boundary "vesper-agent-messenger-auth is unavailable"
      fi

      if command -v agent-messenger >/dev/null 2>&1; then
        record warn communications_mutation_cli "unrestricted upstream agent-messenger is on PATH; Vesper intentionally does not install it"
      else
        record ok communications_mutation_cli "unrestricted upstream agent-messenger is not on PATH"
      fi

      comms_config="''${AGENT_MESSENGER_CONFIG_DIR:-$HOME/.config/agent-messenger}"
      comms_state_root="''${VESPER_COMMUNICATIONS_STATE_DIR:-$HOME/.local/state/vesper/communications}"
      comms_status="$comms_state_root/status.json"
      if [ ! -d "$comms_config" ]; then
        record info communications "communications intelligence is not configured yet: Agent Messenger account state is absent"
      elif [ ! -r "$comms_status" ]; then
        record warn communications "Agent Messenger account state exists but no communications intake status has been recorded yet"
      else
        comms_transport="$(jq -r '.transport // "unknown"' "$comms_status" 2>/dev/null || printf 'invalid')"
        comms_state="$(jq -r '.state // "unknown"' "$comms_status" 2>/dev/null || printf 'invalid')"
        comms_detail="$(jq -r '.detail // "no detail"' "$comms_status" 2>/dev/null || printf 'invalid status record')"
        if [ "$comms_transport" != "agent-messenger" ]; then
          record warn communications "communications intake status belongs to stale transport '$comms_transport'; run communications-radar to refresh it"
        else
          case "$comms_state" in
            ready) record ok communications "Agent Messenger intake: $comms_detail" ;;
            unconfigured) record info communications "Agent Messenger intake is not configured: $comms_detail" ;;
            unavailable) record warn communications "Agent Messenger intake unavailable: $comms_detail" ;;
            degraded) record warn communications "Agent Messenger intake degraded: $comms_detail" ;;
            *) record warn communications "Agent Messenger intake status is $comms_state: $comms_detail" ;;
          esac
        fi
      fi

      if [ -r /sys/power/mem_sleep ]; then
        record info suspend "suspend modes: $(cat /sys/power/mem_sleep)"
      fi

      if [ "$json_mode" = true ]; then
        jq -cn \
          --argjson checks "$checks" \
          '{healthy: ([ $checks[] | select(.level == "warn") ] | length == 0), checks:$checks}'
      fi
    '';
  };
in
{
  home.packages = [ vesperDoctor ];
}
