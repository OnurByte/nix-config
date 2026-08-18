{
  inputs,
  pkgs,
  ...
}:
let
  agents = inputs.llm-agents.packages.${pkgs.system};
  hermesAgent = import ./packages/hermes-agent.nix { inherit inputs pkgs; };

  # hermes-desktop wraps hermes-agent, so point it at the same fixed derivation
  # rather than silently pulling the original broken Hermes closure back in.
  hermesDesktop = agents.hermes-desktop.override {
    hermes-agent = hermesAgent;
  };
in
{
  home.packages = [
    inputs.codexbar.packages.${pkgs.system}.default

    # Official xAI Grok Build package from nixpkgs.
    pkgs.grok-build

    # Hermes is fully declarative: CLI, native desktop shell and optional HUD.
    hermesAgent
    hermesDesktop
    agents.hermes-hud

    # AgentsView owns the durable local session/activity archive. ccusage is an
    # accounting cross-check and TurnLens stays the per-turn Codex/Claude lens.
    agents.agentsview
    agents.ccusage
    inputs.self.packages.${pkgs.system}.turnlens

    # Stable Zed from the locked nixpkgs revision. The FHS wrapper keeps Zed's
    # downloadable extensions and language-server binaries usable on NixOS.
    pkgs.zed-editor.fhs
  ];
}
