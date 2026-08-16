{
  inputs,
  pkgs,
  ...
}:
let
  agenticT3Code = pkgs.t3code.override {
    enableClaude = true;
    enableCodex = true;
    enableOpencode = true;
  };
in
{
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

    session-desktop
    telegram-desktop

    ghostty
    thunar
    tumbler
    file-roller
    mpv
    imv

    bottles

    obsidian
    agenticT3Code
  ];

  xdg.mimeApps = {
    enable = true;
    defaultApplications = {
      "text/html" = [ "zen-beta.desktop" ];
      "application/xhtml+xml" = [ "zen-beta.desktop" ];
      "x-scheme-handler/http" = [ "zen-beta.desktop" ];
      "x-scheme-handler/https" = [ "zen-beta.desktop" ];
    };
  };
}
