{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:
let
  home = config.home.homeDirectory;
  jobs = import ./hermes-jobs.nix;
  hermesAgent = import ./packages/hermes-agent.nix { inherit inputs pkgs; };
  hermesRuntime = pkgs.callPackage ./packages/hermes-runtime.nix { };
  hermesAutomations = pkgs.callPackage ./packages/hermes-automations.nix {
    inherit hermesAgent;
  };

  generatedJobScripts = lib.mapAttrs' (
    name: _spec:
    lib.nameValuePair ".hermes/scripts/vesper-${name}.sh" {
      executable = true;
      text = ''
        #!/usr/bin/env bash
        set -euo pipefail
        exec ${hermesAutomations}/bin/vesper-hermes-automations trigger ${lib.escapeShellArg name}
      '';
    }
  ) jobs;

  morningCompatibilityScript = ''
    #!/usr/bin/env bash
    set -euo pipefail
    exec ${hermesAutomations}/bin/vesper-hermes-automations trigger morning-check
  '';
in
{
  home.packages = [
    hermesRuntime
    hermesAutomations
  ];

  home.sessionVariables.VESPER_HERMES_JOB_REGISTRY = "${home}/.config/vesper/hermes-jobs.json";

  home.file = generatedJobScripts // {
    ".config/vesper/hermes-jobs.json".text = builtins.toJSON jobs;

    # Keep both historical entrypoints as short aliases until every mutable
    # jobs.json has been reconciled. Long model work never runs inside these
    # Hermes no_agent scripts.
    ".hermes/scripts/morning-check-deliver.sh" = {
      executable = true;
      text = morningCompatibilityScript;
    };
    ".hermes/scripts/sabah-check-deliver.sh" = {
      executable = true;
      text = morningCompatibilityScript;
    };
  };

  # Hermes remains the only scheduler. This activation step only reconciles
  # machine-owned `vesper:*` records after Home Manager has linked the scripts.
  # A missing local Hermes setup must not make an otherwise valid NixOS switch fail.
  home.activation.hermesCronSync = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    if ! ${hermesAutomations}/bin/vesper-hermes-automations sync-cron --prune; then
      echo "warning: Hermes cron reconciliation failed; run 'vesper-hermes-automations sync-cron --prune' after Hermes is configured" >&2
    fi
  '';
}
