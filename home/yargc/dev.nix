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
  bunx = "${pkgs.bun}/bin/bunx";
  mcpCache = "${config.home.homeDirectory}/.cache/vesper-mcp/bun";
  vesperControl = pkgs.callPackage ./packages/vesper-control.nix { };

  # GCC is the global system toolchain. Keep Clang's compiler frontends
  # available without adding Clang's entire wrapper output to Home Manager:
  # the full wrapper also exports linker shims such as ld.gold that collide
  # with GCC's wrapper in buildEnv.
  clangFrontends = pkgs.runCommand "clang-frontends" { } ''
    mkdir -p "$out/bin"
    for tool in clang clang++ clang-cpp clang-cl; do
      if [ -e "${pkgs.clang}/bin/$tool" ]; then
        ln -s "${pkgs.clang}/bin/$tool" "$out/bin/$tool"
      fi
    done
  '';

  githubMcpArchive = pkgs.fetchurl {
    url = "https://github.com/github/github-mcp-server/releases/download/v1.9.0/github-mcp-server_Linux_x86_64.tar.gz";
    hash = "sha256-y/OL0zZFGMz4C2olWH1e8RZVsV1jy7SLwGY4TQtbWWQ=";
  };

  githubMcpServer = pkgs.runCommand "github-mcp-server-1.9.0" {
    nativeBuildInputs = [
      pkgs.gnutar
      pkgs.gzip
    ];
  } ''
    mkdir -p "$out/bin"
    tar -xzf ${githubMcpArchive}
    install -Dm755 github-mcp-server "$out/bin/github-mcp-server"
  '';

  githubMcp = pkgs.writeShellApplication {
    name = "vesper-github-mcp";
    runtimeInputs = [ vesperControl ];
    text = ''
      exec ${vesperControl}/bin/vesper-control credential exec github -- \
        ${githubMcpServer}/bin/github-mcp-server stdio \
        --toolsets=context,repos,issues,pull_requests,actions
    '';
  };
in
{
  programs = {
    mcp = {
      enable = true;
      servers = {
        nixos = {
          command = "${pkgs.mcp-nixos}/bin/mcp-nixos";
        };

        context7 = {
          command = bunx;
          args = [
            "@upstash/context7-mcp@4.0.2"
          ];
          env = {
            BUN_INSTALL_CACHE_DIR = mcpCache;
          };
        };

        github = {
          command = "${githubMcp}/bin/vesper-github-mcp";
        };

        "helium-devtools" = {
          command = bunx;
          args = [
            "chrome-devtools-mcp@1.7.0"
            "--executable-path=${helium}/bin/helium"
            "--user-data-dir=${config.home.homeDirectory}/.local/share/vesper/helium-mcp"
            "--usage-statistics=false"
            "--performance-crux=false"
          ];
          env = {
            BUN_INSTALL_CACHE_DIR = mcpCache;
            CHROME_DEVTOOLS_MCP_NO_UPDATE_CHECKS = "1";
          };
        };

        "zen-devtools" = {
          command = bunx;
          args = [
            "@mozilla/firefox-devtools-mcp@0.9.15"
            "--firefox-path"
            "${zen}/bin/zen-beta"
            "--profile-path"
            "${config.home.homeDirectory}/.local/share/vesper/zen-mcp"
            "--tool-preset"
            "developer"
          ];
          env = {
            BUN_INSTALL_CACHE_DIR = mcpCache;
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
    ++ [
      plannotator
      githubMcp
    ];
}
