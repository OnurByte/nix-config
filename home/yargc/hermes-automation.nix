{
  config,
  lib,
  pkgs,
  ...
}:
let
  home = config.home.homeDirectory;
  support = ./hermes/automation-support.py;
  fleet = ./hermes/automation-fleet.py;
  upstreamMonitor = ./hermes/upstream-edge-monitor.py;
  cronRetention = ./hermes/cron-retention.py;

  cronSync = pkgs.writeShellApplication {
    name = "vesper-hermes-cron-sync";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.python3
      pkgs.uv
    ];
    text = ''
      set -euo pipefail

      hermes_home="''${HERMES_HOME:-${home}/.hermes}"
      fleet_script="${home}/.config/vesper/hermes/automation-fleet.py"

      if [[ ! -f "$fleet_script" ]]; then
        echo "Missing $fleet_script. Run nh os switch first." >&2
        exit 1
      fi

      candidates=(
        "$hermes_home/hermes-agent/venv/bin/python"
        "$hermes_home/hermes-agent/.venv/bin/python"
      )

      for python_bin in "''${candidates[@]}"; do
        if [[ -x "$python_bin" ]] && "$python_bin" -c 'import cron.jobs' >/dev/null 2>&1; then
          exec "$python_bin" "$fleet_script" "$@"
        fi
      done

      if python3 -c 'import cron.jobs' >/dev/null 2>&1; then
        exec python3 "$fleet_script" "$@"
      fi

      if [[ -f "$hermes_home/hermes-agent/pyproject.toml" ]]; then
        exec uv run --project "$hermes_home/hermes-agent" python "$fleet_script" "$@"
      fi

      cat >&2 <<'EOF'
Could not locate a Python runtime that can import Hermes cron modules.
Expected a Hermes checkout under $HERMES_HOME/hermes-agent or an importable
Nix/runtime installation. The cron store was not modified.
EOF
      exit 1
    '';
  };
in
{
  # This file is executed directly by the Vesper wrapper rather than by Hermes'
  # cron script sandbox, so a normal Home Manager source link is fine here.
  home.file.".config/vesper/hermes/automation-fleet.py" = {
    source = fleet;
    executable = false;
  };

  # Hermes resolves pre-run/no-agent scripts with realpath and rejects anything
  # outside ~/.hermes/scripts. Home Manager source links point into /nix/store,
  # so install real copies instead of exposing those scripts through home.file.
  home.activation.installHermesAutomationScripts = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
    scripts_dir="${home}/.hermes/scripts"
    ${pkgs.coreutils}/bin/mkdir -p "$scripts_dir"

    for name in \
      frontier-github-collect.py \
      frontier-reddit-collect.py \
      free-ai-linuxdo-collect.py \
      vesper-health-watch.py \
      vesper-skill-integrity-watch.py \
      project-inventory.py \
      ai-usage-snapshot.py
    do
      ${pkgs.coreutils}/bin/rm -f "$scripts_dir/$name"
      ${pkgs.coreutils}/bin/install -m 0755 ${support} "$scripts_dir/$name"
    done

    ${pkgs.coreutils}/bin/rm -f \
      "$scripts_dir/upstream-edge-monitor.py" \
      "$scripts_dir/vesper-cron-retention.py"

    ${pkgs.coreutils}/bin/install -m 0755 ${upstreamMonitor} "$scripts_dir/upstream-edge-monitor.py"
    ${pkgs.coreutils}/bin/install -m 0755 ${cronRetention} "$scripts_dir/vesper-cron-retention.py"
  '';

  home.packages = [ cronSync ];
}
