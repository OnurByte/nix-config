# Vesper installation and storage

Vesper is a single-machine configuration. Do not make the host generic and do not copy another machine's generated hardware configuration.

## Before installing

The target is the Lenovo IdeaPad Gaming 3 16ARH7 documented in the root README. Verify the actual machine before touching storage:

```bash
lsblk -f
lspci -nn | grep -E 'VGA|3D|Display'
bootctl status
```

The repository intentionally does **not** contain a destructive Disko layout or invented disk identifiers. If a declarative installer is added later, use a stable `/dev/disk/by-id/...` value captured from Vesper itself.

## Recommended disk shape

For a fresh install, the preferred design is:

```text
GPT
├── EFI System Partition -> /boot
└── LUKS2
    └── Btrfs
        ├── @root -> /
        ├── @home -> /home
        └── @nix  -> /nix
```

Use Btrfs compression (`compress=zstd`) on the normal subvolumes. The config automatically enables monthly Btrfs scrubs once the generated hardware configuration reports a Btrfs filesystem.

If `/home` is a separate Btrfs mount, Vesper gives it its own Snapper timeline. Otherwise home remains inside the root snapshot boundary. Snapper creates short-term recovery points; Restic is the actual backup layer.

## Encryption and Secure Boot

Prefer LUKS2 for the main data partition on this laptop. Disk encryption should be decided at install time; do not fake LUKS declarations after the machine is already installed.

Secure Boot is intentionally deferred until the real NixOS install is stable. If Lanzaboote is added later, enroll keys on the real machine and keep private signing material out of Git.

## Hibernate

Vesper currently uses zram and does not declare disk-backed swap for hibernation. Suspend-to-RAM works independently, but hibernation must not be enabled until a real swap partition/file and resume parameters are known.

If hibernation is wanted after install:

1. create a disk-backed swap target suitable for Btrfs;
2. determine the real resume device and, for a swapfile, its resume offset;
3. add those values to the host-specific hardware configuration;
4. verify one full hibernate/resume cycle before relying on it.

Never invent resume UUIDs or offsets in this repository.

## Generate the real hardware configuration

After the installer has mounted the intended filesystems:

```bash
sudo nixos-generate-config --root /mnt
```

After the installed system boots, replace the placeholder with the generated file:

```bash
sudo cp /etc/nixos/hardware-configuration.nix \
  ~/nix-config/hosts/vesper/hardware-configuration.nix
sudo chown "$USER:$(id -gn)" \
  ~/nix-config/hosts/vesper/hardware-configuration.nix
```

Review it before committing:

```bash
lsblk -f
findmnt /
findmnt /home || true
findmnt /nix || true
```

The generated file should describe the real initrd modules, filesystems, swap devices and disk UUIDs. Machine policy such as NVIDIA PRIME stays in `hosts/vesper/hardware.nix`.

## First validation

From the repository:

```bash
nh os test
vesper-doctor
```

Check that:

- the root filesystem is Btrfs;
- monthly Btrfs scrub timers exist;
- Snapper timers exist after Btrfs becomes active;
- `amd_pstate` reports `active`;
- the RTX 3050 is visible and `nvidia-offload` exists;
- the internal panel is actually near 165 Hz;
- the local web stack is stopped until requested;
- no unexpected systemd units are failed.

Only after that:

```bash
nh os switch
```

Once the placeholder hardware file is gone, GitHub Actions also stops skipping the full `system.build.toplevel` build.
