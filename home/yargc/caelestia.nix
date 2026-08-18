{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:
let
  codexbar = inputs.codexbar.packages.${pkgs.system}.default;
  agentCockpit = pkgs.callPackage ./packages/agent-cockpit.nix { };
  privacyHud = pkgs.callPackage ./packages/privacy-hud.nix { };
  vesperControl = pkgs.callPackage ./packages/vesper-control.nix { };
  aiHub = pkgs.callPackage ./packages/ai-hub.nix {
    inherit
      codexbar
      agentCockpit
      privacyHud
      ;
  };
  mcpServerNames = builtins.attrNames config.programs.mcp.servers;
  caelestiaPatch = pkgs.writeText "caelestia-ai-hub.patch" (
    builtins.readFile ./packages/caelestia-ai-hub.patch + "\n"
  );

  agenticCaelestia = inputs.caelestia-shell.packages.${pkgs.system}.with-cli.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [ caelestiaPatch ];

    postPatch = (old.postPatch or "") + ''
      substitute ${./packages/CodexUsage.qml} modules/bar/components/CodexUsage.qml \
        --subst-var-by aiHub ${aiHub}/bin/vesper-ai-hub
      substitute ${./packages/AiHub.qml} modules/dashboard/AiHub.qml \
        --subst-var-by aiHub ${aiHub}/bin/vesper-ai-hub
      substituteInPlace modules/dashboard/AiHub.qml \
        --replace-fail 'qsTr("Vesper Hub")' 'qsTr("AI")' \
        --replace-fail 'AI Hub returned invalid data' 'AI returned invalid data'
      substitute ${./packages/AgentCockpit.qml} modules/bar/components/AgentCockpit.qml \
        --subst-var-by agentCockpit ${agentCockpit}/bin/vesper-agent-cockpit
      substitute ${./packages/PrivacyHud.qml} modules/bar/components/PrivacyHud.qml \
        --subst-var-by privacyHud ${privacyHud}/bin/vesper-privacy-hud
      substitute ${./packages/HermesBriefing.qml} modules/bar/components/HermesBriefing.qml \
        --subst-var-by aiHub ${aiHub}/bin/vesper-ai-hub
      substitute ${./packages/AiPage.qml} modules/nexus/pages/AiPage.qml \
        --subst-var-by vesperControl ${vesperControl}/bin/vesper-control \
        --subst-var-by aiHub ${aiHub}/bin/vesper-ai-hub
      substitute ${./packages/AiCredentials.qml} modules/nexus/pages/AiCredentials.qml \
        --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperNetworkSettings.qml} modules/nexus/pages/VesperNetworkSettings.qml \
        --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperProxyPage.qml} modules/nexus/pages/VesperProxyPage.qml \
        --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperAppsSettings.qml} modules/nexus/pages/VesperAppsSettings.qml \
        --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperAppControls.qml} modules/nexus/pages/apps/VesperAppControls.qml \
        --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperThemeSettings.qml} modules/nexus/pages/VesperThemeSettings.qml \
        --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperNavLocations.qml} modules/nexus/navpane/NavLocations.qml
      substituteInPlace modules/nexus/PageRegistry.qml \
        --replace-fail 'label: qsTr("Wallpaper & style")' 'label: qsTr("Appearance")' \
        --replace-fail 'description: qsTr("Wallpaper, fonts, colours")' 'description: qsTr("Wallpaper, colours, icons")' \
        --replace-fail 'description: qsTr("Wi-Fi, ethernet, VPN")' 'description: qsTr("Wi-Fi, ethernet, VPN, proxy")' \
        --replace-fail 'description: qsTr("Default apps, favourites, hidden apps")' 'description: qsTr("Defaults, permissions, wellbeing, icons")'
      ${pkgs.coreutils}/bin/install -Dm644 ${./packages/SystemMonitor.qml} modules/bar/components/SystemMonitor.qml
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
        iconThemeLight = "Vesper-Adaptive";
        iconThemeDark = "Vesper-Adaptive";
        # Upstream currently writes adw-gtk3-dark even in light mode. Correct
        # that final dconf key after palette generation without forking the CLI.
        # The icon engine consumes the same generated palette and recompiles
        # canonical assets locally. No remote AI work happens in this hook.
        postHook = ''
          if [ "$SCHEME_MODE" = "light" ]; then
            ${pkgs.dconf}/bin/dconf write /org/gnome/desktop/interface/gtk-theme "'adw-gtk3'"
          else
            ${pkgs.dconf}/bin/dconf write /org/gnome/desktop/interface/gtk-theme "'adw-gtk3-dark'"
          fi
          ${vesperControl}/bin/vesper-control icon sync-theme "$SCHEME_MODE" || true
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
    vesperControl
    codexbar
    pkgs.adw-gtk3
    pkgs.papirus-icon-theme
    pkgs.qtengine
    pkgs.darkly
  ];

  # Caelestia renders user templates into ~/.local/state/caelestia/theme.
  # Keep only the primary accent as the icon compiler input. The icon engine
  # owns material recipes and must not duplicate the full Caelestia palette.
  home.file.".config/caelestia/templates/vesper-icons".text = "{{ primary.hex }}\n";

  # The AI settings page reads the same MCP registry that Home Manager feeds to
  # Codex, Claude Code and OpenCode. Keep this generated inventory value-only.
  home.file.".config/vesper/mcp-servers".text = lib.concatStringsSep "\n" mcpServerNames + "\n";

  home.file."Pictures/Wallpapers/vesper-nix-dracula.png".source = nixDracula.gnomeFilePath;
  home.file."Pictures/Wallpapers/vesper-nix-solarized-dark.png".source = nixSolarized.gnomeFilePath;

  # The configured icon theme name must always resolve, even when adaptive
  # icons are disabled. In that state the generated theme is intentionally
  # empty and inherits Papirus, giving an immediate visual rollback.
  home.activation.vesperAdaptiveIconTheme = lib.hm.dag.entryAfter [ "linkGeneration" ] ''
    ${vesperControl}/bin/vesper-control icon ensure-theme
  '';

  # Filesystem notifications are the primary discovery path. The daemon also
  # performs a bounded periodic full scan as recovery for missed profile or
  # exported Flatpak changes.
  systemd.user.services.vesper-adaptive-icons = {
    Unit = {
      Description = "Vesper adaptive application icon reconciliation";
      After = [ "graphical-session-pre.target" ];
    };
    Service = {
      ExecStart = "${vesperControl}/bin/vesper-icon-engine daemon";
      Restart = "on-failure";
      RestartSec = 5;
    };
    Install.WantedBy = [ "default.target" ];
  };
}
