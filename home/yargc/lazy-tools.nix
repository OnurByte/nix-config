{
  inputs,
  pkgs,
  ...
}:
let
  agents = inputs.llm-agents.packages.${pkgs.system};

  zedPreview = pkgs.writeShellApplication {
    name = "zed-preview";
    runtimeInputs = with pkgs; [
      curl
      coreutils
      gnutar
      gzip
      xz
      gnused
    ];
    text = ''
      set -euo pipefail

      zed="$HOME/.local/zed-preview.app/bin/zed"
      if [ ! -x "$zed" ]; then
        echo "Installing Zed Preview from the official Zed installer..." >&2
        curl -fsSL https://zed.dev/install.sh | ZED_CHANNEL=preview sh
      fi

      exec "$zed" "$@"
    '';
  };
in
{
  home.packages = [
    inputs.codexbar.packages.${pkgs.system}.default

    # Focused cloud/provider agent workflow. No Ollama or local model daemon.
    agents.bb-app
    agents.agent-browser

    # Hermes is fully declarative: CLI, native desktop shell and optional HUD.
    agents.hermes-agent
    agents.hermes-desktop
    agents.hermes-hud

    # Broad historical accounting plus per-turn Codex/Claude measurement.
    agents.ccusage
    inputs.self.packages.${pkgs.system}.turnlens

    zedPreview
  ];

  # Use store-qualified commands so desktop launch does not depend on whatever
  # PATH a display manager happened to inherit.
  xdg.desktopEntries.bb = {
    name = "bb";
    genericName = "Agent IDE";
    comment = "Control plane for Codex, Claude Code, OpenCode and Hermes";
    exec = "${agents.bb-app}/bin/bb-app";
    icon = "applications-development";
    terminal = false;
    categories = [ "Development" "Utility" ];
  };
}
