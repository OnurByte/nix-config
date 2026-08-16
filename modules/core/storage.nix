{
  config,
  lib,
  pkgs,
  username,
  ...
}:
let
  rootFs = config.fileSystems."/" or null;
  homeFs = config.fileSystems."/home" or null;

  rootIsBtrfs = rootFs != null && (rootFs.fsType or "") == "btrfs";
  homeIsBtrfs = homeFs != null && (homeFs.fsType or "") == "btrfs";
  hasBtrfs = lib.any (fs: (fs.fsType or "") == "btrfs") (lib.attrValues config.fileSystems);

  timeline = {
    ALLOW_USERS = [ username ];
    TIMELINE_CREATE = true;
    TIMELINE_CLEANUP = true;
    TIMELINE_LIMIT_HOURLY = 6;
    TIMELINE_LIMIT_DAILY = 7;
    TIMELINE_LIMIT_WEEKLY = 4;
    TIMELINE_LIMIT_MONTHLY = 6;
    TIMELINE_LIMIT_YEARLY = 0;
  };
in
{
  # Vesper is expected to live on Btrfs, but the real filesystem topology stays
  # in the installer-generated hardware-configuration.nix. Everything below is
  # conditional so the repository keeps evaluating while that file is a placeholder.
  boot.supportedFilesystems = [ "btrfs" ];

  services.btrfs.autoScrub = lib.mkIf hasBtrfs {
    enable = true;
    interval = "monthly";
  };

  # Snapper only activates once the generated hardware config proves that / is
  # actually Btrfs. A separately mounted Btrfs /home gets its own snapshot set;
  # otherwise home remains part of the root snapshot boundary.
  services.snapper = lib.mkIf rootIsBtrfs {
    persistentTimer = true;
    snapshotInterval = "hourly";
    cleanupInterval = "1d";

    configs = {
      root = timeline // {
        SUBVOLUME = "/";
      };
    }
    // lib.optionalAttrs homeIsBtrfs {
      home = timeline // {
        SUBVOLUME = "/home";
      };
    };
  };

  # systemd-tmpfiles type `v` creates a Btrfs subvolume when possible. Snapper
  # requires .snapshots to be a subvolume rather than an ordinary directory.
  systemd.tmpfiles.rules =
    lib.optionals rootIsBtrfs [
      "v /.snapshots 0750 root root - -"
    ]
    ++ lib.optionals homeIsBtrfs [
      "v /home/.snapshots 0750 root root - -"
    ];

  environment.systemPackages = with pkgs; [
    btrfs-progs
    snapper
  ];
}
