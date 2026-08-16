<div align="center">

# vesper

### my nixos workstation

**NixOS · Hyprland · Caelestia · Ghostty · PychoVIM · Zed · Codex · Claude · OpenCode · Grok Build · Hermes**

![NixOS](https://img.shields.io/badge/NixOS-unstable-5277C3?style=flat-square&logo=nixos&logoColor=white)
![Hyprland](https://img.shields.io/badge/Hyprland-Wayland-58E1FF?style=flat-square)
![Caelestia](https://img.shields.io/badge/Caelestia-Quickshell-8B5CF6?style=flat-square)

</div>

vesper is built around the stuff i actually care about tor monero privacy tooling and an ai heavy coding workflow
there is no gaming setup here just a linux workstation shaped around how i use my computer every day

this is the nixos config for `yargc@vesper` on a Lenovo IdeaPad Gaming 3
most of the machine lives in nixos and home manager with Hyprland for the compositor and Caelestia for the desktop shell

## stack

| | |
|---|---|
| nix | NixOS unstable + Home Manager |
| compositor | Hyprland |
| desktop shell | Caelestia / Quickshell |
| terminal | Ghostty |
| shell | Zsh + Oh My Zsh + Starship |
| editors | PychoVIM + stable Zed |
| browsers | Zen + Helium + Tor Browser |
| coding agents | Codex · Claude Code · OpenCode · Grok Build · Hermes |
| agent control | bb |
| agent gui | T3 Code Nightly |
| desktop ai | ChatGPT Desktop · Claude Desktop |
| command history | Navi + local Atuin |
| media | Spotify + Spicetify · MPV + MPRIS |
| privacy | Tor · Zapret2 · Monero GUI/CLI · Feather · Eigenwallet · Cuprate |
| containers | Podman · Distrobox |
| virtual machines | libvirt · virt-manager |
| recovery | Btrfs scrub · Snapper · Restic |
| windows apps | Bottles |

## desktop

Hyprland config lives in Lua under `home/yargc/hypr/`
Caelestia handles the bar launcher control center notifications lock idle clipboard screenshots and recording

Vesper uses an Apple/visionOS inspired glass language rather than a dense telemetry-dashboard look
shell surfaces are layered and translucent with readable backdrop blur larger continuous rounding soft shadows and thin quiet borders
colour comes from the active Caelestia palette but the frame stays restrained instead of using neon multi-colour borders

Hyprland currently uses 22 px window rounding 12 px blur with 4 passes and a soft 24 px shadow
Caelestia uses lower-opacity layered surfaces so the wallpaper and depth remain visible without making every application transparent

### keys

| key | action |
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
| `Super + Shift + H` | Hermes Desktop |
| `Super + T` | T3 Code Nightly |
| `Super + U` | CodexBar |
| `Super + N` | PychoVIM |
| `Super + Z` | Zed |

## coding

base toolchain comes from nix

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

Bun is my default js package manager
project versions can still live in `mise` or `nix develop`

local web work uses Apache + PHP + MariaDB and stays off until `vesper-web.target` is started

```bash
web-start
web-stop
web-restart
web-status
```

### agents

`bb` is the main control surface for Codex Claude Code OpenCode and Hermes
its optional telemetry is disabled with `BB_TELEMETRY=false`
Grok Build comes from nixpkgs and T3 Code Nightly is the gui side of the setup

Agent Cockpit watches Codex Claude OpenCode Hermes Grok and bb from local process state plus Git
live sessions are mirrored into `~/.local/state/vesper/agents/` with project branch dirty state first/last seen timestamps and process age

CodexBar TurnLens and `ccusage` cover usage and status
there is no local model service running by default

active agent skills have one canonical home at `~/.agents/skills`
Codex Claude and OpenCode skill paths link back to that tree instead of maintaining separate copies

### hermes cron

Hermes uses its own cron / scheduled automation layer for recurring research
there is no second GitHub Actions or systemd timer trying to run the same jobs
cron is only the heartbeat: every run resumes persistent research state instead of starting from zero or creating more cron jobs

| job | behavior |
|---|---|
| `briefing` | recurring concise digest such as a daily research/news briefing |
| `research` | deeper bounded investigation into one question |
| `watch` | check a condition and only surface something when it meaningfully changes |

The research loop is adaptive
user supplied RSS feeds subreddits repositories channels and sites are starting seeds rather than a permanent allowlist
Hermes can expand through links authors crossposts GitHub docs/issues/PRs citations transcripts curated lists and generated queries then learn which sources and methods keep producing signal

A normal run follows roughly this shape

```text
persistent state
      ↓
orient → cheap intake → expand → verify → rank → deliver → learn
             ↑                                      ↓
       rss/atom first                    source + heuristic state
```

The default balance is roughly 80% exploitation and 20% exploration
exploration can rise when findings repeat source overlap grows novelty drops a topic moves quickly or results cluster inside one source graph

Reusable behavior is learned gradually

```text
observation
  → candidate heuristic
  → repeated trials
  → active heuristic
  → decay / review
  → retained scoped or retired
```

Hermes writes durable briefing output under `~/.local/share/vesper/briefings/`
reports can keep Markdown for reading and JSON for shell/UI integration with fields such as title summary body type priority sources createdAt job and unread
short notifications are for the interrupt while the full report stays in the briefing inbox
`watch` jobs stay quiet when there is no meaningful change

Potential reusable skills go to `~/.local/share/vesper/skill-drafts/`
those drafts are deliberately not active until reviewed and promoted into `~/.agents/skills`

## apps

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

## privacy

there is a system Tor client for apps that support SOCKS while Tor Browser keeps its own bundled Tor

Monero tooling includes Monero GUI/CLI Feather Eigenwallet and Cuprate
node services stay off until i start them

Atuin stays local
anything with noticeable storage bandwidth or background cost is opt in

## packaging

packages come from nixpkgs when they can
anything missing in the form i need comes from an upstream flake or a pinned source or binary derivation

T3 Code Nightly uses the official nightly AppImage
PychoVIM keeps its own updater and config ownership
Zed is the stable `pkgs.zed-editor.fhs` package from the locked nixpkgs revision

## recovery

```text
Nix generations   system rollback
Snapper           short term Btrfs snapshots
Restic            encrypted backups
```

Btrfs scrub runs monthly
Snapper covers `/` and `/home` and keeps the existing root snapshot history under `/.snapshots`

Restic runs daily with 7 daily 4 weekly and 12 monthly snapshots plus a monthly repository check
credentials live outside the nix store in `/etc/vesper/restic.env`

`vesper-doctor` checks the filesystem Btrfs scrub timer AMD pstate NVIDIA/PRIME display refresh rate Tor the local web stack backups and failed systemd units
`vesper-doctor --json` exposes the same checks as structured data for agents and future shell UI

backup setup and restore testing live in [`docs/BACKUP.md`](docs/BACKUP.md)

## host

`vesper` is a Lenovo IdeaPad Gaming 3 16ARH7 82SC

- Ryzen 5 6600H
- Radeon 660M
- RTX 3050 Mobile
- 1920×1200 165 Hz display
- 1 TB NVMe
- 4 GiB EFI partition
- LUKS2 encrypted Btrfs root
- zram swap

btrfs subvolumes

```text
@       /
@home   /home
@root   /root
@srv    /srv
@cache  /var/cache
@tmp    /var/tmp
@log    /var/log
```

mounts use `compress=zstd:1` and `noatime`
AMD drives the desktop and the RTX 3050 is PRIME offload only

current storage uuids and the mount layout are in `hosts/vesper/hardware-configuration.nix` and [`docs/INSTALL.md`](docs/INSTALL.md)
if the disk or subvolume layout changes those values need to be captured again before switching

## layout

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
    ├── skills/
    ├── caelestia.nix
    ├── command-memory.nix
    ├── dev.nix
    ├── doctor.nix
    ├── skills.nix
    └── privacy.nix
```

## using it

once nixos is installed and the hardware config matches the machine

```bash
nh os test
vesper-doctor
vesper-doctor --json
nh os switch
```

update the flake when i want to

```bash
cd ~/nix-config
nix flake update
nh os switch
```

clean old generations

```bash
nh clean all --keep 5
```

install and storage notes are in [`docs/INSTALL.md`](docs/INSTALL.md)
