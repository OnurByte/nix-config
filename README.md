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

### hermes automation

Hermes cron is the only recurring scheduler
`vesper-hermes-automations` is the only research execution owner and uses transient `systemd-run --user` workers only after a cron trigger
there is no second systemd timer or GitHub Actions schedule running the same research fleet

```text
Hermes cron
    ↓
short no-agent trigger
    ↓
watchdog / monitor / dispatch
    ↓
long Hermes one-shot when reasoning is actually needed
    ↓
persistent state + briefing inbox + Obsidian consolidation
```

Daily research keeps three different questions separate:

| lane | question |
|---|---|
| `unknown-frontier-ai` | what useful AI thing exists outside the current knowledge map? |
| `agenda` | what important thing happened or changed today? |
| `free-ai-radar` | what legitimate useful free AI access/workflow appeared or changed? |

Unknown Frontier AI is deliberately not a popularity feed
GitHub and Reddit use broad parallel deterministic collectors that can inspect hundreds of candidates before the model spends time verifying the strongest subset
X uses Hermes' native `x_search` toolset
full candidate pools stay under `~/.local/state/vesper/research/candidate-pools/` while prompt injection is bounded

The frontier synthesis learns useful future discovery routes in a bounded inert `discovery-seeds.json`
seed sources and queries are starting points rather than an allowlist and weak duplicate/hype-heavy routes are allowed to decay

Free AI Radar treats Linux.do as a first-class discovery source then verifies outward against original providers repos releases docs or authors
Agenda is ranked by importance recency consequence and relevance rather than obscurity
Morning Check keeps Agenda Unknown Frontier AI and Free AI Radar as separate sections

Upstream Edge Radar starts with a zero-token deterministic GitHub head monitor
if tracked upstreams did not move the expensive worker is never launched

Research jobs explicitly preload `hermes-research-radar`
nightly second-brain consolidation explicitly preloads Hermes' bundled `obsidian` skill together with `vesper-obsidian-second-brain`
learned procedures remain drafts under `~/.local/share/vesper/skill-drafts/` until reviewed rather than silently becoming active skills

Sunday also runs user-pain mining project archaeology skill-evolution review AI-usage economics and one final weekly intelligence synthesis

Full architecture and commands live in [`docs/HERMES.md`](docs/HERMES.md)

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
│   ├── BACKUP.md
│   └── HERMES.md
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
    ├── hermes-jobs.nix
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
