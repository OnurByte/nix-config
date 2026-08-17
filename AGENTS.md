# AGENTS.md

## Scope

This repository defines **Vesper**, a personal NixOS/Hyprland workstation.
Prefer small declarative changes over installer scripts, duplicated desktop layers or hidden mutable state.

## Writing contract

- Write like the person maintaining the machine, not like a product page or generated project summary.
- Keep README prose short and plain. Prefer lowercase section headings and avoid unnecessary capitalization outside real product names, commands and paths.
- Use punctuation lightly. Avoid semicolons, em dashes, stacked parentheticals and polished marketing-style sentence rhythm when a simpler line works.
- The README opening should keep two short lines that explain the actual Vesper niche: Tor and Monero privacy tooling plus an AI-heavy coding workflow, with no gaming setup.
- Document the current state. Do not mention removed, rejected, abandoned or intentionally absent features unless that absence is operationally important or the user explicitly asks for history.
- Avoid filler, marketing copy, self-congratulatory language and stock AI phrasing. Do not pad documentation with explanations of obvious choices.
- Prefer exact package names, paths, commands and behavior over adjectives or vague claims.
- README describes what exists now. Git history carries old decisions; do not turn README into a changelog.
- Keep negative guardrails in `AGENTS.md` only when they prevent a real regression or protect an operational constraint.

## UX contract

- Caelestia is the only shell/bar and owns network, Bluetooth, audio, notifications, idle/lock, clipboard, capture, launcher and wallpaper UI.
- Keep the two `wl-paste -> cliphist store` watchers; they are Caelestia's clipboard backend.
- Do not reintroduce `nm-applet`, Blueman UI, Pavucontrol, Waybar, hypridle or hyprlock while Caelestia owns those surfaces.
- Hyprland config is Lua. Keep `home/yargc/hyprland.nix` as wiring and compositor logic under `home/yargc/hypr/*.lua`.
- The Vesper visual language is Apple/visionOS-inspired controlled glass. Shell surfaces should use layered translucency, readable backdrop blur, calm spacing, generous continuous rounding, soft shadow and thin luminous borders.
- Prefer quiet neutral or palette-tinted glass over neon multi-colour borders, opaque telemetry cards or a heavy Material-dashboard look.
- Keep glass concentrated in the shell, launchers, drawers, popovers and HUD surfaces. Do not turn every application transparent.
- Prefer native Caelestia panels/drawers for shell information when practical instead of spawning a terminal-shaped dashboard.
- Wallpaper assets should come from maintained/public sources or nixpkgs. Do not generate bespoke wallpapers unless explicitly requested.
- Turkish Q stays the default layout; US switching remains available.
- Zsh stays minimal: Home Manager features + small Oh My Zsh layer + Starship.
- Command memory is Navi + local Atuin. `Super + /` copies from the desktop palette; `Ctrl + G` inserts into the current prompt; `Ctrl + R` searches history.
- Keep Atuin local unless sync is explicitly requested.
- Spotify uses Spicetify and remains Caelestia's default player. MPV is the local audio/video player and exposes MPRIS rather than adding another media shell.
- Discord uses Vesktop + system Vencord.
- Bun is the user-facing JavaScript package manager; do not add pnpm/yarn globally.
- Cloud/provider agents are first class. Grok Build is the official xAI CLI entry point; use nixpkgs `grok-build`, not the unrelated `grok-cli` package or a mutable installer wrapper.
- Do not add Ollama, LM Studio or another local-model daemon unless requested.
- `bb` is the primary multi-agent control plane. Keep `BB_TELEMETRY=false` and use bb's existing automation/plugin model instead of adding another orchestrator.
- T3 Code Nightly is the GUI coding surface.
- Keep T3 Code on an official pinned nightly AppImage and expose Codex, Claude Code and OpenCode to its PATH.
- TurnLens, ccusage and CodexBar have different jobs; keep them distinct.
- The Agent Cockpit may observe live processes and Git state, but persistent session snapshots belong under `~/.local/state/vesper/agents/` rather than hidden ad-hoc files elsewhere.
- `~/.agents/skills` is the canonical active skill tree. Agent-specific skill paths should link back to it instead of becoming separately maintained copies.
- Hermes research may propose reusable skills under `~/.local/share/vesper/skill-drafts/`. Drafts stay inactive until reviewed and promoted.
- Hermes' own cron/scheduled automation layer owns recurring Hermes research. Do not duplicate the same jobs with GitHub Actions, systemd timers or a second cron layer.
- Hermes cron is only the heartbeat: scheduled runs resume persistent research state and should not rediscover the workflow from scratch or create more cron jobs.
- Monero GUI/CLI, Feather and Eigenwallet are first-class privacy tools. Cuprate remains opt-in/experimental and must not replace `monerod` silently.
- Keep the system Tor client available separately from Tor Browser's bundled Tor.
- Do not auto-enable blockchain nodes, mining or P2Pool.
- Podman and Distrobox are the container stack; libvirt + virt-manager provide local virtual machines.
- Bottles is a Windows-compatibility utility, not permission to restore a gaming stack.
- Do not add gaming packages unless explicitly requested.
- Do not re-add a night-light daemon unless explicitly requested.

## Reliability contract

- First-party Vesper runtime/control-plane code must not be Python. Do not add tracked `.py` files or a global Python/uv/ruff development stack. Use Rust for native Vesper services and CLIs, with Nix/QML/Lua/jq in their existing roles. Upstream packages may internally depend on Python; do not vendor that implementation into this repository.
- CI must fail if a first-party `.py` file appears again.
- Vesper is a single laptop, not a reusable host framework. Do not introduce multi-host abstractions without a real second host.
- The verified disk is GPT -> 4 GiB EFI + LUKS2 -> Btrfs. Preserve that model unless a destructive reinstall is explicitly requested.
- `hosts/vesper/hardware-configuration.nix` now contains the verified live storage topology; it is not an installer and must not gain formatting/partitioning commands.
- Verified Btrfs subvolumes are: `@` -> `/`, `@home` -> `/home`, `@root` -> `/root`, `@srv` -> `/srv`, `@cache` -> `/var/cache`, `@tmp` -> `/var/tmp`, `@log` -> `/var/log`.
- The EFI UUID is `D804-0279`, LUKS2 UUID is `abb7c069-db97-472e-ba70-38cf58bd9fc4`, and unlocked Btrfs UUID is `af2e7549-434c-413b-a077-dceea390b1a1`. If the disk is reformatted, recapture identifiers before changing them.
- Root already has a `.snapshots` Btrfs subvolume with existing Snapper history. Preserve it unless an explicit reset is requested.
- Btrfs scrub and Snapper are local recovery; Restic is the real backup layer. Do not describe snapshots as backups.
- Backup credentials and Restic passwords stay outside the Nix store. `/etc/vesper/restic.env` and the password file are machine-local secrets.
- The local Apache/PHP/MariaDB stack is opt-in through `vesper-web.target` and must stay stopped at boot.
- Do not add disk-backed hibernation until the real swap target and resume parameters are known.
- Secure Boot may be added only after the real NixOS installation is stable; private signing keys must never enter Git.
- PychoVIM is Onur's own project and intentionally keeps its upstream-managed mutable config/updater. Do not "fix" that by replacing its ownership model.
- Zed is the stable `pkgs.zed-editor.fhs` package from the locked nixpkgs revision. Do not reintroduce the mutable Preview installer unless explicitly requested.

## Nix contract

- Prefer NixOS/Home Manager modules and pinned packages over mutable installers except for PychoVIM's explicit updater-owned config.
- Check nixpkgs and upstream Nix support before writing a custom derivation.
- Grok Build must come from `pkgs.grok-build` so its version follows the pinned nixpkgs input.
- Preserve `flake.lock`; update pins only intentionally.
- Keep unfree packages deliberate.
- Keep Apache/PHP/MariaDB local-only and opt-in unless asked otherwise.
- Keep custom Caelestia patches small and build-tested.
- Keep Caelestia theme propagation explicitly whitelisted.
- Keep Zapret2 narrow by default.
- Follow the kernel line selected by pinned nixpkgs unless Vesper has a measured hardware reason to override it.
- Project-specific language versions may use `mise` or `nix develop`; avoid stacking multiple version managers for the same project.

## Change checklist

1. Reject tracked first-party `.py` files.
2. Parse every changed `.nix` file with `nix-instantiate --parse`.
3. Compile changed first-party Rust control-plane code.
4. Parse every Hyprland Lua file with `luac -p`.
5. Run `nix flake metadata --no-write-lock-file`.
6. Evaluate `.#nixosConfigurations.vesper.config.networking.hostName` and expect `vesper`.
7. Evaluate the complete Home Manager activation derivation with `nix eval --raw '.#nixosConfigurations.vesper.config.home-manager.users.yargc.home.activationPackage.drvPath'`.
8. If touching Caelestia/QML/CodexBar, build the configured Caelestia package before the full system build.
9. Build `.#nixosConfigurations.vesper.config.system.build.toplevel`; the hardware topology is concrete, so this must not be skipped.
10. If touching T3 Code Nightly, build `.#t3code-nightly`.
11. If touching TurnLens, build `.#turnlens`.
12. If touching Cuprate, build `.#cuprated`.
13. If touching storage or backup logic, update `docs/INSTALL.md` or `docs/BACKUP.md` when the operational contract changes.
14. Keep README user-facing; implementation guardrails belong here.
