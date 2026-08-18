# apps settings

Vesper extends Caelestia's native Apps page instead of adding a separate settings application.

Vesper Store is a separate native Qt 6 / QML application used to discover and install new applications. `Settings -> Apps` is the canonical place to inspect and manage applications after installation.

## Find New Apps

Apps exposes a `Find New Apps` action near the top of the page.

```text
Find New Apps
Discover and install applications with Vesper Store
```

The action launches `vesper-store`. Prefer single-instance activation so an existing Store window is focused instead of opening duplicates.

Do not add `Open in Vesper Store` to installed application details. Once an application is installed its normal management surface is Apps.

`MARKETPLACE.md` is authoritative for Store architecture, package sources, catalogue identity and install transactions.

## installed application list

Apps shows the installed desktop applications known through real desktop-entry reconciliation.

Each row should stay compact:

- active Vesper/adaptive icon
- application name
- short description when available
- source/ownership only when it affects an action

Selecting a row opens the application detail view inside Settings.

## application detail

The detail view is the canonical installed-app management surface.

The primary header contains:

- active icon, including the current Tinted appearance when Tinted mode is active
- application name
- short description
- `Open` primary action
- `Remove` destructive action when the source is actually removable from this surface

The main information includes:

- installed size
- installed version when known
- source and ownership when relevant
- native or Flatpak sandbox state
- wellbeing usage
- per-app adaptive icon state and actions

Do not duplicate Store screenshots, ratings, discovery categories or marketing metadata here. This page is for the application that is already installed.

### open

`Open` launches the resolved installed desktop entry rather than reconstructing a command from package metadata.

If there are multiple launchable desktop entries, use the canonical application desktop ID from the shared identity layer. Never run arbitrary catalogue command text.

### remove

Removal follows the real owner:

- Vesper Store managed Nix app -> run the Store transaction backend to remove it from the Store manifest/profile
- Store managed Flatpak -> use the Flatpak removal transaction
- externally managed app -> do not claim ownership
- Vesper config managed app -> show `Managed by Vesper config` and do not silently edit `home/yargc/apps.nix`

For removable applications, `Remove` is explicit and destructive. Require a small confirmation sheet that names the application. Do not use a generic system-wide warning dialog.

After a successful removal, reconcile desktop entries and adaptive-icon state immediately.

### installed size

Show a human-readable installed size such as `184 MB` or `1.3 GB`.

The backend owns the calculation.

For Store-managed Nix applications, use Nix store/closure information for the realized package. Nix dependencies can be shared by several applications, so this number is the application's realized closure size rather than a promise that removing it will free exactly that many bytes.

For Flatpak, use Flatpak's installed/deployed size information.

Do not estimate size from download metadata and do not invent a value when the source cannot provide one. Show `Unknown` instead.

### icon

The detail header uses the same installed icon identity as the launcher.

When Vesper adaptive icons are active, preview the currently active appearance. In Tinted mode the detail page therefore shows the actual tinted icon rather than the original catalogue artwork.

Keep the existing per-app actions such as regenerate, retry, revert or exclude below the primary application information. Global appearance selection remains in Appearance and global AI generation controls remain in AI.

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

`ADAPTIVE-ICONS.md` remains the single source of truth for adaptive icon discovery, conversion and appearance generation.

Store catalogue icons remain ordinary read-only catalogue assets before installation. After installation the real `.desktop` entry is reconciled and the existing adaptive-icon pipeline takes over.

Apps keeps per-application icon status and actions. It does not create a second Store icon pipeline.
