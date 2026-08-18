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
    ./hermes.nix
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

    sessionVariables = {
      TERMINAL = "ghostty";
      BROWSER = "zen-beta";

      # bb stays the control plane without sending its optional telemetry.
      BB_TELEMETRY = "false";

      # Use absolute paths here: Home Manager session variables are shell-escaped
      # values, so a literal $HOME would not be expanded a second time.
      VESPER_AGENT_STATE_DIR = "/home/${username}/.local/state/vesper/agents";
      VESPER_RESEARCH_STATE_DIR = "/home/${username}/.local/state/vesper/research";
      VESPER_BRIEFING_DIR = "/home/${username}/.local/share/vesper/briefings";
      VESPER_SKILL_DRAFT_DIR = "/home/${username}/.local/share/vesper/skill-drafts";
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
