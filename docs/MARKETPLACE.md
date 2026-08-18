# Vesper Store

This document is the single source of truth for Vesper Store.

Vesper Store is a separate native desktop application for discovering and installing new desktop applications on Vesper.

The application name is exactly `Vesper Store`.

Nixpkgs is the default and primary source. Flathub is optional and disabled by default.

Installed application management belongs to `Settings -> Apps`, not to a second management UI inside Vesper Store.

## product boundary

Vesper Store is the discovery and installation surface.

It owns:

- search
- categories
- application discovery
- pre-install application information
- source selection
- install planning
- install transactions
- optional Flathub source settings
- the shared transaction/core code used when Settings removes a Store-managed application

`Settings -> Apps` owns installed application management after installation:

- installed application list
- name and description
- active/tinted icon
- Open
- Remove when the real owner supports removal
- installed size
- installed version
- source/ownership state
- permissions
- wellbeing
- adaptive icon controls

Vesper Store must not grow a second installed-app detail system that competes with Apps.

The Store is not a generic nixpkgs browser, NixOS option editor, flake editor, service manager or frontend for every derivation in nixpkgs.

Normal discovery contains user-facing desktop applications. Libraries, headers, development outputs, runtimes, kernels, drivers, language package sets, services without a desktop application and hidden helpers stay out of normal results.

Default source priority:

1. Nixpkgs from the same locked revision used by Vesper
2. reviewed Vesper integration recipes when a package alone is not enough
3. Flathub only after explicit user opt-in

There is no apt, rpm, pacman or PackageKit backend.

There is no conversion from Nix packages into deb or rpm packages.

If Vesper Store is ever reused outside NixOS, Nix should remain the application layer instead of translating packages into the host distribution format.

## native application contract

Vesper Store is a normal native application with its own process, window, application ID, desktop entry and icon.

Target identity:

```text
name        Vesper Store
app id      io.vesper.Store
desktop id  io.vesper.Store.desktop
binary      vesper-store
```

The reverse-DNS ID may change once before implementation if packaging requires it. After persisted state exists it is stable.

Vesper Store appears in the launcher and can be pinned to the dock like any other application.

### standard UI stack

Vesper Store uses:

```text
Qt 6
Qt Quick
QML
Rust backend
SQLite
Nix CLI / established Nix interfaces
```

Qt/QML is the standard Store presentation stack.

GTK and libadwaita are not globally forbidden in Vesper. They are simply not the Store UI stack.

Do not implement Vesper Store with Electron, Tauri/WebView, an embedded browser, a localhost web frontend or a Store page that only exists while Caelestia is running.

Recommended shape:

```text
Qt Quick / QML application
          │
          ▼
       Rust core
          │
          ├── local SQLite catalogue
          ├── Nix planning and transactions
          ├── application identity
          ├── installed-state reconciliation
          ├── source adapters
          └── rollback state
```

QML is presentation logic. Do not put Nix parsing, Flatpak output parsing, SQLite query construction or transaction state machines in QML JavaScript.

Use a maintained Rust/Qt integration approach that builds cleanly from pinned nixpkgs. Keep the bridge narrow and typed.

## upstream-first rule

Do not rebuild package infrastructure that already exists upstream.

Use existing projects and standards as building blocks.

### NixOS/nixos-search and flake-info

Reuse the package export/index concepts and `flake-info` where appropriate.

The important lesson from NixOS Search is that a giant client-side package JSON stopped scaling. Vesper Store therefore does not evaluate all of nixpkgs for each query and does not call `search.nixos.org` on every keystroke.

### snowfallorg/nixos-appstream-data

Reuse or adapt the established NixOS AppStream generation path when it maps correctly to Vesper's pinned nixpkgs revision.

Do not build a new AppStream ecosystem from scratch.

### snowfallorg/nix-software-center

Use it as prior art for Nix GUI-store metadata mapping, installation behavior and edge cases.

Reuse code only when license and architecture make that cleaner than a small Vesper layer.

Do not fork its GTK UI. Vesper Store has its own Qt/QML UI and Vesper visual language.

### Nix itself

Nix remains authoritative for:

- dependency closure resolution
- binary substituters
- trusted keys
- realization
- package profiles/generations where used
- actual build failures

Do not create a second resolver, downloader or binary-cache protocol.

### Flatpak itself

When optional Flathub support is enabled, use Flatpak's native CLI, remote and AppStream mechanisms.

Do not scrape flathub.org and do not depend on an unofficial web API for core behavior.

## relationship with Settings -> Apps

The integration is intentionally one-way for discovery and shared for state.

```text
Settings -> Apps
       │
       └── Find New Apps
               │
               ▼
          Vesper Store
               │
             Install
               │
               ▼
       Settings -> Apps
       manages installed app
```

### Find New Apps

Add a prominent `Find New Apps` action near the top of `Settings -> Apps`.

Suggested row:

```text
Find New Apps
Discover and install applications with Vesper Store
```

It launches:

```text
vesper-store
```

Prefer single-instance activation so an existing Store window is focused.

### no Open in Vesper Store on installed details

Do not add `Open in Vesper Store` to installed application details.

Once installed, Apps is the canonical detail and management surface.

The Store can still show `Installed` on a search result or pre-install detail when reconciliation says the application already exists. It should not recreate the Apps management page there.

If a future stable Caelestia deep-link exists, an already-installed result in Store may offer `Manage` that opens the corresponding Apps detail. This is optional and must route to Settings rather than duplicating controls.

### shared application identity

Store and Apps use one compatible identity model.

Preferred identity keys:

1. AppStream component ID
2. desktop file ID
3. Nix package attribute when known
4. Flatpak application ID when relevant
5. reviewed aliases for known mismatches

A successful Store install must become visible in Apps immediately after desktop-entry reconciliation.

Do not create a second incompatible installed-app registry in Store.

## Store UX

The Store should stay focused on finding and adding applications.

Recommended navigation:

```text
Vesper Store
├── Discover
├── Categories
├── Search
└── Sources
```

Do not add a full `Installed` manager or a second per-app settings section.

Global system updates remain under `System -> Updates`.

### wide layout

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Vesper Store                                                  window │
├────────────────┬─────────────────────────────────────────────────────┤
│ Search         │ Featured / category / search results                │
│                │                                                     │
│ Discover       │  app      app      app                              │
│ Categories     │                                                     │
│                │  selected catalogue detail                          │
│ Sources        │  screenshots                                        │
│                │  description                                        │
│                │  source  version  sandbox                           │
│                │                                      [ Install ]    │
└────────────────┴─────────────────────────────────────────────────────┘
```

Narrow layouts use normal page navigation rather than crushing navigation, results and detail into tiny columns.

### result presentation

A result row/tile should contain only what helps discovery:

- catalogue icon
- name
- short summary
- source only when relevant
- installed state if already present
- one primary action

Do not put closure size, maintainers, package attr, architecture or permission controls on every result.

### pre-install detail

Store detail is catalogue/install information, not the installed-app management page.

It may show:

- catalogue icon
- name
- summary
- description
- screenshots
- source
- available version
- homepage
- license
- expected sandbox type
- local-build warning from the real Nix plan
- package attribute under advanced information
- Install

If already installed, replace Install with a quiet `Installed` state. Do not show Remove/Open/permission/icon-management controls here.

Those belong to Apps.

### interaction

Store is keyboard usable.

- typing from Discover can focus search
- `Ctrl+F` focuses search
- arrow keys move through results
- Enter opens catalogue detail
- Escape closes a sheet or backs out
- Install remains explicit

Do not invent fake install percentages. Prefer real phases:

```text
planning
resolving
downloading
building
installing
reconciling
complete
failed
```

## Vesper design language

Vesper Store follows the Apple/visionOS-inspired controlled-glass direction without turning the whole window into a transparent shell overlay.

Use:

- calm palette-tinted surfaces
- selective translucency in navigation, floating controls and sheets where technically appropriate
- generous continuous rounding
- soft shadow
- thin quiet borders
- restrained hierarchy
- clear spacing
- active Vesper palette integration

Avoid:

- neon source colours
- thick glowing borders
- dense telemetry cards
- browser-like store chrome
- dozens of separately blurred tiles
- an unrelated spacing, typography or radius system

### theme adapter

Do not copy arbitrary numeric values from Caelestia QML and let them drift.

Create a small Qt/QML Vesper application theme layer that receives the active palette and exposes semantic values such as:

```text
surface
surfaceContainer
surfaceContainerHigh
onSurface
onSurfaceVariant
primary
primaryContainer
secondaryContainer
error
outline
roundingSmall
roundingMedium
roundingLarge
roundingExtraLarge
spacingSmall
spacingMedium
spacingLarge
```

The Store should look related to Caelestia without importing private shell page internals.

## catalogue architecture

Nixpkgs browsing is local and fast.

The catalogue is built for Vesper's pinned nixpkgs revision and `x86_64-linux`.

```text
locked nixpkgs
     │
     ├── package metadata / flake-info
     ├── NixOS AppStream data
     └── small reviewed Vesper overrides
             │
             ▼
       catalogue builder
             │
             ▼
     normalized SQLite
             │
       FTS5 search index
             │
             ▼
        Vesper Store
```

The final Nixpkgs catalogue is a Nix derivation tied to `flake.lock`.

Suggested output:

```text
/nix/store/...-vesper-store-catalog/
└── share/vesper/store/
    ├── catalog.sqlite
    ├── icons/
    └── catalog-meta.json
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

Do not store large screenshot blobs inside SQLite. Keep URLs/metadata and lazy-cache media only when detail is opened.

### metadata authority

Nixpkgs is authoritative for:

- attribute path
- pname
- version
- supported platform
- broken state
- license
- known vulnerabilities/insecure state
- homepage
- main program where declared

AppStream enriches presentation with:

- component ID
- desktop ID
- display name
- generic name
- summary
- long description
- categories
- keywords
- screenshots
- icon metadata

AppStream never overrides the package version Nix will actually install.

### catalogue eligibility

Normal results require strong desktop-app identity such as an AppStream desktop component or a visible `Type=Application` desktop entry.

Hide by default:

- broken packages
- unsupported platforms
- libraries
- headers/dev outputs
- language package sets
- kernels/drivers
- services without desktop apps
- Flatpak runtimes
- `NoDisplay=true` helpers
- duplicate outputs of one app

If Nixpkgs marks a package insecure, block ordinary one-click installation by default and show the reason.

Vesper currently allows unfree packages so they may be listed. License state remains visible in catalogue detail.

### normalized identity

A package attribute alone is not a stable user-facing identity.

Normalize from:

1. AppStream component ID
2. desktop file ID
3. package attribute path
4. reviewed project/homepage aliases only when needed

Do not fuzzy-merge applications because names happen to look similar.

### database shape

The exact schema may evolve but should cover:

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

Use SQLite FTS5 for name, generic name, aliases, keywords, package attr and summary.

Ranking should roughly prefer:

1. exact display-name match
2. exact alias/package attr
3. name prefix
4. generic name/keywords
5. summary
6. description

No AI provider is required for Store search.

## Nix install and management core

Do not rewrite hand-maintained `home/yargc/apps.nix` when Install is pressed.

Do not let Store install actions rebuild unrelated dirty work in `/home/yargc/nix-config`.

Use a dedicated manifest-driven Nix profile for apps owned by Vesper Store.

```text
manifest = desired Store-owned Nix apps
profile  = realized environment for that manifest
```

The manifest is authoritative for Store-owned selections. Existing packages declared by Vesper config stay separate and are never silently migrated.

Recommended state:

```text
~/.config/vesper/store/
└── manifest.json

~/.local/state/vesper/store/
├── profile
├── generations.json
└── transactions/

~/.cache/vesper/store/
└── media/

$XDG_RUNTIME_DIR/vesper/
└── store.lock
```

Minimal manifest:

```json
{
  "version": 1,
  "nix": [
    {
      "appId": "org.mozilla.firefox",
      "attrPath": ["firefox"]
    }
  ],
  "flatpak": []
}
```

Do not store arbitrary executable Nix expressions in remote or mutable Store metadata.

### pinning

Store installs use the same exact nixpkgs revision as the Store/catalogue build.

Do not allow `nixpkgs#foo` to silently follow a newer registry revision than Vesper.

Keep these coherent:

```text
Vesper system revision
Vesper Store catalogue revision
Vesper Store package revision
```

When `flake.lock` changes, the new Store build and catalogue move together. Store-managed packages can then be reconciled against that revision.

### transaction model

All package mutations are serialized even when the UI action originates from Settings.

Nix install flow:

1. acquire Store lock
2. validate current manifest
3. calculate desired manifest
4. resolve installables against pinned nixpkgs
5. ask Nix for a dry-run plan
6. show local-build or policy warnings
7. realize desired packages
8. atomically switch Store profile
9. atomically persist new manifest
10. refresh installed application identity
11. let the existing adaptive-icon pipeline discover the new desktop entry
12. release lock

Removal initiated by `Settings -> Apps` uses the same core with the desired manifest minus that Store-owned application.

A failed realization leaves the previous profile and manifest active.

Never persist desired state first and hope the build succeeds later.

### cache awareness

Do not guess cache availability from popularity or Hydra metadata. Ask Nix with the appropriate dry-run/build plan.

If local builds are required, show a concise warning before install.

Vesper Store uses the machine's existing substituters and trusted public keys. It never auto-adds a third-party cache.

### rollback and GC

Keep several successful Store generations and manifest snapshots.

Rollback restores a known-good Store generation as a whole.

The active profile and retained rollback generations remain GC roots.

Verify normal `nh clean` does not remove active Store apps.

## installed size handoff to Apps

Installed size is displayed in `Settings -> Apps`, not as a speculative Store catalogue badge.

The shared Rust core should expose a size query for the installed owner.

For Store-managed Nix applications, query the realized Nix store closure. Dependencies are shared, so the value is the application's closure size and is not guaranteed to equal bytes freed by removal.

For Flatpak, use the real deployed/installed size reported by Flatpak metadata or CLI.

Do not estimate installed size from download size. If reliable data is unavailable, return unknown.

## Vesper-config-managed applications

Applications declared in Vesper configuration remain config-owned.

Apps shows:

```text
Managed by Vesper config
```

Store may recognize that the application is already installed, but it does not claim ownership.

The shared remove backend must refuse to remove a config-owned package by mutating the Store profile.

Do not silently rewrite `home/yargc/apps.nix`.

## packages needing system integration

Some nixpkgs packages require more than a user package. They may need a NixOS module, service, user/group, PAM rule, firewall setting or similar integration.

Store install classes:

```text
package
recipe
unsupported-system-integration
```

`package` is the normal path and ships first.

A `recipe` is reviewed local Vesper Nix code for a real application that needs system integration. Remote catalogue metadata can never provide executable Nix.

If an app needs system integration and Vesper has no reviewed recipe, block the misleading one-click install and explain that system configuration is required.

Do not build a generic recipe engine before actual applications require it.

## Flathub

Flathub is optional and disabled by default.

Strict fresh-system state:

```text
Flatpak service available        yes
Flathub remote auto-added        no
Flathub catalogue downloaded     no
Flathub results shown            no
Flathub preferred over Nixpkgs   no
Flathub beta enabled             no
```

Flatpak service availability exists because Vesper already has real Flatpak permission controls. It does not enable Flathub discovery.

### Sources

Initial Store source state:

```text
Nixpkgs   On    Vesper default
Flathub   Off   Optional sandboxed applications
```

Only explicit user action may enable Flathub.

After opt-in, prefer a user-scoped Flathub remote for Store-managed Flatpaks unless a real system-wide requirement appears.

Use native Flatpak operations and AppStream data. Do not scrape Flathub pages.

Useful operations include:

```text
flatpak remotes
flatpak remote-ls --app
flatpak remote-info
flatpak update --appstream
```

Normalize and cache remote metadata locally so search remains fast.

### source deduplication

When both sources provide the same application, merge only with strong identity:

1. exact AppStream component ID
2. exact desktop ID
3. reviewed alias mapping

Nixpkgs remains the default variant.

Do not merge apps only because display names are similar.

### disabling Flathub

Turning off Flathub discovery stops catalogue refreshes and hides Flathub-only discovery.

It must not silently uninstall installed Flatpaks.

Installed Flatpaks retain their update/removal path through Apps/shared Flatpak management.

## adaptive icon integration

Do not run adaptive-icon AI jobs for every catalogue entry.

Before installation:

- show ordinary AppStream/catalogue icon
- keep it read-only
- do not create icon conversion jobs

After installation:

- the real desktop entry appears
- existing Vesper adaptive-icon discovery resolves it
- Apps displays the active installed icon, including Tinted when that appearance is active

There is one installed adaptive-icon system, not a Store-specific icon pipeline.

## installed reconciliation

The shared core does not trust only the Store manifest when deciding what exists.

Reconcile:

- visible desktop entries
- Store Nix manifest/profile
- installed Flatpaks
- known Vesper-config ownership
- existing application identity data

A native app present outside Store state is external/config managed rather than falsely claimed by Store.

A Store manifest entry whose desktop identity disappears becomes a reconciliation warning instead of being silently forgotten.

## Rust backend shape

Prefer a reusable Rust core shared by `vesper-store` and Apps integration commands.

Suggested repository shape:

```text
home/yargc/packages/vesper-store/
├── default.nix
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── catalog.rs
│   ├── identity.rs
│   ├── installed.rs
│   ├── size.rs
│   ├── nix.rs
│   ├── flatpak.rs
│   ├── manifest.rs
│   └── transaction.rs
├── qml/
│   ├── Main.qml
│   ├── DiscoverPage.qml
│   ├── SearchPage.qml
│   ├── CatalogueDetailPage.qml
│   ├── SourcesPage.qml
│   └── components/
└── data/
    ├── io.vesper.Store.desktop
    └── io.vesper.Store.metainfo.xml

home/yargc/packages/marketplace-catalog.nix
```

Do not add `InstalledPage.qml` or a second installed application manager to Store.

`vesper-control` may expose thin JSON commands for Caelestia Apps when that is cleaner than directly linking the Store core.

Likely integration surface:

```text
vesper-control app-status <desktop-id>
vesper-control app-open <desktop-id>
vesper-control app-size <desktop-id>
vesper-control app-remove <desktop-id>
```

Exact command names should follow the existing `vesper-control` structure and avoid duplicate implementations.

## update semantics

Nixpkgs Store-managed apps follow the Vesper nixpkgs lock. Store never becomes a second faster-moving unstable channel.

Global update presentation belongs in `System -> Updates`.

Flatpak updates may move independently after Flathub is enabled because Flatpak owns that source, but update presentation should still integrate with the existing Updates surface rather than creating a second Store update center.

## network and privacy

Nixpkgs search works from the local catalogue without network access.

Network is used for:

- Nix install/update realization through configured substituters
- uncached screenshots
- optional catalogue artifact refresh if implemented
- Flatpak metadata/transactions after Flathub opt-in

Do not add telemetry, popularity beacons or recommendation tracking.

Do not send each search query to a web service.

Use a bounded screenshot/media cache.

## security and trust

Vesper Store is not a new package trust root.

For Nix:

- use locked nixpkgs
- use existing substituters
- use existing trusted keys
- respect platform/broken/insecure rules
- never auto-add caches

For Flatpak:

- use configured remote trust
- Flathub only after opt-in
- do not execute arbitrary `.flatpakref` URLs from catalogue text

For recipes:

- recipe code lives in Vesper Git
- remote metadata cannot provide executable Nix

No application description may be treated as a shell command.

## failure behavior

If the catalogue is missing, Store opens into a real error state. `Settings -> Apps` still works independently.

If catalogue and backend nixpkgs revisions do not match, block Nix mutations until reconciled. Browsing may continue only with a clear stale state when schema compatibility is known.

If Nix is unavailable, show the integrity error. Never fall back to curl installers.

A Flathub outage must not degrade Nixpkgs search.

A failed build/install/remove keeps the previous successful manifest/profile state active.

A corrupted manifest is preserved for diagnostics and blocks mutation. Never replace unreadable state with an empty manifest automatically.

## performance targets

- opening Store does not evaluate all nixpkgs
- local search feels immediate
- result scrolling does not spawn one process per row
- screenshots load lazily
- search stays usable while media loads
- transactions happen off the UI thread
- long Nix operations do not freeze the Qt event loop
- Apps size/status queries are cached enough to avoid spawning heavy Nix work per visible row
- no resident daemon is added until measurement proves it useful

## implementation phases

### phase 0: upstream and identity proof

Verify with real packages:

- locked nixpkgs metadata extraction
- AppStream mapping
- package attr -> AppStream ID -> desktop ID
- missing icon/screenshot behavior
- unfree/insecure states
- local-build case
- dedicated Store profile across reboot and GC
- installed identity reconciliation
- Nix closure size query

### phase 1: native Store discovery shell

Implement:

- Qt 6/QML application package
- Store `.desktop` and AppStream metadata
- Vesper theme adapter
- local SQLite catalogue and FTS
- Discover/Categories/Search/Sources
- catalogue detail
- installed detection
- Settings `Find New Apps`

Acceptance:

- Store launches independently
- launcher/dock sees it as a normal app
- Nixpkgs search needs no network
- no full nixpkgs evaluation per query
- already-installed apps are recognized without Store claiming management

### phase 2: Nix install transactions

Implement:

- Store manifest/profile
- lock
- dry-run planning
- install
- rollback
- installed reconciliation
- adaptive-icon handoff

Acceptance:

- failed install preserves previous state
- Store never rewrites `apps.nix`
- Store never deploys unrelated dirty repo work
- packages use the locked Store/Vesper nixpkgs revision
- normal package install is user-level

### phase 3: Apps installed management integration

Implement in `Settings -> Apps`:

- installed app list/detail
- name
- description
- active/tinted icon preview
- Open
- Remove for removable Store/Flatpak-owned apps
- installed size
- installed version/source
- existing permissions/wellbeing/adaptive icon controls

The Remove action calls the shared Store/Flatpak transaction core rather than opening Vesper Store.

Acceptance:

- no `Open in Vesper Store` button exists in installed detail
- Store-managed app removal updates manifest/profile atomically
- config-managed apps are not falsely removable
- Open uses resolved desktop entry
- size is sourced from the installed package owner
- Tinted appearance shows the actual active tinted icon

### phase 4: Updates integration

Implement revision comparison and update availability in the existing `System -> Updates` surface. Keep Store from becoming a second update center.

### phase 5: optional Flathub

Only after Nix path is stable:

- explicit source opt-in
- user remote management
- local normalized AppStream cache
- strong-identity dedup
- Flatpak install
- Apps removal/permission handoff

Acceptance:

- fresh Vesper adds no Flathub remote
- no Flathub metadata before opt-in
- Nixpkgs remains default for duplicates
- disabling discovery never removes installed Flatpaks

### phase 6: reviewed recipes and polish

Add recipes only for real apps that prove package-only installation insufficient.

Then finish keyboard navigation, responsive layout, screenshot cache, accessibility, reduced motion, transaction history and catalogue diagnostics.

Do not prioritize ratings, recommendation feeds or decorative content over reliable installation and Apps management.

## test matrix

Catalogue:

- exact name/package attr/keyword search
- Unicode names
- duplicate IDs
- missing icon/description/screenshot
- unsupported/broken/insecure/unfree package

Nix transactions:

- cached install
- large closure
- local build required
- build/network failure
- rollback
- concurrent mutation
- corrupted manifest
- GC with active generations

Apps integration:

- app in `home.packages`
- app installed by Store
- app from another Nix profile
- Flatpak app
- multiple desktop files
- hidden helper desktop entries
- Open
- Remove Store app
- reject remove for config-managed app
- Nix installed size
- Flatpak installed size
- unknown size
- active Tinted icon

Flathub:

- absent on fresh system
- enable source
- metadata refresh failure
- install Flatpak
- permission controls after install
- duplicate Nixpkgs/Flatpak app
- disable source while installed app remains

Qt/QML Store UI:

- independent native launch
- single-instance activation
- Settings `Find New Apps`
- keyboard-only navigation
- narrow window
- long translated metadata
- offline search
- install failure
- stale catalogue
- long transaction does not block UI thread

## validation against Vesper rules

Implementation must still pass the repository change checklist:

- no first-party Python
- parse changed Nix
- compile first-party Rust
- validate Qt/QML resources
- build Vesper Store package
- build configured Caelestia after Apps changes
- run `nix flake metadata --no-write-lock-file`
- evaluate Home Manager activation
- build full Vesper system
- keep lock changes intentional

## decisions locked for v1

- product name is `Vesper Store`
- Store is a separate native desktop application
- Store UI stack is Qt 6 + Qt Quick/QML
- backend/core is Rust
- GTK/libadwaita is not globally banned
- Store is discovery/install focused
- installed application detail and management lives in `Settings -> Apps`
- Apps has `Find New Apps`
- Apps does not have `Open in Vesper Store`
- Apps detail includes name, description, active/tinted icon, Open, removable-source Remove and installed size
- Store does not have a duplicate Installed manager
- global updates live in `System -> Updates`
- Nixpkgs is primary/default
- local catalogue follows Vesper's locked nixpkgs revision
- use upstream Nix/AppStream tooling instead of rebuilding ecosystems
- Store-owned Nix apps use isolated manifest/profile state
- Vesper-config apps are never silently migrated
- Flatpak service availability does not enable Flathub
- Flathub is opt-in and disabled by default
- Nixpkgs wins duplicate source preference
- no PackageKit
- no apt/rpm conversion
- no AI requirement for search
- no automatic third-party caches
- no remote executable recipes
- no fake sandbox controls for native Nix
- no duplicate adaptive-icon pipeline

## research references

Primary upstream references:

- Nixpkgs reference manual: https://nixos.org/manual/nixpkgs/unstable/
- Nix profile reference: https://nix.dev/manual/nix/2.35/command-ref/new-cli/nix3-profile-add.html
- Nix build reference: https://nix.dev/manual/nix/2.35/command-ref/new-cli/nix3-build.html
- Nix substituter configuration: https://nix.dev/manual/nix/2.33/command-ref/conf-file
- NixOS Search: https://github.com/NixOS/nixos-search
- Nix Software Center: https://github.com/snowfallorg/nix-software-center
- NixOS AppStream data: https://github.com/snowfallorg/nixos-appstream-data
- Flatpak command reference: https://docs.flatpak.org/en/latest/flatpak-command-reference.html
- Flatpak repository documentation: https://docs.flatpak.org/en/latest/repositories.html
- Flatpak AppStream conventions: https://docs.flatpak.org/en/latest/conventions.html
- Flathub user installation: https://docs.flathub.org/docs/for-users/installation
- desktop entry specification: https://specifications.freedesktop.org/desktop-entry/latest/
