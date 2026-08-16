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
  # Vesper's real root is Btrfs. Keep these checks conditional anyway so this
  # module remains honest if the storage topology is deliberately changed later.
  boot.supportedFilesystems = [ "btrfs" ];

  services.btrfs.autoScrub = lib.mkIf hasBtrfs {
    enable = true;
    interval = "monthly";
  };

  # The verified machine mounts / and /home as distinct Btrfs subvolumes.
  # Root keeps the existing /.snapshots subvolume/history; Home gets its own
  # Snapper namespace without changing the @home mount itself.
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

  # systemd-tmpfiles type `v` creates a Btrfs subvolume only when the path does
  # not already exist. Existing root Snapper history is therefore preserved.
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
