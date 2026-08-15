{
  inputs,
  pkgs,
  ...
}:
{
  home.packages = with pkgs; [
    # Browsers: Zen is the daily default, Helium is the Chromium-side companion.
    inputs.zen-browser.packages.${pkgs.system}.default
    inputs.helium.packages.${pkgs.system}.default
    tor-browser

    # Desktop / files / media.
    ghostty
    thunar
    tumbler
    file-roller
    mpv
    imv

    # Communication / knowledge / editor-adjacent apps.
    session-desktop
    telegram-desktop
    obsidian
    t3code
  ];

  # ChatGPT has no official Linux desktop build. Claude Desktop now has an
  # official Linux beta, but upstream supports Ubuntu/Debian packages rather
  # than NixOS. Until there is a first-party/Nixpkgs package, keep both as
  # isolated app-mode Helium windows instead of adding another packaging trust
  # dependency to the base system.
  xdg.desktopEntries = {
    chatgpt = {
      name = "ChatGPT";
      genericName = "AI Assistant";
      exec = "helium --app=https://chatgpt.com";
      icon = "helium";
      terminal = false;
      categories = [ "Network" "Utility" ];
    };

    claude = {
      name = "Claude";
      genericName = "AI Assistant";
      exec = "helium --app=https://claude.ai";
      icon = "helium";
      terminal = false;
      categories = [ "Network" "Utility" ];
    };
  };

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
