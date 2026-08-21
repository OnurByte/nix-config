{
  config,
  inputs,
  lib,
  pkgs,
  ...
}:
let
  sessionDesktop = pkgs.callPackage ./packages/session-desktop.nix { };
  vesperStartpage = inputs.self.packages.${pkgs.system}.vesper-startpage;
  heliumPackage = inputs.helium.packages.${pkgs.system}.default;
  torBrowserPackage = pkgs.tor-browser;
  heliumStartpage = pkgs.writeShellApplication {
    name = "helium";
    text = ''
      if [ "$#" -eq 0 ]; then
        exec ${heliumPackage}/bin/helium "http://127.0.0.1:3210/"
      fi
      exec ${heliumPackage}/bin/helium "$@"
    '';
  };
  torBrowserStartpage = pkgs.writeShellApplication {
    name = "tor-browser";
    text = ''
      if [ "$#" -eq 0 ]; then
        exec ${torBrowserPackage}/bin/tor-browser "http://127.0.0.1:3210/"
      fi
      exec ${torBrowserPackage}/bin/tor-browser "$@"
    '';
  };
  startpageArgs = [
    "--web-root"
    "${vesperStartpage}/share/vesper-startpage"
    "--helium-history"
    "${config.home.homeDirectory}/.config/net.imput.helium/Default/History"
    "--helium-preferences"
    "${config.home.homeDirectory}/.config/net.imput.helium/Default/Preferences"
    "--zen-history"
    "${config.xdg.configHome}/zen/default/places.sqlite"
    "--zen-history"
    "${config.xdg.configHome}/zen/default-release/places.sqlite"
    "--zen-history"
    "${config.home.homeDirectory}/.var/app/app.zen_browser.zen/.zen/wax43dc1.Default (release)/places.sqlite"
    "--briefings-index"
    "${config.home.homeDirectory}/.local/share/vesper/briefings/index.json"
    "--source-registry"
    "${config.home.homeDirectory}/.local/state/vesper/research/unknown-frontier-ai/source-registry.json"
    "--tor-browser"
    (lib.getExe torBrowserPackage)
  ];
in
{
  imports = [ inputs.zen-browser.homeModules.beta ];

  programs.zen-browser = {
    enable = true;
    setAsDefaultBrowser = true;
    profiles.default.name = "default";
    profiles.default.isDefault = true;
    policies.Homepage = {
      URL = "http://127.0.0.1:3210/";
      Locked = true;
      StartPage = "homepage";
    };
  };

  xdg.desktopEntries.helium = {
    name = "Helium";
    genericName = "Web Browser";
    comment = "Helium with the Vesper local startpage";
    exec = "${heliumStartpage}/bin/helium %U";
    icon = "web-browser";
    terminal = false;
    categories = [ "Network" "WebBrowser" ];
    mimeType = [ "text/html" "application/xhtml+xml" "x-scheme-handler/http" "x-scheme-handler/https" ];
  };

  xdg.desktopEntries.tor-browser = {
    name = "Tor Browser";
    genericName = "Privacy Web Browser";
    comment = "Tor Browser with the Vesper local startpage";
    exec = "${torBrowserStartpage}/bin/tor-browser %U";
    icon = "${torBrowserPackage}/share/icons/hicolor/128x128/apps/tor-browser.png";
    terminal = false;
    categories = [ "Network" "WebBrowser" "Security" ];
    mimeType = [
      "text/html"
      "text/xml"
      "application/xhtml+xml"
      "application/vnd.mozilla.xul+xml"
      "x-scheme-handler/http"
      "x-scheme-handler/https"
    ];
  };

  programs.vicinae = {
    enable = true;
    systemd = {
      enable = true;
      autoStart = true;
    };
    settings = {
      imports = [ "vesper.json" ];
    };
  };

  programs.vesktop = {
    enable = true;
    vencord.useSystem = true;

    settings = {
      appBadge = true;
      arRPC = true;
      checkUpdates = false;
      customTitleBar = false;
      disableMinSize = true;
      hardwareAcceleration = true;
      minimizeToTray = true;
      tray = true;
      discordBranch = "stable";
    };

    vencord.settings = {
      autoUpdate = false;
      autoUpdateNotification = false;
      notifyAboutUpdates = false;
    };
  };

  home.packages = with pkgs; [
    heliumStartpage
    vesperStartpage
    torBrowserStartpage

    inputs.chatgpt-desktop.packages.${pkgs.system}.default
    inputs.claude-desktop.packages.${pkgs.system}.claude-desktop-with-fhs

    sessionDesktop
    telegram-desktop

    ghostty
    thunar
    tumbler
    file-roller
    imv

    bottles

    obsidian
    inputs.self.packages.${pkgs.system}.t3code-nightly
    inputs.self.packages.${pkgs.system}.vesper-store
  ];

  # Keep Store-owned binaries and desktop entries visible after a transaction
  # without replacing the system's existing PATH or XDG data directories.
  home.sessionPath = [ "$HOME/.local/state/vesper/store/profile/bin" ];
  home.sessionSearchVariables.XDG_DATA_DIRS = [
    "$HOME/.local/state/vesper/store/profile/share"
  ];

  systemd.user.sockets.vesper-startpage = {
    Unit = {
      Description = "Vesper local browser startpage socket";
      PartOf = [ "graphical-session.target" ];
    };
    Socket = {
      ListenStream = "127.0.0.1:3210";
      Accept = false;
      Service = "vesper-startpage.service";
    };
    Install.WantedBy = [ "graphical-session.target" ];
  };

  systemd.user.services.vesper-startpage = {
    Unit = {
      Description = "Vesper local browser startpage and research surface";
      PartOf = [ "graphical-session.target" ];
      After = [ "vesper-startpage.socket" ];
    };
    Service = {
      ExecStart = lib.concatStringsSep " " (
        [ "${vesperStartpage}/bin/vesper-startpage" ]
        ++ lib.map lib.escapeShellArg startpageArgs
      );
      Restart = "on-failure";
      RestartSec = 2;
    };
  };

  xdg.mimeApps = {
    enable = true;
    defaultApplications = {
      "text/html" = [ "zen-beta.desktop" ];
      "application/xhtml+xml" = [ "zen-beta.desktop" ];
      "x-scheme-handler/http" = [ "zen-beta.desktop" ];
      "x-scheme-handler/https" = [ "zen-beta.desktop" ];

      "audio/mpeg" = [ "mpv.desktop" ];
      "audio/flac" = [ "mpv.desktop" ];
      "audio/ogg" = [ "mpv.desktop" ];
      "audio/x-wav" = [ "mpv.desktop" ];
      "video/mp4" = [ "mpv.desktop" ];
      "video/webm" = [ "mpv.desktop" ];
      "video/x-matroska" = [ "mpv.desktop" ];
    };
  };
}
