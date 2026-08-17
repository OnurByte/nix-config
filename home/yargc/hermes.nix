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
  hermesResearch = pkgs.callPackage ./packages/hermes-research-cli.nix {
    inherit hermesAgent;
  };

  researchEnv = ''
    export VESPER_REDDIT_SEEDS="opsec,selfhosted,programming,opensource,linux,rust,golang,cybersecurity,webdev"
    export VESPER_REDDIT_COMMENT_SEEDS="MoneroMeansMoney,Monero,vibecoding,ClaudeCode,codex,opencodeCLI,opsec"
  '';

  # Hermes resolves cron script paths before enforcing containment under
  # ~/.hermes/scripts. Home Manager home.file entries are symlinks into the
  # Nix store, so they would be rejected as symlink escapes at fire time.
  # Build immutable sources here, then copy them into the scripts directory
  # as real files during activation.
  jobScriptSources = lib.mapAttrs (
    name: _spec:
    pkgs.writeShellScript "vesper-hermes-${name}" ''
      set -euo pipefail
      ${researchEnv}
      exec ${hermesAutomations}/bin/vesper-hermes-automations trigger ${lib.escapeShellArg name}
    ''
  ) jobs;

  compatibilityScript = pkgs.writeShellScript "vesper-hermes-morning-check-compat" ''
    set -euo pipefail
    exec ${hermesAutomations}/bin/vesper-hermes-automations trigger morning-check
  '';

  installJobScripts = lib.concatStringsSep "\n" (
    lib.mapAttrsToList (
      name: source: ''
        target="${home}/.hermes/scripts/vesper-${name}.sh"
        rm -f "$target"
        ${pkgs.coreutils}/bin/install -Dm755 ${source} "$target"
      ''
    ) jobScriptSources
  );
in
{
  home.packages = [
    hermesRuntime
    hermesAutomations
    hermesResearch
  ];

  home.sessionVariables = {
    VESPER_HERMES_JOB_REGISTRY = "${home}/.config/vesper/hermes-jobs.json";

    # `r/opsec` is a high-signal discovery/comment seed without making it an
    # immortal source. The existing defaults are repeated because these env
    # variables intentionally replace, rather than append to, Python defaults.
    VESPER_REDDIT_SEEDS = "opsec,selfhosted,programming,opensource,linux,rust,golang,cybersecurity,webdev";
    VESPER_REDDIT_COMMENT_SEEDS = "MoneroMeansMoney,Monero,vibecoding,ClaudeCode,codex,opencodeCLI,opsec";
  };

  home.file.".config/vesper/hermes-jobs.json".text = builtins.toJSON jobs;

  # Install physical files, not Home Manager symlinks. Hermes deliberately
  # resolves symlinks and rejects anything whose resolved target escapes its
  # scripts directory.
  home.activation.hermesCronScripts = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    mkdir -p "${home}/.hermes/scripts"
    ${installJobScripts}

    for target in \
      "${home}/.hermes/scripts/morning-check-deliver.sh" \
      "${home}/.hermes/scripts/sabah-check-deliver.sh"
    do
      rm -f "$target"
      ${pkgs.coreutils}/bin/install -Dm755 ${compatibilityScript} "$target"
    done
  '';

  # Hermes remains the only scheduler. This activation step only reconciles
  # machine-owned `vesper:*` records after the physical scripts are installed.
  # A missing local Hermes setup must not make an otherwise valid NixOS switch fail.
  home.activation.hermesCronSync = lib.hm.dag.entryAfter [ "hermesCronScripts" ] ''
    if ! ${hermesAutomations}/bin/vesper-hermes-automations sync-cron --prune; then
      echo "warning: Hermes cron reconciliation failed; run 'vesper-hermes-automations sync-cron --prune' after Hermes is configured" >&2
    fi
  '';
}
