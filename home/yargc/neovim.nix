{
  pkgs,
  ...
}:
let
  pychoBootstrap = pkgs.writeShellApplication {
    name = "pycho";
    runtimeInputs = with pkgs; [
      bash
      coreutils
      curl
      git
      gnutar
      gzip
      unzip
      ripgrep
      gnugrep
      gnused
      gawk
      findutils
      gnumake
      gcc
      nodejs_24
      xdg-utils
      neovim
      tree-sitter
    ];
    text = ''
      set -euo pipefail

      pycho_bin="$HOME/.local/bin/pycho"
      config_dir="''${XDG_CONFIG_HOME:-$HOME/.config}/nvim"

      if [ ! -x "$pycho_bin" ] || [ ! -f "$config_dir/init.lua" ]; then
        echo "Bootstrapping OnurByte/PSYCHOVIM..." >&2
        curl -fsSL https://raw.githubusercontent.com/OnurByte/PSYCHOVIM/main/install.sh | bash
      fi

      exec "$pycho_bin" "$@"
    '';
  };
in
{
  # Nix supplies the engine and system dependencies. PSYCHOVIM owns the mutable
  # editor config, updater and marketplace in ~/.config/nvim and ~/.local.
  programs.neovim = {
    enable = true;
    defaultEditor = false;
    viAlias = false;
    vimAlias = false;
  };

  home.packages = [
    pychoBootstrap
    pkgs.tree-sitter
    pkgs.xdg-utils
  ];
}
