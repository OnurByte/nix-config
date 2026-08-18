# adaptive icons

This is the single source of truth for Vesper adaptive application icons.

Do not create additional adaptive-icon design documents. Future architecture, AI, Apple-compatibility, renderer, auto-fit, identity, queue or export changes belong here.

The target is an independent Linux implementation of the public Apple Icon Composer authoring/rendering model, adapted to NixOS, Hyprland, freedesktop icon themes and Vesper-owned Caelestia surfaces. Do not depend on Apple's private `.icon` serialization, `actool`, AssetCatalog runtime or private shader implementation.

## mission

Replace the experimental manual icon queue with an automatic system that:

1. discovers every effective installed desktop application;
2. resolves the best real packaged source icon without ingesting Vesper's own generated outputs;
3. fingerprints source artwork and deduplicates identical source work;
4. reuses a valid canonical cache when possible;
5. sends every new or materially changed uncached source through semantic decomposition using the configured Vesper AI provider;
6. preserves reliable original vector geometry locally instead of needlessly redrawing brand curves;
7. reconstructs raster or structurally unsuitable sources into semantic vector artwork when required;
8. stores the result as a multi-layer canonical `.vicon` package;
9. validates identity, geometry, optical size, appearances and safety locally;
10. renders Apple-style adaptive appearances and material depth deterministically;
11. exposes a generated freedesktop icon theme plus richer live rendering in Vesper-owned surfaces;
12. detects installs, upgrades, removals, identity changes and provider-readiness changes automatically;
13. persists conversion work across restarts and respects provider rate limits;
14. never requires AI merely because wallpaper, accent, appearance or renderer recipe changed;
15. can export every accepted icon in bulk without making new AI requests.

The feature must fail visually safe. A provider outage, bad package, unresolved source or one broken icon must never leave missing icons or block desktop startup or theme switching.

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
installed application inventory
        ↓
canonical application identity
        ↓
source icon resolver
        ↓
self-ingestion guard
        ↓
source fingerprint + local geometry analysis
        ↓
source-hash dedup
        ↓
valid .vicon cache?
        ├─ yes → reuse
        └─ no
             ↓
        persistent conversion queue
             ↓
        configured vision-capable provider
             ↓
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

AI decomposes or reconstructs the original icon. It does not generate a finished shiny PNG and it does not own theme colors or the final Glass shader.

## application discovery

Use freedesktop/XDG application discovery as the primary source of truth.

Build effective paths from `XDG_DATA_HOME`, `XDG_DATA_DIRS` and the actual environment. Correctly cover NixOS/Home Manager profiles, Flatpak exports and other effective desktop-entry locations without blindly crawling `/nix/store`.

For every effective `.desktop` entry:

- use desktop id as the primary launcher identity;
- parse `Icon=` exactly;
- support absolute icon paths and theme icon names;
- follow freedesktop icon-theme lookup and inheritance;
- prefer the highest-quality official vector source;
- otherwise select the highest useful raster source;
- support SVG, PNG, WebP, XPM, JPEG and real formats encountered in installed entries;
- apply normal XDG precedence and deduplicate shadowed entries;
- respect hidden and disabled entries;
- retain enough provenance to notice source changes after upgrades.

Do not guess an icon from display name when the desktop entry provides resolvable identity.

## source resolver and recovery chain

A broken `Icon=` reference must not immediately make an application permanently unadaptable.

Use a deterministic source-resolution chain equivalent to:

```text
.desktop Icon=
    ↓ if unresolved
freedesktop theme inheritance / hicolor / pixmaps
    ↓ if unresolved
AppStream / metainfo icon metadata
    ↓ if unresolved
known package or application-exported assets
    ↓ if applicable
AppImage / bundle / executable-associated icon metadata
    ↓ if unresolved
original fallback + explicit unresolved state
```

Rules:

- prefer exact package/application metadata over fuzzy filename searches;
- do not crawl the entire filesystem or Nix store looking for visually similar files;
- do not use translated application names as authoritative identity;
- do not fetch arbitrary remote logos as a normal recovery path;
- record which resolver stage produced the accepted source;
- re-run resolution when the package/profile generation changes;
- if no trustworthy source can be found, retain the original launcher behavior and mark the app unresolved instead of inventing branding.

## self-ingestion guard

Vesper must never use its own generated icon theme, compiled exports, preview cache or `.vicon` renders as the upstream source for a later canonicalization pass.

This is a hard invariant.

When `Icon=firefox` or another theme name is resolved while the Vesper generated theme is active, source discovery must skip every Vesper-owned generated theme root and continue through inherited/original themes until it reaches a trustworthy packaged source.

Conceptually:

```text
Icon=firefox
    ↓
Vesper generated theme      → reject as source
Vesper export/cache roots   → reject as source
    ↓
inherited theme / package asset
    ↓
real upstream source
```

Maintain explicit provenance such as:

- source resolver origin;
- source path or stable package reference locally;
- source fingerprint;
- generated-output roots excluded during resolution.

Reject recursive lineage where a candidate source fingerprint can be traced to an earlier Vesper-generated artifact.

A theme rebuild must never cause canonical generation to feed on its own previous output.

## canonical application identity

A generated icon theme alone is insufficient on Linux because running windows can expose different identifiers.

Maintain one canonical Vesper application identity resolver that reconciles exact evidence such as:

```text
desktop id
StartupWMClass
Wayland app_id
X11 WM_CLASS
Flatpak app id
Snap/application package id when present
executable identity
Electron explicit app_id
Steam app id
Wine/Proton generated desktop identity
browser PWA application id
explicit known aliases
        ↓
canonical Vesper app id
```

Do not use window title or fuzzy translated display-name matching as the primary resolver.

### Steam

For Steam-launched applications, retain Steam application id and generated desktop-entry identity when available. Do not collapse different games into the Steam client merely because their process ancestry is shared.

### Wine and Proton

For Wine/Proton applications, reconcile generated `.desktop` ids, executable identity and stable prefix/application metadata where available. Do not key solely on `wine`, `wine64` or another shared runtime executable.

### browser PWAs

For Chrome/Chromium/Edge/Firefox-like installed web apps, retain the browser-provided PWA/app id and generated desktop id. Multiple PWAs from one browser must remain separate canonical applications.

### Electron

Prefer a stable explicit Electron/Wayland app id and desktop identity over process names such as `electron`.

For Vesper-owned surfaces the same canonical identity must drive:

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
- Snap/application export changes when relevant;
- Steam shortcut/game desktop-entry changes;
- Wine/Proton generated launcher changes;
- PWA install/remove/update;
- Home Manager activation;
- NixOS/profile generation changes;
- selected AI provider/model capability changes;
- provider credential becoming configured or removed.

A periodic full scan may exist as recovery, not as the primary watcher.

## source-hash deduplication

Canonical AI work is primarily keyed by trustworthy source content plus the canonical contract, not by the number of desktop entries that happen to reference that source.

If several application identities resolve to the exact same source artwork and the same semantic treatment is safe, perform one canonicalization job and reuse the accepted canonical geometry.

Conceptually:

```text
desktop A ─┐
desktop B ─┼→ same trustworthy source hash → one canonicalization
desktop C ─┘                              → identity-specific aliases
```

Requirements:

- no duplicate in-flight AI requests for the same canonical work key;
- keep application identity metadata separate from shared canonical geometry;
- do not merge apps merely because filenames or display names are similar;
- only deduplicate after content hashing and provenance checks;
- allow an app-specific override when identical artwork legitimately needs different semantic treatment;
- source/schema/prompt changes invalidate only affected work;
- deduplication must reduce provider usage without making uninstalling one alias delete a still-used canonical package.

## persistent conversion queue

Initial enablement may discover hundreds of applications. Do not launch one remote request per app simultaneously.

Use a persistent Rust-owned conversion queue stored under Vesper state. Queue state must survive service restart, logout and reboot.

Represent jobs with states equivalent to:

```text
pending
ready
running
retry-wait
blocked-no-provider
blocked-no-consent
succeeded
failed
superseded
cancelled
```

The queue must support:

- bounded provider concurrency;
- per-provider rate-limit awareness;
- Retry-After or equivalent server guidance when available;
- exponential or bounded backoff for transient failures;
- pause/resume;
- crash-safe recovery of `running` jobs;
- source-hash in-flight deduplication;
- cancellation/superseding when an application source changes before processing;
- fair scheduling so a huge old backlog does not indefinitely starve a newly installed app;
- progress counters such as `38 / 147 canonicalized`;
- optional configurable request/spend guardrails if the existing AI control plane exposes them cleanly.

Do not create an unbounded retry loop. Permanent identity/safety failures require an explicit new source, contract revision or user retry before repeated provider spend.

The queue should converge toward zero remote work once the installed application set is canonicalized.

## provider readiness and automatic catch-up

A missing provider or API key is a temporary capability state, not a reason to require manual per-app regeneration later.

If adaptive icons are enabled while no capable provider/key is ready:

```text
app
 ↓
local analysis
 ↓
legacy-auto-fit / original fallback
 ↓
blocked-no-provider canonicalization state
```

When a capable provider or credential later becomes available, automatically rescan eligible fallback states and enqueue canonicalization without requiring the user to press `Regenerate` for every application.

Trigger catch-up when:

- an OpenAI or other supported key becomes configured;
- selected provider changes from unavailable to ready;
- selected model gains required image/structured-output capability;
- remote conversion consent changes from off to on.

Do not automatically reprocess already valid `.vicon` packages merely because a new provider was added. Catch-up targets only uncached, degraded, blocked or explicitly invalidated canonical states.

If the provider becomes unavailable again, keep accepted canonical packages and local rendering fully functional.

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

`combined` composes member artwork before material treatment. `individual` allows member layers to respond separately while retaining group order and semantic role.

Do not split every path into its own material surface. Do not flatten genuinely separate overlapping surfaces merely to reduce group count.

Depth comes primarily from ordered material interactions, not perspective or extrusion.

```text
viewer
  ↑
Group 4  foreground detail
Group 3  primary/front fragment
Group 2  secondary/rear fragment
Group 1  base surface
Background
```

One semantic object may occupy multiple depths. A ribbon, flame or line that passes behind something and returns to the front may be split into rear/front fragments while retaining one semantic object id.

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

A group or layer may express bounded semantics equivalent to:

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

AI may recommend semantic classes, never arbitrary shader code or numeric compositor programs. Numeric rendering values belong to versioned local renderer recipes.

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

`auto` is the normal recommendation. Renderer recipes may resolve it differently by appearance and luminance.

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

`auto` considers edge contrast, luminance, group render mode, detail density and renderer recipe. Thin or dense details should often reduce or disable specular.

## selective refraction

Refraction is depth-aware and local, not a global glass-strength effect.

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

`mono` is the semantic source for Clear/Tinted. It is not merely grayscale Default.

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

`Glass` is material behavior, not an artwork appearance. `Original` is a per-app diagnostic or escape hatch, not part of the adaptive appearance matrix.

Clear uses a pinned tested icon material recipe independent from general shell/panel transparency.

## mono accessibility requirements

Mono must preserve strong luminance structure and recognition. Validate very light, dark and saturated tint colors plus bright/dark backgrounds.

A defining foreground feature may remain white or near-white when necessary for contrast. If the icon becomes ambiguous without hue, fix geometry or luminance semantics rather than special-casing one palette.

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

Fit core recognizable geometry uniformly inside the calibrated safe guide. Do not stretch or crop merely to fill the square.

### full-bleed

Determine whether full bleed is intentional background artwork or accidental packaging. Preserve intentional background semantics; reconstruct accidental oversized packaging.

Double enclosure is a hard style failure.

## Apple-grid calibration

Do not hard-code one universal circular shrink percentage.

Do not treat `824 x 824` on `1024 x 1024` as the current universal Apple source of truth. `824 / 1024` may remain only as a historical/regression reference for flattened output until current calibration confirms an equivalent footprint.

Implement a developer/build-time calibration harness based on current public Apple design resources and representative Icon Composer output. Runtime must not depend on Apple resources or tooling.

Record and version at least:

- design canvas;
- flattened enclosure alpha bounds;
- designated circular-artwork guide;
- primary-content guide;
- optical safe region;
- corner/enclosure geometry;
- representative Default/Dark/Mono bounds;
- source/revision/measurement date.

Commit derived constants declaratively under a versioned grid revision. A later Apple revision creates a new grid/renderer revision rather than silently moving all icons.

Circular artwork sizing uses its own calibrated guide. It is not sized from the outer flattened enclosure footprint.

## legacy auto-fit fallback

When no validated canonical package is available, provide a safe compatibility path:

```text
installed source
    ↓
isolate core artwork
    ↓
remove or ignore external effect footprint
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

For every new or materially changed trustworthy source fingerprint without a valid canonical package, run semantic decomposition through the selected configured vision-capable provider.

Even a clean SVG still needs semantic depth grouping for the final `.vicon` model. For clean official vector sources, however, AI acts primarily as semantic director while local code owns reliable geometry:

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

Only generate replacement vector geometry when raster or structure quality makes that necessary. This minimizes brand drift.

For raster inputs, local segmentation/vector candidates may be provided as extra evidence. GPT may reconstruct semantic vector layers when needed.

## provider and credential integration

Reuse Vesper's existing API-key-only AI control plane and Secret Service credential.

If the OpenAI key is already configured in `AI`, adaptive icons use it without asking again. Do not create an icon-specific key store or OAuth flow.

Provider/model selection is capability-driven:

- accepts image input;
- supports schema-constrained structured output;
- can return sufficient vector/metadata content;
- enabled in the existing Vesper provider configuration.

OpenAI/GPT is a first-class path. Use the current Responses-style multimodal/structured-output API or supported successor, not an image-generation endpoint.

## remote input and privacy boundary

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

## local and AI reconciliation

Remote output is a proposal. Before accepting it, reconcile:

- deterministic local measurements;
- AI semantic classification/decomposition;
- calibrated grid rules;
- source provenance and exact vector geometry;
- previous known-good canonical metadata.

Hard measurable geometry overrides contradictory model guesses. AI is authoritative only for semantic intent that local analysis cannot reliably infer.

If disagreement is material:

1. optionally retry once within bounded retry policy using a corrective structured prompt;
2. otherwise use legacy auto-fit or original rather than activating questionable artwork.

## identity protection

Never accept a generated icon that:

- replaces an official mark with a generic symbol;
- invents letters or text;
- removes defining geometry solely for style;
- hallucinates unrelated decoration/background;
- materially distorts a trademark;
- embeds the original raster as fake SVG.

A correct original or legacy fallback is better than a polished wrong logo.

## safety validation

Treat model SVG/XML as untrusted input.

Reject or sanitize at least:

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

Create a neighboring-icon reference board. Compare candidate optical footprint, visual weight, enclosure size, background luminance, depth intensity and specular strength beside stable known-good Vesper icons. A technically valid but obviously too large, small or heavy icon fails style validation.

Track a compliance score, but hard safety or identity failures reject regardless of score.

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

Never feed flattened outputs back into canonical cache or source resolution.

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

## palette and theme behavior

Caelestia remains the palette owner. Adaptive icons are another consumer of the same palette.

Palette, wallpaper, accent, light/dark, Clear/Tinted or renderer recipe changes must never trigger AI for already valid canonical packages.

Debounce rapid changes, compile to staging and atomically switch the active generated theme where practical. Avoid mixed generations on screen.

## tray and status icons

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

Cache accepted canonical packages by a stable work key including:

- trustworthy source content fingerprint;
- canonical schema version;
- semantic/prompt contract revision;
- provider/model family information needed for invalidation;
- validator revision when it changes canonical acceptance.

Do not include wallpaper, accent or current appearance in the canonical key.

Keep state tiers visible:

```text
canonical-ai
legacy-auto-fit
original-fallback
blocked-no-provider
blocked-no-consent
unresolved-source
failed
```

If reliable vector geometry was preserved locally inside an AI-directed package, provenance may record that without inventing a separate product mode.

Store non-secret provider/model/prompt/grid/renderer provenance and failure category. Do not retain raw authorization headers or unnecessary provider payloads.

Reference-count or otherwise safely track shared canonical geometry so source-hash deduplication does not cause one app removal to delete assets still used by another app.

## failure and fallback chain

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

Failures are per icon and per appearance. They do not block startup or local theme switching.

Use bounded retries and backoff. Do not create a tight provider retry loop.

## icon theme installation

Publish normal application outputs through a generated freedesktop Vesper icon theme under the user's XDG data root. Inherit from a maintained fallback such as the existing Papirus configuration for non-generated applications, symbolic UI icons and unrelated assets.

Do not rewrite every `.desktop` file and do not modify packages or the Nix store. Keep an immediate rollback path to the previous configured icon theme.

The generated Vesper theme path must be registered in the source resolver's hard exclusion set so it can never become canonical input.

## settings ownership

### AI → Adaptive icons

Own generation/provider concerns:

- automatic canonicalization on/off;
- remote conversion consent/on/off if separately useful;
- selected capable provider;
- selected model or `Auto`;
- provider credential ready/missing;
- discovered/canonicalized/pending/running/retry/failed/blocked counts;
- progress such as `38 / 147 canonicalized`;
- current conversion activity;
- pause/resume queue;
- retry failed/regenerate operations.

If the existing OpenAI key is configured, show ready. Never show another OpenAI-key field here.

Provider readiness changes should automatically trigger catch-up of eligible fallback/blocked icons.

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
- source resolver status;
- queue state when pending;
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

Use stable sanitized canonical application/desktop ids for filenames. Do not rely on translated display names.

Snapshot accepted inventory, render into staging, record per-app failures, write final manifest and publish atomically where practical. Export must never mutate or corrupt the active cache.

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

Exported files and export staging directories are permanently excluded from source discovery.

## XDG data layout

Use documented XDG roots and clearly separate:

- application inventory and identity map;
- source fingerprints and resolver provenance;
- persistent conversion queue;
- canonical `.vicon` packages;
- shared-canonical alias/reference metadata;
- validation/provenance state;
- compiled active theme generations;
- failures/retry/blocked metadata;
- disposable previews/provider caches;
- export staging and completed exports when user-selected.

Do not retain duplicate packaged source icons indefinitely when they can be resolved again from installed applications.

Generated, cache, preview and export roots must be explicitly marked Vesper-owned and excluded from upstream source resolution.

## implementation shape

Prefer one coherent Rust subsystem rather than scripts scattered around the rice. Extend `vesper-control` or use a dedicated Rust `vesper-icons` worker when separation is cleaner.

Reasonable module boundaries:

```text
discover
desktop
identity
identity/steam
identity/wine
identity/pwa
icon_resolver
source_guard
source_analysis
segmentation
queue
ai/provider
ai/schema
ai/reconcile
canonical
canonical/dedup
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

1. finalize `.vicon` schema and canonical/source provenance;
2. implement XDG discovery and deterministic source resolver;
3. implement self-ingestion exclusions for generated theme/cache/export roots;
4. implement runtime identity inventory including Steam, Wine/Proton, PWA and Electron special cases;
5. implement local source geometry/silhouette/effect analysis;
6. implement source-hash canonical work deduplication and alias/reference accounting;
7. implement persistent crash-safe conversion queue with concurrency/rate-limit/backoff policy;
8. implement strict SVG/package safety validator;
9. implement Apple-grid calibration constants/harness and optical normalization;
10. implement deterministic static renderer for Default/Dark/Mono-derived appearances;
11. generate/activate Vesper freedesktop icon theme atomically;
12. reuse existing AI provider capability and Secret Service key path;
13. implement GPT image/vector input and structured decomposition schema;
14. preserve exact official vector geometry locally where possible;
15. implement AI/local reconciliation and identity validation;
16. implement provider-readiness catch-up for blocked/degraded icons;
17. add watcher/incremental reconciliation and safe retry/fallback state;
18. add versioned system gradient/blend/material recipes;
19. add Caelestia live layered lighting/specular/refraction renderer;
20. implement neighboring-icon validation board;
21. implement tray/status exclusion/symbolic path;
22. replace manual queue UI with AI/Appearance/Apps ownership split and queue progress;
23. implement bulk export backend and **Export all icons** UI;
24. migrate or garbage-collect obsolete queue state and document final XDG paths;
25. run full Nix/Rust/QML build/eval checks required by `AGENTS.md`.

## acceptance criteria

The feature is complete only when all of these are true:

1. one `ADAPTIVE-ICONS.md` document is the sole adaptive-icon architecture source of truth;
2. installed apps are discovered from effective XDG `.desktop` entries and trustworthy source resolution;
3. Vesper generated theme/cache/preview/export outputs can never be re-ingested as upstream icon sources;
4. unresolved `Icon=` references use the deterministic recovery chain before becoming `unresolved-source`;
5. runtime app identity reconciles desktop id, WMClass, app_id, package and executable evidence;
6. Steam applications retain distinct Steam app ids rather than collapsing into the client;
7. Wine/Proton applications retain generated launcher/prefix/executable identity rather than collapsing into `wine`;
8. browser PWAs remain distinct via stable PWA/generated desktop ids;
9. Electron applications use explicit app identity where available rather than generic runtime process names;
10. every new or materially changed trustworthy source without valid cache receives semantic AI decomposition when a provider is ready;
11. missing provider/consent produces a blocked/degraded state rather than a broken icon;
12. adding a capable provider/key later automatically queues eligible blocked/legacy icons without per-app manual regeneration;
13. already valid canonical packages are not regenerated merely because another provider becomes available;
14. identical trustworthy source hashes share canonical work when semantically safe;
15. duplicate desktop entries cannot cause duplicate in-flight AI calls for the same canonical work key;
16. shared canonical geometry remains valid while any referencing application still uses it;
17. initial-library conversion is processed through a persistent queue rather than an unbounded request burst;
18. queue state survives restart/logout/reboot;
19. queue concurrency is bounded and provider rate limits/Retry-After/backoff are respected;
20. stale jobs are superseded when their source changes before completion;
21. an already configured OpenAI/Vesper API key is reused without asking for another key;
22. image analysis plus structured output is used, not an image-generation endpoint;
23. clean official vector geometry is preserved locally where reliable while AI supplies semantic grouping;
24. canonical output is a multi-layer `.vicon`, not a flattened SVG;
25. `.vicon` uses a shared unmasked 1024-square canvas;
26. background is separate from one-to-four normal foreground depth groups;
27. groups support combined/individual material treatment;
28. one semantic object can be split across depth fragments when the source genuinely weaves through depth;
29. blend intent is bounded semantic metadata;
30. generated blur, glow, shadow, specular, refraction and final mask are not baked into canonical source artwork;
31. source silhouette is classified as enclosed, circular, glyph, irregular or full-bleed;
32. circular/glyph/irregular normalization uses calibrated guides, not one fixed shrink percentage;
33. `824/1024` is never treated as universal current Apple truth unless calibration explicitly confirms it;
34. already-enclosed icons are never double-enclosed;
35. material renderer owns system lighting, specular, selective refraction, translucency and depth shadows;
36. canonical annotations are Default/Dark/Mono and compiler derives Clear/Tinted variants locally;
37. Glass remains a material axis separate from appearance;
38. Clear material remains independent from general shell transparency;
39. all outputs survive small-size and neighboring-icon optical validation;
40. unsafe or identity-drifting AI output falls back instead of activating;
41. provider outage never breaks existing icons or local theme switching;
42. Vesper-owned live surfaces and freedesktop outputs share identical normalized geometry;
43. renderer recipe upgrades recompile existing canonical packages without AI;
44. tray/status icons are not run through the full app-icon squircle pipeline;
45. launcher/running/switcher/app-grid icons resolve to the same canonical application identity in Vesper surfaces;
46. generated theme switching is staged/atomic enough to avoid mixed visual generations;
47. the user can see automatic conversion progress and pending/blocked/failed states;
48. normal successful operation requires no manual per-app approval;
49. the user can bulk-export accepted icons via **Export all icons**;
50. export can produce current SVG/PNG, all appearances, canonical `.vicon` packages and a complete archive;
51. export never triggers AI and never mutates the active cache;
52. exported files can never become future source inputs;
53. disabling adaptive icons returns immediately to the configured fallback/original icon theme;
54. no first-party Python service or script is introduced for the feature.
