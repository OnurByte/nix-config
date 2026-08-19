{ pkgs, home }:
let
  version = "2.36.0";
  package = "agent-messenger@${version}";
  cacheDir = "${home}/.cache/vesper-agent-messenger/bun";
  configDir = "${home}/.config/agent-messenger";

  runtime = ''
    export AGENT_MESSENGER_CONFIG_DIR="${configDir}"
    export BUN_INSTALL_CACHE_DIR="${cacheDir}"
  '';

  full = pkgs.writeShellApplication {
    name = "agent-messenger";
    runtimeInputs = [ pkgs.bun ];
    text = ''
      set -euo pipefail
      ${runtime}
      exec ${pkgs.bun}/bin/bunx --package ${package} agent-messenger "$@"
    '';
  };

  # Agent Messenger intentionally exposes both read and write operations.
  # Vesper's scheduled communications intake gets a separate executable whose
  # command grammar contains only the read operations it actually needs.
  readOnly = pkgs.writeShellApplication {
    name = "vesper-agent-messenger-read";
    runtimeInputs = [ pkgs.bun ];
    text = ''
      set -euo pipefail
      ${runtime}

      if [ "$#" -lt 3 ]; then
        echo "usage: vesper-agent-messenger-read PLATFORM RESOURCE ACTION [ARGS...]" >&2
        exit 64
      fi

      platform="$1"
      resource="$2"
      action="$3"

      case "$platform:$resource:$action" in
        whatsapp:auth:status|telegram:auth:status|instagram:auth:status|discord:auth:status|\
        whatsapp:chat:list|telegram:chat:list|instagram:chat:list|\
        whatsapp:message:list|telegram:message:list|instagram:message:list|discord:message:list|\
        discord:dm:unread|discord:mention:unread)
          ;;
        *)
          echo "denied: Agent Messenger operation is outside Vesper's read-only communications allowlist" >&2
          exit 64
          ;;
      esac

      exec ${pkgs.bun}/bin/bunx --package ${package} agent-messenger "$@"
    '';
  };
in
{
  inherit version full readOnly;
}
