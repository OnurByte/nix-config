{
  inputs,
  pkgs,
  ...
}:
let
  # Nixpkgs enables Codex in T3 Code by default, but leaves Claude/OpenCode
  # disabled. Kraken treats all three as first-class backends in the same UI.
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

    # Updates belong to Nix, while tray behaviour should feel like a normal
    # desktop app under Caelestia.
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
    # Browsers: Zen is the daily default, Helium is the Chromium-side companion.
    inputs.zen-browser.packages.${pkgs.system}.default
    inputs.helium.packages.${pkgs.system}.default
    tor-browser

    # Native AI desktop apps. Keep the rich product surfaces.
    inputs.chatgpt-desktop.packages.${pkgs.system}.default
    inputs.claude-desktop.packages.${pkgs.system}.claude-desktop-with-fhs

    # Communication.
    session-desktop
    telegram-desktop

    # Desktop / files / media.
    ghostty
    thunar
    tumbler
    file-roller
    mpv
    imv

    # Opt-in Windows compatibility without making gaming part of the base rice.
    bottles

    # Knowledge / coding surfaces.
    obsidian
    agenticT3Code
    inputs.self.packages.${pkgs.system}.zcode
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
