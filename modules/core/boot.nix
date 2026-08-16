{ lib, ... }:
{
  boot.loader.systemd-boot.enable = lib.mkDefault true;
  boot.loader.efi.canTouchEfiVariables = lib.mkDefault true;
  boot.tmp.cleanOnBoot = true;

  zramSwap = {
    enable = true;
    # The verified live machine exposes a ~27.1 GiB zram device, matching
    # approximately 100% of installed RAM rather than NixOS's 50% default.
    memoryPercent = 100;
  };

  services.fstrim.enable = true;
  services.fwupd.enable = true;
}
