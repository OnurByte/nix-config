{ pkgs, ... }:
{
  programs = {
    mcp.enable = true;

    codex = {
      enable = true;
      enableMcpIntegration = true;
    };

    claude-code = {
      enable = true;
      enableMcpIntegration = true;
    };

    opencode = {
      enable = true;
      enableMcpIntegration = true;
    };

    # Keep Nix as the system baseline, but allow project-local runtime pinning
    # for repos that expect .tool-versions / mise.toml workflows.
    mise = {
      enable = true;
      enableZshIntegration = true;
      enableMutableConfig = true;
    };

    lazygit.enable = true;
  };

  home.packages = with pkgs; [
    # Nix
    nixd
    nixfmt-rfc-style

    # Shell / systems
    shellcheck
    shfmt
    gcc
    clang
    gdb
    cmake
    gnumake
    pkg-config

    # Rust
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt

    # Go
    go
    gopls

    # Python
    python3
    uv
    ruff

    # JS / TS
    nodejs_24
    bun
    pnpm
    typescript
    typescript-language-server

    # PHP
    php
    php84Packages.composer
    intelephense

    # Java
    jdk21
    jdt-language-server

    # Lua
    lua
    lua-language-server
    stylua
  ];
}
