{
  pkgs,
  username,
  ...
}:
let
  backupRunner = pkgs.writeShellApplication {
    name = "vesper-restic-run";
    runtimeInputs = with pkgs; [
      coreutils
      mariadb
      restic
      systemd
    ];
    text = ''
      env_file=/etc/vesper/restic.env
      if [ ! -r "$env_file" ]; then
        echo "Missing $env_file; backup is intentionally not configured yet." >&2
        exit 2
      fi

      set -a
      # shellcheck disable=SC1090
      source "$env_file"
      set +a

      : "''${RESTIC_REPOSITORY:?RESTIC_REPOSITORY must be set in /etc/vesper/restic.env}"
      : "''${RESTIC_PASSWORD_FILE:?RESTIC_PASSWORD_FILE must be set in /etc/vesper/restic.env}"

      # For removable local repositories, set RESTIC_REPOSITORY_CHECK_PATH to
      # the drive mount point. A missing drive becomes a clean skip instead of a
      # noisy failed backup. Remote repositories can simply omit this variable.
      if [ -n "''${RESTIC_REPOSITORY_CHECK_PATH:-}" ] && [ ! -e "$RESTIC_REPOSITORY_CHECK_PATH" ]; then
        echo "Backup target is not mounted: $RESTIC_REPOSITORY_CHECK_PATH; skipping." >&2
        exit 0
      fi

      staging=/var/lib/vesper-backup
      install -d -m 0700 "$staging"
      rm -f "$staging/mariadb.sql"

      # MariaDB is opt-in at runtime. If the local web stack is running, capture
      # a consistent logical dump instead of copying live database files.
      if systemctl is-active --quiet mysql.service; then
        mariadb-dump \
          --all-databases \
          --single-transaction \
          --quick \
          --skip-lock-tables \
          > "$staging/mariadb.sql"
      fi

      paths=()
      for path in \
        "/home/${username}" \
        "$staging"
      do
        if [ -e "$path" ]; then
          paths+=("$path")
        fi
      done

      if [ "''${#paths[@]}" -eq 0 ]; then
        echo "No backup paths exist." >&2
        exit 1
      fi

      restic backup "''${paths[@]}" \
        --exclude "/home/${username}/.cache" \
        --exclude "/home/${username}/Downloads" \
        --exclude "/home/${username}/.local/share/Trash" \
        --exclude "/home/${username}/.local/share/containers" \
        --exclude "/home/${username}/.local/share/Steam" \
        --exclude "/home/${username}/.steam" \
        --exclude "/home/${username}/.cargo/registry" \
        --exclude "/home/${username}/.rustup" \
        --exclude "/home/${username}/.npm" \
        --exclude "/home/${username}/.gradle" \
        --exclude "/home/${username}/.bun/install/cache"

      restic forget --prune \
        --keep-daily 7 \
        --keep-weekly 4 \
        --keep-monthly 12
    '';
  };

  checkRunner = pkgs.writeShellApplication {
    name = "vesper-restic-check";
    runtimeInputs = [ pkgs.restic ];
    text = ''
      env_file=/etc/vesper/restic.env
      if [ ! -r "$env_file" ]; then
        echo "Missing $env_file; backup is intentionally not configured yet." >&2
        exit 2
      fi

      set -a
      # shellcheck disable=SC1090
      source "$env_file"
      set +a

      : "''${RESTIC_REPOSITORY:?RESTIC_REPOSITORY must be set in /etc/vesper/restic.env}"
      : "''${RESTIC_PASSWORD_FILE:?RESTIC_PASSWORD_FILE must be set in /etc/vesper/restic.env}"

      if [ -n "''${RESTIC_REPOSITORY_CHECK_PATH:-}" ] && [ ! -e "$RESTIC_REPOSITORY_CHECK_PATH" ]; then
        echo "Backup target is not mounted: $RESTIC_REPOSITORY_CHECK_PATH; skipping check." >&2
        exit 0
      fi

      exec restic check
    '';
  };
in
{
  environment.systemPackages = with pkgs; [
    restic
    backupRunner
    checkRunner
  ];

  # Secrets stay outside the Nix store. The directory exists declaratively, but
  # restic.env and the password file must be created locally on Vesper.
  systemd.tmpfiles.rules = [
    "d /etc/vesper 0700 root root - -"
    "d /var/lib/vesper-backup 0700 root root - -"
  ];

  systemd.services.vesper-backup = {
    description = "Vesper Restic backup";
    unitConfig.ConditionPathExists = "/etc/vesper/restic.env";
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${backupRunner}/bin/vesper-restic-run";
      Nice = 10;
      IOSchedulingClass = "idle";
    };
  };

  systemd.timers.vesper-backup = {
    description = "Daily Vesper Restic backup";
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "daily";
      Persistent = true;
      RandomizedDelaySec = "30m";
    };
  };

  systemd.services.vesper-backup-check = {
    description = "Verify the Vesper Restic repository";
    unitConfig.ConditionPathExists = "/etc/vesper/restic.env";
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${checkRunner}/bin/vesper-restic-check";
      Nice = 10;
      IOSchedulingClass = "idle";
    };
  };

  systemd.timers.vesper-backup-check = {
    description = "Monthly Vesper Restic repository verification";
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = "monthly";
      Persistent = true;
      RandomizedDelaySec = "2h";
    };
  };
}
