# Vesper installation and storage

Status: **current**

Vesper is a single-machine configuration. Do not make the host generic and do not copy another machine's generated hardware configuration.

## repository checkout contract

The GitHub repository is named `OnurByte/vesper`, but the current Vesper configuration still treats this local path as canonical:

```text
/home/yargc/nix-config
```

`programs.nh.flake`, shell aliases and command-memory helpers currently depend on that path. Until those consumers are migrated together, clone the repository explicitly into `~/nix-config` rather than relying on Git's default `~/vesper` directory name:

```bash
git clone https://github.com/OnurByte/vesper.git ~/nix-config
cd ~/nix-config
```

Do not partially rename the checkout path. A future migration to `~/vesper` must update every hard-coded consumer in one coherent change and then update this section.

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
```

The live mount topology is also verified:

```text
@       -> /
@home   -> /home
@root   -> /root
@srv    -> /srv
@cache  -> /var/cache
@tmp    -> /var/tmp
@log    -> /var/log
```

All Btrfs mounts use `noatime` and `compress=zstd:1`. `/tmp` is tmpfs. The machine has no disk-backed swap; zram is the only swap layer currently present.

The root subvolume already contains `/.snapshots` as a Btrfs subvolume with existing Snapper history. The NixOS recovery module preserves that path rather than recreating or deleting it.

## Declarative hardware file

`hosts/vesper/hardware-configuration.nix` now describes the verified storage layout directly. It is intentionally limited to facts needed to boot and mount the current machine:

- NVMe is included in initrd;
- the existing LUKS2 partition is opened by UUID;
- the existing Btrfs filesystem is mounted by UUID with the verified subvolume names;
- the EFI partition is mounted at `/boot`;
- `/tmp` remains tmpfs;
- there is no disk-backed swap declaration.

This file is **not** a Disko installer. Evaluating or switching the NixOS configuration does not repartition or format the disk.

## Storage policy

The existing machine already has the storage foundation Vesper wants: GPT, an EFI system partition, LUKS2 and Btrfs with zstd compression. There is no reason to repartition merely to satisfy the Nix configuration.

Preserve this topology unless there is an explicit reason to reinstall from scratch. If the disk is ever reformatted, recapture all UUIDs and subvolume names before modifying `hardware-configuration.nix`.

Vesper enables monthly Btrfs scrub. Snapper handles short-term local recovery for root and Home; Restic is the actual backup layer.

## Encryption and Secure Boot

The real Vesper disk is already LUKS2-encrypted. The current NixOS config opens:

```text
UUID=abb7c069-db97-472e-ba70-38cf58bd9fc4
```

No discard option is forced because the current capture did not establish that policy. Change LUKS performance/security options only after checking the live cryptsetup configuration deliberately.

Secure Boot is intentionally deferred until the NixOS install itself is stable. If Lanzaboote is added later, enroll keys on the real machine and keep private signing material out of Git.

## Hibernate

Vesper currently uses zram and has no disk-backed swap. zram cannot be used as the persistent resume target for hibernation.

Suspend-to-RAM works independently, but hibernation must not be enabled until a real disk-backed swap partition/file and resume parameters are known.

If hibernation is wanted later:

1. create a disk-backed swap target suitable for Btrfs;
2. determine the real resume device and, for a swapfile, its resume offset;
3. add those values to the host-specific configuration;
4. verify one full hibernate/resume cycle before relying on it.

Never invent resume UUIDs or offsets in this repository.

## Installing NixOS without changing the disk layout

The existing filesystem can be mounted for installation instead of being recreated. The important rule is to mount the verified subvolumes at the correct paths and let the installer write NixOS into them only when that is actually intended.

Before an install or migration, verify the machine again:

```bash
lsblk -o NAME,PATH,SIZE,TYPE,FSTYPE,FSVER,LABEL,UUID,PARTUUID,MOUNTPOINTS,MODEL
sudo blkid
findmnt -R / -o TARGET,SOURCE,FSTYPE,OPTIONS
sudo btrfs subvolume list /
```

If the topology differs from the values above, stop and update the repository before switching the NixOS configuration.

## First NixOS validation

From the repository:

```bash
nh os test
vesper-doctor
```

Check that:

- the LUKS prompt appears and the existing encrypted root opens;
- `/`, `/home`, `/root`, `/srv`, `/var/cache`, `/var/tmp` and `/var/log` resolve to the intended Btrfs subvolumes;
- `/boot` is the FAT32 EFI partition;
- the root filesystem is Btrfs with zstd compression and noatime;
- monthly Btrfs scrub timers exist;
- Snapper timers exist and existing root snapshots remain visible;
- `amd_pstate` reports `active`;
- the RTX 3050 is visible and `nvidia-offload` exists;
- the internal panel is actually near 165 Hz;
- the local web stack is stopped until requested;
- no unexpected systemd units are failed.

Only after that:

```bash
nh os switch
```

GitHub Actions now builds `.#nixosConfigurations.vesper.config.system.build.toplevel` because the hardware file is no longer a placeholder.
