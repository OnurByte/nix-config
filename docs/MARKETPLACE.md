# marketplace

This document is the implementation plan for a native Vesper application marketplace.

The goal is not to bolt GNOME Software, Discover or another package-manager GUI onto the desktop. The marketplace should use the package infrastructure Vesper already trusts, fit inside the existing Caelestia Apps surface and keep Nixpkgs as the default application source.

Flathub is optional. Vesper must not add the Flathub remote, fetch its catalogue or prefer Flatpak applications by default.

## product boundary

Marketplace is for desktop applications.

It is not a general Nix package browser, a NixOS option editor or a frontend for every derivation in nixpkgs. Libraries, development outputs, runtimes, kernels, services and command line dependencies do not belong in the normal catalogue.

The default source order is:

1. Nixpkgs from the same locked revision used by Vesper
2. reviewed Vesper integration recipes where a normal package is not enough
3. Flathub only after the user explicitly enables it

There is no apt, rpm, pacman or PackageKit backend in the Vesper implementation. There is also no conversion from Nix packages to deb or rpm packages.

If this work is reused on a non-NixOS distribution later, Nix should remain the application layer instead of translating packages into the host distribution format.

## why this shape

Vesper already has the pieces that should remain authoritative:

- Caelestia owns the shell and settings surfaces
- installed application controls already live in Apps
- native Nix applications are intentionally marked unsandboxed
- Flatpak applications already expose their real per-app overrides
- adaptive icons already discover installed desktop entries
- Nixpkgs is pinned by `flake.lock`
- unfree packages are already deliberately allowed by the Vesper Nix configuration
- Vesper first-party control-plane code is Rust

A marketplace must extend those systems rather than create parallel state for permissions, icons, updates or application identity.

The important distinction is between catalogue state and installed state.

The catalogue can be generated from Nixpkgs and AppStream metadata. Installed applications must be reconciled against the real desktop entries and the source that owns them.

## prior art and lessons

The plan is informed by existing attempts rather than starting from assumptions.

### nixos search

`search.nixos.org` moved away from sending one giant package JSON file to the frontend because that stopped scaling as the package set grew. The current project imports package information with `flake-info` into a search backend.

Vesper should take the same lesson but not reproduce the server architecture locally. A single laptop does not need Elasticsearch or OpenSearch. It needs a compact local index generated for one architecture and one pinned nixpkgs revision.

### nix software center

`snowfallorg/nix-software-center` proves that a graphical Nix application store is practical. It supports package search, `configuration.nix`, `nix profile`, updates and running packages without installing them.

It also exposes the areas that need stronger boundaries for Vesper:

- AppStream coverage is incomplete for some icons and screenshots
- package installation is not equivalent to enabling every NixOS service
- package configuration can become a second configuration editor
- the UI must understand the difference between a desktop app and a package that needs system integration

Vesper should reuse the useful data model and avoid pretending every nixpkgs attribute is a one-click desktop application.

### appstream

AppStream is the correct enrichment format for application-store metadata. It provides application IDs, names, summaries, descriptions, icons, categories, screenshots and launchable desktop IDs.

Nixpkgs metadata remains the authority for the Nix package itself. AppStream enriches that package with store-facing presentation data.

The two datasets must be joined. Neither should replace the other.

### flatpak and flathub

Flatpak repositories already publish AppStream metadata and the Flatpak command line exposes configured remotes, available applications and remote details.

Marketplace should use those native interfaces only when Flathub support is enabled. It must not depend on an unofficial Flathub web API.

## ux placement

Do not add a separate GTK or Qt software-center application.

Do not add a new top-level settings category only for Marketplace.

Keep the existing `Apps & AI` sidebar hierarchy. The Apps page becomes the application control surface with two internal modes:

```text
Apps
├── Installed
└── Marketplace
```

`Installed` remains the default mode and keeps the existing defaults, permissions, wellbeing and adaptive-icon behavior.

`Marketplace` is the catalogue and install surface.

Global system updates remain under the existing `System -> Updates` page. Marketplace may show an inline update state for an individual application but must not create a second global update center.

## visual contract

Marketplace follows the existing Vesper controlled-glass language.

Use existing Caelestia `Tokens`, `Colours`, `StyledRect`, `StyledText`, `StateLayer`, `MaterialIcon` and animation primitives. Do not hardcode a second radius scale, palette or typography system.

The surface should feel like the rest of Vesper rather than a web store embedded inside settings.

### layout

The main page uses one calm glass content surface instead of putting a blurred card around every piece of metadata.

Recommended wide layout:

```text
┌────────────────────────────────────────────────────────────────────┐
│ Apps                                                               │
│ [ Installed ] [ Marketplace ]                                      │
│                                                                    │
│  Search applications...                         [ Sources / filter ]│
│                                                                    │
│  Productivity   Development   Media   Internet   Utilities         │
│                                                                    │
│  application results                         application detail     │
│  ┌───────────────────────────────┐           ┌───────────────────┐ │
│  │ icon  name             action │           │       icon        │ │
│  │       short summary           │           │ name              │ │
│  ├───────────────────────────────┤           │ source / version  │ │
│  │ icon  name             action │           │ description       │ │
│  │       short summary           │           │ screenshots       │ │
│  └───────────────────────────────┘           │ install state     │ │
│                                              └───────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

On narrower layouts the detail pane becomes an in-page sheet or pushed detail view. Do not shrink both columns until text becomes dense.

### result rows

Prefer compact application rows or grouped tiles over a dashboard card grid.

Each result needs only:

- application icon
- application name
- one short summary
- source when it matters
- installed or update state
- one primary action

Do not put license, architecture, closure size, maintainers, runtime and package attribute on every result row. Those belong in detail or advanced information.

### detail view

The detail view may show:

- larger icon
- name
- summary and long description
- source
- package version
- installed version
- screenshots when metadata provides them
- homepage
- license
- sandbox state
- source-build warning when Nix reports one
- package attribute under advanced information
- install, remove or update action

If an application is already declared by the main Vesper configuration, show `Managed by Vesper config` instead of offering a misleading remove action.

If the application came from Marketplace, show `Managed by Marketplace`.

If it is a Flatpak, show `Flatpak` and preserve the existing per-app permission controls after installation.

### source selector

Do not show a source chooser for an application that has only one usable source.

When the same application is present in Nixpkgs and enabled Flathub, default to Nixpkgs and expose the alternative in the detail view.

The source selector should communicate behavior rather than only logos:

```text
Nixpkgs
native package · Vesper default

Flathub
sandboxed Flatpak · optional source
```

### glass usage

Use glass for the page shell, search/filter strip, selected detail surface and modal sheets.

Result rows should normally use quiet palette-tinted surface layers inside the parent glass surface. Avoid dozens of independently blurred rectangles.

Use generous continuous rounding, soft shadow and thin quiet borders. Avoid neon source colors, thick borders or a Material dashboard look.

### interaction

Search should be keyboard-first.

- opening Marketplace focuses search when the user starts typing
- `Ctrl+F` focuses the search field
- arrow keys move through results
- Enter opens detail
- install and remove actions must remain explicit
- Escape closes a detail sheet before leaving Marketplace

Status must never be color-only. Use icon, text and state together.

Do not invent progress percentages when Nix does not expose reliable percentage data. Use meaningful phases such as evaluating, downloading, building and applying.

## catalogue architecture

Browsing Marketplace must not run a full nixpkgs evaluation on every query and must not call `search.nixos.org` for every keystroke.

The Nixpkgs catalogue is a local, read-only artifact built for Vesper's pinned nixpkgs revision and `x86_64-linux`.

Recommended pipeline:

```text
locked nixpkgs
     │
     ├── package metadata / flake-info
     │
     ├── AppStream catalogue
     │
     └── Vesper overrides / recipes
             │
             ▼
       catalogue builder
             │
             ▼
   normalized SQLite + assets
             │
             ▼
     Marketplace search
```

### authority

Nixpkgs metadata is authoritative for:

- attribute path
- pname and version
- platform support
- broken state
- license
- known vulnerabilities
- homepage
- main program when declared
- source provenance where available

AppStream is authoritative only for presentation metadata it actually owns:

- component/application ID
- desktop ID
- display name
- generic name
- summary
- long description
- categories
- keywords
- screenshots
- application icon metadata

A conflicting AppStream version must not override the package version Nix will install.

### building the index

Use `flake-info` or an equivalent Nix-native exporter against the exact locked nixpkgs input.

Join that result with AppStream data generated for NixOS. `snowfallorg/nixos-appstream-data` is useful prior art and can be used as an input if its revision and package mapping are verified. It must not silently become the authority for package availability.

The catalogue builder must be first-party Rust or a Nix build pipeline around existing upstream tools. Do not add a tracked Python catalogue generator.

The final catalogue should be a Nix derivation so it follows `flake.lock` and can be built by normal Vesper validation.

Suggested output:

```text
/nix/store/...-vesper-marketplace-catalog/
└── share/vesper/marketplace/
    ├── catalog.sqlite
    ├── icons/
    └── catalog-meta.json
```

`catalog-meta.json` should contain at least:

```json
{
  "schemaVersion": 1,
  "system": "x86_64-linux",
  "nixpkgsRevision": "...",
  "generatedAt": "..."
}
```

Do not ship large screenshot blobs in SQLite. Keep screenshot URLs in metadata and lazy-cache media only when a detail page requests it.

### eligibility filter

The normal catalogue should prefer real desktop applications.

An entry is eligible when there is strong application identity such as an AppStream desktop component or a visible `Type=Application` desktop entry and it is supported on the current platform.

Hide by default:

- `meta.broken = true`
- unsupported platform packages
- libraries
- headers and development outputs
- language package sets
- kernels and drivers
- fonts unless a future Fonts surface explicitly needs them
- services without a desktop application
- Flatpak runtimes
- `NoDisplay=true` desktop helpers
- duplicate outputs of the same application

Known-vulnerable packages must not look like ordinary installable results. If Nixpkgs marks a package insecure, Marketplace should block it by default and show the reason in detail.

Vesper currently allows unfree packages. Marketplace may therefore list them, but the detail view must show the license state clearly.

### application identity

A package attribute is not a stable user-facing identity by itself.

Use a normalized identity record built from:

1. AppStream component ID when available
2. desktop file ID
3. package attribute path
4. canonical homepage/project identity as a fallback for deduplication

This identity also lets Nixpkgs and Flathub variants of the same application appear as source choices instead of duplicate search results.

Do not perform fuzzy cross-source merges when identity is uncertain. Two separate correct entries are better than one incorrectly merged application.

### suggested database shape

The exact schema can change during implementation, but the normalized model should cover these concepts:

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

Use SQLite FTS5 for name, generic name, keywords, package attribute and summary search.

Ranking order should roughly prefer:

1. exact display-name match
2. exact alias or package-attribute match
3. name prefix
4. generic name and keywords
5. summary
6. long description

No AI service is required for normal application search. The catalogue should remain fast, local and deterministic without an API key.

## nix package installation

The Marketplace source of truth should not be a sequence of ad-hoc shell installs.

At the same time, pressing Install must not rebuild an unrelated dirty Vesper checkout or require Marketplace to rewrite hand-maintained `apps.nix`.

Use a dedicated manifest-driven Nix profile for Marketplace-owned user applications.

The important distinction is:

```text
manifest = desired Marketplace application state
profile  = realized Nix environment derived from that state
```

The profile is not the authoritative database.

### state paths

Recommended paths:

```text
~/.config/vesper/marketplace/
└── manifest.json

~/.local/state/vesper/marketplace/
├── profile
├── generations.json
└── transactions/

~/.cache/vesper/marketplace/
└── media/

$XDG_RUNTIME_DIR/vesper/
└── marketplace.lock
```

The manifest contains only Marketplace-owned selections and source information. Existing Nix packages declared in Vesper config are detected separately and never copied into this manifest automatically.

A minimal shape is enough:

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

Do not store arbitrary Nix expressions in the manifest.

Package attribute paths must come from the trusted local catalogue and be represented as string path segments. The backend constructs Nix installables from validated values.

### pinning

Marketplace Nix installs must use the same nixpkgs revision as the Vesper build that provides the running Marketplace backend.

Do not let `nixpkgs#foo` silently follow a newer registry revision than the system.

Expose the locked nixpkgs revision to the backend at Nix build time. Installables can then be resolved against the exact revision.

When Vesper updates `flake.lock`, the new Marketplace backend and catalogue receive the new revision together. Marketplace can then offer a reconcile/update transaction for its managed applications.

This keeps these three things coherent:

```text
Vesper system revision
Marketplace catalogue revision
Marketplace package install revision
```

### transaction model

Every install, remove and update is a serialized transaction.

Recommended flow:

1. acquire the Marketplace runtime lock
2. load and validate the current manifest
3. calculate the desired next manifest
4. resolve every Nix installable against the pinned revision
5. ask Nix for a dry-run plan
6. surface any local-build or policy warning
7. realize the new package set
8. update the dedicated profile atomically
9. write the new manifest atomically
10. refresh installed-app inventory
11. let the existing adaptive-icon supervisor discover the new desktop entry
12. release the lock

If realization fails, the previous manifest and profile stay active.

Never update the manifest first and hope the package build succeeds later.

### dry-run and cache state

Do not guess whether a package is cached by looking at package popularity or Hydra metadata.

Ask Nix.

`nix build --dry-run --json --no-link` or the equivalent Nix library/CLI flow should be the authority for the requested installable.

If Nix reports that local builds are required, show a clear warning before the transaction continues.

Example language:

```text
Local build required
This package is not fully available from your configured binary caches.
```

Do not label every cache miss as an error. Some packages are intentionally built locally.

Marketplace uses the machine's existing trusted substituters and trusted public keys. It must never add a third-party binary cache automatically because an application wants one.

### rollback

Marketplace-owned Nix packages need an application-level rollback path independent of full system rollback.

Keep the previous successful profile generation and manifest snapshot for at least the last few Marketplace transactions.

Rollback should restore the whole Marketplace package set from one known-good generation, not attempt to reverse individual file changes inside `/nix/store`.

This fits Nix profile generation semantics and avoids inventing another package rollback mechanism.

### garbage collection

The active Marketplace profile and retained rollback generations must remain GC roots.

Old Marketplace generations can be pruned by an explicit retention policy. Do not let normal `nh clean` remove the currently active Marketplace applications.

The implementation should verify this with a real garbage-collection test before release.

## applications already managed by Vesper

Vesper already declares normal desktop applications in Home Manager.

Marketplace must detect installed desktop applications that are not part of its own manifest.

For a Nix application already provided by Vesper config:

```text
Installed
Managed by Vesper config
```

The Marketplace remove button is disabled or replaced with an informational action.

Do not silently migrate applications out of `home.packages` into Marketplace state.

A future `Manage in Marketplace` migration can exist only if it performs an explicit reviewed config change. It is not needed for v1.

## packages that need nixos integration

A desktop store should not pretend a package-only install is enough when an application depends on a NixOS module, service, user/group, firewall rule, PAM integration or another system setting.

Marketplace therefore supports three install classes:

```text
package
recipe
unsupported-system-integration
```

### package

A normal desktop application that works as a user package.

This is the default and should cover most of Marketplace.

### recipe

A reviewed Vesper integration for an application that needs more than a package.

Recipes are static code in the repository. They are never generated from catalogue text and Marketplace never accepts arbitrary Nix expressions from a remote source.

A recipe declares exactly what it changes and the detail view previews that before applying it.

Possible future recipe layout:

```text
modules/marketplace/
├── default.nix
└── recipes/
    └── <reviewed-id>.nix
```

Do not build the recipe engine before there are real desktop applications that need it. Package-only Marketplace should ship first.

### unsupported system integration

If Vesper has no reviewed recipe, show the application but do not present a false one-click install when it would produce a broken setup.

The detail page can explain that system configuration is required.

## flathub

Flathub is an optional source and is disabled by default.

This requirement is strict.

Default Vesper behavior:

```text
Flatpak service available        yes
Flathub remote auto-added        no
Flathub catalogue downloaded     no
Flathub search results shown     no
Flathub preferred over Nixpkgs   no
Flathub beta enabled             no
```

The existing Flatpak service can remain enabled because Vesper already has real Flatpak permission controls. That does not mean Flathub is enabled as a Marketplace source.

### enabling flathub

Marketplace has a small Sources sheet or menu.

Initial state:

```text
Nixpkgs     On    Vesper default
Flathub     Off   Optional sandboxed applications
```

Enabling Flathub requires an explicit user action.

Only then may Marketplace add or use the user Flathub remote and refresh its AppStream metadata.

Prefer the user installation scope for Marketplace-managed Flatpaks unless a real system-wide need appears.

Use the official remote definition and native Flatpak commands. Do not scrape flathub.org pages.

### flathub catalogue

When enabled, use Flatpak's configured remote and AppStream data.

Useful native operations include:

- `flatpak remotes`
- `flatpak remote-ls --app`
- `flatpak remote-info`
- `flatpak update --appstream`

Rich metadata comes from the remote AppStream catalogue.

Cache normalized Flathub metadata locally so search does not spawn a large Flatpak command on every keystroke.

### flathub install state

Flatpak remains the authority for Flatpak transactions and rollback semantics.

Marketplace records enough identity to merge installed Flatpaks into the same application model, but it must not fake Nix ownership of Flatpak state.

After installation the existing Vesper Apps controls remain authoritative for Flatpak permissions.

### disabling flathub

Turning off Flathub as a Marketplace source must stop new catalogue refreshes and hide Flathub-only discovery results.

It must not silently uninstall applications.

If installed Flatpaks still depend on the remote, keep the remote usable for those applications or require a separate explicit removal flow. Do not destroy an installed application's update path just because discovery was disabled.

## source deduplication

When Flathub is disabled there is nothing to merge.

When enabled, variants may be grouped only when identity is strong.

Preferred match keys:

1. exact AppStream component ID
2. exact desktop application ID
3. reviewed alias mapping

Do not merge applications only because their display names are similar.

Nixpkgs remains the default variant.

The detail view can show why someone might choose the alternative, particularly native integration versus Flatpak sandboxing.

## adaptive icon integration

Marketplace catalogue icons and installed adaptive icons are different problems.

Do not send every catalogue icon through AI conversion.

Before installation:

- use AppStream or packaged catalogue icon assets
- keep them read-only
- do not create adaptive-icon jobs

After installation:

- the real `.desktop` entry appears in the user environment
- the existing adaptive icon discovery pipeline resolves the packaged icon
- the installed app follows the normal Vesper icon generation rules

This preserves one adaptive-icon source of truth.

Flatpak applications follow the same installed-app discovery path after installation.

## installed application reconciliation

Marketplace should not trust only its manifest when deciding what exists on the desktop.

The installed inventory merges:

- visible desktop entries
- Marketplace Nix manifest
- installed Flatpaks
- current Vesper per-app identity status

A native desktop application that exists but is not in the Marketplace manifest is treated as config-managed or externally managed.

A manifest entry whose desktop entry disappeared is a reconciliation problem and should be surfaced instead of silently forgotten.

## backend shape

Keep the QML thin.

Do not put Nix parsing, Flatpak parsing, SQLite queries or transaction state machines in QML JavaScript.

Extend the existing Rust control plane with a Marketplace module first. Split into a separate service only if process startup or long-running transaction requirements prove that a CLI boundary is insufficient.

Suggested source layout:

```text
home/yargc/packages/
├── vesper-control.rs
├── vesper-marketplace.rs
├── VesperMarketplace.qml
├── VesperMarketplaceDetail.qml
└── marketplace-catalog.nix
```

Exact filenames may change if Caelestia patch structure makes another placement cleaner.

### command surface

A first CLI contract can look like:

```text
vesper-control marketplace status
vesper-control marketplace search <query>
vesper-control marketplace categories
vesper-control marketplace app <id>
vesper-control marketplace installed
vesper-control marketplace plan-install <id> [source]
vesper-control marketplace install <id> [source]
vesper-control marketplace remove <id>
vesper-control marketplace update <id>
vesper-control marketplace rollback
vesper-control marketplace sources
vesper-control marketplace source flathub on|off
vesper-control marketplace refresh
```

All machine-readable output is JSON.

Commands that mutate package state acquire the Marketplace lock.

Search commands do not need the mutation lock.

### process model

For v1, QML can keep using the same `Quickshell.Io Process` pattern already used by Vesper Apps.

Debounce text search before spawning a query process.

If measured search latency becomes visible, move search to a small long-lived Rust service or Unix socket. Do not introduce a daemon before profiling shows it is useful.

## transaction ui

Only one package mutation runs at a time.

A transaction should expose explicit phases:

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

The backend may collapse phases when Nix does not provide a distinction.

The UI shows the phase and the package name. It does not parse terminal spinner characters.

Cancellation is allowed only while it is safe. Once an atomic profile switch or Flatpak deployment is in its final commit phase, the UI should finish the transaction instead of pretending cancellation succeeded.

Errors should retain the useful Nix or Flatpak reason, but the first line shown to the user should be concise.

## updates

Marketplace application updates must respect the source owner.

### nixpkgs applications

An update becomes available when Vesper itself has moved to a newer locked nixpkgs revision and the selected application's resolved version or store path changes.

Marketplace does not independently chase nixpkgs unstable ahead of the Vesper system lock.

This prevents the application store from becoming a second rolling channel with a different package set.

### flatpak applications

When Flathub is enabled, Flatpak can report application updates independently because Flatpak owns that source.

Global presentation still belongs in `System -> Updates` once integration exists.

Marketplace detail may show `Update available` for the selected app.

## network behavior

Nixpkgs browsing should work from the local catalogue without a network request.

Network is used when:

- explicitly refreshing a catalogue artifact if that update model is implemented
- opening uncached screenshot media
- installing or updating packages through Nix substituters
- using Flathub after it has been enabled

Do not add telemetry, popularity beacons or recommendation tracking.

Do not load screenshots for off-screen result rows.

Use a bounded media cache and allow it to be cleared from Marketplace storage controls later.

## security and trust

Marketplace is not a new package trust root.

For Nix packages:

- use the existing locked nixpkgs source
- use existing Nix substituters
- use existing trusted public keys
- respect Nixpkgs broken/platform/insecure checks
- never add a binary cache automatically

For Flatpak:

- use the configured Flatpak remote trust model
- add Flathub only after explicit opt-in
- do not install arbitrary `.flatpakref` URLs from catalogue text

For Vesper recipes:

- recipe code lives in the Vesper repository
- recipe IDs are mapped to reviewed local Nix files
- remote metadata cannot supply executable Nix

No Marketplace operation should execute a shell command copied from an application description.

## privacy

Core Marketplace search is local.

No search queries need to leave the machine for Nixpkgs results.

Flathub queries should normally operate on locally refreshed remote metadata after opt-in rather than sending each search string to a web service.

Do not collect install history for analytics.

Local transaction history exists only for rollback and debugging and follows the same Vesper local-state convention as other control-plane state.

## failure behavior

Marketplace must degrade clearly.

### catalogue missing

Show the installed Apps surface normally and a Marketplace error state with a retry/rebuild action. Do not break the whole Apps page.

### nixpkgs revision mismatch

If the running catalogue revision and the backend's compiled nixpkgs revision do not match, block Nix Marketplace mutations until they are reconciled.

Browsing may continue with a visible stale-catalogue warning if the schema is compatible.

### nix unavailable

This is a Vesper integrity failure. Show the actual error and do not fall back to curl installers.

### flathub unavailable

Nixpkgs Marketplace remains fully usable.

A Flathub outage must not degrade Nixpkgs search.

### package build failure

Keep the previous Marketplace profile and manifest active.

Offer details and retry. Do not leave a half-installed application entry.

### corrupted local manifest

Refuse mutations, preserve the file and show a repair action. Never replace an unreadable manifest with an empty one automatically.

## performance targets

These are design targets, not fake guarantees.

- opening Marketplace should not trigger a full nixpkgs evaluation
- local text search should feel immediate
- search should stay usable while screenshots are loading
- scrolling result lists should not spawn one process per visible row
- application detail may perform one lazy metadata query when needed
- package realization time is controlled by Nix and the network, so the UI should optimize perceived responsiveness rather than promise fixed install times

Use measurement before introducing a resident daemon or a more complex database service.

## implementation phases

### phase 0: fixtures and architecture proof

Build a small catalogue fixture from the current locked nixpkgs revision.

Prove these mappings with real packages:

- package attr to AppStream component
- package attr to desktop file ID
- package attr to icon
- duplicate package variants
- unfree application
- application with missing screenshots
- application that would require a local build

Prove a dedicated Marketplace Nix profile survives logout, reboot and garbage collection.

Do not start with the full UI before these identities are reliable.

### phase 1: read-only nixpkgs marketplace

Implement:

- catalogue derivation
- SQLite schema and FTS search
- categories
- read-only Marketplace mode in Apps
- app detail
- source and version metadata
- installed detection
- `Managed by Vesper config` state

No install button until the installed identity mapping is trustworthy.

Acceptance:

- no network is required for Nixpkgs search
- broken and unsupported packages stay out of normal results
- search does not evaluate nixpkgs per query
- current Apps controls remain unchanged

### phase 2: nix install transactions

Implement:

- Marketplace manifest
- dedicated Nix profile
- transaction lock
- dry-run planning
- install
- remove
- profile reconciliation
- rollback
- adaptive-icon refresh after successful install

Acceptance:

- failed install leaves previous state active
- Marketplace never edits `apps.nix`
- Marketplace never deploys unrelated dirty Vesper repository changes
- installed package uses the same locked nixpkgs revision as the running Marketplace build
- normal install does not require root

### phase 3: update integration

Implement:

- catalogue/profile revision comparison
- update availability
- bulk reconcile after a Vesper nixpkgs update
- retained Marketplace generations
- System Updates integration without creating another update page

Acceptance:

- Marketplace cannot silently move ahead of Vesper's nixpkgs lock
- rollback restores the previous Marketplace generation

### phase 4: optional flathub

Implement only after the Nix path is stable.

Add:

- Sources UI
- explicit Flathub opt-in
- user Flathub remote management
- AppStream refresh
- normalized Flathub catalogue cache
- Nix/Flatpak deduplication by strong identity
- Flatpak install/remove/update operations
- existing permission-control handoff after installation

Acceptance:

- fresh Vesper has no Flathub remote added by Marketplace
- no Flathub metadata is downloaded before opt-in
- Nixpkgs remains the default source for duplicates
- disabling discovery never silently removes installed Flatpaks

### phase 5: reviewed recipes

Only add the recipe layer for real applications that prove package-only installation is insufficient.

Each recipe requires:

- a stable ID
- reviewed local Nix code
- a preview of system changes
- a rollback path
- full Vesper build validation

Do not turn Marketplace into a generic NixOS options editor.

### phase 6: polish

After the data and transaction paths are stable:

- keyboard navigation
- screenshot media cache
- empty states
- source badges
- transaction history
- accessibility review
- reduced-motion behavior where Caelestia exposes it
- catalogue diagnostics

Do not prioritize recommendation feeds, ratings or visual decoration ahead of install reliability.

## test matrix

### catalogue

- exact name search
- package attr search
- keyword search
- Unicode application names
- duplicate AppStream IDs
- missing icon
- missing description
- missing screenshot
- unsupported platform
- broken package
- insecure package
- unfree package

### nix transactions

- successful cached install
- install with a large dependency closure
- install that requires local build
- build failure
- network failure
- cancellation before profile switch
- remove
- update after nixpkgs revision change
- rollback
- concurrent install attempts
- corrupted manifest
- GC with active and retained generations

### installed identity

- app declared in `home.packages`
- app installed by Marketplace
- app installed outside Marketplace through another Nix profile
- Flatpak app
- app with multiple desktop files
- app with `NoDisplay` helper entries

### flathub

- Flathub absent on fresh system
- enable source
- metadata refresh failure
- install Flatpak
- permission controls visible after install
- duplicate Nixpkgs and Flatpak app
- disable source with installed Flatpak remaining
- remove final Flatpak then remove remote

### ui

- keyboard-only navigation
- narrow layout
- long application name
- long translated description
- no screenshots
- no network
- install failure
- stale catalogue
- active transaction while leaving and returning to Apps

## validation against Vesper rules

Implementation must still pass the repository change checklist.

In particular:

- no first-party Python files
- parse changed Nix files
- compile changed Rust
- build the configured Caelestia package after QML changes
- run `nix flake metadata --no-write-lock-file`
- evaluate the Home Manager activation package
- build the full Vesper system
- keep `flake.lock` changes intentional

Marketplace must not weaken the existing Apps permissions or adaptive-icon behavior.

## planned repository changes

A likely implementation touches these areas:

```text
docs/MARKETPLACE.md
home/yargc/caelestia.nix
home/yargc/packages/VesperAppsSettings.qml
home/yargc/packages/VesperMarketplace.qml
home/yargc/packages/VesperMarketplaceDetail.qml
home/yargc/packages/vesper-control.rs
home/yargc/packages/vesper-marketplace.rs
home/yargc/packages/marketplace-catalog.nix
```

Additional Nix wiring should stay small and follow the existing Home Manager layout.

Do not create multiple Marketplace architecture documents. This file is the single source of truth for Marketplace design and implementation decisions.

## decisions locked for v1

These choices should not be reopened during implementation without a concrete technical reason:

- Marketplace lives inside the existing Apps surface
- Nixpkgs is the default and primary source
- catalogue browsing is local
- catalogue data is tied to Vesper's locked nixpkgs revision
- the UI does not query `search.nixos.org` per search
- normal Marketplace packages are user-level Nix installs
- Marketplace state is isolated from hand-maintained Vesper package declarations
- existing Vesper-config applications are not silently migrated
- Flatpak service availability does not imply Flathub availability
- Flathub is opt-in and disabled on a fresh Vesper install
- Flathub beta is not enabled
- Nixpkgs wins source preference when both Nixpkgs and Flathub provide the same application
- no PackageKit backend
- no apt/rpm conversion
- no AI requirement for search
- no automatic third-party binary caches
- no remote executable recipe definitions
- no fake sandbox controls for native Nix applications
- no duplicate adaptive-icon pipeline

## research references

Primary and upstream references used for this plan:

- Nixpkgs reference manual: https://nixos.org/manual/nixpkgs/unstable/
- Nix profile reference: https://nix.dev/manual/nix/2.35/command-ref/new-cli/nix3-profile-add.html
- Nix build reference: https://nix.dev/manual/nix/2.35/command-ref/new-cli/nix3-build.html
- Nix substituter configuration: https://nix.dev/manual/nix/2.33/command-ref/conf-file
- Nix binary cache guide: https://nix.dev/guides/recipes/add-binary-cache.html
- NixOS search source: https://github.com/NixOS/nixos-search
- Nix Software Center: https://github.com/snowfallorg/nix-software-center
- NixOS AppStream data: https://github.com/snowfallorg/nixos-appstream-data
- NixOS Flatpak manual: https://nixos.org/manual/nixos/unstable/
- Flatpak command reference: https://docs.flatpak.org/en/latest/flatpak-command-reference.html
- Flatpak repository documentation: https://docs.flatpak.org/en/latest/repositories.html
- Flatpak AppStream conventions: https://docs.flatpak.org/en/latest/conventions.html
- Flathub installation documentation: https://docs.flathub.org/docs/for-users/installation
- freedesktop Desktop Entry specification: https://specifications.freedesktop.org/desktop-entry/latest/
