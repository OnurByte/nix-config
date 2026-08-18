# Vesper Store

This document is the single source of truth for Vesper Store.

Vesper Store is a separate native desktop application for discovering, installing, removing and updating applications on Vesper.

It has its own window, process, desktop entry and icon. It is not embedded inside Settings.

Nixpkgs is the default and primary source. Flathub is optional and disabled by default.

## product boundary

Vesper Store is for user-facing desktop applications.

It is not a generic nixpkgs browser, NixOS option editor, flake editor, service manager or frontend for every derivation in nixpkgs.

The normal catalogue excludes libraries, development outputs, runtimes, kernels, drivers, language package sets, hidden helpers and services without a real desktop application.

Default source order:

1. Nixpkgs from the same locked revision used by Vesper
2. reviewed local Vesper recipes when a package alone is not enough
3. Flathub only after explicit opt-in

No apt, rpm, pacman or PackageKit backend belongs in Vesper Store.

Do not convert Nix packages into deb or rpm packages.

## upstream-first rule

Do not rebuild package infrastructure that already exists upstream.

Use existing projects and standards as building blocks:

- `NixOS/nixos-search` and `flake-info` for nixpkgs catalogue/export ideas
- `snowfallorg/nixos-appstream-data` for NixOS AppStream generation
- `snowfallorg/nix-software-center` as prior art for Nix GUI install/search/update behavior
- Nix itself for evaluation, substituters, builds, profiles and generations
- AppStream for store presentation metadata
- Flatpak CLI and remote AppStream metadata when optional Flathub support is enabled

Vesper-owned code should mostly be the native application UI, normalization/index layer, transaction policy, identity reconciliation and integration glue.

Do not write a second Nix resolver, binary cache client, AppStream ecosystem or package manager.

## native application contract

Application identity:

```text
name        Vesper Store
app id      io.vesper.Store
desktop id  io.vesper.Store.desktop
binary      vesper-store
```

The reverse-DNS ID may change once during implementation if packaging requires it, then it becomes stable.

Vesper Store must launch like a normal Linux application and appear in the launcher and dock.

It should support single-instance application activation and deep links to application details.

### required application stack

Vesper Store uses:

```text
GTK4
libadwaita
Rust
gtk-rs / libadwaita-rs
SQLite
Nix CLI / established Nix interfaces
```

The UI is native GTK. The backend is Rust.

Do not use:

- Qt
- QML
- Qt Quick
- Kirigami
- Electron
- Tauri/WebView
- embedded Chromium/WebKit as the primary UI
- localhost web frontends
- a Quickshell-only Store implementation

The Store should reuse upstream Rust crates and native libraries where practical rather than wrapping every operation in shell parsing.

Nix CLI invocation is acceptable where Nix has no stable library interface for the operation, but arguments must be constructed from validated catalogue identities rather than arbitrary user-provided Nix expressions.

## relationship with Settings -> Apps

Settings and Vesper Store have different jobs.

`Settings -> Apps` remains the installed-application control surface for:

- default applications
- real Flatpak permissions
- native/unsandboxed state
- wellbeing
- installed application identity
- per-app adaptive icon status and actions

Vesper Store owns:

- discovery
- search
- categories
- application detail
- source choice
- install/remove/update
- Store-owned rollback
- Store source settings

Do not duplicate permissions, wellbeing or adaptive-icon editors inside Vesper Store.

### Find New Apps

Add a `Find New Apps` row to `Settings -> Apps`.

Suggested text:

```text
Find New Apps
Discover and install applications with Vesper Store
```

The action launches `vesper-store`.

If application activation is implemented, use it instead of creating duplicate Store windows.

### Open in Vesper Store

For an installed application with a reliable catalogue identity, Settings may expose:

```text
Open in Vesper Store
```

Deep-link contract:

```text
vesper-store --app <catalogue-id>
```

Do not deep-link by display name.

Hide the action when identity is uncertain.

## visual contract

Vesper Store follows the Vesper Apple/visionOS-inspired controlled-glass language while remaining a normal GTK application.

The Store is not a clone of GNOME Software's visual structure just because it uses GTK/libadwaita.

Use libadwaita as the native widget/window foundation, then apply a small Vesper CSS/theme layer.

Use:

- calm palette-tinted surfaces
- selective translucency where GTK/compositor behavior makes it reliable
- generous continuous rounding
- soft shadows
- thin quiet borders
- restrained hierarchy
- large clean application artwork
- comfortable spacing
- smooth but short transitions

Avoid:

- neon multi-colour borders
- dense Material-dashboard cards
- opaque telemetry panels
- web-store layouts
- excessive blur on every row
- copying GNOME Software pixel-for-pixel

### glass rule

Vesper's strongest glass remains concentrated in shell surfaces.

For Vesper Store, prefer controlled application surfaces over forcing the whole window transparent.

Glass can be used for header/navigation overlays, sheets and floating controls when Wayland/compositor support is reliable. Readability wins over transparency.

### shared design values

Do not make the GTK application depend on Caelestia's QML internals.

Create a small Vesper GTK theme layer with shared semantic values derived from the active Vesper palette:

```text
surface
surface-container
surface-container-high
on-surface
on-surface-variant
primary
secondary-container
error
outline
radius-small
radius-medium
radius-large
radius-extra-large
spacing-small
spacing-medium
spacing-large
```

The Store may read exported Vesper palette state, but it must remain usable when Caelestia is not currently running.

## main window

Recommended desktop layout:

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Vesper Store                                      search      sources │
├──────────────┬───────────────────────────────────────────────────────┤
│ Discover     │ featured / useful categories                          │
│ Categories   │                                                       │
│ Installed    │ application grid/list                                 │
│ Updates      │                                                       │
│              │                                                       │
│              │ selected app opens a full detail view                 │
└──────────────┴───────────────────────────────────────────────────────┘
```

Use adaptive libadwaita navigation patterns for narrow windows rather than compressing desktop columns until they become unreadable.

Recommended primitives include `AdwApplication`, `AdwApplicationWindow`, `AdwNavigationSplitView` or current equivalent, `AdwToolbarView`, `AdwHeaderBar`, `AdwViewStack`, `AdwPreferencesGroup` for compact settings-like sections and standard GTK list/grid models.

Do not force a widget choice if the pinned libadwaita version has a better current replacement.

### search

Search is local and keyboard-first.

- `Ctrl+F` focuses search
- typing from Discover may focus search
- Escape clears/closes the active search/detail layer according to context
- Enter opens the selected result
- search must not evaluate all of nixpkgs per keystroke

Use SQLite FTS5 for fast local search.

### result presentation

A normal result needs only:

- icon
- application name
- one short summary
- source only when useful
- installed/update state
- primary action when appropriate

Do not expose package internals in every card.

Package attribute, architecture, license details, sandbox type and build warnings belong in the detail view or Advanced section.

### application detail

The detail view can show:

- large icon
- name
- summary
- description
- screenshots when available
- source
- package version
- installed version
- homepage
- license
- sandbox state
- local-build warning
- package attribute under Advanced
- Install / Remove / Update

If an app is declared by the main Vesper config, show:

```text
Installed
Managed by Vesper config
```

Do not offer a fake remove action.

If Store owns it, show `Managed by Vesper Store`.

If it is a Flatpak, show `Flatpak` and keep permissions in Settings -> Apps.

## catalogue architecture

Do not query `search.nixos.org` for every search.

The Nixpkgs catalogue is a local read-only artifact built for the same locked nixpkgs revision and architecture as Vesper.

Pipeline:

```text
locked nixpkgs
     │
     ├── flake-info / package metadata
     ├── NixOS AppStream data
     └── reviewed Vesper mappings
             │
             ▼
       catalogue builder
             │
             ▼
   normalized SQLite + icons
             │
             ▼
          Vesper Store
```

The final catalogue should be a Nix derivation so it follows `flake.lock` and participates in normal Vesper validation.

Suggested output:

```text
/nix/store/...-vesper-store-catalog/
└── share/vesper/store/
    ├── catalog.sqlite
    ├── catalog-meta.json
    └── icons/
```

`catalog-meta.json` contains at least:

```json
{
  "schemaVersion": 1,
  "system": "x86_64-linux",
  "nixpkgsRevision": "...",
  "generatedAt": "..."
}
```

Do not store large screenshot blobs in SQLite. Keep URLs and use a bounded lazy media cache.

## metadata authority

Nixpkgs is authoritative for:

- attribute path
- pname/version
- platform support
- broken state
- license
- known vulnerability/insecure state
- homepage
- main program where present

AppStream is used for presentation:

- component ID
- desktop ID
- display name
- generic name
- summary
- long description
- categories
- keywords
- screenshots
- application icon metadata

AppStream must not override the package version that Nix will actually install.

## catalogue eligibility

Prefer entries with strong desktop identity such as an AppStream desktop component or visible `Type=Application` desktop entry.

Hide from normal discovery:

- broken packages
- unsupported platforms
- libraries
- headers/dev outputs
- kernels/drivers
- language package sets
- fonts unless a future Fonts surface explicitly wants them
- service-only packages
- Flatpak runtimes
- `NoDisplay=true` helpers
- duplicate outputs of the same application

Known-insecure packages must not look like normal installable results. Block by default and explain why.

Vesper currently permits deliberate unfree packages, so the Store may list them while clearly showing license state.

## application identity and deduplication

Normalized identity should use:

1. AppStream component ID
2. desktop file ID
3. package attribute
4. Flatpak ID when applicable
5. reviewed alias mappings
6. canonical project/homepage only as a cautious fallback

Do not merge applications only because names look similar.

When the same app is available from Nixpkgs and enabled Flathub, present one application with multiple source variants only when identity is strong.

Nixpkgs remains the default variant.

## database shape

Suggested normalized model:

```text
apps
  id
  name
  generic_name
  summary
  description
  appstream_id
  desktop_id
  homepage
  icon_key
  primary_category

variants
  app_id
  source_kind
  source_id
  package_attr
  package_version
  flatpak_id
  license
  sandbox_kind
  install_kind
  supported
  broken
  insecure

categories
app_categories
screenshots
keywords
aliases
```

Use SQLite FTS5 for name, generic name, aliases, package attribute, keywords and summary.

Ranking should prefer exact name/alias matches, then prefix matches, then keywords/summary, then long description.

No AI service is required for search.

## Nix installation model

Store state must not be a pile of ad-hoc `nix profile install` commands with no desired-state record.

At the same time, Store installs must not rewrite hand-maintained `home/yargc/apps.nix` or deploy unrelated dirty changes from the Vesper checkout.

Use a Store-owned manifest plus dedicated Nix profile.

```text
manifest = desired Store-owned application state
profile  = realized environment generated from that state
```

Recommended paths:

```text
~/.config/vesper/store/manifest.json
~/.local/state/vesper/store/profile
~/.local/state/vesper/store/generations.json
~/.local/state/vesper/store/transactions/
~/.cache/vesper/store/media/
$XDG_RUNTIME_DIR/vesper/store.lock
```

The manifest contains only Store-owned choices.

Existing Vesper-config packages remain external to this manifest and are detected separately.

Do not store arbitrary Nix expressions in the manifest.

Package attributes must come from the trusted local catalogue.

## revision pinning

Vesper Store Nix installs must use the same nixpkgs revision as the running Vesper Store/catalogue build.

Do not let plain `nixpkgs#foo` silently resolve against a newer registry revision.

Expose the locked revision to Store packaging at build time.

When `flake.lock` changes, system, Store and catalogue move together.

```text
Vesper system revision
Vesper Store catalogue revision
Vesper Store package revision
```

These must stay coherent.

## transaction model

Every install/remove/update is serialized.

Flow:

1. acquire Store lock
2. validate current manifest
3. compute next desired state
4. resolve validated installables against the pinned revision
5. ask Nix for a dry-run plan
6. show local-build or policy warnings
7. realize the new package set
8. atomically switch the Store profile
9. atomically write the new manifest
10. reconcile desktop entries
11. let the existing adaptive-icon system discover new installed apps
12. release lock

If realization fails, previous manifest/profile stay active.

Never commit desired state before realization succeeds.

### binary cache behavior

Do not guess cache availability.

Ask Nix through dry-run/planning output.

If local builds are required, show a warning such as:

```text
Local build required
This application is not fully available from your configured binary caches.
```

The Store uses the machine's existing substituters and trusted public keys.

Never add a third-party binary cache automatically.

### rollback

Keep several known-good Store generations and matching manifest snapshots.

Rollback restores the complete Store-owned application set from a previous successful generation.

Do not attempt file-level rollback inside `/nix/store`.

The active and retained Store profiles must remain GC roots.

## apps already managed by Vesper

Applications already declared through Home Manager or NixOS remain owned by Vesper config.

Store shows them as installed but does not silently migrate them into Store state.

A future explicit migration feature can exist later if it performs a reviewed config change. It is not v1 work.

## packages requiring system integration

Store install classes:

```text
package
recipe
unsupported-system-integration
```

### package

Normal user-facing package that works from the Store-owned profile.

This is the v1 default.

### recipe

A reviewed local Vesper integration for an app that genuinely needs NixOS module/service/PAM/firewall/user-group changes.

Recipes are static repository code.

Remote catalogue metadata can never provide executable Nix.

Do not build a general recipe engine before real applications require it.

### unsupported-system-integration

If package-only installation would be misleading and no reviewed recipe exists, show the app but block one-click install with a clear explanation.

## Flathub

Flathub is optional and disabled by default.

Fresh Vesper Store behavior:

```text
Flatpak service available        yes
Flathub remote auto-added        no
Flathub catalogue downloaded     no
Flathub search results shown     no
Flathub preferred over Nixpkgs   no
Flathub beta enabled             no
```

Flatpak service availability does not mean Flathub is enabled.

### enabling Flathub

Expose a Sources page/sheet in Vesper Store:

```text
Nixpkgs     On    Vesper default
Flathub     Off   Optional sandboxed applications
```

Only after explicit opt-in may Store add/use the user Flathub remote and refresh its AppStream metadata.

Use official Flatpak remote operations and AppStream data. Do not scrape flathub.org pages or depend on an unofficial web API.

Prefer user-scope Flatpak installs for Store-owned apps unless a real system-wide requirement appears.

Disabling Flathub discovery must not silently uninstall installed Flatpaks or destroy their update path.

## Flatpak ownership

Flatpak remains authoritative for Flatpak install/remove/update state.

Store merges Flatpak identity into its application model but does not pretend those apps are Nix-owned.

After Flatpak installation, Settings -> Apps remains the authority for per-app network/home overrides.

## adaptive icon integration

Do not run the entire Store catalogue through adaptive-icon AI generation.

Before install, use normal AppStream/catalogue artwork.

After install, the real desktop entry appears and the existing Vesper adaptive-icon pipeline takes ownership as usual.

There is one adaptive-icon source of truth.

## backend shape

The Store application should be split into small Rust modules rather than one huge command file.

Likely shape:

```text
home/yargc/packages/vesper-store/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── catalog.rs
│   ├── identity.rs
│   ├── nix.rs
│   ├── transactions.rs
│   ├── flatpak.rs
│   └── state.rs
├── resources/
│   ├── io.vesper.Store.desktop
│   ├── io.vesper.Store.metainfo.xml
│   ├── style.css
│   └── icons/
└── ui/
    └── GTK composite templates or narrowly scoped UI resources
```

Exact layout may change to match the selected gtk-rs architecture.

Keep business logic testable without constructing GTK widgets.

## Settings integration implementation

The existing Caelestia Apps QML remains QML because it belongs to the existing Quickshell shell.

That is not permission to use Qt/QML for the Store itself.

`VesperAppsSettings.qml` should gain a normal `Find New Apps` action that launches `vesper-store`.

Per-app controls may gain `Open in Vesper Store` when identity resolution is reliable.

This is a shell integration exception to the first-party standalone-app GTK rule.

## updates

Nix Store-owned applications update only when Vesper moves to a new locked nixpkgs revision and the resolved package changes.

Vesper Store must not independently chase nixpkgs unstable ahead of the system lock.

Flatpak updates may move independently after Flathub opt-in because Flatpak owns that source.

Global system/component update presentation remains under `System -> Updates`. Vesper Store may still show app-level update state and its own Installed/Updates section.

## network and privacy

Nixpkgs discovery is local.

Network is used only for actions such as:

- package installation/update through configured Nix substituters
- lazy screenshot/media fetches
- optional Flathub metadata refresh after opt-in
- Flatpak installation/update after opt-in

No telemetry, popularity beacon, recommendation tracking or remote search-query logging is needed.

Keep screenshot/media caching bounded.

## security

Vesper Store is not a new trust root.

For Nix:

- use locked nixpkgs
- use existing substituters
- use existing trusted keys
- respect broken/platform/insecure policy
- never auto-add binary caches

For Flatpak:

- use configured remote trust
- add Flathub only after explicit opt-in
- do not execute arbitrary `.flatpakref` URLs from catalogue descriptions

For recipes:

- executable Nix lives only in reviewed local repository files
- remote metadata maps only to known local recipe IDs

Never execute shell commands copied from application descriptions.

## failure behavior

Catalogue missing:

- Store opens with a clear local catalogue error
- Settings -> Apps still works

Revision mismatch:

- browsing may remain available with a stale warning
- block Nix mutations until backend/catalogue revisions agree

Nix unavailable:

- report the integrity failure
- never fall back to curl installers

Flathub unavailable:

- Nixpkgs remains fully usable

Build failure:

- keep previous Store generation active
- retain useful error details

Corrupted manifest:

- refuse mutations
- preserve the bad file
- offer repair instead of silently replacing it with an empty manifest

## performance targets

- Store launch must not perform a full nixpkgs evaluation
- local search should feel immediate
- scrolling must not spawn one process per row
- screenshots load lazily
- install time is controlled by Nix/network, so show real phases instead of fake percentages

Transaction phases may include:

```text
planning
resolving
fetching
building
installing
reconciling
complete
failed
```

## implementation phases

### phase 0 - upstream and catalogue proof

Prove against the current lock:

- attr -> AppStream identity
- attr -> desktop ID
- attr -> icon
- duplicate variants
- unfree app
- missing AppStream data
- local-build case
- dedicated Store profile surviving reboot and GC

### phase 1 - native GTK Store shell

Implement:

- `vesper-store` Rust package
- GTK4/libadwaita application/window
- `.desktop` and AppStream metainfo
- Vesper GTK CSS/theme adapter
- local catalogue open/search
- Discover/Categories/detail navigation
- launcher integration
- `Find New Apps` in Settings -> Apps

No install button until identity mapping is reliable.

### phase 2 - Nix transactions

Implement:

- Store manifest
- dedicated Nix profile
- transaction lock
- dry-run planning
- install/remove
- rollback
- installed-app reconciliation
- adaptive-icon handoff

### phase 3 - updates

Implement:

- lock/catalogue/profile comparison
- update availability
- reconcile after Vesper nixpkgs updates
- retained generations
- integration with System Updates where useful

### phase 4 - optional Flathub

Only after the Nix path is stable:

- Sources UI
- explicit opt-in
- user Flathub remote management
- local normalized Flatpak/AppStream cache
- strong-identity deduplication
- Flatpak install/remove/update
- Settings permission handoff

### phase 5 - reviewed recipes

Only for real apps that prove package-only installation is insufficient.

### phase 6 - polish

- keyboard navigation
- accessibility
- reduced motion
- media cache controls
- diagnostics
- empty/error states
- source comparison details

Do not prioritize ratings, recommendation feeds or decorative complexity over install reliability.

## validation

Implementation must follow the normal Vesper repository checklist plus Store-specific checks.

Required:

- no first-party Python
- no Qt/QML in the standalone Store implementation
- compile all Store Rust code
- build the Store derivation
- validate `.desktop` and AppStream metainfo
- test GTK launch under Wayland
- test search without network
- test install/remove/rollback
- test active Store generations against GC
- build Caelestia when Settings integration QML changes
- build the full Vesper system

## decisions locked for v1

- product name is `Vesper Store`
- Vesper Store is a separate native application
- standalone Store UI is GTK4 + libadwaita
- backend is Rust
- Qt/QML is forbidden in the Store implementation
- Caelestia Settings integration remains QML because it is part of the existing shell
- Settings -> Apps contains `Find New Apps`
- installed apps may deep-link to Vesper Store by stable ID
- Nixpkgs is the primary/default source
- catalogue search is local
- catalogue/package revision follows Vesper's lock
- normal Store packages are user-level Nix installs in Store-owned state
- Store never edits hand-maintained `apps.nix` for normal installs
- existing config-managed apps are not silently migrated
- Flathub is opt-in and absent from discovery by default
- Nixpkgs wins duplicate-source preference
- no PackageKit
- no apt/rpm conversion
- no AI requirement for search
- no automatic third-party caches
- no remote executable recipes
- no fake sandbox controls for native Nix apps
- no duplicate adaptive-icon pipeline
- upstream package/index/cache mechanisms are reused instead of reimplemented

## research references

Primary upstream references for implementation:

- `NixOS/nixos-search`
- `snowfallorg/nix-software-center`
- `snowfallorg/nixos-appstream-data`
- Nix reference and profile/build/substituter documentation
- GTK4 documentation
- libadwaita documentation
- gtk-rs and libadwaita-rs documentation
- AppStream specification
- Flatpak command and repository documentation
- freedesktop Desktop Entry specification
