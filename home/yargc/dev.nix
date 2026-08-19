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
  mcpCacheRoot = "${config.home.homeDirectory}/.cache/vesper-mcp";
  bunMcpCache = "${mcpCacheRoot}/bun";
  hypruseJournal = "${config.home.homeDirectory}/.local/state/vesper/mcp/hypruse/journal.ndjson";

  # Keep Context7 at the currently selected 4.0.2 release without moving the
  # workstation-wide nixpkgs pin. This is the nixpkgs package recipe with the
  # newer immutable source/dependency hashes, so MCP startup never downloads JS.
  context7Src = pkgs.fetchFromGitHub {
    owner = "upstash";
    repo = "context7";
    tag = "@upstash/context7-mcp@4.0.2";
    hash = "sha256-mRjDG+hGG7gU+05CMAtBy7oVFRNSQgQMWgMEnfmmlSM=";
  };

  context7Mcp = pkgs.context7-mcp.overrideAttrs (_old: {
    version = "4.0.2";
    src = context7Src;
    pnpmDeps = pkgs.fetchPnpmDeps {
      pname = "context7-mcp";
      version = "4.0.2";
      src = context7Src;
      pnpm = pkgs.pnpm_10;
      fetcherVersion = 3;
      hash = "sha256-F3c2/y3fgtPiUQOsg3hFdAp9b85AFs2mCTO1Eoa0i5E=";
    };
  });

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
    runtimeInputs = [
      pkgs.gh
      githubMcpServer
    ];
    text = ''
      if token="$(gh auth token 2>/dev/null)" && [ -n "$token" ]; then
        export GITHUB_PERSONAL_ACCESS_TOKEN="$token"
      fi

      exec github-mcp-server stdio --toolsets=context,repos,issues,pull_requests,actions
    '';
  };

  # Upstream Python is acceptable inside a pinned external package; Vesper's
  # first-party control plane remains Rust. Build the published universal wheel
  # into the Nix store so desktop control never downloads code at MCP startup.
  hyprusePackage = pkgs.python3Packages.buildPythonApplication (finalAttrs: {
    pname = "hypruse";
    version = "0.9.4";
    format = "wheel";

    src = pkgs.fetchPypi {
      inherit (finalAttrs) pname version;
      format = "wheel";
      dist = "py3";
      python = "py3";
      hash = "sha256-v2QV5fUtbiUIfyn1pKaqL0qsk1RUIJXA28kZ8b+/f84=";
    };

    dependencies = [ pkgs.python3Packages.mcp ];
    pythonImportsCheck = [ "hypruse" ];
  });

  # Confinement allows input only in windows the MCP launched itself; auth
  # dialogs remain guarded, human seat changes fail closed and clipboard stays
  # disabled. The journal records calls/refusals without persisting typed text.
  hypruseMcp = pkgs.writeShellApplication {
    name = "vesper-hypruse-mcp";
    runtimeInputs = [
      hyprusePackage
      pkgs.grim
      pkgs.wtype
      pkgs.imagemagick
      pkgs.systemd
    ];
    text = ''
      export HYPRUSE_CONFINE="launched"
      export HYPRUSE_AUTH_GUARD="strict"
      export HYPRUSE_STRICT="1"
      export HYPRUSE_MARK="1"
      export HYPRUSE_JOURNAL="${hypruseJournal}"
      unset HYPRUSE_CLIPBOARD
      unset HYPRUSE_JOURNAL_TEXT

      exec hypruse
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
          command = "${context7Mcp}/bin/context7-mcp";
        };

        github = {
          command = "${githubMcp}/bin/vesper-github-mcp";
        };

        hypruse = {
          command = "${hypruseMcp}/bin/vesper-hypruse-mcp";
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
            BUN_INSTALL_CACHE_DIR = bunMcpCache;
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
            BUN_INSTALL_CACHE_DIR = bunMcpCache;
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
      hypruseMcp
    ];
}
