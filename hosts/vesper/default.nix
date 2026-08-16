{
  hostname,
  ...
}:
{
  imports = [
    ./hardware-configuration.nix
    ./hardware.nix
    ../../modules/core
    ../../modules/desktop
    ../../modules/development
    ../../modules/privacy
  ];

  networking.hostName = hostname;
  time.timeZone = "Europe/Istanbul";

  # Keep the whole desktop/session language in English. Hyprland and Wayland
  # applications inherit these locale categories; keyboard layout is separate.
  i18n = {
    defaultLocale = "en_US.UTF-8";
    extraLocaleSettings = {
      LC_ADDRESS = "en_US.UTF-8";
      LC_IDENTIFICATION = "en_US.UTF-8";
      LC_MEASUREMENT = "en_US.UTF-8";
      LC_MONETARY = "en_US.UTF-8";
      LC_NAME = "en_US.UTF-8";
      LC_NUMERIC = "en_US.UTF-8";
      LC_PAPER = "en_US.UTF-8";
      LC_TELEPHONE = "en_US.UTF-8";
      LC_TIME = "en_US.UTF-8";
      LC_MESSAGES = "en_US.UTF-8";
    };
  };

  # Keep this at the version used for the first installation.
  system.stateVersion = "26.05";
}
