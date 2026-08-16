{
  config,
  lib,
  pkgs,
  ...
}:
let
  hasBtrfs = lib.any (fs: (fs.fsType or "") == "btrfs") (lib.attrValues config.fileSystems);
in
{
  # Vesper is expected to live on Btrfs, but the real filesystem topology stays
  # in the installer-generated hardware-configuration.nix. Keep maintenance
  # conditional so the repository still evaluates while that file is a placeholder.
  boot.supportedFilesystems = [ "btrfs" ];

  services.btrfs.autoScrub = lib.mkIf hasBtrfs {
    enable = true;
    interval = "monthly";
  };

  environment.systemPackages = [ pkgs.btrfs-progs ];
}
