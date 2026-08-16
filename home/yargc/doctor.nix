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
      ok() { printf '[ok] %s\n' "$*"; }
      warn() { printf '[!!] %s\n' "$*"; }
      info() { printf '[--] %s\n' "$*"; }

      printf 'Vesper workstation doctor\n\n'

      root_fs="$(findmnt -n -o FSTYPE / 2>/dev/null || true)"
      if [ "$root_fs" = "btrfs" ]; then
        ok "root filesystem: Btrfs"
        if systemctl list-timers --all --no-legend 2>/dev/null | grep -q 'btrfs-scrub'; then
          ok "Btrfs scrub timer is present"
        else
          warn "Btrfs scrub timer is not visible"
        fi
      elif [ -n "$root_fs" ]; then
        warn "root filesystem is $root_fs, Vesper expects Btrfs"
      else
        warn "could not determine root filesystem"
      fi

      if [ -r /sys/devices/system/cpu/amd_pstate/status ]; then
        amd_state="$(cat /sys/devices/system/cpu/amd_pstate/status)"
        if [ "$amd_state" = "active" ]; then
          ok "amd_pstate: active"
        else
          warn "amd_pstate: $amd_state"
        fi
      else
        warn "amd_pstate status is unavailable"
      fi

      if systemctl is-active --quiet power-profiles-daemon.service; then
        ok "power-profiles-daemon is active"
      else
        warn "power-profiles-daemon is not active"
      fi

      if command -v nvidia-smi >/dev/null 2>&1; then
        gpu="$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -n1 || true)"
        if [ -n "$gpu" ]; then
          ok "NVIDIA driver: $gpu"
        else
          warn "nvidia-smi exists but could not query the GPU"
        fi
      else
        warn "nvidia-smi is unavailable"
      fi

      if command -v nvidia-offload >/dev/null 2>&1; then
        ok "PRIME offload command is installed"
      else
        warn "nvidia-offload command is unavailable"
      fi

      if [ -n "''${HYPRLAND_INSTANCE_SIGNATURE:-}" ]; then
        monitor_summary="$(hyprctl monitors -j 2>/dev/null | jq -r '.[] | "\(.name): \(.width)x\(.height) @ \(.refreshRate) Hz scale=\(.scale)"' || true)"
        if [ -n "$monitor_summary" ]; then
          info "Hyprland monitors:"
          printf '%s\n' "$monitor_summary" | sed 's/^/     /'
          if printf '%s\n' "$monitor_summary" | grep -Eq '@ 16[45]\.'; then
            ok "internal panel appears to be running near 165 Hz"
          else
            warn "no monitor currently reports ~165 Hz; verify the internal panel mode"
          fi
        else
          warn "could not read Hyprland monitor state"
        fi
      else
        info "Hyprland is not the current session; skipping live monitor check"
      fi

      if systemctl is-active --quiet tor.service; then
        ok "system Tor client is active"
      else
        warn "system Tor client is not active"
      fi

      if systemctl is-active --quiet vesper-web.target; then
        info "local web stack: active"
      else
        ok "local web stack: stopped"
      fi

      if [ -r /etc/vesper/restic.env ]; then
        ok "Restic backup configuration exists"
        if systemctl list-timers --all --no-legend 2>/dev/null | grep -q 'vesper-backup'; then
          ok "Vesper backup timers are present"
        else
          warn "Vesper backup timers are not visible"
        fi
      else
        warn "Restic is not configured yet: /etc/vesper/restic.env is missing"
      fi

      failed="$(systemctl --failed --no-legend --plain 2>/dev/null || true)"
      if [ -z "$failed" ]; then
        ok "no failed systemd units"
      else
        warn "failed systemd units:"
        printf '%s\n' "$failed" | sed 's/^/     /'
      fi

      if [ -r /sys/power/mem_sleep ]; then
        info "suspend modes: $(cat /sys/power/mem_sleep)"
      fi
    '';
  };
in
{
  home.packages = [ vesperDoctor ];
}
