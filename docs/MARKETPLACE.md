# Vesper Store

This document is the single source of truth for Vesper Store.

Vesper Store is a separate native desktop application for discovering installing removing and updating desktop applications on Vesper.

The application name is exactly `Vesper Store`.

Nixpkgs is the default and primary source. Flathub is optional and disabled by default.

## product boundary

Vesper Store is a desktop application store.

It is not a general Nix package browser, NixOS option editor, flake editor, service manager or frontend for every derivation in nixpkgs.

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

The reverse-DNS ID may change once before implementation if repository naming requires it. After persisted state or deep links exist it is stable.

Vesper Store appears in the launcher and can be pinned to the dock like any other desktop application.

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

Qt/QML is the standard Vesper Store presentation stack.

GTK and libadwaita are not globally forbidden in Vesper. They are simply not the Store UI stack.

Do not implement Vesper Store with Electron, Tauri/WebView, an embedded browser, a localhost web application or a shell-only Quickshell page.

The Store is a normal Qt application rather than a page that only exists while Caelestia is running.

Recommended shape:

```text
Qt Quick / QML application
          │
          ▼
       Rust core
          │
          ├── local SQLite catalogue
          ├── Nix package planning and transactions
          ├── installed-app reconciliation
          ├── source adapters
          └── transaction and rollback state
```

QML is presentation logic. Do not put Nix expression parsing, Flatpak output parsing, SQLite query construction or transaction state machines in QML JavaScript.

Use a maintained Rust/Qt integration approach available from pinned nixpkgs rather than inventing a large custom protocol just to connect one application to itself. Keep the bridge narrow and typed.

## upstream-first rule

Do not rebuild package infrastructure that already exists upstream.

Use existing projects and standards as building blocks:

### NixOS/nixos-search and flake-info

Reuse the package-export/index concepts and `flake-info` where appropriate.

The lesson from NixOS Search is important: one giant package JSON stopped scaling. Vesper Store should not evaluate all of nixpkgs on each search and should not fetch search.nixos.org on every keystroke.

### snowfallorg/nixos-appstream-data

Reuse or adapt the established NixOS AppStream generation path when it matches the pinned nixpkgs revision.

Do not write a new AppStream ecosystem from scratch.

### snowfallorg/nix-software-center

Use it as prior art for Nix GUI-store behavior, metadata mapping and transaction edge cases.

Reuse code only when its license, architecture and maintenance cost make that cleaner than a small Vesper implementation.

Do not fork its GTK UI. Vesper Store has its own Qt/QML UI and Vesper design language.

### Nix itself

Nix remains authoritative for:

- dependency closure resolution
- binary substituters
- trusted keys
- realization
- package profiles/generations where used
- actual build failures

Do not create a second resolver, downloader or cache format.

### Flatpak itself

When Flathub is enabled, use Flatpak's native CLI/AppStream/remote mechanisms.

Do not scrape flathub.org and do not depend on an unofficial web API for core behavior.

## relationship with Settings -> Apps

Settings and Vesper Store have different jobs.

`Settings -> Apps` remains the installed-application control surface for:

- default applications
- real Flatpak permissions
- native/unsandboxed status
- wellbeing
- installed application identity
- per-app adaptive icon status and actions

Vesper Store owns:

- discovery
- search
- categories
- application details
- source choice
- install/remove/update transactions
- Store-owned rollback
- optional Flathub source management

Do not duplicate the full permissions, wellbeing or adaptive-icon editors inside Vesper Store.

### Find New Apps

Add a prominent `Find New Apps` action near the top of `Settings -> Apps`.

It launches Vesper Store.

Suggested Settings row:

```text
Find New Apps
Discover and install applications with Vesper Store
```

Use the existing Caelestia `RowButton` or an equivalent native Settings component. Do not turn it into a large promotional card.

Basic launch contract:

```text
vesper-store
```

Single-instance activation is preferred. If the Store is already running, activating it should focus its existing window rather than creating duplicate windows.

### Open in Vesper Store

When Settings is showing an installed application that can be resolved to a Store catalogue identity, expose:

```text
Open in Vesper Store
```

Deep-link contract:

```text
vesper-store --app <catalogue-id>
```

Use a stable catalogue ID. Never deep-link by display name.

Hide the action when identity is uncertain.

### shared application identity

Settings and Store must resolve installed applications through one compatible identity model.

Preferred identity keys:

1. AppStream component ID
2. desktop file ID
3. Nix package attribute when known
4. Flatpak application ID when relevant
5. reviewed alias mappings for known mismatches

A Store install should appear in existing Apps controls immediately after desktop-entry reconciliation.

The Store must not create a second incompatible installed-app registry.

## Vesper design language

Vesper Store follows the same Apple/visionOS-inspired controlled-glass direction as Vesper without turning the whole application into a transparent shell overlay.

The repository contract intentionally concentrates the strongest backdrop glass effects in shell surfaces. Vesper Store is a normal desktop app and should remain readable over any wallpaper or window behind it.

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
- a grid of dozens of independently blurred cards
- a second unrelated spacing, typography or radius system

### design token ownership

Do not copy arbitrary numeric values out of Caelestia QML and let them drift.

Create a small Vesper application theme layer for Qt/QML that receives the active Vesper palette and exposes stable semantic values such as:

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

The Store can match Caelestia without importing private shell page implementations.

### window layout

Wide layout:

```text
┌──────────────────────────────────────────────────────────────────────┐
│ Vesper Store                                                  window │
├────────────────┬─────────────────────────────────────────────────────┤
│ Search         │ Featured / category / search results                │
│                │                                                     │
│ Discover       │  app      app      app                              │
│ Categories     │                                                     │
│ Installed      │  selected app detail                                │
│ Updates        │                                                     │
│                │  screenshots                                        │
│ Sources        │  description                                        │
│                │  source  version  sandbox                           │
│                │                                      [ Install ]    │
└────────────────┴─────────────────────────────────────────────────────┘
```

Narrow layout uses normal page navigation rather than crushing sidebar, results and details into three tiny columns.

### result presentation

Prefer clean application tiles or rows with:

- icon
- name
- short summary
- installed/update state
- source only when relevant
- one primary action

License, architecture, package attribute, closure details and maintainers belong in detail or advanced information.

### app detail

Detail may show:

- large icon
- name
- summary
- long description
- screenshots
- source
- package version
- installed version
- homepage
- license
- sandbox state
- cache/local-build warning
- package attribute under advanced information
- install, remove or update action

Config-managed apps show:

```text
Installed
Managed by Vesper config
```

Store-managed apps show:

```text
Installed
Managed by Vesper Store
```

Flatpaks show their actual sandbox/source state.

### interaction

Store is keyboard usable.

- typing from Discover focuses search where reasonable
- `Ctrl+F` focuses search
- arrow keys move through results
- Enter opens the selected application
- Escape closes a sheet or backs out of detail
- install/remove always remain explicit actions

Do not invent fake install percentages. Prefer real phases such as:

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

## catalogue architecture

Nixpkgs browsing is local and fast.

The Store catalogue is built for Vesper's pinned nixpkgs revision and `x86_64-linux`.

Pipeline:

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

The final Nixpkgs catalogue should be a Nix derivation tied to `flake.lock`.

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

Do not store large screenshot blobs inside SQLite. Store URLs/metadata and lazy-cache screenshot media when the user opens detail.

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

AppStream must not override the package version Nix will install.

### catalogue eligibility

Normal Store results require strong desktop-app identity such as an AppStream desktop component or a visible `Type=Application` desktop entry.

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
- duplicate outputs of one application

If Nixpkgs marks a package insecure, block normal one-click installation by default and show the reason.

Vesper currently allows unfree packages, so they may be listed. License state remains visible in detail.

### normalized identity

A package attribute alone is not the user-facing identity.

Normalize from:

1. AppStream component ID
2. desktop file ID
3. package attribute path
4. reviewed homepage/project aliases only when needed

Do not fuzzy-merge apps because names happen to look similar.

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

No AI provider is needed for Store search.

## Nix installation model

Do not rewrite hand-maintained `home/yargc/apps.nix` every time someone presses Install.

Also do not let Install rebuild unrelated dirty work in `/home/yargc/nix-config`.

Use a dedicated manifest-driven Nix profile for applications owned by Vesper Store.

```text
manifest = desired Vesper Store Nix apps
profile  = realized environment for that manifest
```

The manifest is authoritative for Store-owned Nix selections. The profile is the realized state.

Existing packages declared in Vesper config are detected separately and never silently migrated.

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

Package attrs come from the trusted local catalogue and are validated before use.

### pinning

Store installs use the exact nixpkgs revision compiled into the running Store/catalogue.

Do not allow `nixpkgs#foo` to silently follow a newer registry revision than Vesper.

These stay coherent:

```text
Vesper system revision
Vesper Store catalogue revision
Vesper Store package revision
```

When `flake.lock` changes, the new Store build and catalogue move together. The Store can then offer reconciliation/update of its managed packages.

### transaction model

All package mutations are serialized.

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
10. refresh installed-app identity
11. allow existing adaptive-icon discovery to handle new desktop entries
12. release lock

A failed realization leaves the previous profile and manifest active.

Never write desired state first and hope the build succeeds later.

### cache awareness

Do not guess cache availability from popularity or Hydra assumptions.

Ask Nix using an appropriate dry-run/build plan.

When local builds are required, show a warning such as:

```text
Local build required
Some of this application is not available from your configured binary caches.
```

A cache miss is not automatically an error.

Vesper Store uses the machine's existing substituters and trusted public keys. It never adds a third-party cache automatically.

### rollback and GC

Keep several successful Store generations and manifest snapshots.

Rollback restores a known-good Store generation as a whole.

The active Store profile and retained rollback generations remain GC roots.

Verify that normal `nh clean` does not remove the active Store apps.

## Vesper-config-managed applications

Apps already declared by Vesper config remain config-owned.

Show:

```text
Installed
Managed by Vesper config
```

Do not offer a misleading Store Remove action.

Do not silently move an app from `home.packages` into Store state.

A future explicit migration flow is separate work.

## packages needing system integration

Some nixpkgs packages require more than a user package. They may need a NixOS module, service, user/group, PAM rule, firewall setting or similar integration.

Store install classes:

```text
package
recipe
unsupported-system-integration
```

### package

Normal desktop application that works as a user package.

This is the default and ships first.

### recipe

A reviewed Vesper integration for a real application that needs more.

Recipe code is static local code in the Vesper repository.

Remote Store metadata cannot provide executable Nix.

Potential future layout:

```text
modules/store/
├── default.nix
└── recipes/
    └── <reviewed-id>.nix
```

Do not build a generic recipe engine before actual applications need it.

### unsupported-system-integration

If a package is known to need system integration and Vesper has no reviewed recipe, do not pretend one-click install is complete.

Show the requirement clearly and block the misleading action.

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

Flatpak service availability exists because Vesper already has real Flatpak permission controls. That does not enable Flathub discovery.

### Sources

Vesper Store has a Sources page/sheet.

Initial state:

```text
Nixpkgs   On    Vesper default
Flathub   Off   Optional sandboxed applications
```

Only explicit user action may enable Flathub.

After opt-in, prefer a user-scoped Flathub remote for Store-managed Flatpaks unless a real system-wide requirement appears.

Use native Flatpak operations and AppStream data. Do not scrape the Flathub website.

Useful operations include:

```text
flatpak remotes
flatpak remote-ls --app
flatpak remote-info
flatpak update --appstream
```

Normalize and cache remote metadata locally so search stays fast.

### source deduplication

When both sources provide the same application, merge only with strong identity:

1. exact AppStream component ID
2. exact desktop ID
3. reviewed alias mapping

Nixpkgs remains the default variant.

The detail page can expose Flathub as an alternative sandboxed source.

Do not merge applications only because names are similar.

### disabling Flathub

Turning off Flathub discovery stops catalogue refreshes and hides Flathub-only discovery.

It must not silently uninstall installed Flatpaks.

Installed Flatpaks retain a usable update path until explicitly removed.

## adaptive icon integration

Do not run adaptive-icon AI jobs for every Store catalogue entry.

Before installation:

- show AppStream/catalogue icon
- keep it read-only
- do not create icon conversion jobs

After installation:

- the real desktop entry appears
- existing Vesper adaptive-icon discovery resolves it
- the normal adaptive icon pipeline takes over

There is one installed adaptive-icon system, not a separate Store icon system.

## installed reconciliation

Store does not trust only its own manifest when deciding what is installed.

Installed inventory reconciles:

- visible desktop entries
- Store Nix manifest/profile
- installed Flatpaks
- known Vesper-config package ownership
- existing app identity data

A native app present outside Store state is shown as config/external-managed rather than falsely claimed by Store.

A Store manifest entry whose desktop identity disappears becomes a reconciliation warning instead of being silently forgotten.

## Rust backend shape

Prefer a reusable Rust crate/core shared by the Store executable and thin Vesper control commands where useful.

Suggested repository shape:

```text
home/yargc/packages/vesper-store/
├── default.nix
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── catalog.rs
│   ├── identity.rs
│   ├── nix.rs
│   ├── flatpak.rs
│   ├── manifest.rs
│   └── transaction.rs
├── qml/
│   ├── Main.qml
│   ├── DiscoverPage.qml
│   ├── AppDetailPage.qml
│   ├── InstalledPage.qml
│   ├── UpdatesPage.qml
│   ├── SourcesPage.qml
│   └── components/
└── data/
    ├── io.vesper.Store.desktop
    └── io.vesper.Store.metainfo.xml

home/yargc/packages/marketplace-catalog.nix
```

Exact placement may change to keep the package clean. Do not scatter Store implementation across unrelated QML patches.

`vesper-control` only needs integration commands if Settings cannot activate/deep-link the Store directly.

## update semantics

### Nixpkgs apps

Store-managed Nix updates follow the Vesper nixpkgs lock.

Vesper Store never becomes an independent faster-moving unstable channel.

An update exists after Vesper moves to a new locked nixpkgs revision and a managed app resolves to a new package/store path.

### Flatpak apps

Flatpak updates may move independently after Flathub is enabled because Flatpak owns that source.

Global system/update aggregation still belongs in the existing `System -> Updates` surface once integrated.

The Store may also have an Installed/Updates view for applications it manages, but must not pretend it owns NixOS system upgrades.

## network and privacy

Nixpkgs search works from the local catalogue without network access.

Network is used for:

- Nix installs/updates through configured substituters
- uncached screenshots
- optional catalogue artifact refresh if implemented
- Flatpak metadata/transactions after Flathub opt-in

Do not add telemetry, popularity beacons or recommendation tracking.

Do not send each search string to a web service.

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

### catalogue missing

Store opens into a real error state with retry/diagnostic information.

Settings -> Apps still works independently.

### revision mismatch

If Store catalogue and Store backend are built for different nixpkgs revisions, block Nix mutations until reconciled.

Browsing may continue with a clear stale state only when schema compatibility is known.

### Nix unavailable

Show the integrity error. Never fall back to curl installers.

### Flathub unavailable

Nixpkgs Store remains fully usable.

### build/install failure

Keep previous Store state active and offer retry/details.

### corrupted manifest

Preserve the file, block mutation and offer repair diagnostics.

Never replace unreadable state with an empty manifest automatically.

## performance targets

- opening Store does not evaluate all nixpkgs
- local search feels immediate
- result scrolling does not spawn one process per row
- screenshots load lazily
- search remains usable while media loads
- transaction work happens off the UI thread
- long Nix operations never freeze the Qt event loop
- no resident daemon is added until measurement proves it is needed

## implementation phases

### phase 0: upstream and identity proof

Before building the full UI:

- verify `flake-info`/Nixpkgs metadata extraction against the locked revision
- verify AppStream generation/mapping with real desktop packages
- verify package attr -> AppStream ID -> desktop ID
- verify missing-icon and missing-screenshot behavior
- verify unfree and insecure package states
- verify a package requiring local build
- verify Store profile survives reboot and garbage collection

### phase 1: native Store shell and read-only catalogue

Implement:

- Qt 6/QML application package
- `.desktop` and AppStream metadata for Vesper Store itself
- Vesper application theme adapter
- local SQLite catalogue
- FTS search
- categories
- app detail
- installed detection
- Settings `Find New Apps`
- Settings `Open in Vesper Store`

Acceptance:

- Vesper Store launches independently
- launcher/dock sees it as a normal app
- Nixpkgs search needs no network
- no full nixpkgs evaluation per query
- config-managed apps are identified correctly

### phase 2: Nix transactions

Implement:

- Store manifest
- dedicated profile
- transaction lock
- dry-run planning
- install
- remove
- rollback
- installed reconciliation
- adaptive-icon handoff after install

Acceptance:

- failed install preserves previous state
- Store never rewrites `apps.nix`
- Store never deploys unrelated dirty repo work
- installed apps use Store/Vesper locked nixpkgs revision
- normal package install is user-level

### phase 3: updates and generations

Implement:

- revision comparison
- update availability
- bulk Store reconcile after Vesper lock update
- retained generations
- System Updates integration where appropriate

### phase 4: optional Flathub

Only after Nix path is stable:

- Sources UI
- explicit Flathub opt-in
- user remote management
- local normalized AppStream cache
- strong-identity dedup
- Flatpak install/remove/update
- handoff to existing Apps permission controls

Acceptance:

- fresh Vesper has no Flathub remote added by Store
- no Flathub metadata before opt-in
- Nixpkgs remains default for duplicates
- disabling discovery never removes installed Flatpaks

### phase 5: reviewed recipes

Only for real apps that prove package-only install insufficient.

Each recipe needs:

- stable ID
- reviewed local Nix code
- preview of system changes
- rollback path
- full Vesper validation

### phase 6: polish

After data and transactions are reliable:

- keyboard navigation
- responsive/narrow layout
- screenshot cache
- accessible labels and focus order
- reduced motion
- transaction history
- catalogue diagnostics
- source badges

Do not prioritize ratings, recommendations or decorative feeds over install reliability.

## test matrix

### catalogue

- exact name search
- package attr search
- keyword search
- Unicode names
- duplicate IDs
- missing icon
- missing description
- missing screenshot
- unsupported platform
- broken package
- insecure package
- unfree package

### Nix transactions

- cached install
- large closure
- local build required
- build failure
- network failure
- cancellation before final switch
- remove
- update after nixpkgs lock change
- rollback
- concurrent mutation attempt
- corrupted manifest
- garbage collection with active generations

### installed identity

- app in `home.packages`
- app installed by Store
- app from another Nix profile
- Flatpak app
- app with multiple desktop files
- hidden helper desktop entries

### Flathub

- absent on fresh system
- enable source
- metadata refresh failure
- install Flatpak
- permission controls after install
- duplicate Nixpkgs/Flatpak app
- disable source while installed Flatpak remains

### Qt/QML UI

- independent native launch
- single-instance activation
- Settings `Find New Apps`
- deep link to known app
- unknown deep link
- keyboard-only navigation
- narrow window
- long application name
- long translated description
- no screenshots
- offline search
- install failure
- stale catalogue
- active transaction while switching pages
- long transaction does not block UI thread

## validation against Vesper rules

Implementation must still pass the repository change checklist.

In particular:

- no first-party Python
- parse changed Nix
- compile first-party Rust
- validate Qt/QML resources at build time
- build Vesper Store package
- build configured Caelestia after Settings integration changes
- run `nix flake metadata --no-write-lock-file`
- evaluate Home Manager activation
- build full Vesper system
- keep lock changes intentional

## decisions locked for v1

These are implementation constraints unless a concrete technical failure forces a change:

- product name is `Vesper Store`
- Vesper Store is a separate native desktop application
- Store standard UI stack is Qt 6 + Qt Quick/QML
- backend/core is Rust
- GTK/libadwaita is not globally banned
- Store is not embedded inside Settings
- Settings -> Apps has `Find New Apps`
- known installed apps can expose `Open in Vesper Store`
- Nixpkgs is primary/default
- local catalogue is tied to Vesper's locked nixpkgs revision
- core search is local
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

Primary upstream references for this design:

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
