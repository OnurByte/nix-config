# AGENTS.md

## Scope

This repository defines **Kraken**, an agentic NixOS/Hyprland workstation.
Prefer small declarative changes over installer scripts, duplicated desktop layers,
or convenience hacks that make the system harder to reproduce.

## UX contract

- **Caelestia is the only shell/bar.** Do not add Waybar or a parallel widget stack for a feature Caelestia can own.
- **Spotify uses Spicetify + Caelestia MPRIS.** Keep music controls, Now Playing and hardware media keys on the same player service.
- **Discord uses Home Manager Vesktop + system Vencord.** Do not patch the stock Discord client with BetterDiscord.
- **Bun is the user-facing JavaScript package manager.** Do not add pnpm or yarn globally. Node may remain for runtime/LSP compatibility.
- **Cloud/provider agents are first class.** Do not add Ollama, LM Studio, llama.cpp services or other local-model daemons unless explicitly requested.
- **ChatGPT Desktop and Claude Desktop stay first-class apps.** They are not replaced by generic local-assistant frontends.
- **bb is the primary multi-agent control plane.** Avoid adding overlapping agent dashboards unless they have a clearly distinct role.
- **ZCode is the GLM surface.** Keep it packaged from the official, pinned Linux artifact.
- Turkish Q is the default keyboard layout for this host, but layout switching must remain available; do not turn a personal input preference into a project-wide restriction.
- Do not reintroduce gaming packages unless explicitly requested.

## Nix contract

- Never guess filesystem UUIDs, partition identifiers or mount topology. `hosts/kraken/hardware-configuration.nix` stays a placeholder until generated on the actual machine.
- Prefer NixOS/Home Manager modules and pinned derivations over `curl | sh` installers.
- Preserve `flake.lock`; update pins only when the change is intentional.
- Prefer store-qualified executable paths in desktop entries and Hyprland bindings when practical.
- Keep unfree packages deliberate and documented.
- Keep the Apache/PHP/MariaDB development stack local-only unless explicitly asked otherwise.
- Keep custom Caelestia patches small and build-tested; do not fork the whole shell for one widget.

## Change checklist

1. Parse every changed `.nix` file with `nix-instantiate --parse`.
2. Run `nix flake metadata --no-write-lock-file`.
3. Evaluate `.#nixosConfigurations.kraken.config.networking.hostName` and expect `kraken`.
4. If touching ZCode, build `.#zcode` with unfree packages allowed.
5. If touching Caelestia/QML/CodexBar, build the configured Caelestia package.
6. Keep README user-facing. Implementation guardrails and agent instructions belong here.
