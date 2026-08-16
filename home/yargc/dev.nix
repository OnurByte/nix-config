{
  config,
  inputs,
  pkgs,
  ...
}:
let
  plannotator = inputs.llm-agents.packages.${pkgs.system}.plannotator;
  plannotatorBin = "${plannotator}/bin/plannotator";
  helium = inputs.helium.packages.${pkgs.system}.default;
  zen = inputs.zen-browser.packages.${pkgs.system}.default;
  npx = "${pkgs.nodejs_24}/bin/npx";
  mcpCache = "${config.home.homeDirectory}/.cache/vesper-mcp/npm";
in
{
  programs = {
    mcp = {
      enable = true;
      servers = {
        nixos = {
          command = "${pkgs.mcp-nixos}/bin/mcp-nixos";
        };

        "helium-devtools" = {
          command = npx;
          args = [
            "-y"
            "chrome-devtools-mcp@1.7.0"
            "--executable-path=${helium}/bin/helium"
            "--user-data-dir=${config.home.homeDirectory}/.local/share/vesper/helium-mcp"
            "--usage-statistics=false"
            "--performance-crux=false"
          ];
          env = {
            CHROME_DEVTOOLS_MCP_NO_UPDATE_CHECKS = "1";
            NPM_CONFIG_CACHE = mcpCache;
            NPM_CONFIG_UPDATE_NOTIFIER = "false";
          };
        };

        "zen-devtools" = {
          command = npx;
          args = [
            "-y"
            "@mozilla/firefox-devtools-mcp@0.9.15"
            "--firefox-path"
            "${zen}/bin/zen-beta"
            "--profile-path"
            "${config.home.homeDirectory}/.local/share/vesper/zen-mcp"
            "--tool-preset"
            "developer"
          ];
          env = {
            NPM_CONFIG_CACHE = mcpCache;
            NPM_CONFIG_UPDATE_NOTIFIER = "false";
          };
        };
      };
    };

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
      mcp-nixos

      # Shell / systems
      shellcheck
      shfmt
      gcc
      # Keep both compiler frontends available without making their bundled
      # linker wrappers compete for the same Home Manager buildEnv paths.
      (lib.lowPrio clang)
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
