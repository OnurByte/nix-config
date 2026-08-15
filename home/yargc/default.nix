{
  inputs,
  username,
  ...
}:
{
  imports = [
    inputs.spicetify-nix.homeManagerModules.spicetify
    ./apps.nix
    ./caelestia.nix
    ./cli.nix
    ./dev.nix
    ./git.nix
    ./hyprland.nix
    ./lazy-tools.nix
    ./media.nix
    ./neovim.nix
    ./zsh.nix
  ];

  home = {
    inherit username;
    homeDirectory = "/home/${username}";
    stateVersion = "26.05";

    # PSYCHOVIM and Zed Preview intentionally retain their upstream-managed
    # user-space launchers. Agent tooling is declarative through Nix.
    sessionPath = [ "$HOME/.local/bin" ];

    sessionVariables = {
      EDITOR = "pycho";
      VISUAL = "pycho";
      GIT_EDITOR = "pycho";
      TERMINAL = "ghostty";
      BROWSER = "zen-beta";
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
