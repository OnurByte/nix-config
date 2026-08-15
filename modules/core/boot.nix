{ lib, ... }:
{
  boot.loader.systemd-boot.enable = lib.mkDefault true;
  boot.loader.efi.canTouchEfiVariables = lib.mkDefault true;
  boot.tmp.cleanOnBoot = true;

  zramSwap.enable = true;
  services.fstrim.enable = true;
  services.fwupd.enable = true;
}
