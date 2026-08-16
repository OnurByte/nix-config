{
  inputs,
  username,
  ...
}:
{
  imports = [
    inputs.spicetify-nix.homeManagerModules.spicetify
    inputs.sops-nix.homeManagerModules.sops
    ./apps.nix
    ./caelestia.nix
    ./cli.nix
    ./command-memory.nix
    ./dev.nix
    ./doctor.nix
    ./git.nix
    ./hyprland.nix
    ./lazy-tools.nix
    ./media.nix
    ./neovim.nix
    ./privacy.nix
    ./secrets.nix
    ./skills.nix
    ./zsh.nix
  ];

  home = {
    inherit username;
    homeDirectory = "/home/${username}";
    stateVersion = "26.05";

    # PychoVIM intentionally keeps its own updater and mutable config checkout.
    # Zed is Nix-managed from the locked stable nixpkgs package.
    sessionPath = [ "$HOME/.local/bin" ];

    sessionVariables = {
      EDITOR = "pycho";
      VISUAL = "pycho";
      GIT_EDITOR = "pycho";
      TERMINAL = "ghostty";
      BROWSER = "zen-beta";

      # bb stays the control plane without sending its optional telemetry.
      BB_TELEMETRY = "false";

      # Shared Vesper state paths used by the cockpit and Hermes workflows.
      VESPER_AGENT_STATE_DIR = "$HOME/.local/state/vesper/agents";
      VESPER_BRIEFING_DIR = "$HOME/.local/share/vesper/briefings";
      VESPER_SKILL_DRAFT_DIR = "$HOME/.local/share/vesper/skill-drafts";
    };
  };

  xdg = {
    enable = true;
    userDirs = {
      enable = true;
      createDirectories = true;
    };
  };

  programs.home-manager.enable = true;
}
