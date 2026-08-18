# adaptive icons

This is the single source of truth for Vesper adaptive application icons.

Do not create additional adaptive-icon design documents. Future architecture, AI, Apple-compatibility, renderer, auto-fit or export changes belong here.

The target is an independent Linux implementation of the public Apple Icon Composer authoring/rendering model, adapted to NixOS, Hyprland, freedesktop icon themes and Vesper-owned Caelestia surfaces. Do not depend on Apple's private `.icon` serialization, `actool`, AssetCatalog runtime or private shader implementation.

## mission

Replace the experimental manual icon queue with an automatic system that:

1. discovers installed Linux applications from effective XDG `.desktop` entries;
2. resolves the real packaged icon;
3. fingerprints the source and reuses a valid canonical cache when possible;
4. sends each new or materially changed source through semantic icon decomposition using the configured Vesper AI provider;
5. preserves reliable original vector geometry locally where possible instead of needlessly redrawing brand curves;
6. reconstructs raster/unsuitable sources into semantic vector artwork when required;
7. stores the result as a multi-layer canonical `.vicon` package;
8. validates identity, geometry, optical size, appearances and safety locally;
9. renders Apple-style adaptive appearances and material depth deterministically;
10. exposes a generated freedesktop icon theme plus richer live rendering in Vesper-owned surfaces;
11. detects installs/upgrades/removals automatically;
12. never requires AI merely because wallpaper, accent, appearance or renderer recipe changed;
13. can export every accepted icon in bulk without making new AI requests.

The feature must fail visually safe: a provider outage or one broken icon must never leave missing icons or block desktop startup/theme switching.

## repository constraints

Follow `AGENTS.md`.

In particular:

- first-party runtime services and CLIs are Rust, not Python;
- Nix/Home Manager owns installation and service wiring;
- Caelestia remains the only shell/settings surface;
- do not create a duplicate settings app or palette daemon;
- reuse Vesper's existing AI credential/control plane;
- never write secrets to the Nix store, command arguments, logs or icon metadata;
- never overwrite packaged icons or `/nix/store` assets;
- keep mutable state inside documented XDG data/state/cache roots;
- keep custom Caelestia patches small and build-tested.

Read current implementation before coding, especially:

- `home/yargc/packages/vesper-control.rs`
- `home/yargc/packages/vesper-control.nix`
- `home/yargc/packages/vesper-icons.rs`
- `home/yargc/packages/VesperAppsSettings.qml`
- `home/yargc/packages/VesperAppControls.qml`
- `home/yargc/packages/AiPage.qml`
- `home/yargc/packages/VesperThemeSettings.qml`
- `home/yargc/caelestia.nix`
- `home/yargc/skills/vesper-adaptive-icons/SKILL.md`

The old `icon request`/manual-review queue is temporary scaffolding, not the final product contract.

## final system model

```text
.desktop inventory
      ↓
source icon resolver
      ↓
source fingerprint + local geometry analysis
      ↓
valid .vicon cache?
      ├─ yes → reuse
      └─ no
           ↓
      GPT / selected vision-capable provider
      semantic decomposition
           ↓
      local geometry preservation/reconstruction
           ↓
      reconciliation + strict validation
           ↓
      canonical multi-layer .vicon
           ↓
      appearance semantics
      default / dark / mono
           ↓
      versioned material renderer
           ↓
      ┌─────────────────────────────┐
      │ Vesper live layered output  │
      │ freedesktop flattened output│
      └─────────────────────────────┘
```

AI decomposes/reconstructs the original icon. It does not generate a finished shiny PNG and it does not own theme colors or the final Glass shader.

## application discovery

Use freedesktop/XDG application discovery as the source of truth.

Build effective paths from `XDG_DATA_HOME`, `XDG_DATA_DIRS` and the actual environment. Correctly cover NixOS/Home Manager profiles and Flatpak exports without blindly crawling `/nix/store`.

For every effective `.desktop` entry:

- use desktop id as a stable launcher identity;
- parse `Icon=` exactly;
- support absolute icon paths and theme icon names;
- follow freedesktop icon-theme lookup/inheritance;
- prefer the highest-quality official vector source;
- otherwise select the highest useful raster source;
- support SVG, PNG, WebP, XPM and real formats encountered in installed entries;
- apply normal XDG precedence and deduplicate shadowed entries;
- respect hidden/disabled entries;
- retain enough provenance to notice source changes after upgrades.

Do not guess an icon from display name when the desktop entry provides resolvable identity.

## runtime application identity

A generated icon theme alone is insufficient on Linux because running windows can expose different identifiers.

Maintain one canonical Vesper application identity resolver that reconciles exact evidence such as:

```text
desktop id
StartupWMClass
Wayland app_id
X11 WM_CLASS
Flatpak app id
executable identity
explicit known aliases
        ↓
canonical Vesper app id
```

Do not use window title or fuzzy translated display-name matching as the primary resolver.

For Vesper-owned surfaces the same identity must drive:

- launcher/app grid;
- running-task/dock surface if present;
- switcher/Alt-Tab if controlled by Vesper;
- Apps settings;
- icon generation/status/export.

Acceptance invariant:

```text
launcher icon == running-state icon == switcher icon == app-grid icon
```

If an application supplies a conflicting runtime bitmap hint, Vesper-owned surfaces should prefer the known canonical Vesper identity/icon. Do not globally mutate the application's own icon API merely for styling.

## automatic reconciliation

Run a complete scan at service startup. Watch effective application/icon export directories using filesystem notifications where practical, debounce package-manager noise and batch reconciliation.

Handle:

- install/uninstall;
- desktop entry replacement;
- icon replacement after application upgrade;
- Flatpak install/remove;
- Home Manager activation;
- NixOS/profile generation changes.

A periodic full scan may exist as recovery, not as the primary watcher.

## canonical package: `.vicon`

The canonical result is not a single flattened SVG.

Use a Vesper-owned semantic multi-layer package, conceptually:

```text
firefox.vicon/
├── manifest.json
├── background/
│   └── background.svg       # only if actual background artwork is needed
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
│   ├── 03-detail/
│   └── 04-optional/
└── appearances/
    ├── default.json
    ├── dark.json
    └── mono.json
```

All artwork uses the same unmasked `1024 x 1024` coordinate system. The package does not contain the final rounded-square mask.

One logical background plus one to four foreground depth/effect groups is the normal complexity target. Exceed four only when identity genuinely cannot be preserved with fewer groups and validation proves the result remains usable.

A flattened SVG/PNG is a compiled output, never the canonical source.

## artwork rules

Canonical source artwork should be simple, frontal and material-neutral.

Prefer:

- crisp vector geometry;
- small numbers of filled overlapping shapes;
- stable semantic ids;
- explicit back-to-front order;
- original brand geometry and recognizable silhouette;
- outlined text only when text is essential to the mark.

Do not bake generated material effects into canonical artwork:

- no generated specular highlight;
- no generated glass blur;
- no generated external drop shadow;
- no glow/bevel/refraction simulation;
- no final squircle mask;
- no fake full-frame glass screenshot;
- no embedded base64 raster merely to call the result SVG;
- no external fonts/resources/URLs/scripts/event handlers.

Ordinary shading that is intrinsic to the actual brand illustration may remain.

## background model

Background is a first-class semantic surface. Prefer metadata instead of painting a giant SVG rectangle when the background is a simple solid/gradient.

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

For common colored icons, store the stable brand color and let a versioned Vesper system-gradient recipe generate the background instead of letting each AI request invent arbitrary gradient stops.

Define versioned System Light and System Dark recipes. Do not use pure white/black universally unless required by the actual brand.

Legacy circular/irregular transparent sources should prefer system-style enclosure backgrounds rather than random AI-generated brand tiles. A successful canonical redesign may intentionally define a brand background.

## groups, layers and depth

Foreground groups are ordered material/depth surfaces. Layers inside a group are artwork components.

Each group must support material treatment equivalent to:

```json
{ "renderMode": "combined" }
```

or:

```json
{ "renderMode": "individual" }
```

`combined` composes member artwork before material treatment. `individual` allows member layers to respond separately while retaining group order/semantic role.

Do not split every path into its own material surface. Do not flatten genuinely separate overlapping surfaces merely to reduce group count.

Depth comes primarily from ordered material interactions, not perspective/extrusion.

```text
viewer
  ↑
Group 4  foreground detail
Group 3  primary/front fragment
Group 2  secondary/rear fragment
Group 1  base surface
Background
```

One semantic object may occupy multiple depths. A ribbon/flame/line that passes behind something and returns to the front may be split into rear/front fragments while retaining one semantic object id.

Do not invent depth weaving absent from the original icon.

## why the icons look three-dimensional

The Apple-like dimensional effect is renderer-created rather than pre-painted 3D artwork.

The renderer combines:

- z-ordered material surfaces;
- translucency;
- contour-aware specular response;
- small depth-separation shadows;
- selective refraction of lower layers;
- bounded blur where useful;
- appearance-specific fills/blending;
- a shared system lighting model.

The result is flat vector artwork gaining physical depth. Do not turn icons into perspective 3D objects.

## material metadata

Artwork and material intent are separate.

A group/layer may express bounded semantics equivalent to:

```json
{
  "effects": true,
  "renderMode": "combined",
  "blendMode": "auto",
  "specular": "auto",
  "refraction": "auto",
  "translucency": "auto",
  "blur": "auto",
  "shadow": "auto",
  "materialTint": "participate"
}
```

AI may recommend semantic classes, never arbitrary shader code/numeric compositor programs. Numeric rendering values belong to versioned local renderer recipes.

Effects must be disableable per group/layer for dense details, text-like marks, retained raster fragments or identity-critical pieces.

## bounded blend modes

Support a bounded blend intent set equivalent to:

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

Blend intent belongs in metadata, not unconstrained CSS/filter logic in generated SVG.

`auto` is the normal recommendation; renderer recipes may resolve it differently by appearance/luminance.

## system lighting and specular

Do not paint one static diagonal white highlight across every icon.

The live renderer owns a stable versioned system light direction and derives response from surface contour, local/background luminance and depth.

Specular strategy supports:

```text
auto
inside
outside
off
```

`auto` considers edge contrast, luminance, group render mode, detail density and renderer recipe. Thin/dense details should often reduce or disable specular.

## selective refraction

Refraction is depth-aware and local, not a global "glass strength" effect.

The required useful path is refraction between icon layers/groups. Vesper-owned surfaces may additionally use backdrop information when available.

Avoid strong refraction on text, tiny glyphs, thin outlines, dense logos and identity-critical shapes.

Static freedesktop outputs flatten an approximation and must not pretend they provide live backdrop refraction.

## shadows and translucency

Generate shadows from canonical surface geometry. Old baked shadow pixels do not determine content scale.

Use bounded recipes equivalent to:

```text
off
neutral
restrained-chromatic
auto
```

Shadows communicate depth separation, not desktop-window elevation.

Generated glass translucency is renderer metadata. Intrinsic brand opacity may remain artwork semantics.

## appearance model

Canonical annotations are:

```text
default
dark
mono
```

The compiler derives:

```text
default
dark
clear-light
clear-dark
tinted-light
tinted-dark
```

`mono` is the semantic source for Clear/Tinted; it is not merely grayscale Default.

Core recognizable geometry remains consistent across appearances. Dark may alter fills, blend/material participation and background while preserving identity.

Tinted/Clear must remain recognizable with most brand hue removed. Build identity around silhouette, geometry and luminance contrast, not hue alone.

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

`Glass` is material behavior, not a seventh artwork appearance. `Original` is a per-app diagnostic/escape hatch, not part of the adaptive appearance matrix.

Clear uses a pinned tested icon material recipe independent from general shell/panel transparency.

## mono accessibility requirements

Mono must preserve strong luminance structure and recognition. Validate very light, dark and saturated tint colors plus bright/dark backgrounds.

A defining foreground feature may remain white/near-white when necessary for contrast. If the icon becomes ambiguous without hue, fix geometry/luminance semantics rather than special-casing one palette.

## source classification and shape analysis

Every source is independently classified by silhouette:

```text
enclosed
circular
glyph
irregular
full-bleed
```

Local analysis should measure deterministic facts including:

- alpha/vector bounds;
- canvas dimensions;
- occupied/opaque/alpha area ratios;
- aspect ratio;
- circularity;
- edge proximity;
- connected regions where practical;
- optical/visual-mass centroid;
- clipping/mask structure;
- likely external shadow/effect footprint.

Do not include shadow/glow bounds when sizing the core mark. Track `coreBounds` and `effectBounds` separately.

### enclosed

Preserve an already coherent application tile. Do not double-enclose it.

### circular

Treat the circle as foreground artwork, not the final app boundary. Fit it to the calibrated circular-content guide and place it on an enclosure/background when appropriate.

### glyph

Fit the isolated mark to the calibrated primary-content guide and preserve aspect ratio.

### irregular

Fit core recognizable geometry uniformly inside the calibrated safe guide; do not stretch/crop merely to fill the square.

### full-bleed

Determine whether full bleed is intentional background artwork or accidental packaging. Preserve intentional background semantics; reconstruct accidental oversized packaging.

Double enclosure is a hard style failure.

## Apple-grid calibration

Do not hard-code one universal circular shrink percentage.

Do not treat `824 x 824` on `1024 x 1024` as the current universal Apple source of truth. `824 / 1024` may remain only as a historical/regression reference for flattened output until current calibration confirms an equivalent footprint.

Implement a developer/build-time calibration harness based on current public Apple design resources and representative Icon Composer output. Runtime must not depend on Apple resources/tooling.

Record/version at least:

- design canvas;
- flattened enclosure alpha bounds;
- designated circular-artwork guide;
- primary-content guide;
- optical safe region;
- corner/enclosure geometry;
- representative Default/Dark/Mono bounds;
- source/revision/measurement date.

Commit derived constants declaratively under a versioned grid revision. A later Apple revision creates a new grid/renderer revision rather than silently moving all icons.

Circular artwork sizing uses its own calibrated guide; it is not sized from the outer flattened enclosure footprint.

## legacy auto-fit fallback

When no validated canonical package is available, provide a safe compatibility path:

```text
installed source
    ↓
isolate core artwork
    ↓
remove/ignore external effect footprint
    ↓
classify silhouette
    ↓
fit using calibrated guide
    ↓
apply system-style enclosure when required
    ↓
compile compatibility icon
```

Store state as `legacy-auto-fit`. It is a fallback quality tier, not the final target. Replace it opportunistically with validated canonical AI output.

Never distort aspect ratio or make circular artwork fill the entire outer enclosure.

## AI canonicalization policy

For every new or materially changed source fingerprint without a valid canonical package, run semantic decomposition through the selected configured vision-capable provider.

This intentionally differs from the old single-SVG optimization: even a clean SVG still needs semantic depth grouping for the final `.vicon` model.

For clean official vector sources, however, AI acts primarily as semantic director while local code owns reliable geometry:

```text
official SVG
    ↓
sanitize + extract exact geometry locally
    ↓
render visual reference + structural summary
    ↓
GPT semantic decomposition
    ↓
assign existing geometry to background/groups/layers
    ↓
local .vicon construction
```

Only generate replacement vector geometry when raster/structure quality makes that necessary. This minimizes brand drift.

For raster inputs, local segmentation/vector candidates may be provided as extra evidence; GPT may reconstruct semantic vector layers when needed.

## provider and credential integration

Reuse Vesper's existing API-key-only AI control plane and Secret Service credential.

If the OpenAI key is already configured in `AI`, adaptive icons use it without asking again. Do not create an icon-specific key store or OAuth flow.

Provider/model selection is capability-driven:

- accepts image input;
- supports schema-constrained structured output;
- can return sufficient vector/metadata content;
- enabled in the existing Vesper provider configuration.

OpenAI/GPT is a first-class path. Use the current Responses-style multimodal/structured-output API or supported successor, not an image-generation endpoint.

## remote input/privacy boundary

The model sees the original source, not a stylized Vesper output.

For raster sources send a normalized high-quality render with transparency preserved.

For vector sources send both when useful:

- normalized rendered preview;
- sanitized SVG/XML geometry or safe structural representation.

Remote conversion is explicit. The UI must state that application icon artwork may be sent to the selected provider.

Do not send unrelated data such as:

- full `.desktop` contents;
- `Exec=` commands;
- absolute paths;
- username/hostname;
- Nix configuration;
- usage/wellbeing data;
- window titles/process lists;
- unrelated application inventory.

Keys never enter generated assets, provenance or ordinary logs.

## structured AI response

Do not parse conversational prose. Require a versioned schema that can represent the whole package.

Conceptually:

```json
{
  "schemaVersion": 2,
  "sourceAssessment": {
    "shapeClass": "circular",
    "confidence": 0.97,
    "identityRisk": "low"
  },
  "normalization": {
    "needsEnclosure": true,
    "opticalOffsetX": 0.0,
    "opticalOffsetY": -0.01
  },
  "background": {
    "strategy": "system-brand-gradient",
    "brandColor": "#rrggbb"
  },
  "groups": [
    {
      "id": "primary",
      "z": 1,
      "renderMode": "combined",
      "blendMode": "auto",
      "material": {
        "effects": true,
        "specular": "auto",
        "refraction": "auto"
      },
      "layers": [
        { "id": "mark", "svg": "<svg>...</svg>" }
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

The exact serialization may evolve, but equivalent information must exist.

## local/AI reconciliation

Remote output is a proposal. Before accepting it, reconcile:

- deterministic local measurements;
- AI semantic classification/decomposition;
- calibrated grid rules;
- source provenance/exact vector geometry;
- previous known-good canonical metadata.

Hard measurable geometry overrides contradictory model guesses. AI is authoritative only for semantic intent that local analysis cannot reliably infer.

If disagreement is material:

1. optionally retry once/bounded with a corrective structured prompt;
2. otherwise use legacy auto-fit/original rather than activating questionable artwork.

## identity protection

Never accept a generated icon that:

- replaces an official mark with a generic symbol;
- invents letters/text;
- removes defining geometry solely for style;
- hallucinates unrelated decoration/background;
- materially distorts a trademark;
- embeds the original raster as fake SVG.

A correct original/legacy fallback is better than a polished wrong logo.

## safety validation

Treat model SVG/XML as untrusted input.

Reject/sanitize at least:

- malformed XML/SVG;
- scripts/event handlers;
- `foreignObject`;
- external URLs/resources;
- embedded base64/data raster payloads except explicitly permitted retained-raster layers;
- external fonts;
- dangerous XML constructs;
- unsupported/problematic filters;
- pathological DOM/path complexity;
- unreasonable dimensions/view boxes;
- invisible/empty output;
- geometry outside expected bounds.

Render using a safe renderer before activation.

## visual validation

Every accepted package must pass more than syntax validation.

Test:

- source identity preservation;
- silhouette classification;
- optical centering;
- calibrated enclosure/circular/primary bounds;
- no double enclosure;
- edge clarity;
- small-size recognition at 16/24/32/48/64/128/256 px;
- Default;
- Dark;
- Mono;
- Clear Light/Dark;
- Tinted Light/Dark;
- light/dark sample backgrounds;
- light/dark/saturated accent colors;
- material/depth readability;
- layer overlap/refraction safety.

Create a neighboring-icon reference board. Compare candidate optical footprint, visual weight, enclosure size, background luminance, depth intensity and specular strength beside stable known-good Vesper icons. A technically valid but obviously too large/small/heavy icon fails style validation.

Track a compliance score, but hard safety/identity failures reject regardless of score.

## render targets

The same `.vicon` drives two outputs.

### Vesper-owned Caelestia surfaces

Render live layered composition using:

- unmasked canonical layers;
- live enclosure mask;
- current palette;
- versioned system-gradient backgrounds;
- system lighting;
- specular response;
- depth shadows;
- selective intra-icon refraction;
- optional backdrop-aware effects.

### freedesktop compatibility theme

Compile self-contained SVG/raster outputs using:

- identical normalized geometry;
- concrete colors;
- versioned enclosure geometry;
- flattened material approximation;
- no dependency on backdrop sampling.

Never feed flattened outputs back into canonical cache.

The same app must not change optical size when moving between Vesper and ordinary Linux surfaces.

## renderer versioning

Keep canonical artwork independent from renderer generations:

```text
canonical .vicon vN
    ↓
standard renderer recipe
glass-current renderer recipe
static-freedesktop renderer recipe
future renderer revisions
```

Renderer upgrades recompile locally and must not require AI reconstruction.

The current Glass recipe should favor sharper edges, restrained blur, lower baseline translucency than early Tahoe-style imitation, selective refraction and shadows that clarify depth rather than add haze.

## palette/theme behavior

Caelestia remains the palette owner. Adaptive icons are another consumer of the same palette.

Palette, wallpaper, accent, light/dark, Clear/Tinted or renderer recipe changes must never trigger AI for already valid canonical packages.

Debounce rapid changes, compile to staging and atomically switch the active generated theme where practical. Avoid mixed generations on screen.

## tray/status icons

Do not apply the full adaptive app-icon pipeline to tray/status icons.

A 1024-square layered glass squircle is usually illegible in a tiny status area.

Use a separate optional symbolic derivation:

```text
canonical app artwork
    ├── app icon → launcher / app grid / dock / switcher
    └── symbolic → tray / StatusNotifier / AppIndicator
```

Tray/status output should normally be monochrome/template-like, high contrast, background-free and optically filled for its actual tiny size.

Prefer an application's maintained symbolic/status icon when one exists.

## cache and state

Cache accepted canonical packages by a stable key including:

- source content fingerprint;
- canonical schema version;
- semantic/prompt contract revision;
- model/provider family information needed for invalidation;
- material validator revision when it changes canonical acceptance.

Do not include wallpaper/accent/current appearance in the canonical key.

Keep state tiers visible:

```text
canonical-ai
legacy-auto-fit
original-fallback
failed
```

If reliable vector geometry was preserved locally inside an AI-directed package, provenance may record that without inventing a separate product mode.

Store non-secret provider/model/prompt/grid/renderer provenance and failure category. Do not retain raw authorization headers or unnecessary provider payloads.

## failure/fallback chain

For an adaptive requested appearance:

```text
last-known-good requested appearance
        ↓
valid canonical mono-derived local fallback
        ↓
legacy-auto-fit adaptive wrapper
        ↓
original packaged icon
```

A failed Tinted/Clear generation must not immediately inject a bright multicolor original into an otherwise coherent theme when an adaptive local fallback is possible.

Failures are per icon/per appearance. They do not block startup or local theme switching.

Use bounded retries/backoff; do not create a tight provider retry loop.

## icon theme installation

Publish normal application outputs through a generated freedesktop Vesper icon theme under the user's XDG data root. Inherit from a maintained fallback such as the existing Papirus configuration for non-generated applications, symbolic UI icons and unrelated assets.

Do not rewrite every `.desktop` file and do not modify packages/Nix store. Keep an immediate rollback path to the previous configured icon theme.

## settings ownership

### AI → Adaptive icons

Own generation/provider concerns:

- automatic canonicalization on/off;
- remote conversion consent/on/off if separately useful;
- selected capable provider;
- selected model or `Auto`;
- provider credential ready/missing;
- discovered/generated/pending/failed counts;
- current conversion activity;
- retry failed/regenerate operations.

If the existing OpenAI key is configured, show ready; never show another OpenAI-key field here.

### Appearance/Theme → Adaptive icons

Own rendering/system appearance:

- appearance: Automatic / Default / Dark / Clear / Tinted;
- material: Standard / Glass;
- follow Caelestia accent;
- active renderer/grid revision status;
- rebuild local icon theme;
- **Export all icons**.

Do not expose raw shader numbers in normal UI.

### Apps → per-app controls

Own individual application state:

- original/active preview;
- canonical state;
- use original;
- regenerate/retry;
- exclude from adaptation;
- re-run normalization for diagnostics;
- export this icon.

Normal successful operation requires no manual per-app approval.

## bulk export

The user must be able to export generated icons in bulk via **Export all icons** in Appearance/Theme.

Export is local. It must never trigger a new AI request.

Support at least:

```text
Current appearance (SVG)
Current appearance (PNG)
All appearances
Canonical .vicon packages
Complete archive
```

A complete archive is self-describing, conceptually:

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

Use stable sanitized canonical application/desktop ids for filenames; do not rely on translated display names.

Snapshot accepted inventory, render into staging, record per-app failures, write final manifest and publish atomically where practical. Export must never mutate/corrupt the active cache.

Export metadata may include:

- export schema/timestamp;
- renderer recipe revision;
- calibrated grid revision;
- active appearance/material;
- application id;
- source fingerprint;
- canonical schema/state;
- available appearances;
- output filenames;
- degraded state.

Never export API keys, authorization material, unrelated personal paths or raw private provider data.

## XDG data layout

Use documented XDG roots and clearly separate:

- application inventory/identity map;
- source fingerprints;
- canonical `.vicon` packages;
- validation/provenance state;
- compiled active theme generations;
- failures/retry metadata;
- disposable previews/provider caches;
- export staging.

Do not retain duplicate packaged source icons indefinitely when they can be resolved again from installed applications.

## implementation shape

Prefer one coherent Rust subsystem rather than scripts scattered around the rice. Extend `vesper-control` or use a dedicated Rust `vesper-icons` worker when separation is cleaner.

Reasonable module boundaries:

```text
discover
desktop
identity
icon_resolver
source_analysis
segmentation
ai/provider
ai/schema
ai/reconcile
canonical
svg/safety
calibration
appearance
material
renderer
validator
theme
export
watcher
```

Nix/Home Manager declaratively wires packages, user service, environment and generated-theme activation. Caelestia/QML owns only UI/runtime presentation, not hidden mutable orchestration.

## implementation order

1. finalize `.vicon` schema v1/v2 and canonical provenance;
2. implement XDG discovery, source resolver and runtime identity inventory;
3. implement local source geometry/silhouette/effect analysis;
4. implement strict SVG/package safety validator;
5. implement Apple-grid calibration constants/harness and optical normalization;
6. implement deterministic static renderer for Default/Dark/Mono-derived appearances;
7. generate/activate Vesper freedesktop icon theme atomically;
8. reuse existing AI provider capability + Secret Service key path;
9. implement GPT image/vector input and structured decomposition schema;
10. preserve exact official vector geometry locally where possible;
11. implement AI/local reconciliation and identity validation;
12. add watcher/incremental reconciliation and safe retry/fallback state;
13. add versioned system gradient/blend/material recipes;
14. add Caelestia live layered lighting/specular/refraction renderer;
15. implement neighboring-icon validation board;
16. implement tray/status exclusion/symbolic path;
17. replace manual queue UI with AI/Appearance/Apps ownership split;
18. implement bulk export backend and **Export all icons** UI;
19. migrate/garbage-collect obsolete queue state and document final XDG paths;
20. run full Nix/Rust/QML build/eval checks required by `AGENTS.md`.

## acceptance criteria

The feature is complete only when all of these are true:

1. one `ADAPTIVE-ICONS.md` document is the sole adaptive-icon architecture source of truth;
2. installed apps are discovered from effective XDG `.desktop` entries and real `Icon=` resolution;
3. runtime app identity reconciles desktop id/WMClass/app_id/Flatpak/executable aliases for Vesper surfaces;
4. every new/materially changed source without valid cache receives semantic AI decomposition;
5. an already configured OpenAI/Vesper API key is reused without asking for another key;
6. image analysis + structured output is used, not an image-generation endpoint;
7. clean official vector geometry is preserved locally where reliable while AI supplies semantic grouping;
8. canonical output is a multi-layer `.vicon`, not a flattened SVG;
9. `.vicon` uses a shared unmasked 1024-square canvas;
10. background is separate from one-to-four normal foreground depth groups;
11. groups support combined/individual material treatment;
12. one semantic object can be split across depth fragments when the source genuinely weaves through depth;
13. blend intent is bounded semantic metadata;
14. generated blur/glow/shadow/specular/refraction/final mask are not baked into canonical source artwork;
15. source silhouette is classified as enclosed/circular/glyph/irregular/full-bleed;
16. circular/glyph/irregular normalization uses calibrated guides, not one fixed shrink percentage;
17. `824/1024` is never treated as universal current Apple truth unless calibration explicitly confirms it;
18. already-enclosed icons are never double-enclosed;
19. material renderer owns system lighting, specular, selective refraction, translucency and depth shadows;
20. canonical annotations are Default/Dark/Mono and compiler derives Clear/Tinted variants locally;
21. Glass remains a material axis separate from appearance;
22. Clear material remains independent from general shell transparency;
23. all outputs survive small-size and neighboring-icon optical validation;
24. unsafe/identity-drifting AI output falls back instead of activating;
25. provider outage never breaks existing icons or local theme switching;
26. Vesper-owned live surfaces and freedesktop outputs share identical normalized geometry;
27. renderer recipe upgrades recompile existing canonical packages without AI;
28. tray/status icons are not run through the full app-icon squircle pipeline;
29. launcher/running/switcher/app-grid icons resolve to the same canonical application identity in Vesper surfaces;
30. generated theme switching is staged/atomic enough to avoid mixed visual generations;
31. the user can bulk-export accepted icons via **Export all icons**;
32. export can produce current SVG/PNG, all appearances, canonical `.vicon` packages and a complete archive;
33. export never triggers AI and never mutates the active cache;
34. disabling adaptive icons returns immediately to the configured fallback/original icon theme;
35. no first-party Python service/script is introduced for the feature.
