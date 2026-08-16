<div align="center">

# VESPER

### NixOS config for my daily workstation

**NixOS · Hyprland · Caelestia · Ghostty · PychoVIM · Codex · Claude · OpenCode · Grok Build · Hermes**

![NixOS](https://img.shields.io/badge/NixOS-unstable-5277C3?style=flat-square&logo=nixos&logoColor=white)
![Hyprland](https://img.shields.io/badge/Hyprland-Wayland-58E1FF?style=flat-square)
![Caelestia](https://img.shields.io/badge/Caelestia-Quickshell-8B5CF6?style=flat-square)

</div>

This is the NixOS configuration for `yargc@vesper`, my Lenovo IdeaPad Gaming 3.

Most of the machine is managed through NixOS and Home Manager. Hyprland is the compositor, Caelestia provides the desktop shell, and the rest of the repo is split between system modules, user config and a small set of custom packages.

## Stack

| | |
|---|---|
| Nix | NixOS unstable + Home Manager |
| compositor | Hyprland |
| desktop shell | Caelestia / Quickshell |
| terminal | Ghostty |
| shell | Zsh + Oh My Zsh + Starship |
| editors | PychoVIM + Zed Preview |
| browsers | Zen + Helium + Tor Browser |
| coding agents | Codex · Claude Code · OpenCode · Grok Build · Hermes |
| agent control | bb |
| agent GUI | T3 Code Nightly |
| desktop AI | ChatGPT Desktop · Claude Desktop |
| command history | Navi + local Atuin |
| media | Spotify + Spicetify · MPV + MPRIS |
| privacy | Tor · Zapret2 · Monero GUI/CLI · Feather · Eigenwallet · Cuprate |
| containers | Podman · Distrobox |
| virtual machines | libvirt · virt-manager |
| recovery | Btrfs scrub · Snapper · Restic |
| Windows apps | Bottles |

## Desktop

Hyprland config lives in Lua under `home/yargc/hypr/`. Caelestia handles the bar, launcher, control center, notifications, lock screen, idle handling, clipboard, screenshots and recording.

The desktop is dark with translucent Caelestia surfaces, Hyprland blur, soft shadows and thin borders. The default wallpaper is the Dracula Nix wallpaper from `nixos-artwork`; Solarized Dark is installed as an alternative.

### Keys

| Key | Action |
|---|---|
| `Super + Space` | launcher |
| `Super + C` | control center |
| `Super + /` | command palette |
| `Super + Shift + /` | keybind sheet |
| `Ctrl + G` | Navi |
| `Ctrl + R` | Atuin history |
| `Super + A` | ChatGPT |
| `Super + Shift + A` | Claude Desktop |
| `Super + G` | Grok Build |
| `Super + Shift + D` | bb |
| `Super + T` | T3 Code Nightly |
| `Super + U` | CodexBar |

## Coding setup

The base toolchain is installed with Nix:

```text
Git / gh
Rust
Go
Python / uv / ruff
Node 24 / Bun / TypeScript
PHP / Composer
Java 21
Lua
nixd / nixfmt
GCC / Clang
CMake / GDB
Lazygit
mise
```

Bun is my default JS package manager. Project-specific versions can still live in `mise` or `nix develop`.

The local web stack is Apache + PHP + MariaDB. It is installed declaratively but stays off until `vesper-web.target` is started.

```bash
web-start
web-stop
web-restart
web-status
```

### Agents

`bb` is the main control surface for Codex, Claude Code, OpenCode and Hermes. Grok Build is installed from nixpkgs. T3 Code Nightly provides the GUI side of the same setup.

CodexBar, TurnLens and `ccusage` are used for usage/status visibility.

There is no local model service running by default.

## Applications

- Zen Browser
- Helium
- Tor Browser
- ChatGPT Desktop
- Claude Desktop
- Vesktop + Vencord
- Spotify + Spicetify
- MPV
- Session
- Telegram
- Obsidian
- Thunar
- Bottles
- T3 Code Nightly
- Grok Build

## Privacy

A system Tor client is available for applications that support SOCKS. Tor Browser keeps using its own bundled Tor instance.

The Monero setup includes Monero GUI/CLI, Feather, Eigenwallet and Cuprate. Node services do not start automatically.

Atuin stays local. Services with noticeable storage, bandwidth or background cost are opt-in.

## Packaging

Packages are taken from nixpkgs when possible. For software that is not available there in the required form, the repo uses upstream flakes or pinned source/binary derivations.

T3 Code Nightly is packaged from the official nightly AppImage. PychoVIM keeps its own updater/config ownership, while Zed Preview uses the upstream Preview installer.

## Recovery

There are three separate recovery layers:

```text
Nix generations   system rollback
Snapper           short-term Btrfs snapshots
Restic            encrypted backups
```

Btrfs scrub runs monthly. Snapper covers `/` and `/home` and keeps the existing root snapshot history under `/.snapshots`.

Restic runs daily with 7 daily, 4 weekly and 12 monthly snapshots, plus a monthly repository check. Credentials live outside the Nix store in `/etc/vesper/restic.env`.

`vesper-doctor` checks the filesystem, Btrfs scrub timer, AMD pstate, NVIDIA/PRIME, display refresh rate, Tor, the local web stack, backups and failed systemd units.

See [`docs/BACKUP.md`](docs/BACKUP.md) for backup setup and restore testing.

## Host

`vesper` is a Lenovo IdeaPad Gaming 3 16ARH7 (82SC):

- Ryzen 5 6600H
- Radeon 660M
- RTX 3050 Mobile
- 1920×1200 / 165 Hz display
- 1 TB NVMe
- 4 GiB EFI partition
- LUKS2-encrypted Btrfs root
- zram swap

Btrfs subvolumes:

```text
@       /
@home   /home
@root   /root
@srv    /srv
@cache  /var/cache
@tmp    /var/tmp
@log    /var/log
```

The Btrfs mounts use `compress=zstd:1` and `noatime`. AMD drives the desktop; the RTX 3050 is configured for PRIME offload.

The current storage UUIDs and mount layout are recorded in `hosts/vesper/hardware-configuration.nix` and [`docs/INSTALL.md`](docs/INSTALL.md).

If the disk is reformatted or the subvolume layout changes, those values need to be captured again before switching the configuration.

## Repo layout

```text
.
├── flake.nix
├── flake.lock
├── docs/
│   ├── INSTALL.md
│   └── BACKUP.md
├── hosts/
│   └── vesper/
├── modules/
│   ├── core/
│   ├── desktop/
│   ├── development/
│   └── privacy/
└── home/yargc/
    ├── hypr/
    ├── packages/
    ├── caelestia.nix
    ├── command-memory.nix
    ├── dev.nix
    ├── doctor.nix
    └── privacy.nix
```

## Using it

After NixOS is installed and the hardware config matches the machine:

```bash
nh os test
vesper-doctor
nh os switch
```

Update the flake explicitly:

```bash
cd ~/nix-config
nix flake update
nh os switch
```

Old generations can be cleaned with:

```bash
nh clean all --keep 5
```

Installation and storage notes are in [`docs/INSTALL.md`](docs/INSTALL.md).
