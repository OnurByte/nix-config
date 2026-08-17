# apps settings

Vesper extends Caelestia's native Apps page instead of adding a separate settings application.

## permissions

Flatpak applications expose real per-app network and home-directory overrides through `flatpak override --user`.

Native Nix applications are shown as native and unsandboxed. Vesper does not pretend that Flatpak-style toggles can restrict a normal native process.

## wellbeing

`vesper-control wellbeing-daemon` samples the active Hyprland window class every five seconds and stores daily local counters under:

```text
~/.local/state/vesper/wellbeing/
```

No usage data is uploaded.

## adaptive icons

The current experimental toggle only enables a review queue. The original packaged icon is never overwritten automatically.

This queue is temporary scaffolding. `ADAPTIVE-ICONS.md` defines the replacement implementation: automatic `.desktop` discovery, real `Icon=` source resolution, AI-assisted canonical SVG conversion when needed, deterministic Original/Light/Dark/Tinted/Clear/Glass compilation and a generated freedesktop Vesper icon theme.

In the target UI, Apps keeps only per-application icon status and actions such as regenerate, retry, revert or exclude. Global AI generation controls belong in the AI page and global icon appearance belongs in the theme/appearance page.
