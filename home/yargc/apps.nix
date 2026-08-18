{
  inputs,
  pkgs,
  ...
}:
let
  sessionDesktop = pkgs.callPackage ./packages/session-desktop.nix { };
in
{
  programs.vicinae = {
    enable = true;
    systemd = {
      enable = true;
      autoStart = true;
    };
    settings = {
      close_on_focus_loss = true;
      pop_to_root_on_close = true;
      launcher_window.layer_shell.enabled = true;
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
    inputs.zen-browser.packages.${pkgs.system}.default
    inputs.helium.packages.${pkgs.system}.default
    tor-browser

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
