{
  inputs,
  pkgs,
  ...
}:
let
  plannotator = inputs.llm-agents.packages.${pkgs.system}.plannotator;
  plannotatorBin = "${plannotator}/bin/plannotator";

  # GCC is the global system toolchain. Keep the wrapped Clang compiler
  # frontends available too, but do not add Clang's entire wrapper output to
  # Home Manager: it also exports linker shims such as ld.gold that collide
  # with GCC's wrapper in buildEnv.
  clangFrontends = pkgs.runCommand "clang-frontends" { } ''
    mkdir -p "$out/bin"
    for tool in clang clang++ clang-cpp clang-cl; do
      if [ -e "${pkgs.clang}/bin/$tool" ]; then
        ln -s "${pkgs.clang}/bin/$tool" "$out/bin/$tool"
      fi
    done
  '';
in
{
  programs = {
    mcp.enable = true;

    codex = {
      enable = true;
      enableMcpIntegration = true;

      settings.features.hooks = true;
      hooks.Stop = [
        {
          hooks = [
            {
              type = "command";
              command = plannotatorBin;
              timeout = 345600;
              statusMessage = "Opening Plannotator review";
            }
          ];
        }
      ];
    };

    claude-code = {
      enable = true;
      enableMcpIntegration = true;

      settings.hooks = {
        PreToolUse = [
          {
            matcher = "EnterPlanMode";
            hooks = [
              {
                type = "command";
                command = "${plannotatorBin} improve-context";
                timeout = 5;
              }
            ];
          }
        ];
        PermissionRequest = [
          {
            matcher = "ExitPlanMode";
            hooks = [
              {
                type = "command";
                command = plannotatorBin;
                timeout = 345600;
              }
            ];
          }
        ];
      };
    };

    opencode = {
      enable = true;
      enableMcpIntegration = true;
    };

    mise = {
      enable = true;
      enableZshIntegration = true;
      enableMutableConfig = true;
    };

    lazygit.enable = true;
  };

  home.packages =
    (with pkgs; [
      # Nix
      nixd
      nixfmt-rfc-style

      # Shell / systems
      shellcheck
      shfmt
      gcc
      clangFrontends
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

      # JS / TS — Bun is the package manager; Node stays as a runtime/LSP dependency.
      nodejs_24
      bun
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
    ])
    ++ [ plannotator ];
}
