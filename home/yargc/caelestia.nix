{
  inputs,
  pkgs,
  ...
}:
let
  codexbar = inputs.codexbar.packages.${pkgs.system}.default;
  agentCockpit = pkgs.callPackage ./packages/agent-cockpit.nix { };
  privacyHud = pkgs.callPackage ./packages/privacy-hud.nix { };
  aiHub = pkgs.callPackage ./packages/ai-hub.nix {
    inherit
      codexbar
      agentCockpit
      privacyHud
      ;
  };

  agenticCaelestia = inputs.caelestia-shell.packages.${pkgs.system}.with-cli.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [ ./packages/caelestia-ai-hub.patch ];

    postPatch = (old.postPatch or "") + ''
      substitute ${./packages/CodexUsage.qml} modules/bar/components/CodexUsage.qml \
        --subst-var-by aiHub ${aiHub}/bin/vesper-ai-hub
      substitute ${./packages/AiHub.qml} modules/dashboard/AiHub.qml \
        --subst-var-by aiHub ${aiHub}/bin/vesper-ai-hub
      substitute ${./packages/AgentCockpit.qml} modules/bar/components/AgentCockpit.qml \
        --subst-var-by agentCockpit ${agentCockpit}/bin/vesper-agent-cockpit
      substitute ${./packages/PrivacyHud.qml} modules/bar/components/PrivacyHud.qml \
        --subst-var-by privacyHud ${privacyHud}/bin/vesper-privacy-hud
      substitute ${./packages/HermesBriefing.qml} modules/bar/components/HermesBriefing.qml \
        --subst-var-by aiHub ${aiHub}/bin/vesper-ai-hub
      ${pkgs.coreutils}/bin/install -Dm644 ${./packages/SystemMonitor.qml} modules/bar/components/SystemMonitor.qml
      ${pkgs.coreutils}/bin/install -Dm644 ${./packages/VesperThemeSettings.qml} modules/nexus/pages/VesperThemeSettings.qml
    '';
  });

  nixDracula = pkgs.nixos-artwork.wallpapers.dracula;
  nixSolarized = pkgs.nixos-artwork.wallpapers.nineish-solarized-dark;
in
{
  imports = [
    inputs.caelestia-shell.homeManagerModules.default
  ];

  programs.caelestia = {
    enable = true;
    package = agenticCaelestia;
    systemd.enable = false;

    settings = {
      # Shell surfaces follow the Vesper glass contract: layered translucency,
      # readable backdrop blur, calm spacing and larger continuous rounding.
      appearance = {
        rounding.scale = 1.25;
        spacing.scale = 1.05;
        padding.scale = 1.05;
        anim.durations.scale = 0.85;
        transparency = {
          enabled = true;
          base = 0.68;
          layers = 0.34;
        };
      };

      paths.wallpaperDir = "~/Pictures/Wallpapers";

      general = {
        apps = {
          terminal = [ "ghostty" ];
          explorer = [ "thunar" ];
          audio = [
            "caelestia"
            "shell"
            "nexus"
            "open"
          ];
        };

        idle = {
          lockBeforeSleep = true;
          inhibitWhenAudio = true;
          inhibitWhenCharging = false;
          timeouts = [
            {
              timeout = 300;
              idleAction = "lock";
            }
            {
              timeout = 600;
              idleAction = "dpms off";
              returnAction = "dpms on";
            }
          ];
        };
      };

      services = {
        defaultPlayer = "Spotify";
        smartScheme = true;
        useTwelveHourClock = false;
      };

      dashboard = {
        enabled = true;
        showPerformance = true;
        resourceUpdateInterval = 1000;
        performance = {
          showBattery = true;
          showCpu = true;
          showGpu = true;
          showMemory = true;
          showNetwork = true;
          showStorage = true;
        };
      };

      launcher = {
        vimKeybinds = true;
        useFuzzy = {
          apps = true;
          actions = true;
          schemes = true;
          variants = true;
          wallpapers = true;
        };
      };

      utilities = {
        quickToggles = [
          { id = "wifi"; enabled = true; }
          { id = "bluetooth"; enabled = true; }
          { id = "mic"; enabled = true; }
          { id = "settings"; enabled = true; }
          { id = "dnd"; enabled = true; }
          { id = "gameMode"; enabled = false; }
          { id = "vpn"; enabled = false; }
        ];
        toasts.gameModeChanged = false;
      };

      bar.entries = [
        { id = "logo"; enabled = true; }
        { id = "workspaces"; enabled = true; }
        { id = "spacer"; enabled = true; }
        { id = "activeWindow"; enabled = true; }
        { id = "spacer"; enabled = true; }
        { id = "tray"; enabled = true; }
        { id = "systemMonitor"; enabled = true; }
        { id = "agentCockpit"; enabled = true; }
        { id = "privacyHud"; enabled = true; }
        { id = "hermesBriefing"; enabled = true; }
        { id = "aiUsage"; enabled = true; }
        { id = "clock"; enabled = true; }
        { id = "statusIcons"; enabled = true; }
        { id = "power"; enabled = true; }
      ];
    };

    cli = {
      enable = true;
      settings.theme = {
        enableTerm = true;
        enableHypr = true;
        enableDiscord = false;
        enableSpicetify = false;
        enablePandora = false;
        enableFuzzel = true;
        enableBtop = true;
        enableNvtop = true;
        enableHtop = false;
        enableGtk = true;
        enableQt = true;
        enableWarp = false;
        enableChromium = false;
        enableZed = false;
        enableCava = false;
        iconThemeLight = "Papirus-Light";
        iconThemeDark = "Papirus-Dark";
        # Upstream currently writes adw-gtk3-dark even in light mode. Correct
        # that final dconf key after palette generation without forking the CLI.
        postHook = ''
          if [ "$SCHEME_MODE" = "light" ]; then
            ${pkgs.dconf}/bin/dconf write /org/gnome/desktop/interface/gtk-theme "'adw-gtk3'"
          else
            ${pkgs.dconf}/bin/dconf write /org/gnome/desktop/interface/gtk-theme "'adw-gtk3-dark'"
          fi
          hyprctl reload
        '';
      };
    };
  };

  # Caelestia generates live GTK/Qt palettes. Home Manager provides the native
  # toolkit engines so applications consume those generated files.
  qt = {
    enable = true;
    platformTheme = {
      name = "qtengine";
      package = pkgs.qtengine;
    };
    style = {
      name = "Darkly";
      package = pkgs.darkly;
    };
  };

  home.packages = [
    agentCockpit
    privacyHud
    aiHub
    codexbar
    pkgs.adw-gtk3
    pkgs.papirus-icon-theme
    pkgs.qtengine
    pkgs.darkly
  ];

  home.file."Pictures/Wallpapers/vesper-nix-dracula.png".source = nixDracula.gnomeFilePath;
  home.file."Pictures/Wallpapers/vesper-nix-solarized-dark.png".source = nixSolarized.gnomeFilePath;
}
