{
  pkgs,
  ...
}:
let
  psychovimRepo = "https://github.com/OnurByte/PSYCHOVIM.git";
  psychovimRev = "97c842012c65199ee287fecf3f40dbcba018822c";

  pychoRuntimeInputs = with pkgs; [
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
    tree-sitter
  ];

  pychoRuntimePath = pkgs.lib.makeBinPath pychoRuntimeInputs;

  # Nix owns the engine, dependencies and launchers. PSYCHOVIM remains a normal
  # Git checkout in ~/.config/nvim so its own updater/marketplace can keep doing
  # what the project is designed to do. The initial bootstrap is pinned to an
  # exact known commit instead of executing main/install.sh from the network.
  pychoLaunchers = pkgs.runCommand "psychovim-launchers-${builtins.substring 0 8 psychovimRev}" { } ''
    mkdir -p "$out/bin"

    cat > "$out/bin/pycho" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail

export PATH="${pychoRuntimePath}:$PATH"
export PSYCHOVIM_NVIM="${pkgs.neovim}/bin/nvim"

psychovim_repo="${psychovimRepo}"
psychovim_rev="${psychovimRev}"
config_dir="''${XDG_CONFIG_HOME:-$HOME/.config}/nvim"
backup=""

restore_backup() {
  if [[ -n "$backup" && -e "$backup" && ! -e "$config_dir" ]]; then
    mv "$backup" "$config_dir"
  fi
}

if [[ ! -d "$config_dir/.git" || ! -x "$config_dir/bin/pycho" ]]; then
  stamp="$(date +%Y%m%d-%H%M%S)"
  if [[ -e "$config_dir" || -L "$config_dir" ]]; then
    backup="''${config_dir}.backup-''${stamp}"
    mv "$config_dir" "$backup"
    printf 'PychoVIM: existing config moved to %s\n' "$backup" >&2
  fi

  mkdir -p "$(dirname "$config_dir")"
  printf 'PychoVIM: bootstrapping pinned config %s\n' "$psychovim_rev" >&2

  if ! git clone --no-checkout "$psychovim_repo" "$config_dir"; then
    rm -rf "$config_dir"
    restore_backup
    exit 1
  fi

  if ! git -C "$config_dir" checkout -B main "$psychovim_rev"; then
    rm -rf "$config_dir"
    restore_backup
    exit 1
  fi
fi

exec ${pkgs.bash}/bin/bash "$config_dir/bin/pycho" "$@"
EOF

    cat > "$out/bin/nvim" <<'EOF'
#!/usr/bin/env bash
# PYCHOVIM nvim frontend
exec "$(dirname "$0")/pycho" "$@"
EOF

    cat > "$out/bin/pychoUpdate" <<'EOF'
#!/usr/bin/env bash
exec "$(dirname "$0")/pycho" update "$@"
EOF

    cat > "$out/bin/pychoUpdater" <<'EOF'
#!/usr/bin/env bash
exec "$(dirname "$0")/pycho" update "$@"
EOF

    chmod 755 "$out/bin/pycho" "$out/bin/nvim" "$out/bin/pychoUpdate" "$out/bin/pychoUpdater"
  '';
in
{
  programs.neovim = {
    enable = true;
    defaultEditor = false;
    viAlias = false;
    vimAlias = false;
  };

  # Own the exact legacy launcher paths that PSYCHOVIM's standalone installer
  # uses. This cleanly replaces stale mutable launchers on the next HM switch
  # while keeping ~/.config/nvim itself mutable for PSYCHOVIM's updater.
  home.file = {
    ".local/bin/pycho".source = "${pychoLaunchers}/bin/pycho";
    ".local/bin/nvim".source = "${pychoLaunchers}/bin/nvim";
    ".local/bin/pychoUpdate".source = "${pychoLaunchers}/bin/pychoUpdate";
    ".local/bin/pychoUpdater".source = "${pychoLaunchers}/bin/pychoUpdater";
  };

  home.sessionPath = [ "$HOME/.local/bin" ];
  home.sessionVariables = {
    EDITOR = "pycho";
    VISUAL = "pycho";
    GIT_EDITOR = "pycho";
  };

  home.packages = [
    pkgs.tree-sitter
    pkgs.xdg-utils
  ];
}
