# apple-compatible adaptive icon contract

This document is the normative Apple-compatibility layer for `ADAPTIVE-ICONS.md`.

If the older implementation prompt conflicts with this document on icon geometry, layer structure, shape normalization, appearance modes or Liquid Glass behavior, follow this document.

The goal is not to reproduce Apple's private renderer or file format byte-for-byte. The goal is to preserve the design model that makes current Apple app icons coherent while adapting it to Linux, freedesktop icon themes and Vesper-owned Caelestia surfaces.

## source model

Treat the canonical icon as artwork, not as a flattened glass render.

For macOS-style application icons use a square `1024 x 1024` canonical design canvas.

The canonical canvas must remain unmasked. Do not bake the final rounded rectangle into the source artwork. Apple applies the final enclosure mask after composition. Vesper must keep the same separation even though Linux does not provide a universal system app-icon mask.

The appearance compiler may apply the Vesper rounded-rectangle enclosure when producing normal freedesktop assets.

Canonical artwork should use:

- one background definition
- one or more foreground artwork layers
- explicit back-to-front z-order
- no more than four effect/composition groups unless a source genuinely cannot be represented without more
- semantic names and stable layer ids
- vector geometry wherever possible
- a 1024-square coordinate system even when the original installed Linux source uses another view box

Do not model `specular` as painted artwork. Specular behavior belongs to material metadata and the renderer.

Do not model `shadow`, `blur`, `glow` or `refraction` as mandatory painted source layers.

A canonical asset may contain ordinary brand artwork whose geometry naturally includes highlights or shading that are part of the actual logo, but generated glass effects must remain separate from artwork.

## background

The background is a first-class part of the icon model.

Prefer a simple solid or gradient background that supports the foreground mark.

When canonicalizing an imported full background artwork layer, it must be full-bleed and opaque. If the background can be represented as a solid or gradient, store that as semantic background metadata rather than wasting SVG geometry on a large painted rectangle.

Do not let the AI leave accidental transparency holes at the outer background where the compiled icon expects an enclosed app icon.

Do not force every brand into the same background color. The common system feel should come from geometry, optical balance, material behavior and appearance rules rather than erasing brand identity.

## foreground artwork

Prefer clearly defined edges.

Avoid AI-generated feathering around foreground paths. Soft alpha edges, painted blur and fuzzy halos interfere with material highlights and scale poorly.

Prefer a small number of filled, overlapping shapes over complicated outline drawings where the brand can be preserved that way.

The conversion model should simplify details that cannot survive small sizes while keeping the application's defining silhouette and symbol.

Avoid:

- unnecessary fine detail
- very thin line weights
- gratuitously sharp micro-corners
- photographic reconstruction when an illustration can preserve the identity
- screenshots or replicas of application UI
- decorative text that merely repeats the application name

Text is allowed only when it is essential to the mark. Convert required text to outlines. Canonical SVG must not depend on external fonts.

Do not replace a recognizable official logo with a generic symbol merely to satisfy the style.

## optical balance and grid

A mathematically centered icon is not automatically optically centered.

The normalizer must track both geometric bounds and an optical-content region.

Keep primary content centered enough that the final enclosure mask cannot truncate important geometry.

The implementation must define a Vesper app-icon grid derived from the 1024-square Apple-style workflow and use it consistently for:

- maximum content bounds
- recommended primary-glyph bounds
- optical-center correction
- common padding
- enclosure preview
- small-size validation

Do not copy a rounded mask into the canonical artwork. The grid is guidance; the enclosure belongs to rendering.

Allow controlled per-icon optical offsets and scale corrections in metadata. A single rigid percentage for every logo will make visually light and visually heavy logos look inconsistent.

## mandatory shape normalization

This is required, not optional.

Linux application icons frequently arrive as circles, isolated transparent logos, irregular silhouettes or glyphs with no enclosing application-icon background. Vesper must not render those shapes at full canvas size beside enclosed app icons.

Every discovered source must go through a shape-analysis stage before canonicalization or compilation.

Classify the visible source geometry into at least:

- `enclosed` — already has a full square or rounded-square app-icon composition
- `circular` — dominant visible silhouette is circular or near-circular
- `glyph` — isolated logo/symbol on transparent canvas
- `irregular` — non-rectangular silhouette that does not fill an app-icon enclosure
- `full-bleed` — artwork intentionally reaches the full source bounds

The classifier must use actual geometry, not the application name. For raster sources inspect alpha bounds and visible-pixel distribution. For vector sources inspect visible path bounds, fill coverage and clipping/masking structure.

For `circular`, `glyph` and `irregular` sources the normal path is:

```text
source artwork
    ↓
measure alpha/geometry bounds
    ↓
measure optical mass and edge proximity
    ↓
scale down into the Vesper safe area
    ↓
apply per-icon optical offset if needed
    ↓
place on a semantic app-icon background/enclosure
    ↓
compile final rounded enclosure
```

Do not use one fixed scale such as 70% or 75% for every icon.

Determine scale and offset from a combination of:

- alpha bounds
- vector geometry bounds
- occupied-area ratio
- aspect ratio
- edge proximity
- optical center / visual mass
- circularity or silhouette class
- small-size recognition
- enclosure clipping risk

Store the normalization decision in canonical metadata so the same geometry can be recompiled without AI.

The metadata should be able to represent concepts equivalent to:

```json
{
  "shapeClass": "circular",
  "contentScale": 0.72,
  "opticalOffsetX": 0.0,
  "opticalOffsetY": -0.01,
  "needsEnclosure": true,
  "backgroundStrategy": "brand-derived"
}
```

The exact schema may differ, but these decisions must be explicit and deterministic.

### enclosure background selection

Do not let the model invent arbitrary backgrounds independently for each appearance.

Choose the enclosure background using this priority:

1. preserve an official/background color already present in the source when suitable
2. derive a stable brand-supporting background from the official icon palette
3. use canonical light/dark background metadata defined during normalization
4. fall back to a Vesper neutral or current palette-aware surface when the brand has no meaningful background

Tinted and clear outputs may reinterpret the enclosure through the active Caelestia palette, but the default appearance should not unnecessarily erase brand identity.

### already enclosed icons

If the source already has a coherent full app-icon enclosure, do not blindly shrink it and add a second background.

Normalize its canvas, optical size and layer semantics while preserving the existing composition where it already fits the Vesper grid.

### full-bleed artwork

Do not automatically shrink intentional full-bleed artwork into a floating tile. Preserve it when doing so is necessary to retain the icon's identity, while still validating final mask clipping and small-size behavior.

### transparent circular icons

A transparent circular icon must not remain a giant circle touching the same outer bounds as a rounded-square Vesper icon.

The normalizer must reduce it to the primary-glyph region and add an enclosure/background unless the icon is explicitly excluded or marked original-only.

This rule is particularly important on Linux because circular and transparent standalone application icons are common and otherwise destroy the visual rhythm of the generated theme.

## material model

`Glass` is not an artwork appearance.

Liquid Glass is a material/rendering treatment applied to layers and groups. Default, dark, clear and tinted are user-facing appearances.

Store material intent independently from artwork. The canonical sidecar should be able to express group or layer properties equivalent to:

- effects enabled or disabled
- individual versus combined group treatment
- translucency strength
- refraction strength
- blur strength
- specular behavior
- shadow behavior
- optional material tint participation

These are semantic parameters, not instructions to paint permanent highlights into the source SVG.

Vesper does not need to clone Apple's numeric shader implementation. The contract should remain stable while the renderer can improve independently.

## appearance model

Do not expose `Original / Light / Dark / Tinted / Clear / Glass` as six equivalent modes.

Canonical appearance annotations are:

- `default`
- `dark`
- `mono`

The compiler derives:

- default
- dark
- clear-light
- clear-dark
- tinted-light
- tinted-dark

`mono` is the semantic source for clear and tinted outputs. It is not simply a grayscale screenshot of the default icon.

If an icon does not need custom dark or mono geometry, those annotations may inherit the default geometry and vary only material/color properties.

Keep core geometry and recognizable features consistent between all appearances.

Tinted and clear should be more restrained than default. Recognition must survive even when most brand color is removed.

Use separate settings axes:

```text
appearance
  automatic
  default
  dark
  clear
  tinted

material
  standard
  glass
```

`automatic` follows the current Caelestia light/dark state and configured global icon style.

## original brand mode

`Original` may remain available as a diagnostic or per-app escape hatch, but it is not part of the Apple-compatible appearance matrix.

When a user chooses original for one app, use the packaged icon or a minimally normalized version.

## Linux enclosure strategy

Apple's system masks square icon layers after composition. Freedesktop icon loaders generally do not provide an equivalent universal application-icon mask.

Vesper therefore needs two outputs from the same canonical asset.

### normal Linux icon theme

The appearance compiler applies the Vesper enclosure itself and emits a self-contained SVG or raster fallback.

The enclosure must be consistent across applications and applied after artwork composition and shape normalization.

Do not permanently clip the canonical source.

### Vesper-owned surfaces

Caelestia/Quickshell receives the unmasked canonical composition, normalization metadata and material metadata.

The Vesper renderer owns:

- content scaling and optical offset
- enclosure mask
- live tint
- backdrop-aware translucency
- blur
- refraction/distortion when supported
- specular edge response
- shadow
- interaction/lighting changes

## static freedesktop glass fallback

Ordinary GTK, Qt and Electron launchers cannot receive the live wallpaper behind an arbitrary icon.

For those surfaces the compiler may create a flattened glass-looking approximation.

Never feed that compatibility asset back into the canonical cache.

Keep provenance distinguishing:

- canonical artwork
- normalization metadata
- runtime material metadata
- compiled static compatibility output

## source conversion policy

Do not send every SVG through AI.

Classify source quality separately from shape class.

### class A — already suitable

Official vector artwork with clean geometry. Normalize coordinates, shape placement, layers and metadata locally.

### class B — vector but structurally unsuitable

Sanitize and restructure locally where reliable. Use AI only for semantic interpretation that cannot be recovered deterministically.

### class C — raster but simple

Use local vector tracing as a candidate, then semantic cleanup and validation.

### class D — complex raster or photographic icon

Use vision-assisted reconstruction into simple illustrative layers. The goal is brand-preserving abstraction, not pixel tracing.

### class E — unsafe to reinterpret

Keep the original app icon or apply enclosure-only normalization. Do not publish a confident-looking wrong logo.

## AI contract additions

The model must be told that it is producing Icon-Composer-like source artwork, not the final glass render.

Require it to:

- target a 1024-square unmasked canvas
- identify whether the source is enclosed, circular, glyph-like, irregular or full-bleed
- separate background and foreground artwork
- preserve back-to-front layer order
- keep effect groups between one and four where practical
- prefer solid filled shapes
- keep foreground edges crisp
- avoid baked blur, glow, drop shadow, bevel and generated specular highlights
- avoid painting the final rounded enclosure mask
- keep the primary mark optically centered
- return recommended content scale/optical offset/enclosure intent when semantic interpretation is needed
- preserve the same core geometry for default, dark and mono annotations
- use outlines for essential text
- return semantic material metadata separately from SVG artwork

The validator must reject an AI result that simply puts the original raster image inside an SVG or paints a fake full-frame glass screenshot.

## small-size behavior

Validate representative sizes around:

- 16 px
- 24 px
- 32 px
- 48 px
- 64 px
- 128 px
- 256 px

When detail disappears at small sizes, prefer canonical geometry simplification before manually different micro-icons.

Shape normalization must also be checked at these sizes. A circular/glyph icon that looks balanced at 256 px but too small or too dominant at 32 px needs revised scale metadata.

## color

Use sRGB as the canonical Linux output color space unless the renderer gains a verified wide-gamut path.

Tinted output must use the current Caelestia palette as a rendering input. Palette changes must never cause AI regeneration.

Dark, clear and tinted outputs must be checked on both light and dark sample backgrounds.

## validation matrix

Validation must test more than whether the SVG parses.

For every canonical icon test:

- identity recognition against the source
- source shape classification
- occupied-area ratio after normalization
- optical centering
- consistency against neighboring enclosed icons
- enclosure clipping
- clearly defined foreground edges
- no accidental transparency in required full-bleed background
- default appearance
- dark appearance
- mono source
- clear-light output
- clear-dark output
- tinted-light output
- tinted-dark output
- small-size recognition
- representative accent colors
- bright and dark backgrounds for clear/glass previews

A circular or irregular source is not complete merely because it parses and is recognizable. It must also look optically consistent beside enclosed application icons.

## compatibility scoring

Track an internal compliance score per generated canonical asset.

Useful dimensions are:

- source identity preservation
- shape normalization quality
- layer quality
- edge quality
- optical balance
- neighboring-icon size consistency
- small-size legibility
- dark compatibility
- mono compatibility
- clear compatibility
- tinted compatibility
- material suitability

Hard safety failures reject the asset. A low style score should fall back to the previous known-good/original icon.

## settings changes

Use separate controls:

- icon appearance: Automatic / Default / Dark / Clear / Tinted
- icon material: Standard / Glass
- follow Caelestia accent
- glass intensity or refraction only when supported

Per-app controls may additionally expose:

- Use original icon
- Re-run shape normalization
- Enclosure strategy/status for diagnostics

Do not expose raw scale/offset tuning in the normal UI unless a developer/debug view is explicitly added.

## acceptance criteria additions

The adaptive icon implementation is not Apple-compatible enough until all of these hold:

1. canonical artwork uses a 1024-square unmasked design canvas
2. the final rounded enclosure is applied by the compiler/runtime, not painted into canonical artwork
3. canonical source artwork does not contain generated blur, shadow, glow, bevel or specular effects
4. background and foreground artwork are separated
5. effect/composition grouping stays within four groups by default
6. foreground edges remain crisp enough for system-style material treatment
7. the same recognizable core design survives default, dark, clear and tinted appearances
8. default, dark and mono exist as canonical appearance annotations or inherited mappings
9. the compiler can produce default, dark, clear-light, clear-dark, tinted-light and tinted-dark without another AI call
10. Glass is material behavior, not an artwork identity
11. clear and tinted output are tested on bright and dark backgrounds
12. the icon remains recognizable at small Linux icon sizes
13. difficult source icons can fall back to original rather than being force-redesigned
14. Vesper-owned surfaces can apply richer runtime material effects without making freedesktop loaders responsible for live refraction
15. every source is classified as enclosed, circular, glyph, irregular or full-bleed before final compilation
16. circular, glyph and irregular sources are automatically scaled into the shared safe area and enclosed unless a validated exception applies
17. shape scaling is based on measured geometry/optical balance rather than one hard-coded percentage
18. transparent circular icons cannot occupy the full outer icon bounds in the generated Vesper theme
19. already-enclosed sources are not double-enclosed
20. shape normalization decisions are cached as metadata and do not require AI when only the theme or palette changes
21. validation checks normalized occupied area and visual size against neighboring icons

## real-world lessons

Legacy automatic enclosure is not enough for visual consistency. A system can put an old icon inside a container and still get the size wrong if the original circular/glyph artwork is not optically normalized first.

Linux makes this more important because many application icons are transparent circles or isolated symbols rather than complete app-icon compositions.

Cross-platform projects should keep platform-independent brand artwork while compiling platform-specific enclosure/material treatment separately. Vesper follows the same separation: canonical brand artwork first, shape normalization second, platform/rendering treatment last.

The implementation should optimize for semantic artwork quality, consistent optical size and renderer independence rather than trying to make AI generate a finished shiny SVG in one step.
