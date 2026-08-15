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
    ./neovim.nix
    ./zsh.nix
  ];

  home = {
    inherit username;
    homeDirectory = "/home/${username}";
    stateVersion = "26.05";

    sessionVariables = {
      EDITOR = "nvim";
      VISUAL = "nvim";
      TERMINAL = "ghostty";
      BROWSER = "helium";
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
