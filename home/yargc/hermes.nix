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
  hermesCore = pkgs.callPackage ./packages/hermes-core.nix {
    inherit hermesAgent;
  };

  researchEnv = ''
    export VESPER_REDDIT_SEEDS="opsec,selfhosted,programming,opensource,linux,rust,golang,cybersecurity,webdev"
    export VESPER_REDDIT_COMMENT_SEEDS="MoneroMeansMoney,Monero,vibecoding,ClaudeCode,codex,opencodeCLI,opsec"
  '';

  # Hermes resolves cron script paths before enforcing containment under
  # ~/.hermes/scripts. Home Manager home.file entries are symlinks into the
  # Nix store, so build immutable sources here and copy physical wrappers at
  # activation time.
  jobScriptSources = lib.mapAttrs (
    name: _spec:
    pkgs.writeShellScript "vesper-hermes-${name}" ''
      set -euo pipefail
      ${researchEnv}
      exec ${hermesCore}/bin/vesper-hermes-automations trigger ${lib.escapeShellArg name}
    ''
  ) jobs;

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
  home.packages = [ hermesCore ];

  home.sessionVariables = {
    VESPER_HERMES_JOB_REGISTRY = "${home}/.config/vesper/hermes-jobs.json";
    VESPER_REDDIT_SEEDS = "opsec,selfhosted,programming,opensource,linux,rust,golang,cybersecurity,webdev";
    VESPER_REDDIT_COMMENT_SEEDS = "MoneroMeansMoney,Monero,vibecoding,ClaudeCode,codex,opencodeCLI,opsec";
  };

  home.file.".config/vesper/hermes-jobs.json".text = builtins.toJSON jobs;

  home.activation.hermesCronScripts = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    mkdir -p "${home}/.hermes/scripts"
    ${installJobScripts}

    rm -f \
      "${home}/.hermes/scripts/morning-check-deliver.sh" \
      "${home}/.hermes/scripts/sabah-check-deliver.sh"
  '';

  # Hermes remains the only recurring scheduler. Reconcile only Vesper-owned
  # records after physical wrappers exist. A missing local Hermes setup should
  # not make an otherwise valid NixOS activation fail.
  home.activation.hermesCronSync = lib.hm.dag.entryAfter [ "hermesCronScripts" ] ''
    if ! ${hermesCore}/bin/vesper-hermes-automations sync-cron --prune; then
      echo "warning: Hermes cron reconciliation failed; run 'vesper-hermes-automations sync-cron --prune' after Hermes is configured" >&2
    fi
  '';
}
