# apple-style icon auto-fit and compatibility contract

This document is the normative geometry and compatibility contract for Vesper adaptive app icons.

Read it together with `ADAPTIVE-ICONS.md` and `APPLE-ICON-MODEL.md`.

If either older document conflicts with this one on legacy icon auto-fit, irregular/circular icon normalization, current Apple renderer behavior, clear/tinted contrast, or compatibility fallback geometry, follow this document.

The purpose is not to guess Apple's private implementation. The purpose is to reproduce the externally visible design behavior closely enough that Linux application icons have the same optical sizing, enclosure discipline and adaptive-material rules.

## current Apple baseline

Treat the current Apple app icon model as the reference, not the pre-Liquid-Glass Big Sur model.

As of August 2026 the relevant public behavior is:

- iOS, iPadOS and macOS use a `1024 x 1024` square layered design canvas
- the system applies the final rounded-rectangle mask
- circular artwork has a designated frame with more breathing room than the outer canvas
- existing rounded-rectangle-like Mac icons may be masked or extended into the current template
- unique or irregular legacy Mac icons have existing drop shadows removed and are automatically scaled into the rounded-rectangle canvas
- irregular icons may receive a system-provided background
- Default, Dark and Mono are the canonical annotation axes used to derive the user-facing appearances
- Liquid Glass effects belong to the renderer, not baked source artwork
- the 2026/27 renderer is sharper and more defined than the first Tahoe renderer
- current Icon Composer adds selective refraction and updated specular controls
- current guidance reduces translucency compared with the first Liquid Glass release to improve legibility

Do not freeze Vesper to a Tahoe-26 visual recipe when the current Apple renderer has moved forward.

## exactness policy

Do not invent a universal percentage such as `72%` for circular icons.

Do not claim that one community-measured number is Apple's private auto-scale algorithm.

Apple publicly provides the current production grid through Apple Design Resources but does not publish the complete private auto-fit coefficients used for arbitrary legacy artwork.

Therefore Vesper must separate two things:

1. public template geometry that can be measured exactly
2. private legacy auto-fit behavior that must be reproduced from public rules plus deterministic optical fitting

The implementation must capture the current official app-icon production grid at implementation time and encode only the derived numeric guides needed by Vesper.

Do not add a runtime network dependency on Apple resources.

Record provenance in source form, for example:

```json
{
  "gridFamily": "apple-unified-app-icon",
  "gridRevision": "2026-current",
  "canvas": 1024,
  "source": "Apple Design Resources App Icon Template",
  "measuredAt": "YYYY-MM-DD"
}
```

The exact metadata schema may differ.

When Apple revises the public grid, update the versioned Vesper grid constants deliberately rather than silently changing all icons.

## compatibility envelope for flattened Linux output

For normal freedesktop icon-theme output, keep a separate compatibility envelope from the canonical unmasked artwork.

The widely used macOS flattened fallback footprint is an approximately `824 x 824` centered tile on a `1024 x 1024` canvas, leaving approximately `100 px` transparent gutter on each side.

That corresponds to:

```text
canvas           1024 x 1024
fallback tile     824 x 824
left/top gutter   100 px
right/bottom      100 px
base scale        824 / 1024 = 0.8046875
```

Use this as the default static macOS-style enclosure footprint for Vesper freedesktop compatibility output unless the measured current Apple production grid proves that the corresponding guide has changed.

Do not confuse the `824 x 824` outer compatibility tile with the inner designated frame for a circular logo. They are different concepts.

The current official template remains authoritative for the inner circular and optical guides.

Do not make a circular Spotify-style logo fill the whole `824 x 824` enclosure merely because the enclosure itself uses that footprint.

## two normalization paths

Vesper must distinguish legacy compatibility from a true canonical redesign.

### legacy auto-fit

Use this when the installed source is circular, irregular, transparent, pre-masked, or otherwise not structured like a current layered Apple-style app icon.

This path should imitate Apple's automatic compatibility treatment rather than pretending the legacy artwork was intentionally designed for Liquid Glass.

The normal sequence is:

```text
installed source
    ↓
isolate visible artwork
    ↓
identify and remove legacy external shadow/effect footprint
    ↓
classify silhouette
    ↓
fit artwork into the current Apple-derived grid
    ↓
apply system-style enclosure/background when needed
    ↓
apply Vesper material recipe
    ↓
compile Linux compatibility asset
```

For this path, a system-style background is the default for circular/irregular transparent artwork.

Do not invent a different brand-derived square background for every legacy icon merely because dominant-color extraction is available.

A stable neutral or palette-compatible enclosure is closer to Apple's compatibility behavior.

Brand color can still participate when it materially improves recognition, but this must be a controlled rendering decision rather than an AI guess.

Mark this output as compatibility-derived in metadata.

### canonical redesign

Use this when local restructuring or AI reconstruction successfully produces current Icon-Composer-like source artwork.

This path may intentionally define a brand background because the background is now part of the designed canonical asset rather than an automatic wrapper around a legacy logo.

The canonical redesign should therefore be preferred after it passes validation.

Legacy auto-fit is the safety net. It is not the final quality target for every app.

## silhouette classes

Retain the classes defined in `APPLE-ICON-MODEL.md`, but make the behavior stricter.

### enclosed

Artwork already behaves like a coherent rounded-square application icon.

- do not add a second enclosure
- do not blindly shrink it again
- normalize canvas and optical footprint
- remove only incompatible baked effects where safe
- apply the final Vesper mask/material at render time

### circular

The visible silhouette is primarily a circle or near-circle on transparency.

- treat the circle as foreground artwork, not as the final app icon boundary
- strip legacy external shadow before measuring
- fit the circle to the official designated circular-artwork frame from the captured Apple grid
- center geometrically, then apply a small bounded optical correction only if needed
- place it over the compatibility enclosure/background

### glyph

The visible content is an isolated brand mark or symbol.

- fit the mark to the primary-content guide rather than the outer enclosure
- preserve aspect ratio
- use a system-style enclosure in legacy mode
- use intentional background metadata only in canonical-redesign mode

### irregular

The artwork has a non-rectangular silhouette or protrusions.

- separate the core recognizable mark from external shadow/effect pixels
- scale the core uniformly to the current primary-content guide
- keep protrusions inside the final mask-safe region
- do not distort the shape to fill the square

### full-bleed

The artwork intentionally fills a rectangular source.

- determine whether it is actual background artwork or merely an incorrectly exported square
- if it is a valid background, preserve it as background semantics
- if it is accidental full-bleed packaging, reconstruct the foreground instead of shipping an oversized square

## shadow and effect stripping

Apple's legacy adjustment explicitly removes drop shadows before auto-scaling unique-shaped artwork. Vesper must do the same conceptually.

Do not measure the scale from blurred shadow pixels.

For raster sources distinguish:

- high-confidence opaque/core alpha region
- antialiased contour
- soft external shadow or glow

For SVG sources distinguish geometry from filters, shadow primitives and out-of-shape effect bounds.

If a baked effect can be removed safely, remove it before calculating content bounds.

If removing it would destroy meaningful artwork, classify the icon for canonical reconstruction rather than using a destructive heuristic.

Metadata should retain both:

- `coreBounds`
- `effectBounds`

Never let effect bounds determine the base content scale.

## Apple-grid fit transform

The base fit must be deterministic.

For a source-visible bounding box `B` and an Apple-derived target guide `G`:

1. remove external effect bounds from `B`
2. preserve aspect ratio
3. compute a uniform scale that fits `B` entirely inside `G`
4. center the scaled geometry on the guide center
5. calculate visual-mass/optical center separately
6. allow a small bounded translation only if optical imbalance is significant
7. verify the final result against mask clipping and small-size previews

Do not stretch x and y independently.

Do not enlarge tiny source marks indefinitely. If the source intentionally has large internal whitespace, first determine whether that whitespace belongs to the logo composition or is packaging padding.

Do not crop meaningful geometry merely to hit a target occupancy ratio.

## optical-size validation

Validate optical footprint independently from file dimensions.

A `1024 x 1024` image can still look much too large if its alpha or visual mass nearly fills the entire canvas.

Track at least:

- core visible bounding-box width/height ratio to canvas
- opaque-area ratio
- alpha-area ratio
- optical-mass centroid
- distance from core geometry to enclosure boundary
- effect-only extension beyond the core

Use the Apple-derived grid as the pass/fail source of truth.

As a secondary regression signal, compare the final flattened footprint against known macOS-style values. Real-world reports show full-canvas assets around a `0.91` visible bounding-box ratio looking oversized beside common Mac icons around roughly `0.81–0.84`.

This secondary measurement is a test heuristic, not a replacement for the official grid.

## no double enclosure

Double-enclosure is a hard failure.

Detect cases such as:

```text
existing rounded-square icon
    ↓
shrunk into another rounded square
    ↓
final rounded-square mask
```

This creates the obvious "logo inside a badge inside an app icon" look.

Before adding a compatibility enclosure, inspect whether the source already contains:

- a full opaque/gradient background
- rounded-square boundary geometry
- an existing platform tile
- substantial uniform corner transparency consistent with an enclosure

If the source is already enclosed, normalize it instead of wrapping it again.

## no full-canvas flattened tile

A full `1024 x 1024` opaque rounded tile is not acceptable as the default freedesktop compatibility export.

It can look oversized relative to neighboring Mac-style icons even when its nominal dimensions and corner radius are correct.

Keep the final static tile inside the versioned compatibility envelope and reserve outer space for optical balance/effect footprint.

## current 2026/27 material recipe

Vesper must version material rendering separately from canonical artwork.

Define a current recipe equivalent to `apple27` or another clear versioned identifier.

Relative to the first Tahoe Liquid Glass treatment, current Apple guidance indicates a sharper, more defined icon rendering with reduced translucency and improved specular behavior.

The current Vesper glass recipe should therefore prefer:

- lower baseline translucency than the original Tahoe imitation
- crisp edge definition
- restrained blur
- specular highlights that preserve contour contrast
- selective rather than universal refraction
- shadows used to clarify depth, not add haze

Do not regenerate canonical SVG artwork when changing material recipe versions.

A renderer upgrade should recompile existing canonical assets locally.

## specular strategy

Support semantic specular placement equivalent to:

- `auto`
- `inside`
- `outside`
- `off`

`auto` should be the normal default.

The renderer may choose inside/outside behavior based on layer/background luminance and edge contrast.

Do not paint specular highlights into the canonical SVG.

Allow specular to be disabled for narrow, complex or pillowy layers where it harms legibility.

## selective refraction

Refraction is not a global "make glass stronger" slider.

Use it primarily where overlapping layers benefit from the sense that one glass element bends content beneath it.

Avoid strong refraction on:

- tiny glyphs
- dense narrow shapes
- text outlines
- already complex raster artwork
- logos where distortion hurts recognition

Canonical metadata may express refraction intent per group/layer, while the actual numeric recipe belongs to the renderer version.

## clear mode must be stable

Do not bind icon Clear mode directly to Vesper's global shell transparency slider.

Apple's current icon behavior intentionally keeps Clear icon glass at a controlled material level instead of letting a system transparency preference arbitrarily destroy icon legibility.

Vesper should do the same.

The shell can have its own transparency preference, but adaptive app icons need a pinned Clear material recipe with known contrast behavior.

## mono, clear and tinted contrast

A usable default icon is not enough.

The `mono` annotation must preserve recognition with strong luminance structure.

Require:

- at least one prominent foreground feature to remain white or near-white when needed for contrast
- meaningful dark-to-light dynamic range in the mono representation
- no dependence on hue alone for identifying the app
- validation with very light accents, including yellow-like colors
- validation with dark and saturated accents
- validation on both bright and dark wallpaper samples

When tint backgrounds become washed out, prefer a stronger system-style background treatment and preserve a high-contrast foreground rather than increasing random saturation everywhere.

Do not let AI pick a single tinted composition that only works with the palette active during generation.

## colorblind robustness

Recognition must come primarily from shape, silhouette and contrast.

Do not depend on red/green or another hue pair to distinguish essential parts of the mark.

The high-contrast mono representation should act as a built-in accessibility test for the default artwork.

If the icon becomes ambiguous in mono, fix the source geometry or layer contrast instead of special-casing one tint color.

## subtractive artwork and depth

When a logo creates holes by subtracting paths from one flat layer, determine whether the same visual result can be represented as overlapping solid layers.

For Vesper-owned Liquid Glass surfaces, explicit overlapping layers often produce more convincing material depth than one compound cutout.

Do not decompose a trademark arbitrarily, but allow semantic reconstruction where it preserves the exact visible identity.

## opt-out from glass per layer

Not every layer benefits from Liquid Glass.

Canonical metadata must allow effects to be disabled per layer/group.

Use this for:

- pre-rendered raster artwork that must remain visually intact
- watermarks or small critical marks
- layers whose detail collapses under refraction/specular treatment
- photographic or highly textured source elements retained intentionally

The app icon as a whole can still use the common enclosure/material system.

## dark appearance

Do not generate Dark by mechanically multiplying all RGB values.

Dark may keep the same geometry while changing fill/blend/material properties.

Validate every major foreground feature against the dark background.

If a feature disappears:

- adjust that layer's semantic fill
- adjust blend behavior
- adjust material participation
- preserve the defining geometry

Do not swap out the core logo for a different symbol.

## static and runtime renderers

Continue to keep two rendering targets.

### freedesktop compatibility renderer

Produces self-contained SVG/raster output with:

- versioned enclosure geometry
- flattened material approximation
- concrete colors
- no dependency on live wallpaper sampling
- predictable output in GTK, Qt and Electron icon loaders

### Vesper/Caelestia renderer

Can use:

- unmasked canonical layers
- live enclosure mask
- current palette
- backdrop-aware material
- specular response
- selective refraction
- runtime lighting

The two outputs must share geometry and appearance semantics so the same app does not change optical size when moving between the launcher and another Linux surface.

## renderer back-compat

Apple's current Icon Composer can preview/render icons for earlier system releases from the same source.

Vesper should mirror that concept with versioned local renderer recipes.

Canonical artwork should not encode one compositor generation permanently.

Keep:

```text
canonical artwork vN
    ↓
renderer recipe standard
renderer recipe glass-current
renderer recipe static-freedesktop
```

If the material model changes later, recompile rather than ask AI to redraw every app.

## legacy wrapper quality ceiling

Apple's own automatic legacy treatment is a compatibility feature, not a guarantee that an old icon will look identical to a true Icon Composer icon.

Vesper must treat `legacy-auto-fit` as a lower-quality but safe state.

Expose enough status to distinguish:

- `original`
- `legacy-auto-fit`
- `canonical-local`
- `canonical-ai`

The reconciliation worker should opportunistically replace `legacy-auto-fit` with a validated canonical asset when possible.

Do not block the desktop while waiting for that upgrade.

## source and runtime asset mismatch

Real applications sometimes use one packaged icon in the launcher and a different runtime icon while the process is active.

Vesper should avoid creating this problem itself.

- use the normal freedesktop theme resolver as the primary activation path
- do not inject per-process runtime icon overrides merely to force styling
- make Vesper-owned surfaces resolve the same canonical application id and generated asset inventory
- validate an asset before atomically making it active

A malformed icon file must never be able to break an application's startup path.

## cache and atomic switching

Icon changes are visually global and highly cache-sensitive.

Compile into a staging generation first.

Validate the complete changed set, then atomically switch the active theme generation/symlink where practical.

Keep the previous known-good generation until the new one is active.

Invalidate only the icon caches and Vesper surfaces that need refresh.

Do not expose a mixed desktop where half the icons use the previous footprint/material recipe and half use the new one.

## regression fixtures

Add representative source fixtures for the implementation, without committing copyrighted proprietary app icons unless licensing permits it.

The test corpus should contain synthetic equivalents of:

1. a circular brand mark on transparency
2. a glyph on transparency
3. an irregular silhouette with external shadow
4. an already enclosed rounded-square icon
5. a full-bleed accidental square
6. a legitimate full-bleed background composition
7. a thin-line icon that becomes illegible when small
8. overlapping glass-capable layers
9. raster artwork that should opt out of material effects
10. a mono composition with weak contrast

For every fixture verify:

- shape classification
- core/effect bounds separation
- selected target guide
- scale transform
- optical offset bounds
- enclosure decision
- no double enclosure
- default/dark/mono compatibility
- clear/tinted contrast
- small-size recognition
- static/runtime optical-size parity

## acceptance criteria

Do not call Apple-style normalization complete until all of these are true:

1. The implementation uses a versioned `1024 x 1024` Apple-derived grid.
2. Current official production-template geometry is measured and recorded rather than guessed.
3. The static compatibility tile defaults to the established centered `824 x 824` footprint only as a compatibility envelope and can be updated if the current official grid measurement differs.
4. Circular artwork uses the official designated circular guide, not the outer compatibility tile.
5. Irregular legacy artwork has external drop shadow/effect bounds removed before scale calculation.
6. Rounded-square legacy artwork can be masked/extended without being double-wrapped.
7. Circular/glyph/irregular legacy artwork receives a system-style enclosure by default.
8. Intentional brand backgrounds are reserved for canonical design or a documented recognition exception.
9. No source is stretched non-uniformly to fill the canvas.
10. A full-canvas opaque flattened tile is rejected as visually oversized unless explicitly required by a target renderer.
11. Core bounds and effect bounds are tracked separately.
12. Optical centering is bounded and cannot cause mask clipping.
13. The current glass recipe is sharper and less translucent than the first Tahoe imitation.
14. Specular supports automatic/inside/outside/off behavior.
15. Refraction is selective and can be disabled per layer/group.
16. Clear icon material remains legible independently of global shell transparency.
17. Mono/tinted validation covers very light and dark accent colors.
18. Recognition does not depend on color alone.
19. Legacy auto-fit is visibly distinguishable in status from a true canonical conversion.
20. Static freedesktop and Vesper runtime rendering keep the same optical footprint.
21. Renderer upgrades never require AI regeneration when canonical artwork is unchanged.
22. Activation is staged, validated and atomically switched with a known-good rollback.

## implementation references

Use these as research references, not runtime dependencies:

- current Apple Human Interface Guidelines for App icons, updated June 2026
- current Apple Design Resources App Icon Template
- Apple Icon Composer documentation
- WWDC25 `Say hello to the new look of app icons`
- WWDC25 `Create icons with Icon Composer`
- WWDC26 `Icon Composer for Beginners Group Lab`
- WWDC26 Platforms State of the Union icon-rendering updates
- GitHub reports where full-canvas Mac icons appear optically oversized
- GitHub projects that moved from legacy compatibility wrappers to real Icon Composer assets

Implementation must prefer official Apple guidance for semantics and use third-party measurements only to validate observable compatibility behavior that Apple does not publish numerically.
