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
in
{
  home.packages = [
    inputs.codexbar.packages.${pkgs.system}.default

    # Focused cloud/provider agent workflow. Browser control comes from the
    # Helium and Zen MCP servers in dev.nix rather than another browser layer.
    agents.bb-app

    # Official xAI Grok Build package from nixpkgs.
    pkgs.grok-build

    # Hermes is fully declarative: CLI, native desktop shell and optional HUD.
    hermesAgent
    hermesDesktop
    agents.hermes-hud

    # Broad historical accounting plus per-turn Codex/Claude measurement.
    agents.ccusage
    inputs.self.packages.${pkgs.system}.turnlens

    # Stable Zed from the locked nixpkgs revision. The FHS wrapper keeps Zed's
    # downloadable extensions and language-server binaries usable on NixOS.
    pkgs.zed-editor.fhs
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
