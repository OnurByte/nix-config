# Vesper backups

Status: **current**

Vesper uses Restic for real backups and Snapper for short-term local recovery. They solve different problems: Snapper helps with local mistakes; Restic survives loss of the machine when its repository is elsewhere.

## what the service backs up

`vesper-backup.service` backs up the user home plus a staging directory for consistent MariaDB dumps. Obvious caches, downloads, container storage and package-manager caches are excluded.

If MariaDB is running, the service creates an `--all-databases` logical dump first. It does not copy live MariaDB data files.

Retention is:

```text
7 daily
4 weekly
12 monthly
```

The repository is checked monthly with `restic check`.

## keep backup credentials out of Nix

The configuration deliberately does not put repository credentials or passwords in the Nix store. Create them locally on Vesper.

Create a password file:

```bash
sudo install -d -m 0700 /root/.config/restic
sudo sh -c 'umask 077; openssl rand -base64 48 > /root/.config/restic/vesper-pass'
```

Copy that password somewhere safe **off the laptop** before relying on the repository. A Restic repository without its password is intentionally unrecoverable.

Create `/etc/vesper/restic.env`:

```bash
sudo install -d -m 0700 /etc/vesper
sudoedit /etc/vesper/restic.env
```

For a remote repository, the minimal file is:

```bash
RESTIC_REPOSITORY=sftp:backup-host:/backups/vesper
RESTIC_PASSWORD_FILE=/root/.config/restic/vesper-pass
```

For a removable local disk, add a mount guard so scheduled runs cleanly skip when the drive is absent:

```bash
RESTIC_REPOSITORY=/run/media/yargc/BACKUP/vesper-restic
RESTIC_PASSWORD_FILE=/root/.config/restic/vesper-pass
RESTIC_REPOSITORY_CHECK_PATH=/run/media/yargc/BACKUP
```

Then lock the file down:

```bash
sudo chmod 0600 /etc/vesper/restic.env
sudo chown root:root /etc/vesper/restic.env
```

## initialize once

After the destination exists:

```bash
sudo bash -c 'set -a; source /etc/vesper/restic.env; set +a; restic init'
```

Run the first backup manually:

```bash
backup
```

Inspect it:

```bash
backup-status
sudo bash -c 'set -a; source /etc/vesper/restic.env; set +a; restic snapshots'
```

Run a repository check:

```bash
backup-check
```

The daily and monthly timers are already declarative:

```bash
systemctl list-timers 'vesper-backup*'
```

## restore test

A backup is not trusted until a restore has been tested. Periodically restore a small directory into a scratch path:

```bash
sudo mkdir -p /tmp/vesper-restore-test
sudo bash -c '
  set -a
  source /etc/vesper/restic.env
  set +a
  restic restore latest --target /tmp/vesper-restore-test --include /home/yargc/Documents
'
```

Confirm a few real files open correctly, then delete the scratch restore.

## secrets

The Restic repository is encrypted, but a repository containing a full home directory can still contain SSH keys, browser profiles, app credentials and other sensitive state. Protect the Restic password as seriously as the laptop itself.

Vesper already has `sops-nix` for declarative Home Manager/user-service secrets, but Restic intentionally does not consume those secrets today.
The backup service is system-owned and its credentials remain in root-owned machine-local files:

```text
/etc/vesper/restic.env
/root/.config/restic/vesper-pass
```

Do not move these credentials into sops-nix merely for consistency. Change the secret mechanism only if the backup service ownership and consumption path are intentionally redesigned together.

See `SECRETS.md` for the repository-wide secret ownership split.
