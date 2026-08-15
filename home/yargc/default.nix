{
  username,
  ...
}:
{
  imports = [
    ./apps.nix
    ./caelestia.nix
    ./cli.nix
    ./dev.nix
    ./git.nix
    ./hyprland.nix
    ./lazy-tools.nix
    ./neovim.nix
    ./zsh.nix
  ];

  home = {
    inherit username;
    homeDirectory = "/home/${username}";
    stateVersion = "26.05";

    # Mutable upstream tools (PSYCHOVIM, Zed Preview, Hermes) install their
    # launchers here. Keep it ahead of the Nix profile so their official
    # self-update flow remains intact.
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
