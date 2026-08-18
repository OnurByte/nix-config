# apple-compatible adaptive icon contract

This document is the normative Apple-compatibility layer for `ADAPTIVE-ICONS.md`.

If the older implementation prompt conflicts with this document on icon geometry, layer structure, appearance modes or Liquid Glass behavior, follow this document.

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

## material model

`Glass` is not an artwork appearance.

This is the largest correction to the first adaptive-icons prompt.

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

This matters because Apple's own material rendering can change between system versions while the underlying artwork remains valid.

## appearance model

Do not expose `Original / Light / Dark / Tinted / Clear / Glass` as six equivalent modes.

Use an Apple-compatible artwork/appearance model.

Canonical appearance annotations are:

- `default`
- `dark`
- `mono`

The compiler derives the six desktop outputs:

- default
- dark
- clear-light
- clear-dark
- tinted-light
- tinted-dark

`mono` is the semantic source for clear and tinted outputs. It is not simply a grayscale screenshot of the default icon.

If an icon does not need custom dark or mono geometry, those annotations may inherit the default geometry and vary only material/color properties.

Keep core geometry and recognizable features consistent between all appearances. Do not add or remove major logo elements just because the user changed appearance.

Use the default icon as the starting point for dark. Dark should normally be a more subdued complementary treatment, not a completely unrelated recolor.

Tinted and clear should be more restrained than default. Recognition must survive even when most brand color is removed.

The UI may still offer a separate Vesper material control such as `standard` versus `glass`, but this control is orthogonal to appearance.

A useful settings model is therefore:

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

`automatic` follows the current Caelestia light/dark state and any configured global icon-style preference.

For clear and tinted, light/dark remains an internal rendering axis so the compiler can emit both variants.

## original brand mode

`Original` may remain available as a diagnostic or per-app escape hatch, but it must not be part of the Apple-compatible appearance matrix.

When a user chooses original for one app, use the packaged icon or a minimally normalized version without pretending it is an Apple-compatible appearance variant.

This gives difficult brands a clean fallback without contaminating the global appearance semantics.

## Linux enclosure strategy

Apple's system masks square icon layers after composition. Freedesktop icon loaders generally do not provide an equivalent universal application-icon mask.

Therefore Vesper needs two outputs from the same canonical asset.

### normal Linux icon theme

The appearance compiler applies the Vesper enclosure itself and emits a self-contained SVG or raster fallback.

The enclosure should be consistent across applications and must be applied after artwork composition.

Do not permanently clip the canonical source.

### Vesper-owned surfaces

Caelestia/Quickshell receives the unmasked canonical composition and material metadata.

The Vesper renderer owns:

- enclosure mask
- live tint
- backdrop-aware translucency
- blur
- refraction/distortion when supported
- specular edge response
- shadow
- interaction/lighting changes

This is the closest Linux equivalent to Apple's system-rendered icon material.

## static freedesktop glass fallback

Ordinary GTK, Qt and Electron launchers cannot receive the live wallpaper behind an arbitrary icon.

For those surfaces the compiler may create a flattened glass-looking approximation.

That approximation may contain rendered transparency, gradient, border, highlight and shadow because it is a final compatibility asset, not canonical artwork.

Never feed the flattened compatibility SVG back into the canonical cache as source artwork.

Keep a provenance flag that distinguishes:

- canonical artwork
- runtime material metadata
- compiled static compatibility output

## source conversion policy

Do not send every SVG through AI.

Classify sources first.

### class A — already suitable

Official vector artwork with clean geometry.

Normalize coordinates, analyze layers and add metadata locally. Do not redraw it.

### class B — vector but structurally unsuitable

Official SVG with baked effects, excessive groups, text/fonts, unusual transforms or poor semantic separation.

Sanitize and restructure locally where reliable. Use AI only for semantic interpretation that cannot be recovered deterministically.

### class C — raster but simple

Use local vector tracing as a candidate, then semantic cleanup and validation.

### class D — complex raster or photographic icon

Use vision-assisted reconstruction into simple illustrative layers. The goal is brand-preserving abstraction, not pixel tracing.

### class E — unsafe to reinterpret

If reconstruction changes identity too much, keep the original app icon or apply only an enclosure treatment. Do not publish a confident-looking wrong logo.

## AI contract additions

The model must be told that it is producing Icon-Composer-like source artwork, not the final glass render.

Require it to:

- target a 1024-square unmasked canvas
- separate background and foreground artwork
- preserve back-to-front layer order
- keep effect groups between one and four where practical
- prefer solid filled shapes
- keep foreground edges crisp
- avoid baked blur, glow, drop shadow, bevel and generated specular highlights
- avoid painting the final rounded enclosure mask
- keep the primary mark optically centered
- preserve the same core geometry for default, dark and mono annotations
- use outlines for essential text
- return semantic material metadata separately from SVG artwork

The validator must reject an AI result that simply puts the original raster image inside an SVG or paints a fake full-frame glass screenshot.

## small-size behavior

Apple scales app icons down automatically. Vesper must validate the same design at real Linux sizes.

At minimum preview and validate representative sizes around:

- 16 px
- 24 px
- 32 px
- 48 px
- 64 px
- 128 px
- 256 px

The exact exported set can follow the freedesktop theme layout, but validation must include very small sizes.

When detail disappears at small sizes, prefer canonical geometry simplification before creating manually different micro-icons. Only introduce size-specific simplification if tests prove the single design cannot remain legible.

## color

Use sRGB as the canonical Linux output color space unless the renderer gains a verified wide-gamut path.

Do not claim Display P3 support merely because SVG can contain color values. The entire Linux rendering chain would need to preserve it correctly.

Tinted output must use the current Caelestia palette as a rendering input. Palette changes must never cause AI regeneration.

Dark, clear and tinted outputs must be checked on both light and dark sample backgrounds.

## validation matrix

Validation must test more than whether the SVG parses.

For every canonical icon test:

- identity recognition against the source
- content centering
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
- at least several representative accent colors
- both bright and dark wallpapers/background samples for clear/glass previews

An icon that passes default but becomes unreadable in clear or tinted mode is not complete.

## compatibility scoring

Track an internal compliance score per generated canonical asset instead of treating validation as only pass/fail.

Useful dimensions are:

- source identity preservation
- layer quality
- edge quality
- optical balance
- small-size legibility
- dark compatibility
- mono compatibility
- clear compatibility
- tinted compatibility
- material suitability

Hard safety failures still reject the asset. A low style score should fall back to the previous known-good/original icon rather than activating a visibly inconsistent conversion.

## settings changes

Update the planned Vesper Appearance controls.

Do not show `Glass` beside `Dark` and `Tinted` as though they are the same type of choice.

Use separate controls:

- icon appearance: Automatic / Default / Dark / Clear / Tinted
- icon material: Standard / Glass
- follow Caelestia accent
- glass intensity or refraction only when supported by the actual renderer

Per-app controls may additionally expose `Use original icon`.

The AI page continues to own provider and generation status. It does not own appearance or glass styling.

## acceptance criteria additions

The adaptive icon implementation is not Apple-compatible enough until all of these hold:

1. canonical artwork uses a 1024-square unmasked design canvas
2. the final rounded enclosure is applied by the compiler/runtime, not painted into canonical artwork
3. canonical source artwork does not contain generated blur, shadow, glow, bevel or specular effects
4. background and foreground artwork are separated
5. effect/composition grouping stays within four groups by default
6. foreground edges remain crisp enough for system-style material treatment
7. the same recognizable core design survives default, dark, clear and tinted appearances
8. default, dark and mono exist as canonical appearance annotations or explicit inherited mappings
9. the compiler can produce default, dark, clear-light, clear-dark, tinted-light and tinted-dark without another AI call
10. Glass is implemented as material behavior, not as a seventh artwork identity
11. clear and tinted output are tested on bright and dark backgrounds
12. the icon remains recognizable at small Linux icon sizes
13. difficult source icons can fall back to original rather than being force-redesigned
14. Vesper-owned surfaces can apply richer runtime material effects without making freedesktop loaders responsible for live refraction

## real-world lessons

Legacy automatic enclosure is not enough for visual consistency. Projects adapting to macOS Tahoe have found that old icons can still look wrong beside native Liquid Glass icons even when the system places them into transitional containers.

Cross-platform projects that adopted Icon Composer commonly keep platform-independent source artwork while compiling the Apple-specific material representation separately. Vesper should follow the same separation: canonical brand artwork first, platform/rendering treatment second.

Static Linux themes that resemble Tahoe are useful references for coverage and enclosure proportions, but they do not replace the layered runtime model.

The Vesper implementation should therefore optimize for semantic artwork quality and renderer independence rather than trying to make AI generate a finished shiny SVG in one step.
