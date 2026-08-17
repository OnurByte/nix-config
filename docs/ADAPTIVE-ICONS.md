# adaptive app icons implementation prompt

This document is the implementation prompt for replacing Vesper's current experimental icon review queue with an automatic adaptive app icon system.

Do not treat the existing queue and manual review workflow as the target architecture. It is temporary scaffolding.

## mission

Implement a Vesper-native adaptive app icon engine for NixOS, Hyprland and Caelestia.

The system must discover installed Linux desktop applications from their `.desktop` entries, resolve each application's real source icon, convert unsuitable source assets into a canonical semantic SVG with an AI provider when needed, compile deterministic appearance variants, install those variants as a normal freedesktop icon theme and keep the active icons synchronized with the current Caelestia palette.

The result should feel like one coherent adaptive icon system rather than a collection of unrelated icon replacements.

Do not require the user to manually queue, review and activate every generated icon.

## product behavior

The finished behavior should be:

1. Vesper starts or the icon service is enabled.
2. The service scans the effective XDG application directories and discovers `.desktop` entries.
3. It reads the desktop id and `Icon=` value and resolves the actual installed icon asset rather than guessing by app name.
4. It fingerprints the source asset and checks the canonical cache.
5. Clean vector assets are normalized locally when possible.
6. Raster, non-semantic or unsuitable assets are sent to the selected configured vision-capable AI provider and reconstructed as a canonical semantic SVG.
7. The SVG is validated and rendered at several target sizes.
8. A deterministic appearance compiler creates the active Vesper icon theme.
9. The generated icon becomes active automatically only after validation succeeds.
10. If generation or validation fails, Vesper keeps the original icon with no broken placeholder.
11. New applications are detected automatically and processed without the user creating a job.
12. Palette, wallpaper, accent, light/dark mode or icon appearance changes recompile existing canonical assets without another AI request.

The user must be able to disable the feature and immediately return to the original icon theme.

## repository constraints

Follow `AGENTS.md`.

In particular:

- first-party Vesper services and CLIs must be Rust, not Python
- Nix/Home Manager owns installation and service wiring
- Caelestia remains the only shell and settings surface
- do not add another settings application
- keep custom Caelestia patches small
- preserve the Apple/visionOS-inspired controlled glass visual language
- do not overwrite immutable Nix store assets or application packaged icons
- avoid hidden mutable state outside the documented XDG state, cache and data locations

The implementation should extend the existing `vesper-control`, Caelestia settings integration and theme pipeline rather than creating an unrelated second control plane unless splitting a dedicated Rust worker is clearly cleaner.

## current Vesper integration points

Read the current implementation before changing anything, especially:

- `home/yargc/packages/vesper-control.rs`
- `home/yargc/packages/vesper-control.nix`
- `home/yargc/packages/VesperAppsSettings.qml`
- `home/yargc/packages/VesperAppControls.qml`
- `home/yargc/packages/AiPage.qml`
- `home/yargc/packages/VesperThemeSettings.qml`
- `home/yargc/caelestia.nix`
- `home/yargc/skills/vesper-adaptive-icons/SKILL.md`
- `docs/AI.md`
- `docs/APPS-SETTINGS.md`

The current `icon status|on|off` and `icon request` queue behavior is not the final contract.

## application discovery

Use the freedesktop/XDG application model as the source of truth.

Do not hard-code one application directory. Build the effective search path from `XDG_DATA_HOME`, `XDG_DATA_DIRS` and the active user environment, while correctly covering NixOS/Home Manager and Flatpak exported desktop entries.

Typical sources include user-local application entries, per-user Nix profiles, the system profile and Flatpak exports, but the implementation should derive the real effective paths instead of assuming one fixed layout.

For every discovered entry:

- use the desktop filename/id as the stable application identity
- parse `Icon=` exactly
- support absolute icon paths and theme icon names
- resolve theme names through the freedesktop icon lookup rules
- handle SVG, PNG, WebP, XPM and other formats that appear in real desktop entries
- prefer the highest quality installed vector source
- otherwise choose the highest useful raster source
- deduplicate duplicate desktop entries using normal XDG precedence
- respect hidden and disabled desktop entries
- retain enough source metadata to detect when an application's packaged icon changes after an upgrade

Never infer an icon only from the display name when the desktop entry provides a resolvable source.

## automatic change detection

A full scan must run at service startup.

After startup, watch the effective application directories and relevant exported icon locations for changes. Debounce noisy package-manager updates and batch rescans.

A periodic full rescan may exist as a recovery mechanism, but do not use polling as the primary mechanism when filesystem notifications are available.

The service must correctly handle:

- install
- uninstall
- desktop entry replacement
- icon source replacement after package upgrade
- Flatpak install/remove
- Home Manager activation
- NixOS rebuild or profile generation change

Removing an application should remove or garbage-collect its generated active asset without touching the original package.

## canonical icon format

AI output is not the final themed icon. AI output is a canonical semantic asset.

Store one canonical representation per source fingerprint and application identity.

The canonical asset should preserve recognizable brand geometry while exposing semantic roles that the appearance compiler can theme. At minimum support concepts equivalent to:

- background or container
- primary glyph
- secondary glyph/detail
- accent
- highlight/specular layer
- optional mask/safe area information

Use a fixed coordinate system and consistent safe-area rules across applications.

The canonical representation must remain vector-only when accepted as SVG. Do not allow embedded raster blobs inside the canonical SVG merely to satisfy the file extension.

Keep small sidecar metadata for information that does not belong naturally in SVG, such as schema version, source fingerprint, desktop id, source path, generation provider, validation status and semantic layer mapping.

Version the canonical schema from the beginning so future migrations do not require deleting the entire cache.

## AI generation contract

Do not use a normal image-generation workflow as the primary pipeline.

Use a vision-capable model that can inspect the source icon and return structured SVG/text output. The model's job is to reconstruct geometry and semantic layers, not to bake the current Vesper color palette into every icon.

The generation instruction must require the model to:

- preserve the application's recognizable mark and silhouette
- avoid inventing unrelated branding
- simplify details that do not survive small icon sizes
- produce clean vector paths
- keep transparent outer space where appropriate
- follow the Vesper safe area and container geometry
- separate semantic layers consistently
- avoid text unless the original mark genuinely depends on it
- avoid external resources, scripts, remote URLs, embedded raster data and fonts
- output only the requested canonical representation

If a source SVG is already clean and can be normalized deterministically, do not spend an AI request on it.

If a source is visually complex but recognizable vectorization can be done locally, prefer local normalization first. AI should be used where semantic reconstruction is actually useful.

Do not regenerate a canonical icon just because the accent color changed.

## provider integration

Reuse Vesper's existing API-key-only credential system. Do not introduce OAuth.

Provider selection should be capability-driven. A provider is eligible for icon conversion only if the configured model path can accept image input and return sufficiently large structured text/SVG output.

Do not hard-code the icon engine to one vendor if the existing Vesper provider abstraction can support multiple capable providers cleanly.

The settings UI must make it clear which provider is used for remote icon conversion.

Enabling automatic remote conversion is the explicit opt-in that permits source icon pixels/vector content to be sent to the selected provider. Send only the minimum required icon content. Do not upload unrelated desktop files, environment data or application usage history.

API keys must continue to use Secret Service and must not be written to the Nix store, command arguments, logs or generated metadata.

## validation and safety

Never activate raw model output directly.

Every generated canonical SVG must pass a strict local validator before it can enter the active theme.

Reject or sanitize at least:

- malformed XML/SVG
- scripts or event handlers
- `foreignObject`
- external URLs and network references
- embedded base64/data raster payloads
- external fonts
- unsupported filters that break common Linux renderers
- pathological path/node counts
- unreasonable canvas dimensions
- content outside the expected view box
- empty or effectively invisible icons

Render validation previews at multiple real icon sizes. The exact set may be chosen during implementation, but it must cover small launcher/tray-like sizes and larger application-grid sizes.

Validate that the mark remains visible, is not clipped and does not become a blank or solid rectangle.

Keep the previous known-good canonical and compiled asset until a replacement has passed validation.

## appearance compiler

Build a deterministic compiler that consumes canonical assets and the current Vesper/Caelestia palette.

Support these global appearance modes:

- Original
- Light
- Dark
- Tinted
- Clear
- Glass

The modes must not require new AI generations.

`Original` preserves the source brand colors as closely as possible while still using the common Vesper geometry and safe-area rules when a canonical asset exists.

`Light` and `Dark` produce readable self-contained assets for the respective desktop mode.

`Tinted` derives the icon palette from the active Caelestia accent/material palette and should behave like one global icon appearance rather than separately hand-colored icons.

`Clear` should emphasize the glyph and transparent/low-weight container treatment while remaining readable on normal Linux launchers.

`Glass` should provide a self-contained SVG approximation using transparency, gradients, soft borders, specular highlights and controlled shadows. It must still render correctly in ordinary GTK, Qt and Electron icon loaders that only receive an image asset.

Do not depend on `currentColor` alone for the desktop-wide compiled theme. Linux applications and toolkits do not all provide the same contextual color semantics to arbitrary application SVG icons. Compile concrete self-contained SVG colors for the active theme.

Semantic `currentColor` or KDE-style palette roles may still be used inside Vesper/Caelestia-controlled surfaces where the renderer is known.

## real glass inside Vesper surfaces

Separate a glass-looking icon asset from real dynamic backdrop glass.

Normal application launchers should receive self-contained compiled SVGs because freedesktop icon loading does not provide arbitrary icons with the live wallpaper/backdrop texture behind them.

Inside Vesper-owned Quickshell/Caelestia surfaces such as the launcher, dock-like surfaces, drawers or app grids, the canonical icon metadata may be combined with runtime effects such as backdrop sampling, blur, refraction/distortion, tint and specular lighting.

Do not fake a promise that a freedesktop SVG alone can perform live wallpaper refraction in every application.

Keep runtime glass effects bounded and consistent with the existing controlled-glass UX contract.

## icon theme installation

Expose compiled application icons through a normal freedesktop icon theme rather than rewriting every `.desktop` file.

The intended result is a generated Vesper icon theme under the user's XDG data directory, with scalable application icons and the metadata required by the icon theme specification.

Do not modify files in `/nix/store`.

Do not copy modified assets back into application packages.

Do not mutate every desktop entry unless a specific broken application cannot be handled through normal icon theme precedence and there is no cleaner fix.

Preserve an immediate rollback path to the previous configured icon theme.

The generated theme should inherit from a maintained fallback theme so icons that Vesper has not generated, symbolic UI icons and non-application assets do not disappear.

## state, cache and data layout

Use XDG locations consistently.

Keep operational state under the existing Vesper state root, generated persistent user assets under the Vesper user data root and disposable render/provider caches under the user cache root.

The final layout should clearly separate:

- discovered app inventory
- canonical assets
- compiled active theme
- source fingerprints
- generation/validation status
- failures and retry metadata
- disposable previews/cache

Do not retain duplicate source icon copies indefinitely when they can be resolved again from installed packages.

Do not store API keys, full provider responses containing secrets or unrelated app metadata in these directories.

## failure handling

The icon system must fail closed visually.

For any individual application, the fallback order should be equivalent to:

1. last known-good compiled Vesper icon
2. last known-good canonical asset recompiled with the current palette
3. original packaged icon through the inherited icon theme

A failed generation must never leave a missing icon.

Track failures per source fingerprint so one broken app does not block the entire theme rebuild.

Use bounded retries with backoff for remote provider failures. Do not create a tight API retry loop.

A global theme/palette switch must still succeed even if the remote AI provider is offline because existing canonical assets are compiled locally.

## settings integration

Remove the final product concept of `Apps -> Experimental -> AI adaptive icons` as the primary control.

Split responsibilities according to what the user is controlling.

### AI page

Add an `Adaptive icons` section to the existing native `AI` page.

It should expose at least:

- automatic adaptive icons enabled/disabled
- selected capable AI provider
- discovered application count
- canonical/generated count
- pending conversion count
- failed count
- current conversion activity
- regenerate failed
- rebuild canonical assets when explicitly requested

This page owns AI generation because provider choice, remote conversion and generation health are AI concerns.

Do not put global color appearance controls here.

### appearance/theme page

Extend the existing Vesper theme settings page with application icon appearance controls.

It should expose at least:

- appearance mode: Original / Light / Dark / Tinted / Clear / Glass
- follow current Caelestia accent/palette
- useful Glass controls only if the runtime/compiler really supports them
- current generated Vesper icon theme status
- rebuild local icon theme

Changing appearance, accent, wallpaper-derived palette or light/dark mode must trigger local recompilation only. It must not enqueue remote AI work for already canonicalized icons.

Replace the current Papirus-only presentation with the generated Vesper theme when adaptive icons are active while retaining Papirus or another maintained theme as inheritance/fallback.

### Apps page and per-app controls

Keep Apps focused on application-specific information.

Per-app controls may show:

- original source preview/path identity
- canonical status
- active appearance preview
- regenerate this icon
- revert this app to original
- retry after failure
- exclude this app from adaptation

Do not require per-app approval in the normal successful path.

## theme synchronization

Integrate with the existing Caelestia scheme pipeline.

Today Caelestia already propagates one palette to shell, GTK and Qt. Adaptive application icons should become another consumer of that same palette rather than creating a second theme source.

When the effective Caelestia palette changes:

- debounce rapid changes
- recompile only the required icon variants
- update the active generated theme atomically
- invalidate icon caches where necessary
- notify/reload only the surfaces that actually require it

Avoid visible periods where half the icons use the previous palette and half use the new one. Prefer staging plus atomic directory/symlink generation switches where practical.

Do not block wallpaper/theme switching on remote AI generation.

## caching and deduplication

AI calls should be rare after first conversion.

Cache canonical generation by a stable combination of source content fingerprint and canonical schema/generator version.

A palette change must be a cache hit for canonical geometry.

An application upgrade that changes the actual source icon must invalidate that app's canonical fingerprint and regenerate only that app.

If multiple desktop entries resolve to the same source icon and semantics, reuse canonical work where safe while retaining the correct desktop ids in the compiled theme.

Keep enough version metadata to invalidate assets when the canonical schema, prompt contract or validator changes materially.

## source quality and fallback assets

Prefer the application's installed official vector asset when it exists.

Do not automatically replace a good official SVG with an AI approximation merely for consistency.

When the installed source is a poor raster asset, AI semantic reconstruction is appropriate.

A future maintained canonical-logo source may be used as a fallback for common applications, but it must not silently become an unpinned network dependency at every theme rebuild. Any remote canonical source should be fetched deliberately, cached with provenance and treated as input to the same validator.

## research references

Use these projects and subsystems as implementation references, not mandatory dependencies:

- freedesktop Icon Theme Specification for lookup and theme layout
- KDE `KIconLoader` and Breeze SVG color semantics
- Papirus for broad Linux icon-theme inheritance/coverage patterns
- Themix/Oomox for generated icon-theme recoloring ideas
- Color-manager for bulk palette mapping and SVG manipulation ideas
- Kando for practical SVG `currentColor` behavior in a controlled renderer
- Matugen-style wallpaper-to-palette pipelines
- MacTahoe icon themes for Linux packaging/coverage of macOS-like application icons
- macOS Tahoe/Liquid Glass KDE projects for compositor-side glass/refraction ideas

Do not copy an entire third-party theme or make Vesper depend on an abandoned project when the needed behavior is small enough to implement directly.

## implementation shape

Choose the smallest architecture that keeps responsibilities clear.

A reasonable target is:

- Rust discovery/watch/generation orchestration
- Rust or established native command-line SVG tooling for validation/normalization where appropriate
- deterministic local appearance compiler
- generated freedesktop Vesper icon theme
- existing Secret Service credential path for provider access
- existing Caelestia/QML settings surfaces
- Nix/Home Manager for packages, service wiring and defaults

Do not add Python.

Do not introduce a web service or browser UI.

Do not create another agent/orchestrator just for icons.

The existing `vesper-adaptive-icons` skill may remain useful for diagnostics or explicit repair, but routine icon generation must not depend on manually invoking an agent skill.

## migration from the current queue

Replace the queue-centered model rather than layering another automatic system beside it.

During implementation:

- migrate or safely ignore old queued metadata
- stop presenting manual review as the normal workflow
- remove stale queue-only UI and documentation
- preserve generated assets only when they pass the new validator or can be migrated safely
- keep rollback to original packaged icons at every stage

Do not leave two competing activation paths.

## observability

Provide concise machine-readable status through the Vesper control plane so QML does not scrape logs.

Status should be sufficient for the settings pages to show inventory counts, active mode, selected provider, current work and per-app failures.

Logs should identify desktop id, source fingerprint prefix, pipeline stage and error category without leaking API keys or dumping full remote responses.

Normal theme recompilation should be quiet.

## acceptance criteria

Do not call the feature complete until all of these are true:

1. A newly installed application with a valid `.desktop` file is discovered automatically.
2. Its real installed icon is resolved from `Icon=` without guessing the app name.
3. A clean source SVG can avoid a remote AI call.
4. A raster-only application can be converted through a configured capable AI provider into a validated canonical SVG.
5. Successful validated output becomes active automatically when the feature is enabled.
6. A malformed or unsafe AI SVG is rejected and the original icon remains visible.
7. Original, Light, Dark, Tinted, Clear and Glass modes can be switched globally.
8. Changing the Caelestia accent/palette updates Tinted icons without an AI request.
9. Changing light/dark mode updates the active compiled icons without an AI request.
10. The generated icons appear through a normal freedesktop icon theme in GTK/Qt application launch surfaces.
11. Vesper/Caelestia-owned surfaces can use richer runtime glass treatment without making that a requirement for ordinary icon loaders.
12. Disabling adaptive icons returns the desktop to the configured fallback/original icon theme.
13. Provider outage does not break theme switching or existing icons.
14. Application upgrades invalidate only icons whose actual source content changed.
15. Uninstalling an app does not leave a permanently active broken icon entry.
16. No first-party Python is added.
17. No API key enters the Nix store, logs, generated SVG or metadata.
18. NixOS/Home Manager evaluation and the repository's required build checks pass.

## implementation order

Implement in this order unless repository inspection proves a different dependency order is safer:

1. define canonical schema and source fingerprinting
2. implement XDG desktop/icon discovery and inventory
3. implement local SVG normalization and strict validation
4. implement the deterministic appearance compiler
5. generate and activate a Vesper freedesktop icon theme with fallback inheritance
6. connect Caelestia palette/light-dark changes to local recompilation
7. add provider-backed semantic conversion for assets that actually need AI
8. add filesystem watching and incremental reconciliation
9. replace the queue/manual-review control path
10. integrate AI, Appearance and per-app settings surfaces
11. add runtime glass enhancements in Vesper-owned surfaces
12. add migration, rollback, failure recovery and cleanup
13. update docs to describe the implemented behavior rather than this target plan

At each stage keep the desktop usable with original icons if the new path fails.
