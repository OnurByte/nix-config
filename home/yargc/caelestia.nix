{
  inputs,
  pkgs,
  ...
}:
let
  codexbar = inputs.codexbar.packages.${pkgs.system}.default;
  codexbarUi = pkgs.callPackage ./packages/codexbar-ui.nix {
    src = inputs.codexbar-ui-src;
    inherit codexbar;
  };
  agentCockpit = pkgs.callPackage ./packages/agent-cockpit.nix { };
  privacyHud = pkgs.callPackage ./packages/privacy-hud.nix { };
  hermesRuntime = pkgs.callPackage ./packages/hermes-runtime.nix { };

  agenticCaelestia = inputs.caelestia-shell.packages.${pkgs.system}.with-cli.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [ ./packages/caelestia-codexbar.patch ];

    postPatch = (old.postPatch or "") + ''
      substitute ${./packages/CodexUsage.qml} modules/bar/components/CodexUsage.qml \
        --subst-var-by codexbarStatus ${codexbarUi}/bin/codexbar-status \
        --subst-var-by codexbarPopup ${codexbarUi}/bin/codexbar-popup
      substitute ${./packages/AgentCockpit.qml} modules/bar/components/AgentCockpit.qml \
        --subst-var-by agentCockpit ${agentCockpit}/bin/vesper-agent-cockpit
      substitute ${./packages/PrivacyHud.qml} modules/bar/components/PrivacyHud.qml \
        --subst-var-by privacyHud ${privacyHud}/bin/vesper-privacy-hud
      substitute ${./packages/HermesBriefing.qml} modules/bar/components/HermesBriefing.qml \
        --subst-var-by hermesRuntime ${hermesRuntime}/bin/vesper-hermes
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
        enableQt = false;
        enableWarp = false;
        enableChromium = false;
        enableZed = false;
        enableCava = false;
        postHook = "hyprctl reload";
      };
    };
  };

  home.packages = [
    agentCockpit
    privacyHud
    codexbarUi
    pkgs.adw-gtk3
    pkgs.papirus-icon-theme
  ];

  xdg.desktopEntries = {
    vesper-agent-cockpit = {
      name = "Vesper Agent Cockpit";
      genericName = "Coding Agent Monitor";
      comment = "Inspect active coding agents and their Git worktrees";
      exec = "vesper-agent-cockpit";
      icon = "utilities-terminal";
      terminal = false;
      categories = [
        "Development"
        "Utility"
      ];
    };

    vesper-privacy-hud = {
      name = "Vesper Privacy HUD";
      genericName = "Privacy Status";
      comment = "Inspect local Tor, microphone, camera, clipboard and Monero state";
      exec = "vesper-privacy-hud";
      icon = "security-high";
      terminal = false;
      categories = [
        "Utility"
        "Security"
      ];
    };

    vesper-hermes-briefings = {
      name = "Vesper Hermes Briefings";
      genericName = "Research Inbox";
      comment = "Open the persistent Hermes research briefing inbox";
      exec = "vesper-hermes inbox";
      icon = "mail-unread";
      terminal = false;
      categories = [
        "Utility"
        "Development"
      ];
    };
  };

  home.file."Pictures/Wallpapers/vesper-nix-dracula.png".source = nixDracula.gnomeFilePath;
  home.file."Pictures/Wallpapers/vesper-nix-solarized-dark.png".source = nixSolarized.gnomeFilePath;

  xdg.dataFile."codexbar-waybar/icons".source = "${codexbarUi}/share/codexbar-waybar/icons";
}
