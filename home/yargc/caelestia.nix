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
  vesperDoctor = pkgs.callPackage ./packages/vesper-doctor.nix { };
  hermesAgent = inputs.hermes-agent.packages.${pkgs.system}.default;
  hermesCore = pkgs.callPackage ./packages/hermes-core.nix { inherit hermesAgent; };
  ai = pkgs.callPackage ./packages/ai.nix { inherit codexbar agentCockpit privacyHud; };
  mcpServerNames = builtins.attrNames config.programs.mcp.servers;
  caelestiaAiPatch = pkgs.writeText "caelestia-ai.patch" (builtins.readFile ./packages/caelestia-ai.patch + "\n");
  caelestiaAppIconsPatch = pkgs.writeText "caelestia-app-icons.patch" (builtins.readFile ./packages/caelestia-app-icons.patch + "\n");
  caelestiaWellbeingPatch = pkgs.writeText "caelestia-wellbeing-ipc.patch" (builtins.readFile ./packages/caelestia-wellbeing-ipc.patch + "\n");
  caelestiaSettingsNamePatch = pkgs.writeText "caelestia-settings-name.patch" (builtins.readFile ./packages/caelestia-settings-name.patch + "\n");

  agenticCaelestia = inputs.caelestia-shell.packages.${pkgs.system}.with-cli.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [ caelestiaAiPatch caelestiaAppIconsPatch caelestiaWellbeingPatch caelestiaSettingsNamePatch ];
    postPatch = (old.postPatch or "") + ''
      substitute ${./packages/CodexUsage.qml} modules/bar/components/CodexUsage.qml --subst-var-by ai ${ai}/bin/vesper-ai
      substitute ${./packages/Ai.qml} modules/dashboard/Ai.qml --subst-var-by ai ${ai}/bin/vesper-ai
      substitute ${./packages/AgentCockpit.qml} modules/bar/components/AgentCockpit.qml --subst-var-by agentCockpit ${agentCockpit}/bin/vesper-agent-cockpit
      substitute ${./packages/PrivacyHud.qml} modules/bar/components/PrivacyHud.qml --subst-var-by privacyHud ${privacyHud}/bin/vesper-privacy-hud
      substitute ${./packages/HermesBriefing.qml} modules/bar/components/HermesBriefing.qml --subst-var-by ai ${ai}/bin/vesper-ai
      substitute ${./packages/AiPage.qml} modules/nexus/pages/AiPage.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control --subst-var-by ai ${ai}/bin/vesper-ai
      substitute ${./packages/AiCredentials.qml} modules/nexus/pages/AiCredentials.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/AiAppIcons.qml} modules/nexus/pages/AiAppIcons.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/AiRuntimeCredentials.qml} modules/nexus/pages/AiRuntimeCredentials.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/AiProviders.qml} modules/nexus/pages/AiProviders.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/AiSkillsMcp.qml} modules/nexus/pages/AiSkillsMcp.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/AiHermes.qml} modules/nexus/pages/AiHermes.qml --subst-var-by hermesAutomation ${hermesCore}/bin/vesper-hermes-automations
      substitute ${./packages/SystemHealth.qml} modules/nexus/pages/SystemHealth.qml --subst-var-by vesperDoctor ${vesperDoctor}/bin/vesper-doctor
      substitute ${./packages/PrivacyPage.qml} modules/nexus/pages/PrivacyPage.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/BackupRecovery.qml} modules/nexus/pages/BackupRecovery.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperNetworkSettings.qml} modules/nexus/pages/VesperNetworkSettings.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperProxyPage.qml} modules/nexus/pages/VesperProxyPage.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperAppsSettings.qml} modules/nexus/pages/VesperAppsSettings.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      substitute ${./packages/VesperAppControls.qml} modules/nexus/pages/apps/VesperAppControls.qml --subst-var-by vesperControl ${vesperControl}/bin/vesper-control
      ${pkgs.coreutils}/bin/install -Dm644 ${./packages/SystemMonitor.qml} modules/bar/components/SystemMonitor.qml
      ${pkgs.coreutils}/bin/install -Dm644 ${./packages/VesperThemeSettings.qml} modules/nexus/pages/VesperThemeSettings.qml
    '';
  });

  wellbeingRunner = pkgs.writeShellScript "vesper-wellbeing" ''
    export PATH=${lib.makeBinPath [ agenticCaelestia vesperControl ]}:$PATH
    exec ${vesperControl}/bin/vesper-control wellbeing-daemon
  '';
  appIconReconcileRunner = pkgs.writeShellScript "vesper-app-icons-reconcile" ''
    exec ${vesperControl}/bin/vesper-control icons reconcile
  '';
  nixDracula = pkgs.nixos-artwork.wallpapers.dracula;
  nixSolarized = pkgs.nixos-artwork.wallpapers.nineish-solarized-dark;
in
{
  imports = [ inputs.caelestia-shell.homeManagerModules.default ];
  programs.caelestia = {
    enable = true;
    package = agenticCaelestia;
    systemd.enable = false;
    settings = {
      appearance = {
        rounding.scale = 1.25;
        spacing.scale = 1.05;
        padding.scale = 1.05;
        anim.durations.scale = 0.85;
        transparency = { enabled = true; base = 0.68; layers = 0.34; };
      };
      paths.wallpaperDir = "~/Pictures/Wallpapers";
      general = {
        apps = { terminal = [ "ghostty" ]; explorer = [ "thunar" ]; audio = [ "caelestia" "shell" "nexus" "open" ]; };
        idle = {
          lockBeforeSleep = true;
          inhibitWhenAudio = true;
          inhibitWhenCharging = false;
          timeouts = [ { timeout = 300; idleAction = "lock"; } { timeout = 600; idleAction = "dpms off"; returnAction = "dpms on"; } ];
        };
      };
      services = { defaultPlayer = "Spotify"; smartScheme = true; useTwelveHourClock = false; };
      dashboard = {
        enabled = true;
        showPerformance = true;
        resourceUpdateInterval = 1000;
        performance = { showBattery = true; showCpu = true; showGpu = true; showMemory = true; showNetwork = true; showStorage = true; };
      };
      launcher = { vimKeybinds = true; useFuzzy = { apps = true; actions = true; schemes = true; variants = true; wallpapers = true; }; };
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
        enableTerm = true; enableHypr = true; enableDiscord = false; enableSpicetify = false; enablePandora = false;
        enableFuzzel = true; enableBtop = true; enableNvtop = true; enableHtop = false; enableGtk = true; enableQt = true;
        enableWarp = false; enableChromium = false; enableZed = false; enableCava = false;
        iconThemeLight = "Papirus-Light"; iconThemeDark = "Papirus-Dark";
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

  systemd.user.services.vesper-wellbeing = {
    Unit = { Description = "Vesper local application wellbeing tracker"; After = [ "graphical-session.target" ]; PartOf = [ "graphical-session.target" ]; };
    Service = { ExecStart = wellbeingRunner; Restart = "on-failure"; RestartSec = 5; };
    Install.WantedBy = [ "graphical-session.target" ];
  };
  systemd.user.services.vesper-app-icons-reconcile = { Unit.Description = "Reconcile Vesper semantic app icons"; Service = { Type = "oneshot"; ExecStart = appIconReconcileRunner; }; };
  systemd.user.timers.vesper-app-icons-reconcile = {
    Unit.Description = "Periodically discover new or changed app icons";
    Timer = { OnBootSec = "2m"; OnUnitActiveSec = "5m"; RandomizedDelaySec = "20s"; Persistent = true; };
    Install.WantedBy = [ "timers.target" ];
  };

  qt = {
    enable = true;
    platformTheme = { name = "qtengine"; package = pkgs.qtengine; };
    style = { name = "Darkly"; package = pkgs.darkly; };
  };
  home.packages = [ agentCockpit privacyHud ai vesperControl codexbar pkgs.adw-gtk3 pkgs.papirus-icon-theme pkgs.qtengine pkgs.darkly ];
  home.file.".config/vesper/mcp-servers".text = lib.concatStringsSep "\n" mcpServerNames + "\n";
  home.file."Pictures/Wallpapers/vesper-nix-dracula.png".source = nixDracula.gnomeFilePath;
  home.file."Pictures/Wallpapers/vesper-nix-solarized-dark.png".source = nixSolarized.gnomeFilePath;
}
