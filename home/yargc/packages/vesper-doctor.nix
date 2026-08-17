{
  btrfs-progs,
  coreutils,
  gnugrep,
  hyprland,
  jq,
  pciutils,
  systemd,
  util-linux,
  writeShellApplication,
}:
writeShellApplication {
  name = "vesper-doctor";
  runtimeInputs = [
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

    if [ -r /etc/vesper/restic.env ]; then
      record ok restic "Restic backup configuration exists"
      if systemctl list-timers --all --no-legend 2>/dev/null | grep -q 'vesper-backup'; then
        record ok backup_timers "Vesper backup timers are present"
      else
        record warn backup_timers "Vesper backup timers are not visible"
      fi
    else
      record warn restic "Restic is not configured yet: /etc/vesper/restic.env is missing"
    fi

    failed="$(systemctl --failed --no-legend --plain 2>/dev/null || true)"
    if [ -z "$failed" ]; then
      record ok failed_units "no failed systemd units"
    else
      record warn failed_units "failed systemd units: $failed"
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
}
