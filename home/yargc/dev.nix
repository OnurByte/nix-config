{
  inputs,
  pkgs,
  ...
}:
let
  plannotator = inputs.llm-agents.packages.${pkgs.system}.plannotator;
  plannotatorBin = "${plannotator}/bin/plannotator";
in
{
  programs = {
    mcp.enable = true;

    codex = {
      enable = true;
      enableMcpIntegration = true;

      # Codex exposes lifecycle hooks through Home Manager. When a turn ends
      # with a plan, Plannotator opens its local browser review surface; reject
      # feedback is returned to Codex so the same turn can revise the plan.
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

      # Mirror Plannotator's upstream Claude Code plugin hooks without a
      # mutable marketplace install. EnterPlanMode sharpens context; asking to
      # leave plan mode opens the local visual approval/review surface.
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

    # Keep Nix as the system baseline, but allow project-local runtime pinning
    # for repos that expect .tool-versions / mise.toml workflows.
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
    ])
    ++ [ plannotator ];
}
