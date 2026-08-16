# AGENTS.md

## Scope

This repository defines **Vesper**, a personal NixOS/Hyprland workstation.
Prefer small declarative changes over installer scripts, duplicated desktop layers or hidden mutable state.

## UX contract

- Caelestia is the only shell/bar and owns network, Bluetooth, audio, notifications, idle/lock, clipboard, capture, launcher and wallpaper UI.
- Keep the two `wl-paste -> cliphist store` watchers; they are Caelestia's clipboard backend.
- Do not reintroduce `nm-applet`, Blueman UI, Pavucontrol, Waybar, hypridle or hyprlock while Caelestia owns those surfaces.
- Hyprland config is Lua. Keep `home/yargc/hyprland.nix` as wiring and compositor logic under `home/yargc/hypr/*.lua`.
- The Vesper visual language is controlled glass: dark wallpaper, translucent shell surfaces, readable blur, soft shadow, thin border. Do not turn every app transparent.
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
- `bb` is the primary multi-agent control plane. T3 Code Nightly is the GUI coding surface.
- Keep T3 Code on an official pinned nightly AppImage and expose Codex, Claude Code and OpenCode to its PATH.
- ZCode is intentionally removed. Do not restore it unless explicitly requested.
- TurnLens, ccusage and CodexBar have different jobs; keep them distinct.
- Monero GUI/CLI, Feather and Eigenwallet are first-class privacy tools. Cuprate remains opt-in/experimental and must not replace `monerod` silently.
- Keep the system Tor client available separately from Tor Browser's bundled Tor.
- Do not auto-enable blockchain nodes, mining or P2Pool.
- Podman and Distrobox are the container stack. Do not reintroduce libvirt/virt-manager unless explicitly requested.
- Bottles is a Windows-compatibility utility, not permission to restore a gaming stack.
- Do not add gaming packages unless explicitly requested.
- Do not re-add a night-light daemon unless explicitly requested.

## Nix contract

- Never guess filesystem UUIDs or partition topology. `hosts/vesper/hardware-configuration.nix` stays a placeholder until generated on the machine.
- Prefer NixOS/Home Manager modules and pinned packages over `curl | sh`.
- Check nixpkgs and upstream Nix support before writing a custom derivation.
- Grok Build must come from `pkgs.grok-build` so its version follows the pinned nixpkgs input.
- Preserve `flake.lock`; update pins only intentionally.
- Keep unfree packages deliberate.
- Keep Apache/PHP/MariaDB local-only unless asked otherwise.
- Keep custom Caelestia patches small and build-tested.
- Keep Caelestia theme propagation explicitly whitelisted.
- Keep Zapret2 narrow by default.

## Change checklist

1. Parse every changed `.nix` file with `nix-instantiate --parse`.
2. Parse every Hyprland Lua file with `luac -p`.
3. Run `nix flake metadata --no-write-lock-file`.
4. Evaluate `.#nixosConfigurations.vesper.config.networking.hostName` and expect `vesper`.
5. Evaluate the complete Home Manager activation derivation with `nix eval --raw '.#nixosConfigurations.vesper.config.home-manager.users.yargc.home.activationPackage.drvPath'`.
6. If touching T3 Code Nightly, build `.#t3code-nightly`.
7. If touching TurnLens, build `.#turnlens`.
8. If touching Cuprate, build `.#cuprated`.
9. If touching Caelestia/QML/CodexBar, build the configured Caelestia package.
10. Keep README user-facing; implementation guardrails belong here.
