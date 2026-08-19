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
  beeperMcpUrl = "http://127.0.0.1:23373/v0/mcp";
  beeperHermesProfile = "vesper-social";

  # Keep Beeper's mutation-capable MCP out of the default Hermes profile used
  # by Vesper cron jobs. The explicit social profile is cloned once from the
  # default profile so it inherits the operator's model/provider setup, then
  # owns its MCP/OAuth state independently under ~/.hermes/profiles/.
  hermesBeeperMcp = pkgs.writeShellApplication {
    name = "vesper-hermes-beeper-mcp";
    text = ''
      set -euo pipefail

      hermes=${hermesAgent}/bin/hermes
      profile=${lib.escapeShellArg beeperHermesProfile}

      ensure_profile() {
        if ! "$hermes" profile show "$profile" >/dev/null 2>&1; then
          "$hermes" profile create "$profile" \
            --clone-from default \
            --no-alias \
            --description "Interactive Vesper social profile for the local Beeper MCP."
        fi
      }

      case "''${1:-status}" in
        setup)
          ensure_profile
          exec "$hermes" -p "$profile" mcp add beeper \
            --url ${lib.escapeShellArg beeperMcpUrl} \
            --auth oauth
          ;;
        login)
          ensure_profile
          exec "$hermes" -p "$profile" mcp login beeper
          ;;
        test)
          ensure_profile
          exec "$hermes" -p "$profile" mcp test beeper
          ;;
        chat)
          ensure_profile
          exec "$hermes" -p "$profile" chat
          ;;
        status|list)
          ensure_profile
          exec "$hermes" -p "$profile" mcp list
          ;;
        *)
          echo "usage: vesper-hermes-beeper-mcp {setup|login|test|chat|status}" >&2
          exit 2
          ;;
      esac
    '';
  };

  researchEnv = ''
    export VESPER_REDDIT_SEEDS="opsec,selfhosted,programming,opensource,linux,rust,golang,cybersecurity,webdev"
    export VESPER_REDDIT_COMMENT_SEEDS="MoneroMeansMoney,Monero,vibecoding,ClaudeCode,codex,opencodeCLI,opsec"
    export VESPER_BEEPER_BASE_URL="http://127.0.0.1:23373"
    export VESPER_BEEPER_TOKEN_FILE="${home}/.config/vesper/beeper.token"
    export VESPER_COMMUNICATIONS_STATE_DIR="${home}/.local/state/vesper/communications"
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
      ${hermesCore}/bin/vesper-hermes-automations trigger ${lib.escapeShellArg name}

      ${lib.optionalString (name == "vesper-health-watch") ''
        # Optional external dead-man signal. The URL is mutable secret/config
        # state, never Nix/Git state and never a curl command-line argument.
        deadman_file="''${VESPER_DEADMAN_URL_FILE:-${home}/.config/vesper/hermes-deadman.url}"
        deadman_current=""

        if [ -r "$deadman_file" ]; then
          IFS= read -r deadman_url < "$deadman_file" || true
          case "$deadman_url" in
            http://*|https://*)
              if ! printf 'url = "%s"\nfail\nsilent\nshow-error\nmax-time = 15\n' "$deadman_url" \
                | ${pkgs.curl}/bin/curl --config - >/dev/null 2>&1; then
                deadman_current='[Hermes dead-man] external heartbeat ping failed'
              fi
              ;;
            "") ;;
            *) deadman_current='[Hermes dead-man] URL file exists but does not contain an http(s) URL' ;;
          esac
        fi

        deadman_state="''${VESPER_RESEARCH_STATE_DIR:-${home}/.local/state/vesper/research}/watches/hermes-deadman-watch.txt"
        deadman_previous=""
        if [ -r "$deadman_state" ]; then
          IFS= read -r deadman_previous < "$deadman_state" || true
        fi

        if [ "$deadman_current" != "$deadman_previous" ]; then
          ${pkgs.coreutils}/bin/mkdir -p "$(${pkgs.coreutils}/bin/dirname "$deadman_state")"
          deadman_tmp="$deadman_state.tmp.$$"
          printf '%s\n' "$deadman_current" > "$deadman_tmp"
          ${pkgs.coreutils}/bin/mv -f "$deadman_tmp" "$deadman_state"

          if [ -n "$deadman_current" ]; then
            printf '%s\n' "$deadman_current"
          elif [ -n "$deadman_previous" ]; then
            echo '[Hermes dead-man] external heartbeat recovered or was disabled'
          fi
        fi
      ''}
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
  home.packages = [
    hermesCore
    hermesBeeperMcp
  ];

  home.sessionVariables = {
    VESPER_HERMES_JOB_REGISTRY = "${home}/.config/vesper/hermes-jobs.json";
    VESPER_REDDIT_SEEDS = "opsec,selfhosted,programming,opensource,linux,rust,golang,cybersecurity,webdev";
    VESPER_REDDIT_COMMENT_SEEDS = "MoneroMeansMoney,Monero,vibecoding,ClaudeCode,codex,opencodeCLI,opsec";
    VESPER_BEEPER_BASE_URL = "http://127.0.0.1:23373";
    VESPER_BEEPER_MCP_URL = beeperMcpUrl;
    VESPER_COMMUNICATIONS_STATE_DIR = "${home}/.local/state/vesper/communications";
    VESPER_BEEPER_TOKEN_FILE = "${home}/.config/vesper/beeper.token";
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
