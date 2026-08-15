# OnurByte NixOS config

Personal NixOS + Home Manager workstation config for `yargc@kraken`, inspired by
[`bariscodefxy/nix-config`](https://github.com/bariscodefxy/nix-config), Omarchy and a handful of
mature NixOS configs. The gaming/Victus-specific parts of the original base were removed and the
stack was rebuilt around development, AI tooling, privacy and a terminal-first workflow.

## Kraken hardware

Target machine:

- Lenovo IdeaPad Gaming 3 16ARH7 (`82SC`)
- AMD Ryzen 5 6600H / Rembrandt
- AMD Radeon 660M integrated GPU
- NVIDIA GeForce RTX 3050 Mobile (GA107M)
- 1920x1200 165 Hz internal display
- MediaTek MT7921 Wi-Fi
- Realtek RTL8111/8168 Ethernet
- Crucial P3 NVMe
- ~27 GiB RAM

The host uses the latest mainline NixOS kernel and AMD P-State active mode. Hybrid graphics is
configured for an AMD-driven desktop with NVIDIA PRIME offload:

```text
NVIDIA 01:00.0 -> PCI:1:0:0
AMD    05:00.0 -> PCI:5:0:0
```

Run GPU-heavy applications explicitly with:

```bash
nvidia-offload <command>
```

## Desktop

- Hyprland + Caelestia
- Ghostty
- Turkish Q only (`kb_layout = tr`)
- PipeWire + WirePlumber
- Bluetooth + Blueman
- LocalSend with its local-network firewall port enabled
- clipboard history, screenshots, lock/idle handling

### Key bindings

| Key | Action |
|---|---|
| `Super + Return` | Ghostty |
| `Super + Space` | Caelestia launcher |
| `Super + B` | Zen Browser |
| `Super + Shift + B` | Helium |
| `Super + E` | Thunar |
| `Super + N` | PychoVIM |
| `Super + Z` | Zed Preview |
| `Super + A` | ChatGPT app window |
| `Super + Shift + A` | Claude app window |
| `Super + Shift + C` | Codex |
| `Super + Shift + O` | OpenCode |
| `Super + Shift + H` | Hermes Agent |
| `Super + U` | CodexBar usage cards |

## Browsers

The normal browser stack is deliberately small:

- **Zen Browser** — default HTTP/HTTPS browser
- **Helium** — Chromium-side companion browser
- **Tor Browser** — isolated Tor browsing

There is no generic Firefox install in the default app set. ChatGPT and Claude do not rely on
unofficial Linux Electron wrappers; they are app-mode Helium desktop entries.

## Editors and coding agents

### PychoVIM

`OnurByte/PSYCHOVIM` is the default terminal editor. Nix supplies Neovim and the compiler/runtime
dependencies, while PychoVIM keeps ownership of its mutable config, marketplace and updater.

The first `pycho` (or `nvim`, which aliases to `pycho`) bootstraps the official PSYCHOVIM installer.
After that, its own launcher in `~/.local/bin` takes precedence.

### Zed Preview

`zed` maps to `zed-preview`. The first launch runs Zed's official Preview-channel installer. Keeping
the preview binary upstream-managed lets the fast-moving preview channel update independently from
the Nix system generation.

### AI stack

- Codex
- Claude Code
- OpenCode
- Hermes Agent
- T3 Code
- CodexBar Linux CLI

Codex, Claude Code and OpenCode are managed through Home Manager and share one declarative MCP
registry (`programs.mcp.servers`). Hermes is lazy-bootstrapped from Nous Research on first use.

`aiusage` runs `codexbar cards`. CodexBar is pinned as a Nix package from the upstream static Linux
CLI release rather than using the Darwin-only GUI package from nixpkgs.

## Apps

- Session
- Telegram Desktop
- Obsidian
- LocalSend
- T3 Code
- ChatGPT web-app window
- Claude web-app window
- mpv + imv

## Development stack

System baseline:

- Rust + rust-analyzer
- Go + gopls
- Python + uv + Ruff
- Node.js 24 + Bun + pnpm + TypeScript language server
- PHP + Composer + Intelephense
- Java 21 + JDT LS
- Lua + Lua Language Server + Stylua
- nixd + nixfmt
- GCC/Clang/GDB/CMake/Make
- Podman + Distrobox
- libvirt + virt-manager
- GitHub CLI + Lazygit
- mise

Nix owns the reliable machine-wide baseline. `mise` is available for repositories that require
project-specific runtime versions or already ship `mise.toml` / `.tool-versions`.

## XAMPP-style local web stack

Upstream XAMPP is not used. The equivalent stack is native NixOS services:

- Apache HTTPD
- PHP
- MariaDB
- default database: `dev`
- document root: `/srv/http` (owned by `yargc`)
- Apache listens only on `127.0.0.1:80`

Convenience commands:

```bash
xampp-start
xampp-stop
xampp-restart
xampp-status
```

Open the local site at `http://localhost`.

## Nix workflow

Home Manager runs as a NixOS module, so system and user state switch together:

```bash
nh os test
nh os switch
nix flake update
nh clean all --keep 5
```

## Layout

```text
.
├── flake.nix
├── hosts/
│   └── kraken/
│       ├── default.nix
│       ├── hardware.nix
│       └── hardware-configuration.nix
├── modules/
│   ├── core/
│   ├── desktop/
│   └── development/
└── home/
    └── yargc/
        ├── packages/
        └── *.nix
```

## Before the first real switch

`hosts/kraken/hardware-configuration.nix` is intentionally still a placeholder. `hardware.nix`
contains the known model/GPU tuning, but it **does not replace** the generated filesystem, initrd and
device configuration for the actual NixOS installation.

After NixOS generates the machine file, replace the placeholder with the real one and then test:

```bash
git clone git@github.com:OnurByte/nix-config.git ~/nix-config
cd ~/nix-config

sudo cp /etc/nixos/hardware-configuration.nix \
  hosts/kraken/hardware-configuration.nix

nix flake lock
sudo nixos-rebuild test --flake .#kraken
sudo nixos-rebuild switch --flake .#kraken
```

Do not copy another machine's `hardware-configuration.nix` or disk UUIDs.

## Mutable vs declarative boundary

Most of the workstation is reproducible through Nix. Three tools are intentionally allowed to own
their fast-moving user-space installation because that matches their upstream workflow:

- PSYCHOVIM
- Zed Preview
- Hermes Agent

Their launchers live under `~/.local/bin`, which is placed before the Nix profile. Everything else
should stay declarative unless there is a good reason not to.

## Secrets

Do not put API keys, tokens, SSH private keys or `.env` contents directly in the flake. `sops` and
`age` are installed; adding `sops-nix` is the next step if declarative secrets are needed.

## Deliberately not added

- Steam / gaming modules / PrismLauncher
- Half-Life custom packages from the original base
- HP Victus/WMI modules
- another machine's Disko partition layout
- random NVIDIA PRIME IDs guessed from another laptop
- unofficial ChatGPT/Claude Linux desktop wrappers
- impermanence before the first NixOS migration is proven stable

The target is a recoverable workstation config, not the largest possible dotfiles dependency graph.
