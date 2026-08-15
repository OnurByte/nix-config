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

  agenticCaelestia = inputs.caelestia-shell.packages.${pkgs.system}.with-cli.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [ ./packages/caelestia-codexbar.patch ];

    postPatch = (old.postPatch or "") + ''
      substitute ${./packages/CodexUsage.qml} modules/bar/components/CodexUsage.qml \
        --subst-var-by codexbarStatus ${codexbarUi}/bin/codexbar-status \
        --subst-var-by codexbarPopup ${codexbarUi}/bin/codexbar-popup
    '';
  });
in
{
  imports = [
    inputs.caelestia-shell.homeManagerModules.default
  ];

  programs.caelestia = {
    enable = true;
    package = agenticCaelestia;

    # Hyprland starts the shell directly so all compositor-bound services come
    # up in one predictable session path.
    systemd.enable = false;

    settings = {
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

        # Caelestia owns idle/lock state. This preserves the old Kraken timing
        # while removing the parallel hypridle + hyprlock stack.
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

      # Keep Kraken's control center focused on daily-driver controls. Caelestia
      # ships gameMode enabled in its default quick-toggle list; explicitly
      # hide that and VPN until either workflow is requested.
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

      # The stock Caelestia list accepts arbitrary entry IDs; the patched shell
      # gives aiUsage a native QML delegate backed by CodexBar.
      bar.entries = [
        { id = "logo"; enabled = true; }
        { id = "workspaces"; enabled = true; }
        { id = "spacer"; enabled = true; }
        { id = "activeWindow"; enabled = true; }
        { id = "spacer"; enabled = true; }
        { id = "tray"; enabled = true; }
        { id = "aiUsage"; enabled = true; }
        { id = "clock"; enabled = true; }
        { id = "statusIcons"; enabled = true; }
        { id = "power"; enabled = true; }
      ];
    };

    cli = {
      enable = true;
      settings.theme = {
        # Whitelist only surfaces Kraken actually uses. Caelestia treats an
        # omitted enable* key as true, so explicit false values prevent theme
        # hooks from touching unrelated applications.
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

        # enableHypr writes scheme/current.lua; reload so the modular Lua
        # compositor config immediately picks up the wallpaper palette.
        postHook = "hyprctl reload";
      };
    };
  };

  # Caelestia's GTK theme hook selects these names through dconf. Install the
  # actual theme/icon assets declaratively instead of leaving dangling names.
  home.packages = [
    codexbarUi
    pkgs.adw-gtk3
    pkgs.papirus-icon-theme
  ];

  xdg.dataFile."codexbar-waybar/icons".source = "${codexbarUi}/share/codexbar-waybar/icons";
}
