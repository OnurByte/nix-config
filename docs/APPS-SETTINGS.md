# apps settings

Vesper extends Caelestia's native Apps page instead of adding a separate settings application.

Vesper Store is a separate native Qt 6 / QML application. Apps remains the installed-application control surface and links into the Store where discovery or Store details are needed.

## Store integration

Apps exposes a `Find New Apps` action near the top of the page.

```text
Find New Apps
Discover and install applications with Vesper Store
```

The action launches `vesper-store`. Prefer single-instance activation so an existing Store window is focused instead of opening duplicates.

For an installed application with a reliable Store catalogue identity, the per-app page may expose:

```text
Open in Vesper Store
```

using the stable deep-link contract:

```text
vesper-store --app <catalogue-id>
```

Do not resolve Store links by display name. Hide the action when identity is uncertain.

`MARKETPLACE.md` is authoritative for Store architecture, package sources, catalogue identity and transactions.

## permissions

Flatpak applications expose real per-app network and home-directory overrides through `flatpak override --user`.

Native Nix applications are shown as native and unsandboxed. Vesper does not pretend that Flatpak-style toggles can restrict a normal native process.

Vesper Store does not duplicate the permission editor. Installed Flatpaks continue to use Apps for these controls.

## wellbeing

`vesper-control wellbeing-daemon` samples the active Hyprland window class every five seconds and stores daily local counters under:

```text
~/.local/state/vesper/wellbeing/
```

No usage data is uploaded.

## adaptive icons

The current experimental toggle only enables a review queue. The original packaged icon is never overwritten automatically.

This queue is temporary scaffolding. `ADAPTIVE-ICONS.md` defines the replacement implementation: automatic `.desktop` discovery, real `Icon=` source resolution, AI-assisted canonical SVG conversion when needed, deterministic Original/Light/Dark/Tinted/Clear/Glass compilation and a generated freedesktop Vesper icon theme.

Store catalogue icons remain normal catalogue assets before installation. Installing an app creates or exposes its real desktop entry, then the existing adaptive-icon discovery pipeline takes over.

In the target UI, Apps keeps only per-application icon status and actions such as regenerate, retry, revert or exclude. Global AI generation controls belong in the AI page and global icon appearance belongs in the theme/appearance page.
