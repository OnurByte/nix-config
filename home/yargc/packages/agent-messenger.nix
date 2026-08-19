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

  # Human setup surface only. It can create/switch/remove authentication state,
  # but it never exposes chat/message mutation commands such as send, edit,
  # delete or react.
  authOnly = pkgs.writeShellApplication {
    name = "vesper-agent-messenger-auth";
    runtimeInputs = [ pkgs.bun ];
    text = ''
      set -euo pipefail
      ${runtime}

      if [ "$#" -lt 2 ]; then
        echo "usage: vesper-agent-messenger-auth PLATFORM ACTION [ARGS...]" >&2
        echo "platforms: whatsapp telegram instagram discord" >&2
        exit 64
      fi

      platform="$1"
      shift

      case "$platform" in
        whatsapp|telegram|instagram|discord) ;;
        *)
          echo "denied: platform is outside Vesper communications scope" >&2
          exit 64
          ;;
      esac

      exec ${pkgs.bun}/bin/bunx --package ${package} agent-messenger "$platform" auth "$@"
    '';
  };

  # Scheduled intake surface. Agent Messenger intentionally exposes both read
  # and write operations upstream; Vesper gives the communications worker a
  # separate executable whose grammar contains only the reads it needs.
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
  inherit version authOnly readOnly;
}
