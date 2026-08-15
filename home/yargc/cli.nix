{ pkgs, ... }:
{
  programs = {
    bat.enable = true;
    btop = {
      enable = true;
      settings.color_theme = "caelestia";
    };
    eza.enable = true;
    fastfetch.enable = true;
    fzf.enable = true;
    zoxide.enable = true;

    direnv = {
      enable = true;
      nix-direnv.enable = true;
    };
  };

  home.packages = with pkgs; [
    ripgrep
    fd
    jq
    curl
    wget
    aria2
    unzip
    zip
    p7zip
    file
    tree
    gh
    gnupg
    age
    sops
    openssl
    just

    # Hardware / workstation diagnostics.
    pciutils
    usbutils
    lm_sensors
    smartmontools
    nvtopPackages.full
  ];
}
