# Apple icon fidelity and export contract

This document is the normative fidelity layer for the Vesper adaptive icon system.

Read it together with:

- `ADAPTIVE-ICONS.md`
- `ADAPTIVE-ICONS-AI.md`
- `ADAPTIVE-ICONS-LAYERED.md`
- `APPLE-ICON-MODEL.md`
- `APPLE-ICON-AUTOFIT.md`

If an older document conflicts with this one on calibrated icon geometry, flattened compatibility footprints, blend modes, system-gradient backgrounds, runtime application identity, tray/status icon handling, renderer lighting, appearance fallback or bulk export behavior, follow this document.

The goal is to move Vesper from an Apple-looking icon theme toward an independent Linux implementation of the current Apple Icon Composer authoring and runtime model.

## fidelity principles

The adaptive icon system must preserve this separation:

```text
source application artwork
        ↓
AI-assisted semantic decomposition
        ↓
canonical multi-layer `.vicon`
        ↓
appearance semantics
        ↓
versioned material renderer
        ↓
Vesper-owned live output
        +
flattened freedesktop compatibility output
```

Do not encode renderer-generation-specific visual effects permanently into canonical artwork.

Do not ask AI to regenerate icons merely because the Vesper renderer changes.

Do not attempt to clone Apple's private `.icon` serialization or depend on Apple's proprietary `actool`/AssetCatalog toolchain at runtime.

Copy the public authoring/rendering model, not the private build chain.

## canonical model remains multi-layer

`ADAPTIVE-ICONS-LAYERED.md` remains authoritative for the canonical package.

The canonical result is not one flattened SVG.

Conceptually:

```text
app.vicon/
├── manifest.json
├── background/
├── groups/
│   ├── 01-base/
│   ├── 02-primary/
│   ├── 03-detail/
│   └── 04-optional/
└── appearances/
    ├── default.json
    ├── dark.json
    └── mono.json
```

The package uses a shared unmasked `1024 x 1024` coordinate system.

One logical background plus one to four foreground effect/depth groups is the normal target.

Each group may contain multiple artwork layers.

## combined and individual material modes

Every foreground group must support the semantic distinction between combined and individual material treatment.

Conceptually:

```json
{
  "renderMode": "combined"
}
```

means member artwork is composed first and then treated as one material surface.

```json
{
  "renderMode": "individual"
}
```

means member layers can receive material response independently while retaining their common group ordering.

The AI may recommend the mode, but local validation owns acceptance.

Do not split every vector path into an individual material surface.

Do not flatten genuinely separate overlapping surfaces merely to reduce group count.

## bounded blend-mode model

The current canonical/material schema must represent blend intent separately from SVG artwork.

Support a bounded semantic set equivalent to:

```text
auto
normal
multiply
screen
darken
lighten
plus-lighter
plus-darker
```

`auto` is the normal AI recommendation.

The renderer may resolve `auto` differently by appearance, renderer recipe and luminance context.

The AI must not output arbitrary compositor expressions or shader code.

Blend modes belong in group/layer metadata, not as unconstrained CSS/filter tricks embedded into generated SVG.

For dark and mono compositions, prefer a deterministic blend-mode choice that preserves the recognizable geometry before changing the geometry itself.

## background generation must be system-like

Do not let every AI conversion invent a different decorative gradient.

Background intent should be semantic.

Support strategies equivalent to:

```text
brand-solid
brand-gradient
system-brand-gradient
system-light
system-dark
palette-surface
transparent
artwork
```

For a simple colored application background, prefer storing a stable brand color and using a versioned Vesper system-gradient recipe rather than baking arbitrary gradient stops into SVG.

Conceptually:

```json
{
  "background": {
    "strategy": "system-brand-gradient",
    "brandColor": "#rrggbb"
  }
}
```

The material renderer owns the actual gradient recipe.

This allows Vesper to improve the common system look without asking AI to reconstruct every icon again.

### system light and system dark

Define versioned Vesper equivalents of Apple's current system-like light and dark icon backgrounds.

Do not use pure white and pure black as the universal fallback unless the actual brand requires them.

The system-light/system-dark recipes should preserve enough luminance structure for specular edges, shadows and clear/tinted transformations to remain visible.

Legacy circular/irregular icons that need an automatic enclosure should prefer these system-like recipes before an arbitrary AI-invented square background.

## Apple grid calibration harness

`824 x 824` on a `1024 x 1024` canvas must no longer be treated as the universal current Apple source of truth.

It may remain a useful historical/regression reference for flattened macOS-style output, but current Vesper geometry must come from a versioned calibration process.

The implementation must provide a calibration harness that can record measurements derived from the current public Apple design resources and representative Icon Composer output during development.

The harness is a developer/build-time reference process, not a runtime Apple dependency.

At minimum record:

- source canvas size
- final flattened enclosure alpha bounds
- designated circular-artwork guide
- primary-content guide
- optical safe region
- corner/enclosure geometry
- representative default/dark/mono output bounds
- measurement source/revision/date

Conceptual metadata:

```json
{
  "family": "apple-unified-app-icon",
  "revision": "2026-current",
  "canvas": 1024,
  "flattenedFootprint": {
    "width": 0,
    "height": 0
  },
  "circularGuide": {},
  "primaryGuide": {},
  "measuredAt": "YYYY-MM-DD",
  "source": "Apple Design Resources / reference Icon Composer output"
}
```

Exact values are intentionally not hard-coded in this document.

Once measured, commit the derived Vesper constants declaratively and version them.

A later Apple grid revision must create a new renderer/grid revision instead of silently moving every existing icon.

## legacy 824 value

Treat `824 / 1024` only as a compatibility heuristic until the current calibrated reference confirms an equivalent footprint.

Do not size circular foreground artwork directly from the flattened enclosure footprint.

The circular artwork guide and the outer enclosure footprint are separate measurements.

Do not let a Spotify-style circular mark fill the entire outer tile.

## AI geometry ownership

For clean official SVG/vector sources, the AI should act primarily as a semantic director rather than needlessly redrawing exact brand curves.

Preferred path:

```text
official SVG
    ↓
local sanitize + geometry extraction
    ↓
rendered visual reference
    +
structural/vector summary
    ↓
GPT vision semantic decomposition
    ↓
assign existing geometry to background/groups/layers
    ↓
local canonical package construction
```

Use AI-generated replacement vector geometry when the source is raster, structurally unusable or cannot be decomposed while preserving identity.

This reduces brand drift.

For raster sources, local segmentation/vector candidates may be supplied to the model as additional evidence.

AI still decides semantic meaning; local code should preserve reliable original geometry whenever possible.

## depth weaving

The decomposition model must understand that one visual object can occupy different apparent depths.

For example, a ribbon, flame or line may visually pass behind another surface and then return to the foreground.

The canonicalizer may therefore split one semantic visual object into multiple artwork layers when this is necessary to preserve the original image while reproducing the intended depth.

Do not treat semantic-object identity and depth-layer identity as the same thing.

Metadata should be able to associate multiple depth fragments with the same semantic object id.

Conceptually:

```text
semantic object: ribbon
    ├── rear fragment  → group 1
    └── front fragment → group 3
```

Do not invent weaving that does not exist in the source icon.

## system lighting model

Do not fake dimensionality by painting one permanent white diagonal highlight over every icon.

Vesper-owned surfaces must use a renderer-level light model.

The current recipe should define a stable system light direction and derive material response from surface contours and luminance.

Conceptually:

```text
system light direction
        ↓
layer/group contour
        ↓
background + local luminance
        ↓
specular placement
        ↓
shadow/depth response
```

The light model belongs to the renderer recipe, not `.vicon` artwork.

A future renderer revision may change lighting without invalidating canonical geometry.

## specular model

Support semantic strategies equivalent to:

```text
auto
inside
outside
off
```

`auto` is default.

`auto` should consider at least:

- local/background luminance
- edge contrast
- group render mode
- layer width/detail density
- current renderer recipe

Do not render identical specular intensity on every group.

Tiny details, thin strokes and dense marks should often reduce or disable specular response.

## selective refraction

Refraction should be local and depth-aware.

Prefer refraction where an upper translucent surface overlaps meaningful lower artwork.

Refraction may sample:

1. lower icon layers/groups
2. optionally the Vesper surface backdrop where the renderer supports it

The first path is required for convincing icon-internal depth.

The second path is optional and only available in Vesper-owned surfaces.

Do not apply strong refraction to:

- text
- tiny glyphs
- thin outlines
- dense logos
- identity-critical geometry that becomes distorted

The static freedesktop renderer must flatten an approximation without pretending to provide live backdrop refraction.

## shadow model

Shadows communicate separation between depth surfaces.

Generate them from the actual canonical surface geometry.

Do not accept old baked external shadows as scale geometry.

Use renderer recipes equivalent to:

```text
off
neutral
restrained-chromatic
auto
```

Shadow strength and spread must remain bounded and icon-scale aware.

Do not turn the icon into a floating card with a large desktop-window shadow.

## neighboring-icon validation

An icon is not validated in isolation.

Create a reference-board validation step.

Every generated/updated icon should be previewed beside a stable representative set of known-good Vesper icons at multiple sizes.

At minimum compare:

- optical footprint
- occupied-area ratio
- apparent visual weight
- enclosure size
- depth intensity
- background luminance
- specular strength
- small-size recognition

Test at least:

```text
16
24
32
48
64
128
256 px
```

A candidate that is technically valid but visually much larger/smaller than neighboring icons fails style validation.

This is especially important for circular and transparent Linux source icons.

## appearance-aware fallback

Do not fall directly from a failed Tinted or Clear conversion to a bright original multicolor icon if a coherent adaptive fallback can be generated locally.

For a requested adaptive appearance, use a fallback order equivalent to:

```text
last-known-good requested appearance
        ↓
deterministic canonical mono-derived appearance
        ↓
legacy-auto-fit adaptive wrapper
        ↓
original packaged icon
```

Mark the state as degraded when the requested appearance cannot be fully reproduced.

The failure of one appearance must not invalidate known-good outputs for the same app.

## Clear material independence

Clear icon material must remain independent from the shell's general transparency preference.

The icon renderer owns a pinned, tested Clear recipe.

Changing panel/window transparency must not make Clear icons illegible.

Clear Light and Clear Dark remain derived from `mono` semantics plus the current material/background recipe.

## runtime application identity resolver

The generated icon theme is not sufficient by itself on Linux.

Vesper must maintain a canonical application identity resolver so running windows and launchers resolve the same icon.

The resolver should reconcile evidence equivalent to:

```text
desktop id
StartupWMClass
Wayland app_id
X11 WM_CLASS
Flatpak app id
executable identity
known explicit aliases
```

into one stable Vesper application identity.

Do not guess solely from the visible window title.

Do not use fuzzy display-name matching as the primary mapping when exact metadata exists.

The same canonical identity must drive Vesper-owned:

- launcher/app grid
- dock/task surface if present
- window switcher/Alt-Tab surface if controlled by Vesper
- Apps settings page
- per-app adaptive icon status

Acceptance requirement:

```text
launcher icon
== running-state icon
== switcher icon
== app-grid icon
```

for Vesper-owned surfaces.

If an application supplies a runtime bitmap icon hint that conflicts with a known canonical Vesper application identity, Vesper-owned surfaces should prefer the canonical Vesper icon instead of visually breaking the adaptive theme.

Do not mutate the application's own process icon APIs globally merely to enforce styling.

## tray and status icons are a separate class

Do not apply the adaptive app-icon enclosure pipeline to tray/status icons.

A 1024-square layered glass application icon usually becomes illegible when shrunk into a tiny status area.

Use a separate symbolic derivation path:

```text
canonical app artwork
        ├── application icon pipeline
        │      → launcher / app grid / dock / switcher
        │
        └── optional symbolic status derivation
               → tray / StatusNotifier / AppIndicator-like surfaces
```

Tray/status output should normally be:

- monochrome/template-like
- high contrast
- background-free
- optically filled for the target tiny size
- independent from app-icon glass enclosure

Do not automatically reuse the application squircle as the tray icon.

Do not derive a tray icon when the application already supplies a suitable maintained symbolic/status icon.

## bulk export product requirement

The user must be able to export generated icons in bulk.

Add a visible action in the adaptive icon UI equivalent to:

`Export all icons`

This is not an AI-generation operation.

Export uses the current known-good canonical cache and local renderer.

It must not send any additional icon data to a provider merely because the user exports files.

### placement

The primary bulk-export action belongs in the Appearance/Theme adaptive-icon section because it exports rendered/icon-theme artifacts.

The AI page may show generation status but should not own the main export action.

A per-app `Export icon` action may additionally exist in Apps controls.

### export scope

Bulk export should support at least:

- all accepted canonical `.vicon` packages
- current active appearance as flattened SVG
- current active appearance as PNG
- optionally all generated appearances
- metadata/provenance manifest

Do not require every format in one export if the UI provides a compact format selector.

Recommended UI concept:

```text
Adaptive icons

[ Export all icons ]

Format
  Current appearance (SVG)
  Current appearance (PNG)
  All appearances
  Canonical .vicon packages
  Complete archive
```

The exact QML layout may differ.

### complete archive

A complete archive should be self-describing and suitable for backup or use outside the live cache.

Conceptually:

```text
vesper-icons-export/
├── manifest.json
├── canonical/
│   └── *.vicon/
├── current/
│   ├── svg/
│   └── png/
└── appearances/
    ├── default/
    ├── dark/
    ├── clear-light/
    ├── clear-dark/
    ├── tinted-light/
    └── tinted-dark/
```

Only include appearance directories requested by the selected export mode.

### filenames

Use stable sanitized application identities/desktop ids for filenames.

Do not use translated display names as the only file identity.

Avoid filename collisions across multiple desktop entries.

### atomic export

Render into a staging directory first.

Do not leave a partially updated export tree if one icon fails.

For an export request:

1. snapshot the current accepted inventory
2. render requested outputs into staging
3. record per-app export success/failure
4. write final manifest
5. publish/move the completed export atomically where practical

An individual failed icon may be represented as a manifest failure while the rest of the export succeeds if the UI clearly reports the partial result.

Do not corrupt or modify the active icon cache while exporting.

### export metadata

The export manifest should include non-secret provenance equivalent to:

- export schema version
- export timestamp
- Vesper renderer recipe version
- Apple-grid calibration revision
- active appearance/material
- application id
- source fingerprint
- canonical schema version
- canonical state (`canonical-ai`, `canonical-local`, `legacy-auto-fit`, `original`)
- available appearances
- output filenames
- degraded/fallback state

Do not export:

- API keys
- Secret Service identifiers that expose secrets
- raw provider authorization headers
- unrelated user paths
- provider responses containing private metadata

## settings UI requirements

Appearance/Theme adaptive-icon section should expose at least:

- icon appearance: Automatic / Default / Dark / Clear / Tinted
- icon material: Standard / Glass
- follow Caelestia accent
- active renderer recipe
- generated theme status
- rebuild local icon theme
- `Export all icons`

Developer/debug status may additionally expose:

- Apple-grid calibration revision
- renderer recipe revision
- canonical/degraded counts
- identity mapping failures

Do not expose raw per-layer shader numeric controls in the normal UI.

AI page remains responsible for:

- provider readiness
- existing API-key status
- conversion activity
- generated/pending/failed counts
- retry/regenerate operations

Apps/per-app controls may expose:

- canonical state
- active preview
- original preview
- use original
- regenerate
- retry
- exclude
- export this icon

## acceptance criteria

The adaptive icon implementation is not considered fidelity-complete until all of the following hold:

1. canonical output is a semantic multi-layer package rather than a single flattened SVG
2. background and foreground groups remain separate
3. one to four depth/effect groups are the normal ceiling
4. group render mode supports combined and individual material treatment
5. blend intent is a bounded semantic property
6. renderer supports versioned system-like brand/light/dark background recipes
7. current geometry comes from a versioned calibration, not an assumed universal `824 x 824` constant
8. `824 / 1024` is retained only as a regression/reference heuristic unless calibration confirms it
9. circular foreground sizing uses a separate calibrated circular guide
10. clean official vector geometry is preserved locally where possible while AI supplies semantic decomposition
11. one semantic object may have multiple depth fragments when the source composition requires weaving
12. lighting/specular response is renderer-driven rather than painted into source SVG
13. specular supports auto/inside/outside/off semantics
14. refraction is selective and can operate between icon layers
15. Clear material is independent from general shell transparency
16. Tinted/Clear failure uses an adaptive fallback before falling back to original multicolor artwork
17. neighboring-icon reference-board validation catches optical-size and material-strength outliers
18. runtime Vesper surfaces use canonical application identity mapping across launcher/running/switcher states
19. tray/status icons are excluded from the full adaptive app-icon enclosure pipeline
20. the user can export all accepted icons in bulk
21. bulk export does not trigger AI calls
22. bulk export can include canonical packages and flattened SVG/PNG outputs
23. export output includes non-secret provenance and renderer/grid revision metadata
24. exporting cannot damage or mutate the active icon cache
25. renderer recipe upgrades recompile locally without AI reconstruction

## implementation priority additions

When the implementation phase starts, prioritize these fidelity pieces after the core layered canonical pipeline works:

1. Apple-grid calibration constants and developer harness
2. bounded blend-mode schema
3. versioned system-light/system-dark/system-brand-gradient recipes
4. runtime application identity resolver
5. neighboring-icon optical validation board
6. appearance-aware fallback chain
7. tray/status exclusion and symbolic derivation path
8. renderer lighting/specular/refraction fidelity
9. bulk export backend
10. Appearance/Theme `Export all icons` UI

Do not block basic icon generation on the final live-glass renderer. Canonical package generation, identity, validation, deterministic static rendering and export should remain useful independently.
