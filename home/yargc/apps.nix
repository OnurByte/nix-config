{
  inputs,
  pkgs,
  ...
}:
{
  home.packages = with pkgs; [
    inputs.helium.packages.${pkgs.system}.default
    firefox
    tor-browser
    ghostty
    thunar
    tumbler
    file-roller
    mpv
    imv
    vesktop
  ];

  xdg.mimeApps = {
    enable = true;
    defaultApplications = {
      "text/html" = [ "helium.desktop" ];
      "application/xhtml+xml" = [ "helium.desktop" ];
      "x-scheme-handler/http" = [ "helium.desktop" ];
      "x-scheme-handler/https" = [ "helium.desktop" ];
    };
  };
}
