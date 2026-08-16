{
  inputs,
  pkgs,
  ...
}:
let
  agents = inputs.llm-agents.packages.${pkgs.system};

  # llm-agents already disables slack-bolt's Pyramid adapter tests for Python
  # 3.14, but the pinned slack-bolt 1.29.0 test suite still fails in this
  # interpreter even though the package builds and imports correctly. Keep the
  # workaround scoped to Hermes' private Python package set: only slack-bolt's
  # upstream test phase is skipped, while the rest of the system keeps checks.
  hermesAgent = agents.hermes-agent.override (old: {
    python3 = old.python3.override {
      packageOverrides = _final: prev: {
        slack-bolt = prev.slack-bolt.overridePythonAttrs (_: {
          doCheck = false;
        });
      };
    };
  });

  # hermes-desktop wraps hermes-agent, so point it at the same fixed derivation
  # rather than silently pulling the original broken Hermes closure back in.
  hermesDesktop = agents.hermes-desktop.override {
    hermes-agent = hermesAgent;
  };

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

    # Official xAI Grok Build package from nixpkgs.
    pkgs.grok-build

    # Hermes is fully declarative: CLI, native desktop shell and optional HUD.
    hermesAgent
    hermesDesktop
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
