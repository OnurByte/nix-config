{
  inputs,
  pkgs,
  ...
}:
let
  codexbar = inputs.codexbar.packages.${pkgs.system}.default;
  agentCockpit = pkgs.callPackage ./packages/agent-cockpit.nix { };
  privacyHud = pkgs.callPackage ./packages/privacy-hud.nix { };
  niriScreenTime = pkgs.callPackage ./packages/niri-screen-time.nix { };
  vesperSettings = pkgs.callPackage ./packages/vesper-settings.nix {
    inherit niriScreenTime;
  };
  aiHub = pkgs.callPackage ./packages/ai-hub.nix {
    inherit
      codexbar
      agentCockpit
      privacyHud
      ;
  };

  # Caelestia's monitor editor is still pending upstream. Pin the reviewed PR
  # implementation to an exact commit and transplant only its native QML
  # services/pages onto the shell version already locked by this flake.
  displaySettingsSrc = builtins.fetchGit {
    url = "https://github.com/devalentineomonya/caelestia-shell.git";
    rev = "2b79be0d9c609c4fa7ecde0118143db00e213a90";
  };

  # Extend Caelestia in its native QML/Quickshell tree. Vesper shell surfaces
  # stay inside Caelestia; no parallel GTK shell or Caelestia GTK theme layer.
  nativeCaelestia = inputs.caelestia-shell.packages.${pkgs.system}.with-cli.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [
      ./packages/caelestia-display-settings.patch
      ./packages/caelestia-ai-hub.patch
    ];

    postPatch = (old.postPatch or "") + ''
      ${pkgs.coreutils}/bin/install -Dm644 ${displaySettingsSrc}/services/Hyprctl.qml services/Hyprctl.qml
      ${pkgs.coreutils}/bin/install -Dm644 ${displaySettingsSrc}/services/Monitors.qml services/Monitors.qml
      ${pkgs.coreutils}/bin/install -Dm644 ${displaySettingsSrc}/modules/nexus/pages/monitors/MonitorsPane.qml modules/nexus/pages/monitors/MonitorsPane.qml
      ${pkgs.coreutils}/bin/install -Dm644 ${displaySettingsSrc}/modules/nexus/pages/monitors/MonitorDetail.qml modules/nexus/pages/monitors/MonitorDetail.qml

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
      substitute ${./packages/VesperSystemSettings.qml} modules/nexus/pages/VesperSystemSettings.qml \
        --subst-var-by vesperSettings ${vesperSettings}/bin/vesper-settings
      substitute ${./packages/VesperClipboardSettings.qml} modules/nexus/pages/VesperClipboardSettings.qml \
        --subst-var-by vesperSettings ${vesperSettings}/bin/vesper-settings
      substituteInPlace modules/nexus/pages/AppsPage.qml \
        --subst-var-by vesperSettings ${vesperSettings}/bin/vesper-settings
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
    package = nativeCaelestia;
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
        enableGtk = false;
        enableQt = true;
        enableWarp = false;
        enableChromium = false;
        enableZed = false;
        enableCava = false;
        iconThemeLight = "Papirus-Light";
        iconThemeDark = "Papirus-Dark";
        postHook = "hyprctl reload";
      };
    };
  };

  # Caelestia owns shell appearance in native QML. Qt keeps its native
  # platform/style engines; GTK theming is intentionally left outside
  # Caelestia rather than creating a second toolkit-specific theme layer.
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

  # The unit is installable but not enabled declaratively: wellbeing stays
  # opt-in. Nexus uses `systemctl --user enable/disable --now`, so the user's
  # choice persists across login/reboot without forcing tracking on by default.
  systemd.user.services.vesper-wellbeing = {
    Unit = {
      Description = "Vesper local digital wellbeing tracker";
      After = [ "graphical-session.target" ];
    };
    Service = {
      ExecStart = "${niriScreenTime}/bin/niri-screen-time -daemon";
      Environment = "XDG_CURRENT_DESKTOP=Hyprland";
      Restart = "on-failure";
      RestartSec = 3;
    };
    Install.WantedBy = [ "graphical-session.target" ];
  };

  home.packages = [
    agentCockpit
    privacyHud
    aiHub
    codexbar
    vesperSettings
    niriScreenTime
    pkgs.papirus-icon-theme
    pkgs.qtengine
    pkgs.darkly
  ];

  home.file."Pictures/Wallpapers/vesper-nix-dracula.png".source = nixDracula.gnomeFilePath;
  home.file."Pictures/Wallpapers/vesper-nix-solarized-dark.png".source = nixSolarized.gnomeFilePath;
}