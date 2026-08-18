# AI layered adaptive icon contract

This document is the normative canonical-format and depth-rendering contract for Vesper adaptive application icons.

Read it together with `ADAPTIVE-ICONS.md`, `ADAPTIVE-ICONS-AI.md`, `APPLE-ICON-MODEL.md` and `APPLE-ICON-AUTOFIT.md`.

If an older document describes the final canonical result as one semantic SVG, this document overrides that assumption. The final canonical result is a semantic multi-layer icon package. A flattened SVG is a compiled compatibility output, not the canonical source.

The goal is to reproduce the design model behind current Apple Icon Composer icons: simple source artwork separated into meaningful depth layers, then rendered with material-aware translucency, shadows, specular highlights and refraction.

Do not implement this feature by asking an image-generation model to draw a finished shiny icon. The AI step is semantic decomposition and vector reconstruction of the installed application's existing icon.

## core product decision

Every discovered application icon that does not already have a valid canonical package for its current source fingerprint must go through the canonicalization pipeline.

The source may be PNG, JPEG, WebP, SVG, XPM or another resolvable installed icon format.

The canonicalizer must inspect what the original icon actually looks like and reconstruct it into meaningful visual layers.

A good source SVG is useful input, but it is not by itself the final Vesper canonical format. It may contain paths that are grouped for authoring convenience rather than visual depth. The canonicalizer must still determine the semantic foreground/background/depth structure required by the Vesper renderer.

Cache the result by source fingerprint and canonical schema/model contract version so the same unchanged icon is not repeatedly sent to AI.

The normal flow is:

```text
.desktop
    ↓
resolve Icon=
    ↓
installed source icon
PNG / JPG / SVG / WEBP / etc.
    ↓
render normalized visual reference
    +
provide sanitized source geometry when available
    ↓
GPT / selected vision-capable provider
    ↓
understand original icon identity and composition
    ↓
reconstruct semantic layers
    ↓
Vesper layered canonical package
    ↓
local geometry + identity + safety validation
    ↓
material/depth renderer
    ↓
Default / Dark / Clear / Tinted
    ↓
Vesper-owned live icon or flattened freedesktop output
```

## why Apple icons look more three-dimensional

Do not confuse Apple's current app-icon depth with traditional 3D illustration.

The source artwork is intentionally encouraged to remain relatively flat, frontal, simple and free of baked material effects. The dimensionality appears when multiple foreground groups are stacked in the z-plane and the system renders each group or layer with material behavior.

The important contributors are:

- back-to-front layer/group ordering
- separate material surfaces at different visual depths
- translucency that lets lower layers influence upper layers
- per-layer or per-group shadows that visually lift one surface above another
- specular highlights that define the contour of a glass surface
- refraction that bends color/shape from layers behind a translucent surface
- blur where appropriate
- blend/fill/opacity differences between appearance annotations
- environmental/system lighting behavior supplied by the renderer

The result is a two-dimensional vector composition with depth-aware material rendering, not a polygonal 3D model.

Vesper must copy this architecture rather than painting fake bevels and highlights into every SVG.

## canonical package format

Use a Vesper-owned package format. The exact extension is implementation-defined; `.vicon` is the preferred conceptual name.

A package should be equivalent to:

```text
firefox.vicon/
├── manifest.json
├── background/
│   └── background.svg          # only when actual artwork is required
├── groups/
│   ├── 01-base/
│   │   ├── group.json
│   │   └── layers/
│   │       ├── 01.svg
│   │       └── 02.svg
│   ├── 02-primary/
│   │   ├── group.json
│   │   └── layers/
│   │       └── 01.svg
│   └── 03-detail/
│       ├── group.json
│       └── layers/
│           └── 01.svg
└── appearances/
    ├── default.json
    ├── dark.json
    └── mono.json
```

Do not require a painted background SVG when a solid color or gradient can be represented semantically in `manifest.json` or appearance metadata.

All SVG artwork inside the package must share the same `1024 x 1024` unmasked coordinate system so layers align without guesswork.

The package must not contain the final rounded-square mask.

## background model

There is one logical background surface.

Represent it as one of:

- semantic solid color
- semantic gradient
- full-bleed opaque SVG artwork when the brand genuinely requires artwork

Background colors/gradients belong in metadata when possible so appearance variants and material recipes can alter them without redrawing geometry.

A legacy circular or transparent logo may initially receive a system-style enclosure background through `APPLE-ICON-AUTOFIT.md`, but successful AI canonicalization should attempt to reconstruct an intentional layered composition while preserving brand identity.

Do not invent unrelated decorative backgrounds.

## group and layer model

Mirror the useful parts of Icon Composer's mental model.

The canonical package contains:

- one logical background
- one to four foreground effect/depth groups by default
- one or more artwork layers inside a group when required
- explicit z-order from back to front

Four foreground groups are the normal complexity ceiling. Exceed it only when the icon cannot preserve identity with fewer groups and validation proves the extra complexity remains legible.

A group is a material/depth surface. Internal artwork layers are shapes that can either receive material together as a combined surface or individually when the design benefits from separate glass treatment.

Group metadata should support a mode equivalent to:

```json
{
  "renderMode": "combined"
}
```

or:

```json
{
  "renderMode": "individual"
}
```

`combined` means the group's member artwork is composed first and treated as one material surface.

`individual` means member layers may receive material treatment separately while retaining the group's ordering and shared semantic role.

The implementation does not need to serialize exactly these property names, but the distinction must exist.

## z-plane and depth

Every foreground group must have deterministic back-to-front ordering.

Conceptually:

```text
viewer
  ↑
Group 4  foreground detail
Group 3  primary mark
Group 2  secondary mark
Group 1  base glass/artwork
Background
```

Do not model depth by arbitrary large perspective transforms.

The sense of depth should come primarily from material interactions between ordered surfaces.

Vesper may store a bounded semantic depth value or simply derive depth from group order. If explicit depth values exist, they must remain small, normalized and deterministic.

Do not let AI invent unconstrained perspective or extrusion values.

## material metadata

Artwork and material are separate.

Each group must be able to express semantic material intent equivalent to:

```json
{
  "effects": true,
  "specular": "auto",
  "refraction": "auto",
  "translucency": "auto",
  "blur": "auto",
  "shadow": "neutral",
  "materialTint": "participate"
}
```

The exact numeric shader values belong to versioned Vesper renderer recipes, not to AI output.

The AI may recommend semantic intent such as `off`, `low`, `auto`, `high` or a small bounded class set. It must not author arbitrary shader code.

Effects must be disableable per group and per individual artwork layer where necessary.

Use this for dense details, text-like marks, raster fragments or brand elements whose identity is harmed by refraction/specular treatment.

## 3D/depth rendering contract

Inside Vesper-owned Caelestia/Quickshell surfaces, render the canonical package as a live layered composition.

The renderer should conceptually perform:

```text
background
    ↓
render group 1
    ↓
material surface 1
    ↓
render group 2 above it
    ↓
shadow interaction / separation
    ↓
refraction of lower content where enabled
    ↓
specular contour
    ↓
repeat for remaining groups
    ↓
final enclosure mask
    ↓
final lighting/material response
```

The depth effect must remain subtle and icon-like.

Do not turn application icons into rotating 3D objects.

Do not add fake perspective merely to prove that the renderer supports depth.

The target is the Apple-style effect where flat artwork gains physicality because separate material layers appear to sit above one another.

## specular behavior

Specular highlights define edges and make a layer read as a material surface.

Support renderer-level strategies equivalent to:

- `auto`
- `inside`
- `outside`
- `off`

`auto` is the default.

The renderer chooses an appropriate strategy using layer/background luminance and contrast.

Do not paint a white highlight path into the canonical SVG just to simulate specular response.

## refraction behavior

Refraction should communicate overlapping glass surfaces.

When an upper layer is translucent, the renderer may bend/transmit the colors and shapes beneath it.

Use refraction selectively.

Prefer it for:

- broad overlapping foreground surfaces
- translucent logo pieces
- shapes where seeing displaced lower content improves depth

Avoid or disable it for:

- tiny glyphs
- thin strokes
- essential text outlines
- dense marks
- any logo whose identity becomes distorted

Do not require live wallpaper refraction for the effect to work. Refraction between icon layers is already useful. Vesper-owned surfaces may additionally use live backdrop information where available.

## shadows

Use shadows as depth separation rather than decoration.

Support at least:

- neutral system-style shadow
- restrained chromatic shadow when brand color against the background benefits from it
- off

Shadow geometry must be generated by the renderer from canonical surface geometry.

Do not include baked external drop shadows in canonical artwork unless they are inseparable from the actual trademark artwork, which should be rare.

## translucency and opacity

AI should normally reconstruct source artwork as solid/clean geometry and store transparency intent semantically.

The renderer can then apply translucency in a controlled way and adapt it across system/material recipe versions.

Opacity differences that are intrinsic to brand artwork may remain part of artwork semantics, but generated glass translucency must not be baked into SVG pixels.

## appearance annotations

Canonical appearance annotations remain:

- `default`
- `dark`
- `mono`

Do not create four independently redesigned complete icons for Default, Dark, Clear and Tinted.

Use the same package structure and core geometry across appearances.

`dark` may override fills, blend behavior, background metadata or limited composition properties.

`mono` is the source representation used to derive clear and tinted outputs.

The renderer/compiler derives:

- default
- dark
- clear-light
- clear-dark
- tinted-light
- tinted-dark

Material recipes remain a separate axis.

## AI must decompose every new source into layers

This requirement intentionally supersedes the older optimization where a clean official SVG could become the final canonical asset without AI semantic decomposition.

A clean official SVG should still be preserved and supplied to the model as high-quality source geometry, but the canonical package requires semantic depth grouping.

For each new or changed source fingerprint:

1. resolve the highest quality installed original icon
2. sanitize it locally
3. render a normalized 1024-square visual reference
4. if SVG/vector, also supply safe source geometry or a structural summary
5. ask the configured vision-capable model to identify the icon's visual components
6. reconstruct background and foreground elements as separate vector layers
7. assign elements into one to four depth/effect groups
8. provide default/dark/mono semantic annotations
9. provide bounded material recommendations
10. validate everything locally
11. cache the accepted canonical package

The AI request happens once per meaningful source/schema change, not once per theme switch.

If no capable provider/key is configured or the provider is temporarily unavailable, Vesper may use legacy auto-fit/original output as a temporary fallback. That fallback must not be mislabeled as fully canonicalized.

## input to the vision model

The model should see the original icon, not a previously stylized Vesper output.

For raster sources provide:

- a high-quality normalized visual image of the installed source
- transparency preserved where possible
- minimal application identity metadata only when needed to disambiguate what the mark represents

For SVG/vector sources provide:

- a rendered visual reference
- sanitized SVG/XML geometry or a safe structural representation when supported by the provider contract

Do not send unrelated `.desktop` contents, user paths, usage data or environment information.

## required AI understanding

The model must identify concepts equivalent to:

- actual background versus transparent canvas
- primary recognizable logo/symbol
- secondary logo component
- front-most detail
- visually overlapping surfaces
- pieces that should stay in one material group
- pieces that benefit from separate depth
- circular/glyph/irregular/enclosed/full-bleed silhouette class
- brand geometry that must not be changed
- baked shadow/glow/bevel that should be removed rather than reconstructed
- elements that should opt out of glass effects

The model should prefer the smallest number of layers/groups that preserves the visual identity and gives meaningful depth.

Do not split every path into its own layer.

Do not merge all visually distinct depth surfaces into one giant layer.

## structured AI response

The AI provider contract should return a schema-constrained response describing the whole package rather than free-form prose.

Conceptually:

```json
{
  "schemaVersion": 2,
  "shapeClass": "enclosed",
  "background": {
    "type": "gradient",
    "role": "brand-background"
  },
  "groups": [
    {
      "id": "base",
      "z": 1,
      "renderMode": "combined",
      "material": {
        "effects": true,
        "specular": "auto",
        "refraction": "low"
      },
      "layers": [
        {"id": "base-shape", "svg": "<svg>...</svg>"}
      ]
    },
    {
      "id": "primary",
      "z": 2,
      "renderMode": "individual",
      "material": {
        "effects": true,
        "specular": "auto",
        "refraction": "auto"
      },
      "layers": [
        {"id": "logo-main", "svg": "<svg>...</svg>"},
        {"id": "logo-detail", "svg": "<svg>...</svg>"}
      ]
    }
  ],
  "appearances": {
    "default": {},
    "dark": {},
    "mono": {}
  }
}
```

The implementation may choose a different serialization, but equivalent information must be represented.

## vector reconstruction rules

Every generated artwork layer must:

- use the common 1024-square viewBox
- preserve source identity
- remain aligned with all sibling layers
- contain no final app-icon mask
- contain no remote resources
- contain no scripts/event handlers
- contain no embedded base64 raster image as a fake vector conversion
- avoid external fonts
- use outlined text only when text is essential to the logo
- avoid baked specular, generated bevel, fake glass highlight and external drop shadow
- use crisp shapes suitable for material rendering

Raster source fragments may be retained only when vector reconstruction would materially destroy identity and the layer is explicitly marked as raster-preserved/effects-limited. The preferred target remains vector geometry.

## semantic decomposition examples

A simple circular logo might become:

```text
background: semantic gradient/color
Group 1: circular brand field or supporting surface
Group 2: primary logo glyph
```

A Firefox-like illustrated mark might become:

```text
background: semantic brand field
Group 1: rear globe/flame mass
Group 2: primary fox/flame silhouette
Group 3: front highlight/detail geometry that is genuinely part of the mark
```

Do not create a separate layer for a highlight if that highlight is merely a baked simulation of glass. Only preserve it if it is part of the actual recognizable brand illustration.

A Settings-like gear icon might become:

```text
background: semantic background
Group 1: rear gear/body
Group 2: central ring/foreground element
```

Use rounded, bold geometry where the original mark permits it so specular light can travel cleanly along edges at small sizes.

## circular and irregular Linux icons

Shape normalization still applies, but its role changes after full AI decomposition.

Before canonicalization, silhouette classification tells the AI and fallback system how the original source behaves.

After successful canonicalization, do not simply keep a giant circular PNG inside a square wrapper.

The AI should treat the circular source as source identity and rebuild it as intentional foreground artwork over a semantic enclosure/background where appropriate.

Use the Apple-derived circular artwork frame and optical rules from `APPLE-ICON-AUTOFIT.md`.

Legacy auto-fit remains the immediate fallback while canonical AI work is unavailable or invalid.

## no fake finished-image generation

The following pipelines are explicitly wrong:

```text
PNG → image generation → shiny PNG
```

```text
PNG → image generation → one flattened shiny SVG
```

```text
SVG → palette replace → call it layered
```

The correct target is:

```text
original icon
    ↓
vision understanding
    ↓
semantic vector decomposition
    ↓
layer package
    ↓
Vesper material renderer
```

## live versus flattened outputs

The canonical `.vicon` package is renderer-independent source.

### Vesper-owned surfaces

Use the layers directly and render live:

- z-order
- material grouping
- translucency
- per-layer/group shadow
- specular
- refraction
- optional backdrop interaction
- final enclosure

### freedesktop compatibility theme

Compile the same package into self-contained SVG/raster output.

The static output should visually approximate the current Vesper material recipe but cannot perform true live backdrop interaction in generic GTK/Qt/Electron icon loaders.

Never treat a flattened compatibility export as canonical source for future regeneration.

## renderer versioning

Material appearance changes more often than brand artwork.

Keep renderer recipes versioned separately from canonical packages, for example:

```text
canonical package schema v2
    ↓
renderer standard-v1
renderer glass-apple26
renderer glass-current
renderer static-freedesktop-v2
```

When Vesper improves specular/refraction/shadow behavior, re-render locally.

Do not ask GPT to rebuild every icon merely because the glass recipe changed.

## validation

Validation must happen at package level, not only per SVG file.

Validate:

- every SVG for safety and syntax
- common coordinate system
- deterministic z-order
- maximum normal group count
- no empty layers
- no duplicate redundant layers
- correct group membership
- identity preservation compared with the original rendered source
- silhouette and optical scale
- no accidental geometry drift between appearances
- small-size readability
- default/dark/mono legibility
- clear/tinted derived output
- layer overlap behavior
- material effects on/off behavior
- specular edge quality
- refraction not destroying the logo
- shadow not inflating optical bounds
- final flattened freedesktop parity with the Vesper live renderer

Render a diagnostic contact sheet that can compare:

```text
original
canonical flat
standard material
glass material
dark
mono
clear-light
clear-dark
tinted-light
tinted-dark
```

The normal successful path remains automatic; the contact sheet is for validation/debugging rather than mandatory human approval.

## acceptance criteria

The implementation is not complete until all of the following hold:

1. a canonical icon is a multi-layer package, not one flattened SVG
2. every new/changed installed source is semantically decomposed by the selected AI provider unless a valid canonical cache already exists
3. the AI sees the original installed icon visual
4. clean source SVG geometry is preserved as useful input rather than discarded
5. background is separate from foreground depth groups
6. foreground groups have explicit z-order
7. normal complexity is limited to at most four effect/depth groups
8. each group can receive material as one combined surface or as individual member layers
9. generated glass effects are not painted into canonical SVG artwork
10. Vesper can apply translucency, shadow, specular and refraction at renderer time
11. the live Vesper renderer produces visible but restrained depth between overlapping groups
12. the same core geometry powers Default, Dark, Mono, Clear and Tinted
13. renderer/material version changes require no new AI request
14. a failed AI conversion falls back safely to legacy auto-fit/original
15. circular/irregular Linux sources are reconstructed as intentional layered icons when canonicalization succeeds
16. static freedesktop exports are derived outputs and never become canonical source
17. API credentials come from the existing Vesper AI credential system
18. provider input contains only the minimum icon material necessary for conversion
19. the original application identity remains recognizable at 16–256 px
20. the resulting icon feels dimensional because of layered material rendering, not because AI painted a fake 3D illustration

## implementation priority

For the coding agent, the order is:

```text
1. define canonical layered package schema
2. define structured AI response schema
3. update generation pipeline to always produce/cache package per source fingerprint
4. implement package validator
5. implement deterministic flat compositor
6. implement freedesktop compatibility compiler
7. implement Caelestia live layered material renderer
8. add Default/Dark/Mono annotations and Clear/Tinted derivation
9. integrate automatic app discovery/change detection
10. add status/debug surfaces without restoring manual approval as the normal path
```

Do not begin by tuning pretty glass shaders before the package/decomposition model is correct. The semantic layer architecture is the foundation that makes the material rendering behave like Apple's system rather than a collection of pre-rendered icon effects.
