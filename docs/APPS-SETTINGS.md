# apps settings

Status: **partial**

This document owns Vesper-specific installed-application settings behavior, default-application handling, the Spotlight/launcher boundary, and the handoff between `Settings -> Apps` and Vesper Store.

Vesper extends Caelestia's native Apps surface instead of adding a second settings application. Vesper Store is a separate native Qt 6 / QML application for discovery and installation. [Vicinae](https://github.com/vicinaehq/vicinae) is the default Spotlight-style launcher/search surface; it is not an application manager or package owner.

`SETTINGS.md` owns where Apps, App Inspector and Wellbeing appear in the wider Settings information architecture.

## current state

Implemented Vesper-specific pieces include:

- `Find New Apps` in Apps, launching `vesper-store`;
- Vicinae enabled as the default Spotlight-style launcher with a Home Manager user systemd service; the Hyprland session bridge starts and stops `graphical-session.target` for that service;
- bare `Super` toggling Vicinae instead of the Caelestia app launcher, with `Super + Space` retained as an alternate;
- a `Settings -> Vicinae` page for launcher behavior, Vesper theme/accent sync, controlled-glass opacity and Vesper adaptive icons;
- the installed Apps list using the existing Quickshell `DesktopEntries` registry;
- category filtering derived from real desktop-entry `Categories` metadata;
- installed-app rows showing icon, name, description, default-role state and an inline Open action;
- application detail exposing canonical Open plus desktop-entry categories/startup class/default-role information;
- user Flatpak removal through the Vesper control backend with an explicit confirmation step;
- local wellbeing collection through `vesper-control wellbeing-daemon`;
- Flatpak network/home override controls where the backend can enforce them;
- native-app state that does not pretend ordinary Nix applications are Flatpak-sandboxed;
- App Inspector ownership state resolved from the winning desktop entry, its `X-Flatpak`/export evidence and `flatpak info --user/--system` scope;
- per-app adaptive-icon status/actions through the existing Vesper app controls;
- shared adaptive-icon identity after an installed desktop entry is discovered.

The Default Agent selector, physical Assistant/Copilot-key handling and Quake Agent Console are not implemented in the current Hyprland/QML tree. Their shared contract is target work owned by `DESKTOP-ERGONOMICS.md`; the current AI HUD shortcut is `Super + U -> codexbar-popup`.

Caelestia may provide base installed-app list/detail behavior independently of these Vesper extensions. Inspect the current QML and backend before assuming every target field or transaction below is already wired end to end.

The ownership-aware Nix Store removal/size/source transaction contract remains target behavior until the Store transaction backend becomes complete. Flatpak owner/scope state is now reported only when the effective desktop entry and installed Flatpak scope agree.

## ownership boundary

Use this split consistently:

```text
Vicinae
  -> default Spotlight / command-palette surface
  -> app search and launch
  -> file/command/extension search where enabled
  -> never owns package removal or Vesper defaults

Vesper Store
  -> discover applications
  -> inspect catalogue metadata
  -> choose source
  -> plan/install

Settings -> Apps
  -> default applications and generic handlers
  -> browse/filter installed applications
  -> inspect installed applications
  -> launch
  -> installed source/ownership state
  -> remove when the real owner supports it
  -> real enforceable permissions
  -> app/runtime inspection
  -> wellbeing
  -> adaptive icon controls
```

`MARKETPLACE.md` is authoritative for Store architecture, package sources, catalogue identity and install transactions.
`ADAPTIVE-ICONS.md` is authoritative for adaptive icon discovery, conversion and appearance semantics.
`DESKTOP-ERGONOMICS.md` is authoritative for the Quake Agent Console and physical Assistant/Copilot-key interaction.

Do not add a second installed-app management system to Vesper Store or Vicinae.

## Vicinae as the default Spotlight surface

Vicinae is Vesper's primary keyboard-first launcher/search surface, analogous to the role Spotlight plays on macOS.

Default entry point:

```text
Super -> vicinae toggle
Super + Space -> vicinae toggle (alternate)
```

Vesper uses the packaged Nix/Home Manager integration and runs the Vicinae server as a user service. The Hyprland Lua session bridge imports the Wayland environment, starts `graphical-session.target` on session start and stops it on shutdown; do not introduce a separate hand-written daemon wrapper when the upstream/Home Manager service is sufficient.

Vesper owns a small imported runtime file at `~/.config/vicinae/vesper.json`. `vesper-control vicinae-sync-theme` regenerates it and the XDG data themes at `$XDG_DATA_HOME/vicinae/themes/vesper-light.toml` and `vesper-dark.toml` (normally `~/.local/share/vicinae/themes`) from the active Caelestia scheme and `primary` accent. Settings writes only the Vesper state at `~/.config/vesper/vicinae.conf`; it does not rewrite arbitrary Nix source or the user's whole Vicinae configuration.

The initial integration intentionally keeps ownership narrow:

- Vicinae is primary for Spotlight-style search, application launch and command-palette workflows;
- Caelestia remains the desktop shell and keeps shell-native panels, settings, clipboard, emoji, capture and other existing capabilities unless a later migration explicitly moves one;
- do not run two competing primary launcher shortcuts;
- the old Caelestia launcher may remain available internally, but bare `Super` belongs to Vicinae and `Super + Space` remains its compatibility alias;
- Vesper-specific actions should integrate with Vicinae through supported commands/deeplinks/extensions rather than forking Vicinae when practical;
- Vicinae theme integration follows Vesper appearance work without creating a second independent visual identity database;
- the Vicinae Settings page exposes only controls enforced by the imported runtime configuration: focus-loss behavior, root reset, layer-shell, controlled-glass opacity, theme following and adaptive-icon following.

Vicinae's application index and Settings -> Apps both ultimately refer to desktop applications, but Vesper must not create a third app identity database merely to reconcile them. Canonical desktop IDs and `.desktop` metadata remain the common identity layer.

## desktop-entry metadata contract

Installed applications are described from real desktop entries rather than guessed from package names.

Example:

```ini
[Desktop Entry]
Type=Application
Name=My Application
Comment=A description of what the app does
Exec=/absolute/path/to/executable %U
Icon=/absolute/path/to/icon.png
Terminal=false
Categories=Utility;Development;
StartupWMClass=AppName
```

Vesper maps the fields as follows:

```text
Desktop ID       -> canonical installed-app identity
Name             -> application title
Comment          -> primary description
GenericName      -> description fallback when Comment is absent
Icon             -> normal icon resolution -> Vesper adaptive-icon layer when active
Categories       -> Apps category filtering
StartupWMClass   -> window-identity hint, never sole package ownership proof
Exec             -> parsed by the desktop-entry implementation; never execute raw text from QML
Terminal         -> launch semantics owned by the desktop-entry implementation
```

Rules:

- use Quickshell `DesktopEntry` / the canonical desktop-entry resolver already used by Caelestia;
- launch with the resolved desktop entry (`execute()` or equivalent canonical launch path), not by evaluating raw `Exec=` text;
- do not infer package ownership from `Name`, `StartupWMClass` or executable basename alone;
- preserve unknown/empty metadata instead of inventing descriptions or categories;
- adaptive icons may replace presentation of `Icon`, but do not replace the underlying desktop identity.

## Find New Apps

Apps exposes a `Find New Apps` action near the top of the page.

```text
Find New Apps
Discover and install applications with Vesper Store
```

The action launches `vesper-store`.

Prefer single-instance activation so an existing Store window is focused rather than duplicated.

Do not add `Open in Vesper Store` to installed application details. Once an application is installed, Apps is the intended management surface.

## Default Apps

`Settings -> Apps` exposes a native **Default Apps** section for user-level generic handlers. This is a Settings surface, not Vicinae or Store catalogue metadata.

Target structure:

```text
Settings
└── Apps
    └── Default Apps
        ├── Terminal
        ├── Audio
        ├── Media playback
        ├── File manager
        ├── other supported handlers as they become real
        └── Default Agent
```

Installed-app rows/details may display a compact `Default` indicator such as:

```text
Default: Terminal
Default: File manager
```

Do not expose an ambiguous one-click `Make Default` action when an application could fill several roles. A future per-app default action must name the exact role and update the same canonical Default Apps setting.

### Default Agent

`Default Agent` selects which installed supported coding-agent runtime handles the generic Vesper "open my coding agent" intent.

Candidate values are derived from supported installed runtimes rather than a hard-coded provider list. Examples may include, only when installed/supported:

```text
Codex
Claude Code
OpenCode
Gemini CLI
GitHub Copilot CLI
Pi
other Vesper-supported agent runtimes
None
```

Rules:

- `None` is a valid explicit state;
- do not hard-code Codex, Claude, GitHub Copilot or any other provider as the default;
- the selected runtime is an application/agent default, not a provider/model preference;
- provider credentials, model/router policy and per-agent capabilities remain owned by `Settings -> AI`;
- the generic agent launcher, Quake Agent Console and physical Assistant/Copilot key all consume this one canonical selection;
- the physical Copilot key does not imply `GitHub Copilot CLI`;
- do not store separate defaults for keyboard hardware, launcher and console;
- if the selected agent is removed or stops being supported, show it as unavailable and require an explicit new selection rather than silently launching a different agent;
- persist through the normal Vesper settings/config ownership path;
- use native Caelestia/Nexus settings components and Vesper semantic tokens.

## installed application library

`Settings -> Apps -> All apps` is the canonical installed-app browser.

Each row should stay compact and native to Nexus:

```text
[icon]  Application name                         [Open] [>]
        Comment / GenericName
        Default: <role>        # only when applicable
```

Required row data:

- active Vesper/adaptive icon when available, otherwise resolved packaged icon;
- application `Name`;
- `Comment`, falling back to `GenericName`;
- compact default-role indicator only when the app is actually a current default;
- existing favourite state when applicable;
- inline **Open** action using the canonical desktop entry;
- navigation to App Inspector/detail.

Do not infer installed state from Store catalogue membership alone.

### category filter

Category filtering uses `.desktop` `Categories` values directly.

Vesper's user-facing categories are:

```text
All
Development
Internet
Office
Graphics
Audio & Video
Games
Utilities
System
Other
```

Mapping:

```text
Development  <- Development
Internet     <- Network, WebBrowser, Email
Office       <- Office
Graphics     <- Graphics
Audio & Video <- AudioVideo, Audio, Video
Games        <- Game
Utilities    <- Utility
System       <- System, Settings
Other        <- no recognized major category above
```

Rules:

- an app with multiple categories may appear under every matching filter;
- `Other` is a deterministic metadata fallback, not an AI classification bucket;
- do not classify apps from their names/descriptions merely to make the category list look complete;
- filtering changes the view only; it does not mutate desktop entries.

## App Inspector

The application detail should evolve into an **App Inspector** rather than a page full of generic permission toggles.

The header should contain the active icon, application name and short description. Primary application controls belong near the top.

Useful inspectable state, when reliable sources exist, includes:

- canonical desktop entry / desktop ID;
- categories;
- StartupWMClass/window identity hint;
- executable and package/source owner;
- installed version and size;
- native/Flatpak/sandbox ownership;
- current default roles;
- Wayland/XWayland state;
- current processes;
- CPU and memory use;
- GPU activity;
- current network connections;
- autostart state;
- file associations;
- wellbeing usage;
- adaptive-icon state.

Unknown data stays unknown. Do not fabricate process, GPU, network or package ownership from application names alone.

The inspector can combine data from several local sources, but the backend owns attribution and normalization. QML should not scrape `/proc`, shell output or package-manager text directly.

## application actions

### Open

`Open` launches the resolved installed desktop entry.

Never reconstruct a shell command from raw `Exec=` text. The desktop-entry implementation owns field-code parsing and working-directory semantics.

The installed-app list may expose a compact inline Open button; the detail view also exposes Open as a primary action.

### Remove

Removal always follows the real owner and always requires explicit user intent.

Current enforceable behavior:

```text
user-installed Flatpak
  -> Vesper backend
  -> flatpak uninstall --user
  -> explicit confirmation in Apps
```

The backend resolves the effective desktop entry first. A Flatpak ID is accepted only from that entry's `X-Flatpak` value or Flatpak export path and must then pass `flatpak info --user` or `flatpak info --system`. A matching ID in another, lower-precedence desktop source cannot grant permissions or removal controls.

Target routing as Store ownership becomes complete:

```text
Store-managed Nix app
  -> shared Vesper Store transaction core

Store-managed/user Flatpak
  -> Flatpak removal transaction

externally managed app
  -> no ownership claim / no fake Remove button

Vesper-config-managed app
  -> Managed by Vesper config
  -> do not silently edit home/yargc/apps.nix
```

Rules:

- do not show an enabled Remove button unless the backend can identify and invoke the actual owner;
- destructive removal requires confirmation naming the application;
- a system/global Flatpak is not the same as a user Flatpak and must not be removed through the user-only path;
- successful removal triggers normal desktop-entry/adaptive-icon reconciliation;
- removal must never be implemented by deleting a `.desktop` file while leaving the package installed.

### installed size

Show a human-readable installed size such as `184 MB` or `1.3 GB` only when a reliable source can provide it.

The backend owns the calculation.

For Store-managed Nix applications, use Nix store/closure information for the realized package. Dependencies may be shared, so this is realized closure size and not a promise about bytes freed by removal.

For Flatpak, use Flatpak's installed/deployed size information.

Do not estimate from download metadata. Show `Unknown` when the source cannot provide a trustworthy value.

## permissions and sandboxing

The Vesper backend supports real user Flatpak overrides for the permissions it exposes, including network and home-directory access.

Native Nix applications are shown as native/unsandboxed. Vesper must not present Flatpak-style toggles as if they can restrict an ordinary native process.

Vesper Store and Vicinae do not own or duplicate the permission editor.

A future **Vesper sandbox launch profile** may add real isolation for selected native applications through an enforceable backend such as bubblewrap or systemd sandboxing.

Only after a real sandbox profile exists may Apps expose native restriction toggles for that launch path.

Rules:

- a permission toggle must correspond to a real enforcement mechanism;
- native/unsandboxed must remain explicit when no sandbox is active;
- do not imply that observing a process or network connection means Vesper can restrict it;
- sandbox launch profiles must be reversible and must not silently rewrite the underlying application package.

## wellbeing

`vesper-control wellbeing-daemon` samples the active Hyprland window every five seconds, resolves its class through the adaptive-icon canonical identity inventory, and stores daily local counters under:

```text
~/.local/state/vesper/wellbeing/
```

No wellbeing usage data is uploaded by this feature.

The collector now samples only when `loginctl show-session` reports both
`IdleHint=no` and `LockedHint=no` for `XDG_SESSION_ID`. If the session owner
cannot provide those signals, the sample is skipped rather than treated as
active attention.

When the canonical identity inventory is unavailable, the collector keeps the
exact runtime id as a bounded fallback. App summaries use exact canonical or
runtime keys rather than fuzzy name matching.

The remediation contract is:

- keep idle/lock truth on the logind session owner;
- do not increment application usage while the session is idle, locked or unknown;
- do not backfill missed samples as if continuous attention were proven;
- keep foreground sampling explicitly approximate even after idle/lock gating;
- prefer cached/event-driven session state over adding another fast subprocess poll.

Target wellbeing can grow into a local Digital Wellbeing surface with daily/weekly graphs, categories, focus mode, app timers and break reminders.

Do not claim exact human attention time from foreground-window sampling alone.
Do not upload usage history merely to build charts, reminders or category summaries.

## adaptive icons

`ADAPTIVE-ICONS.md` remains the single source of truth for adaptive icon discovery, conversion, rendering and appearance generation.

Store catalogue icons are read-only discovery assets before installation. After installation, the real `.desktop` entry is reconciled and the existing adaptive-icon pipeline owns the installed application identity.

Apps keeps per-application icon status and actions. Vicinae and Vesper Store must not create parallel adaptive-icon pipelines.

## implementation rules

When implementing or extending this surface:

1. inspect the current Caelestia Apps/DesktopEntries behavior first;
2. extend rather than duplicate existing installed-app UI and identity;
3. keep Vicinae as the primary Spotlight/command-palette surface, not an installed-app manager;
4. keep bare `Super` as the primary launcher binding and `Super + Space` as its alternate rather than adding a competing surface;
5. use `.desktop` metadata for icon/name/description/category/window hints;
6. use canonical desktop-entry execution, never raw `Exec=` shell evaluation;
7. keep Default Agent and other defaults in the existing/native Default Apps surface;
8. keep Store transaction logic in the shared Rust Store core, not QML;
9. keep App Inspector normalization and process attribution in a Vesper backend, not QML shell parsing;
10. keep source ownership explicit and never expose Remove without a real owner;
11. never expose a permission/restriction toggle without enforcement;
12. keep wellbeing local by default;
13. update this document's `current state` section when behavior changes.
