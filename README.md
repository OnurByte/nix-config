<div align="center">

# vesper

### my nixos workstation

**NixOS · Hyprland · Caelestia · Ghostty · PychoVIM · Zed · Codex · Claude · OpenCode · Grok Build · Hermes**

![NixOS](https://img.shields.io/badge/NixOS-unstable-5277C3?style=flat-square&logo=nixos&logoColor=white)
![Hyprland](https://img.shields.io/badge/Hyprland-Wayland-58E1FF?style=flat-square)
![Caelestia](https://img.shields.io/badge/Caelestia-Quickshell-8B5CF6?style=flat-square)

</div>

vesper is built around tor monero privacy tooling and an ai heavy coding workflow
there is no gaming setup here just a linux workstation shaped around how i use my computer every day

this is the nixos config for `yargc@vesper` on a Lenovo IdeaPad Gaming 3
most of the machine lives in nixos and home manager with Hyprland for the compositor and Caelestia for the desktop shell

## stack

| | |
|---|---|
| nix | NixOS unstable + Home Manager |
| compositor | Hyprland |
| desktop shell | Caelestia / Quickshell |
| launcher | Vicinae |
| terminal | Ghostty |
| shell | Zsh + Oh My Zsh + Starship |
| editors | PychoVIM + stable Zed |
| browsers | Zen + Helium + Tor Browser |
| coding agents | Codex · Claude Code · OpenCode · Grok Build · Hermes |
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
Caelestia owns the bar control center notifications lock idle clipboard screenshots and recording while Vicinae is the primary `Super` application launcher

Liquid Glass is maintained as the consumer-agnostic sibling project at
`../liquid-glass/`. Vesper only exposes its enforced transparency and launcher
surface settings; it does not own the visual contract or renderer.
current shell behavior follows the active Caelestia theme
Vicinae follows the same light/dark scheme and primary accent; its Vesper-specific controls live in `Settings -> Vicinae`

Hyprland keeps its own baseline window appearance; Liquid Glass profiles are
not applied automatically.

### keys

| key | action |
|---|---|
| `Super` | Vicinae launcher |
| `Super + Space` | Vicinae launcher alternate |
| `Super + C` | control center |
| `Super + /` | command palette |
| `Super + Shift + /` | keybind sheet |
| `Ctrl + G` | Navi |
| `Ctrl + R` | Atuin history |
| `Super + A` | ChatGPT |
| `Super + Shift + A` | Claude Desktop |
| `Super + G` | Grok Build |
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

Bun is the default user-facing js package manager
project versions can still live in `mise` or `nix develop`

first-party Vesper runtime and control-plane code is not Python

local web work uses Apache + PHP + MariaDB and stays off until `vesper-web.target` is started

```bash
web-start
web-stop
web-restart
web-status
```

### agents

Vesper's AI control plane owns provider configuration credentials analytics skills MCP inventory live-agent state and user-facing orchestration boundaries
optional orchestration backends stay replaceable behind that boundary

Agent Cockpit watches supported coding agents from local process state plus Git
live snapshots belong under `~/.local/state/vesper/agents/`

AgentsView owns local session/activity history
CodexBar handles live limits `ccusage` cross-checks accounting and TurnLens handles supported per-turn diagnostics
there is no local model service running by default

active agent skills have one canonical home at `~/.agents/skills`
Codex Claude and OpenCode skill paths link back to that tree instead of maintaining separate copies

### hermes cron

Hermes uses its own cron / scheduled automation layer for recurring research
scheduled runs resume persistent research state instead of creating another scheduler layer

Hermes writes durable briefing output under `~/.local/share/vesper/briefings/`
reusable skill candidates go to `~/.local/share/vesper/skill-drafts/` and stay inactive until reviewed

See `docs/HERMES.md` and the Hermes research docs for the current contracts

## apps

- Vicinae
- Vesper Store
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

## recovery

```text
Nix generations   system rollback
Snapper           short term local recovery
Restic            encrypted backups
```

Btrfs scrub runs monthly
Restic runs daily with 7 daily 4 weekly and 12 monthly snapshots plus a monthly repository check
credentials live outside the nix store in `/etc/vesper/restic.env`

`vesper-doctor` checks the workstation and exposes the same checks as JSON through `vesper-doctor --json`

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

current storage identifiers and subvolumes live in `hosts/vesper/hardware-configuration.nix` and `docs/INSTALL.md`

## repository

```text
.
├── AGENTS.md
├── flake.nix
├── flake.lock
├── docs/
│   └── README.md
├── hosts/vesper/
├── modules/
└── home/yargc/
```

`docs/README.md` is the documentation index and authority map
`AGENTS.md` contains repository-wide agent guardrails

## using it

once nixos is installed and the hardware config matches the machine

```bash
nh os test
vesper-doctor
vesper-doctor --json
nh os switch
```

update intentionally

```bash
cd ~/nix-config
nix flake update
nh os switch
```

clean old generations

```bash
nh clean all --keep 5
```
