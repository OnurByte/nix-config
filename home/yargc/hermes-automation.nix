{ config, pkgs, ... }:
let
  home = config.home.homeDirectory;
  support = ./hermes/automation-support.py;
  fleet = ./hermes/automation-fleet.py;

  supportNames = [
    "frontier-github-collect.py"
    "frontier-reddit-collect.py"
    "free-ai-linuxdo-collect.py"
    "vesper-health-watch.py"
    "vesper-skill-integrity-watch.py"
    "project-inventory.py"
    "ai-usage-snapshot.py"
  ];

  supportFiles = builtins.listToAttrs (
    map (name: {
      name = ".hermes/scripts/${name}";
      value = {
        source = support;
        executable = true;
      };
    }) supportNames
  );

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
  home.file = supportFiles // {
    ".hermes/scripts/upstream-edge-monitor.py" = {
      source = ./hermes/upstream-edge-monitor.py;
      executable = true;
    };
    ".hermes/scripts/vesper-cron-retention.py" = {
      source = ./hermes/cron-retention.py;
      executable = true;
    };
    ".config/vesper/hermes/automation-fleet.py" = {
      source = fleet;
      executable = false;
    };
  };

  home.packages = [ cronSync ];
}
