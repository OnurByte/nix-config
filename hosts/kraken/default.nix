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
  i18n.defaultLocale = "en_US.UTF-8";

  # Keep this at the version used for the first installation.
  system.stateVersion = "26.05";
}
