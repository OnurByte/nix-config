# Vesper installation and storage

Vesper is a single-machine configuration. Do not make the host generic and do not copy another machine's generated hardware configuration.

## Verified current disk inventory

The following values were captured from the real Vesper machine on 2026-08-16. They are not examples.

```text
/dev/nvme0n1                         931.5 GiB NVMe
├─ /dev/nvme0n1p1                    4 GiB EFI /boot
│  filesystem                        FAT32
│  UUID                              D804-0279
│  PARTUUID                          4f24853c-ccf1-412c-bc2b-156681463187
└─ /dev/nvme0n1p2                    927.5 GiB
   filesystem                        LUKS2
   LUKS UUID                         abb7c069-db97-472e-ba70-38cf58bd9fc4
   PARTUUID                          39ed3951-aa72-461c-9b4f-15631240a7fd
   └─ /dev/mapper/luks-abb7c069-db97-472e-ba70-38cf58bd9fc4
      filesystem                     Btrfs
      UUID                           af2e7549-434c-413b-a077-dceea390b1a1
      root source                    [...] /@ 
      root options                   rw,noatime,compress=zstd:1,ssd,...
```

The same unlocked Btrfs filesystem currently provides `/`, `/home`, `/var/log`, `/var/cache`, `/root`, `/var/tmp` and `/srv`. The supplied inventory only proves that the root subvolume is `@`; it does **not** reveal the exact subvolume names used by the other mount points.

Do not manufacture those names. Before replacing `hosts/vesper/hardware-configuration.nix`, capture the complete topology from the running machine:

```bash
findmnt -R / -o TARGET,SOURCE,FSTYPE,OPTIONS
sudo btrfs subvolume list /
cat /etc/fstab
```

Those three outputs are the source of truth for preserving the current subvolume layout.

## Before installing

The target is the Lenovo IdeaPad Gaming 3 16ARH7 documented in the root README. Verify the actual machine before touching storage:

```bash
lsblk -f
lspci -nn | grep -E 'VGA|3D|Display'
bootctl status
```

The repository intentionally does **not** contain a destructive Disko layout or invented disk identifiers. If a declarative installer is added later, use a stable `/dev/disk/by-id/...` value captured from Vesper itself.

## Storage policy

The existing machine already has the storage foundation Vesper wants: GPT, an EFI system partition, LUKS2 and Btrfs with zstd compression. There is no reason to repartition merely to satisfy the Nix configuration.

Preserve the current LUKS/Btrfs topology unless there is an explicit reason to reinstall from scratch. The NixOS configuration automatically enables monthly Btrfs scrubs once the generated hardware configuration reports a Btrfs filesystem.

If `/home` is a separate Btrfs mount, Vesper gives it its own Snapper timeline. Otherwise home remains inside the root snapshot boundary. Snapper creates short-term recovery points; Restic is the actual backup layer.

## Encryption and Secure Boot

The real Vesper disk is already LUKS2-encrypted. Keep the existing LUKS UUID and partition identity only while preserving that partition. Formatting the partition creates new identifiers, so always regenerate the hardware configuration after destructive disk changes.

Secure Boot is intentionally deferred until the real NixOS install is stable. If Lanzaboote is added later, enroll keys on the real machine and keep private signing material out of Git.

## Hibernate

Vesper currently uses a 27.1 GiB zram swap device and no disk-backed swap was present in the supplied inventory. zram cannot be used as the persistent resume target for hibernation.

Suspend-to-RAM works independently, but hibernation must not be enabled until a real disk-backed swap partition/file and resume parameters are known.

If hibernation is wanted after install:

1. create a disk-backed swap target suitable for Btrfs;
2. determine the real resume device and, for a swapfile, its resume offset;
3. add those values to the host-specific hardware configuration;
4. verify one full hibernate/resume cycle before relying on it.

Never invent resume UUIDs or offsets in this repository.

## Generate the real hardware configuration

After the intended filesystems are mounted for NixOS:

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

Review it against the verified inventory before committing:

```bash
lsblk -f
findmnt -R / -o TARGET,SOURCE,FSTYPE,OPTIONS
sudo btrfs subvolume list /
sudo blkid
```

The generated file should describe the real initrd modules, LUKS mapping, filesystems, Btrfs subvolumes, swap devices and disk UUIDs. Machine policy such as NVIDIA PRIME stays in `hosts/vesper/hardware.nix`.

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
