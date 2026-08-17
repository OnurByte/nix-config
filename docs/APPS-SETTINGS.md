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

The experimental adaptive icon toggle only enables a review queue. The original packaged icon is never overwritten automatically.
