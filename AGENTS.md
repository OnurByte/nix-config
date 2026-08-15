# AGENTS.md

## Scope

This repository defines **Kraken**, an agentic NixOS/Hyprland workstation.
Prefer small declarative changes over installer scripts, duplicated desktop layers,
or convenience hacks that make the system harder to reproduce.

## UX contract

- **Caelestia is the only shell/bar and owns desktop UX**: network, Bluetooth, audio surfaces, notifications, idle/lock, clipboard frontend, screenshots/recording, launcher and wallpaper scheme.
- Keep the two `wl-paste -> cliphist store` processes: they are Caelestia's clipboard-history backend, not a competing UI.
- Do not reintroduce `nm-applet`, Blueman, Pavucontrol, Fuzzel-as-launcher, hypridle or hyprlock while Caelestia owns those surfaces.
- **Hyprland config is Lua.** Keep `home/yargc/hyprland.nix` as the Home Manager wiring layer and compositor logic under `home/yargc/hypr/*.lua`; do not recreate `hyprland.conf`.
- Turkish Q is the default keyboard layout for this host, but layout switching must remain available.
- **Zsh keeps a deliberately small Oh My Zsh layer plus Starship.** Do not add an OMZ theme or a large plugin bundle without a concrete workflow reason.
- **Spotify uses Spicetify + Caelestia MPRIS.** Keep music controls, Now Playing and hardware media keys on the same player service.
- **Discord uses Home Manager Vesktop + system Vencord.** Do not patch the stock Discord client with BetterDiscord.
- **Bun is the user-facing JavaScript package manager.** Do not add pnpm or yarn globally. Node may remain for runtime/LSP compatibility.
- **Cloud/provider agents are first class.** Do not add Ollama, LM Studio, llama.cpp services or other local-model daemons unless explicitly requested.
- **ChatGPT Desktop and Claude Desktop stay first-class apps.** They are not replaced by generic local-assistant frontends.
- **bb is the primary multi-agent control plane.** Avoid adding overlapping agent dashboards unless they have a clearly distinct role.
- **ZCode is the GLM surface.** Keep it packaged from the official, pinned Linux artifact.
- **Monero is a first-class privacy toolchain.** Keep the official GUI/CLI available; Feather is the lightweight complementary wallet and Eigenwallet must come from nixpkgs rather than a local binary repack.
- **Cuprate is experimental.** Keep `cuprated` available for opt-in testing, but do not replace `monerod` or auto-start it while upstream still labels the release preview/WIP.
- Keep the system Tor client available on the standard local SOCKS endpoint for privacy-aware CLI software. Do not confuse it with Tor Browser's separate bundled Tor instance.
- Do not auto-enable a Monero node, mining, P2Pool or other always-on blockchain workloads unless explicitly requested; wallet tooling should not silently consume large storage/bandwidth.
- Do not reintroduce gaming packages unless explicitly requested.

## Nix contract

- Never guess filesystem UUIDs, partition identifiers or mount topology. `hosts/kraken/hardware-configuration.nix` stays a placeholder until generated on the actual machine.
- Prefer NixOS/Home Manager modules and pinned derivations over `curl | sh` installers.
- Before writing a custom AppImage, `.deb` or vendor-binary derivation, check nixpkgs and the upstream repository for a maintained Nix package/flake. Use the maintained package when it satisfies the requested channel/features.
- Preserve `flake.lock`; update pins only when the change is intentional.
- Prefer store-qualified executable paths in desktop entries when practical; Lua compositor binds may use profile commands that Home Manager guarantees.
- Keep unfree packages deliberate and documented.
- Keep the Apache/PHP/MariaDB development stack local-only unless explicitly asked otherwise.
- Keep custom Caelestia patches small and build-tested; do not fork the whole shell for one widget.
- Caelestia CLI theme flags are opt-out upstream: explicitly set every `enable*` flag so new rebuilds do not theme unrelated software by accident.
- Keep Zapret2 narrow by default: TLS ClientHello on TCP/443 with host autodetection. Broaden ports/interfaces only for a demonstrated need.

## Change checklist

1. Parse every changed `.nix` file with `nix-instantiate --parse`.
2. Parse every Hyprland Lua file with `luac -p`.
3. Run `nix flake metadata --no-write-lock-file`.
4. Evaluate `.#nixosConfigurations.kraken.config.networking.hostName` and expect `kraken`.
5. If touching ZCode, build `.#zcode` with unfree packages allowed.
6. If touching Cuprate packaging, build `.#cuprated`.
7. If touching Caelestia/QML/CodexBar, build the configured Caelestia package.
8. Keep README user-facing. Implementation guardrails and agent instructions belong here.
