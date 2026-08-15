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
  home.packages = with pkgs; [
    # Browsers: Zen is the daily default, Helium is the Chromium-side companion.
    inputs.zen-browser.packages.${pkgs.system}.default
    inputs.helium.packages.${pkgs.system}.default
    tor-browser

    # Native AI desktop apps. Both flakes package the vendors' official Linux binaries.
    inputs.chatgpt-desktop.packages.${pkgs.system}.default
    inputs.claude-desktop.packages.${pkgs.system}.claude-desktop-with-fhs

    # Desktop / files / media.
    ghostty
    thunar
    tumbler
    file-roller
    mpv
    imv

    # Communication / knowledge / agent surfaces.
    session-desktop
    telegram-desktop
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
