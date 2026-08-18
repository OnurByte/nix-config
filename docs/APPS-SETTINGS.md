# apps settings

Status: **partial**

This document owns Vesper-specific installed-application settings behavior and the handoff between `Settings -> Apps` and Vesper Store.

Vesper extends Caelestia's native Apps surface instead of adding a second settings application. Vesper Store is a separate native Qt 6 / QML application for discovery and installation.

`SETTINGS.md` owns where Apps, App Inspector and Wellbeing appear in the wider Settings information architecture.

## current state

Implemented Vesper-specific pieces include:

- `Find New Apps` in Apps, launching `vesper-store`;
- local wellbeing collection through `vesper-control wellbeing-daemon`;
- Flatpak network/home override controls where the backend can enforce them;
- native-app state that does not pretend ordinary Nix applications are Flatpak-sandboxed;
- per-app adaptive-icon status/actions through the existing Vesper app controls;
- shared adaptive-icon identity after an installed desktop entry is discovered.

Caelestia may provide base installed-app list/detail behavior independently of these Vesper extensions. Inspect the current QML and backend before assuming every target field or transaction below is already wired end to end.

The full ownership-aware remove/size/source transaction contract and App Inspector fields described later in this document are target behavior and depend on their corresponding backends becoming complete.

## ownership boundary

Use this split consistently:

```text
Vesper Store
  -> discover applications
  -> inspect catalogue metadata
  -> choose source
  -> plan/install

Settings -> Apps
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

Do not add a second installed-app management system to Vesper Store.

## Find New Apps

Current behavior: Apps exposes a `Find New Apps` action near the top of the page.

```text
Find New Apps
Discover and install applications with Vesper Store
```

The action launches `vesper-store`.

Prefer single-instance activation so an existing Store window is focused rather than duplicated.

Do not add `Open in Vesper Store` to installed application details. Once an application is installed, Apps is the intended management surface.

## installed application list

Target contract: Apps should show installed desktop applications discovered through real desktop-entry reconciliation.

Keep each row compact:

- active Vesper/adaptive icon;
- application name;
- short description when available;
- source/ownership only when it changes an available action.

Selecting a row should open the application detail view inside Settings.

Do not infer installed state from Store catalogue membership alone.

## App Inspector

The target application detail should evolve into an **App Inspector** rather than a page full of generic permission toggles.

When reliable sources exist, useful inspectable state includes:

- executable and package/source owner
- canonical desktop entry
- installed version and size
- native/Flatpak/sandbox ownership
- Wayland/XWayland state
- current processes
- CPU and memory use
- GPU activity
- current network connections
- autostart state
- file associations
- wellbeing usage
- adaptive-icon state

Unknown data stays unknown. Do not fabricate process, GPU, network or package ownership from application names alone.

The inspector can combine data from several local sources, but the backend owns attribution and normalization. QML should not scrape `/proc`, shell output or package-manager text directly.

## application detail

Target contract: the detail view is the canonical installed-app management surface.

The primary header should contain, when implemented and known:

- active icon, including the actual current Tinted appearance when Tinted mode is active;
- application name;
- short description;
- `Open` primary action;
- `Remove` destructive action only when the real source owner supports removal from this surface.

The main information may include:

- installed size;
- installed version;
- source and ownership;
- native or Flatpak sandbox state;
- App Inspector runtime state;
- wellbeing usage;
- per-app adaptive icon state and actions.

Do not duplicate Store screenshots, ratings, discovery categories or marketing metadata here. Those are discovery concerns.

### open

Target behavior: `Open` launches the resolved installed desktop entry rather than reconstructing a command from package metadata.

If several launchable desktop entries exist, use the canonical application desktop ID from the shared identity layer. Never execute arbitrary catalogue command text.

### remove

Removal must follow the real owner.

```text
Store-managed Nix app
  -> shared Vesper Store transaction core

Store-managed Flatpak
  -> Flatpak removal transaction

externally managed app
  -> no ownership claim

Vesper-config-managed app
  -> Managed by Vesper config
  -> do not silently edit home/yargc/apps.nix
```

This routing is a target contract until the corresponding Store transaction backend is implemented.

For removable applications, `Remove` should be explicit and destructive and use a small confirmation sheet naming the application.

After successful removal, desktop-entry and adaptive-icon state must be reconciled.

### installed size

Target behavior: show a human-readable installed size such as `184 MB` or `1.3 GB` only when a reliable source can provide it.

The backend owns the calculation.

For Store-managed Nix applications, use Nix store/closure information for the realized package. Dependencies may be shared, so this is realized closure size and not a promise about bytes freed by removal.

For Flatpak, use Flatpak's installed/deployed size information.

Do not estimate from download metadata. Show `Unknown` when the source cannot provide a trustworthy value.

### icon

Current Vesper per-app controls use the installed application identity and the adaptive-icon pipeline.

When adaptive icons are active, the detail surface should preview the actual active appearance rather than catalogue artwork. In Tinted mode that means the current tinted icon.

Keep per-app actions such as regenerate, retry, revert, export or exclude with the installed application. Global appearance/material selection remains in Appearance and global remote-generation controls remain in AI.

## permissions and sandboxing

Current Vesper backend behavior supports real user Flatpak overrides for the permissions it exposes, including network and home-directory access.

Native Nix applications are shown as native/unsandboxed. Vesper must not present Flatpak-style toggles as if they can restrict an ordinary native process.

Vesper Store does not own or duplicate the permission editor.

A future **Vesper sandbox launch profile** may add real isolation for selected native applications through an enforceable backend such as bubblewrap or systemd sandboxing.

Only after a real sandbox profile exists may Apps expose native restriction toggles such as network or home access for that launch path.

Rules:

- a permission toggle must correspond to a real enforcement mechanism
- native/unsandboxed must remain explicit when no sandbox is active
- do not imply that observing a process or network connection means Vesper can restrict it
- sandbox launch profiles must be reversible and must not silently rewrite the underlying application package

## wellbeing

Current behavior: `vesper-control wellbeing-daemon` samples the active Hyprland window class every five seconds and stores daily local counters under:

```text
~/.local/state/vesper/wellbeing/
```

No wellbeing usage data is uploaded by this feature.

Target wellbeing can grow into a local Digital Wellbeing surface with:

- daily and weekly graphs
- application categories
- category distribution such as coding/browser/social
- focus mode
- app timers
- break reminders

The existing foreground collector can remain a source where its granularity is sufficient.

Do not claim exact human attention time from foreground-window sampling alone.
Do not upload usage history merely to build charts, reminders or category summaries.

Focus/timer controls must have a real enforcement or notification path before they are shown as active restrictions.

## adaptive icons

`ADAPTIVE-ICONS.md` remains the single source of truth for adaptive icon discovery, conversion, rendering and appearance generation.

Store catalogue icons are read-only discovery assets before installation. After installation, the real `.desktop` entry is reconciled and the existing adaptive-icon pipeline owns the installed application identity.

Apps keeps per-application icon status and actions. It must not create a second Store-specific icon pipeline.

## implementation rule

When implementing any target behavior from this document:

1. inspect the current Caelestia Apps surface first;
2. extend rather than duplicate existing installed-app UI;
3. keep Store transaction logic in the shared Rust Store core, not QML;
4. keep App Inspector normalization and process attribution in a Vesper backend, not QML shell parsing;
5. keep source ownership explicit;
6. never expose a permission/restriction toggle without enforcement;
7. keep wellbeing local by default;
8. update this document's `current state` section when the feature actually lands.
