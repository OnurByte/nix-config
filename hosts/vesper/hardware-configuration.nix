{ ... }:
{
  # IMPORTANT:
  # This remains an intentional placeholder until NixOS generates the complete
  # hardware configuration or every current Btrfs subvolume mount is captured.
  #
  # Verified on the real Vesper machine (2026-08-16):
  #   /dev/nvme0n1p1  FAT32 /boot
  #     UUID:     D804-0279
  #     PARTUUID: 4f24853c-ccf1-412c-bc2b-156681463187
  #   /dev/nvme0n1p2  LUKS2
  #     UUID:     abb7c069-db97-472e-ba70-38cf58bd9fc4
  #     PARTUUID: 39ed3951-aa72-461c-9b4f-15631240a7fd
  #   unlocked Btrfs
  #     UUID:     af2e7549-434c-413b-a077-dceea390b1a1
  #     root:     subvol=@, noatime, compress=zstd:1
  #
  # The exact subvolume names currently mounted at /home, /var/log, /var/cache,
  # /root, /var/tmp and /srv have not been captured yet. Never infer them from
  # distro defaults. See docs/INSTALL.md for the commands that complete the map.
  #
  # Once NixOS has generated the real file:
  #   sudo cp /etc/nixos/hardware-configuration.nix \
  #     ~/nix-config/hosts/vesper/hardware-configuration.nix
  #
  # Do not copy another machine's disk UUIDs, filesystems, GPU or kernel modules.
}
