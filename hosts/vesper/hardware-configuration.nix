{
  lib,
  ...
}:
{
  # Verified from the running Vesper machine on 2026-08-16.
  # This file preserves the existing disk and Btrfs topology; it does not
  # repartition, format or otherwise mutate the disk.

  # Root and EFI live on NVMe, so ensure the NVMe driver is present in initrd.
  boot.initrd.availableKernelModules = [ "nvme" ];

  # Existing LUKS2 partition:
  #   /dev/nvme0n1p2
  #   UUID abb7c069-db97-472e-ba70-38cf58bd9fc4
  boot.initrd.luks.devices."luks-abb7c069-db97-472e-ba70-38cf58bd9fc4" = {
    device = "/dev/disk/by-uuid/abb7c069-db97-472e-ba70-38cf58bd9fc4";
  };

  # All Btrfs mounts below are subvolumes of the same unlocked filesystem:
  #   UUID af2e7549-434c-413b-a077-dceea390b1a1
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/af2e7549-434c-413b-a077-dceea390b1a1";
    fsType = "btrfs";
    options = [
      "subvol=@"
      "compress=zstd:1"
      "noatime"
    ];
  };

  fileSystems."/home" = {
    device = "/dev/disk/by-uuid/af2e7549-434c-413b-a077-dceea390b1a1";
    fsType = "btrfs";
    options = [
      "subvol=@home"
      "compress=zstd:1"
      "noatime"
    ];
  };

  fileSystems."/root" = {
    device = "/dev/disk/by-uuid/af2e7549-434c-413b-a077-dceea390b1a1";
    fsType = "btrfs";
    options = [
      "subvol=@root"
      "compress=zstd:1"
      "noatime"
    ];
  };

  fileSystems."/srv" = {
    device = "/dev/disk/by-uuid/af2e7549-434c-413b-a077-dceea390b1a1";
    fsType = "btrfs";
    options = [
      "subvol=@srv"
      "compress=zstd:1"
      "noatime"
    ];
  };

  fileSystems."/var/cache" = {
    device = "/dev/disk/by-uuid/af2e7549-434c-413b-a077-dceea390b1a1";
    fsType = "btrfs";
    options = [
      "subvol=@cache"
      "compress=zstd:1"
      "noatime"
    ];
  };

  fileSystems."/var/tmp" = {
    device = "/dev/disk/by-uuid/af2e7549-434c-413b-a077-dceea390b1a1";
    fsType = "btrfs";
    options = [
      "subvol=@tmp"
      "compress=zstd:1"
      "noatime"
    ];
  };

  fileSystems."/var/log" = {
    device = "/dev/disk/by-uuid/af2e7549-434c-413b-a077-dceea390b1a1";
    fsType = "btrfs";
    options = [
      "subvol=@log"
      "compress=zstd:1"
      "noatime"
    ];
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/D804-0279";
    fsType = "vfat";
    options = [ "umask=0077" ];
  };

  fileSystems."/tmp" = {
    device = "tmpfs";
    fsType = "tmpfs";
    options = [
      "noatime"
      "mode=1777"
    ];
  };

  # The live system has no disk-backed swap. zram is configured separately in
  # modules/core/boot.nix, so hibernation remains intentionally disabled.
  swapDevices = [ ];

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
}
