# adaptive icon AI provider contract

This document is the normative AI integration contract for Vesper adaptive application icons.

Read it together with `ADAPTIVE-ICONS.md`, `APPLE-ICON-MODEL.md` and `APPLE-ICON-AUTOFIT.md`.

If the older implementation prompt conflicts with this document on provider selection, credential reuse, multimodal input, structured model output, AI/local reconciliation or remote-conversion privacy, follow this document.

The icon engine must reuse Vesper's existing API-key-only AI control plane. Do not create a second credential store, a separate icon-specific API key setting or an OAuth flow.

## goal

When an installed application icon cannot be normalized locally with sufficient quality, Vesper should be able to use an already configured GPT/OpenAI API key to inspect the real source icon and reconstruct a canonical Apple-style semantic vector asset.

The user should configure the OpenAI key once in Vesper's existing AI settings. Adaptive icons then discover that configured provider capability automatically.

The normal path is:

```text
.desktop
    ↓
resolve Icon=
    ↓
real installed SVG/PNG/etc
    ↓
local source analysis
    ↓
local normalization sufficient?
    ├── yes → canonicalize locally
    └── no
         ↓
    configured AI provider
         ↓
    OpenAI/GPT vision when selected and available
         ↓
    structured canonical proposal
         ↓
    local reconciliation
         ↓
    strict local validation
         ↓
    canonical cache
         ↓
    deterministic appearance/material compiler
```

AI must never be required for palette changes, light/dark changes, Clear/Tinted rendering, Glass rendering, icon-theme rebuilds or normal startup when a valid canonical asset already exists.

## credential reuse

Use the existing Vesper credential system and Secret Service integration.

The implementation must:

- detect whether the OpenAI provider already has a configured API key
- use that existing credential without asking the user to enter it again
- keep the secret out of the Nix store
- keep the secret out of generated SVG, metadata and logs
- avoid placing the raw key in command-line arguments
- expose only configured/missing provider state to QML
- preserve the existing API-key-only policy

Do not create files such as:

```text
~/.config/vesper/icon-openai-key
~/.local/share/vesper/adaptive-icons/api-key
```

Do not duplicate Secret Service entries under an icon-specific naming scheme when the existing OpenAI credential can be reused safely.

## provider selection

OpenAI/GPT should be a first-class supported path because a configured GPT model can inspect image input and return structured text output suitable for the canonicalization pipeline.

Do not permanently bind the architecture to one exact GPT model name.

Select by capability:

- accepts image input
- supports the structured response contract required by the icon engine
- can return enough text for SVG geometry and metadata
- is enabled by the user's existing Vesper provider configuration

The AI settings page may expose:

```text
Adaptive icons
  automatic conversion      on
  provider                  OpenAI
  model                     Auto
```

`Auto` should resolve to a configured vision-capable model supported by the provider adapter.

If multiple capable providers exist, the user may select one. OpenAI should work without any additional icon-specific credential step when its API key is already configured.

If the selected provider becomes unavailable, keep existing canonical icons and fall back visually. Never break the active desktop icon theme because remote generation is offline.

## OpenAI transport

For the OpenAI provider, use the current Responses API or its supported successor rather than an image-generation endpoint.

The required model interaction is image analysis plus structured text generation, not image synthesis.

The provider adapter should be able to send:

- an image representation of the installed icon
- conversion instructions derived from Vesper's versioned icon contracts
- sanitized source-vector text when useful

and receive a schema-constrained structured result.

The OpenAI API supports image inputs to Responses and structured output through JSON Schema on supported models. Use those capabilities instead of scraping free-form prose.

Do not use the Images API as the normal adaptive-icon pipeline.

## source payload

Send only what the model needs to understand the icon.

### raster source

For PNG, WebP or another raster source, send a normalized high-quality icon render with transparent background preserved.

Do not send the full desktop screenshot, wallpaper, launcher, application window or unrelated UI.

### SVG source

For SVG, provide both when useful:

1. a local raster preview of the sanitized SVG
2. sanitized SVG/XML source text

This lets the model understand the rendered appearance while preserving exact vector hints that would be lost by vision-only reconstruction.

Before sending SVG text, remove or reject:

- scripts
- event handlers
- external references
- remote URLs
- embedded foreign documents
- irrelevant metadata
- comments containing local filesystem information

Do not send the original absolute source path to the provider.

## privacy boundary

Remote icon conversion is an explicit network feature.

When enabled, the UI must make clear that application icon artwork may be sent to the selected AI provider.

The provider request should contain only the minimum required content:

- icon pixels or sanitized vector artwork
- the versioned canonicalization contract
- a neutral request id if needed

Do not send:

- full `.desktop` contents
- `Exec=` commands
- filesystem paths
- username or hostname
- Home Manager/Nix configuration
- application usage history
- wellbeing data
- window titles
- process lists
- unrelated installed-app inventory

The model does not need the user's personal environment to reconstruct an icon.

The application identity may remain local unless a short stable identifier is genuinely required for caching/debugging. Prefer opaque local request ids over unnecessary metadata disclosure.

## AI input contract

The model should be instructed that it is producing canonical source artwork and semantic analysis, not a finished glossy icon.

The request contract must incorporate the current rules from:

- `APPLE-ICON-MODEL.md`
- `APPLE-ICON-AUTOFIT.md`

Generate the actual provider prompt from a versioned implementation template. Do not ask the model vaguely to "make this look like macOS".

The prompt must communicate at least:

- `1024 x 1024` unmasked canonical canvas
- preserve recognizable brand identity
- classify the source silhouette
- distinguish legacy compatibility from canonical redesign
- separate background and foreground artwork
- preserve back-to-front layer order
- default/dark/mono semantic appearance annotations
- no final rounded enclosure baked into source artwork
- no generated glass baked into canonical artwork
- no baked drop shadow, blur, glow, bevel, refraction or generated specular highlight
- crisp vector edges
- Apple-derived optical grid and safe-area rules
- circular/irregular source handling from the auto-fit contract
- text only when essential to the original mark
- required text converted to vector outlines
- no external fonts
- no embedded raster image inside an accepted canonical SVG
- no invented branding

The model should prefer preserving official geometry over redesigning for style.

## structured model output

Do not parse a conversational answer such as:

```text
Here is your SVG. I made the logo a bit smaller...
```

Require a schema-constrained result.

The exact schema can evolve, but it should represent concepts equivalent to:

```json
{
  "schemaVersion": 1,
  "sourceAssessment": {
    "shapeClass": "circular",
    "confidence": 0.97,
    "requiresAIReconstruction": true,
    "identityRisk": "low"
  },
  "normalization": {
    "needsEnclosure": true,
    "opticalOffsetX": 0.0,
    "opticalOffsetY": -0.01,
    "backgroundIntent": "system-compatibility"
  },
  "artwork": {
    "defaultSvg": "<svg>...</svg>",
    "darkSvg": null,
    "monoSvg": "<svg>...</svg>"
  },
  "materialIntent": {
    "glassEligible": true,
    "refraction": "auto",
    "specular": "auto"
  },
  "notes": []
}
```

The example is conceptual. The implementation may use one shared SVG plus semantic layer metadata instead of three complete SVG strings where that is cleaner.

Use strict schema validation before any SVG-specific validation begins.

A model result that fails the response schema is a failed generation, not something to repair with brittle string extraction.

## semantic role of GPT

GPT is most valuable for semantic interpretation.

Use it to reason about questions such as:

- which geometry is the actual recognizable logo
- which shapes form the background
- which gradients or halos are legacy effects rather than brand artwork
- which visual details can be removed at small sizes
- which shapes must remain in the mono representation
- whether a circular source is a foreground logo or an intentional complete app-icon composition
- whether a raster source can be safely reconstructed without changing identity

Do not ask GPT to decide runtime theme colors that can be derived deterministically from Caelestia.

Do not ask GPT to decide the final Glass shader output.

Do not ask GPT to regenerate the same canonical geometry because the wallpaper changed.

## local analysis remains authoritative for measurable facts

Do not make AI the only source of truth.

The Rust pipeline should independently measure what can be measured deterministically, including:

- alpha bounds
- visible vector bounds
- occupied-area ratio
- aspect ratio
- circularity
- edge proximity
- connected visible regions where practical
- transparency coverage
- canvas dimensions
- clipping structure
- likely external shadow/effect footprint

Compare local measurements with model claims.

For example:

```text
local circularity        0.98
AI shapeClass            circular
AI confidence            0.97
                         ↓
                    agreement
```

But:

```text
local geometry           isolated circle, 80% transparent canvas
AI shapeClass            enclosed
                         ↓
                    disagreement
```

The second result must not be accepted blindly.

## reconciliation

Insert an explicit reconciliation stage between remote output and canonical validation.

The reconciler should combine:

- deterministic local measurements
- AI semantic classification
- Apple-derived grid rules
- source provenance
- previous known-good metadata if available

Hard geometric facts should override contradictory model guesses.

Semantic decisions may use the model when local analysis cannot reliably infer intent.

If disagreement is material and cannot be resolved safely:

1. retry with a corrective structured prompt at most within bounded retry policy
2. otherwise fall back to local legacy auto-fit or the original icon

Do not activate a questionable reconstruction merely because the API request succeeded.

## identity protection

Brand identity preservation is a hard requirement.

The AI result must not:

- replace an official mark with a generic icon
- invent letters or text
- change a recognizable symbol into a different symbol
- remove a defining feature solely to make the icon simpler
- hallucinate a new brand background in legacy compatibility mode
- introduce decorative details not present in the source

For difficult icons, keeping the original or using the legacy auto-fit wrapper is better than a polished but incorrect reconstruction.

## local-first policy

AI calls are a quality fallback, not the first stage for every app.

Use this order:

```text
official clean vector
    ↓
local normalize
    ↓ if insufficient
local sanitize/restructure/trace
    ↓ if insufficient
GPT semantic reconstruction
    ↓ if unsafe or low confidence
legacy auto-fit/original fallback
```

A clean official SVG should normally cost zero API calls.

A palette change should cost zero API calls.

A material change should cost zero API calls.

A valid cached canonical icon should cost zero API calls.

## caching

Cache successful canonical output against a stable key that includes at least:

- source content fingerprint
- canonical schema version
- AI prompt/contract revision
- provider/model family information needed for invalidation
- validator revision when material

Do not include the current accent color or wallpaper in the canonical AI cache key.

The cache should make model usage converge toward zero after the installed app set has been processed.

## failure and retry policy

Remote failures are per-icon failures.

They must not block:

- system startup
- Caelestia startup
- theme switching
- light/dark switching
- palette generation
- existing icon rendering

Use bounded retries with backoff for transient provider errors.

Do not continuously retry a source that repeatedly returns invalid SVG or fails identity validation.

Persist enough failure state to show the error category and offer `retry failed` without logging full provider responses.

## settings integration

The existing Vesper AI page owns provider selection and remote-generation state.

The `Adaptive icons` section should expose at least:

- automatic icon canonicalization on/off
- remote AI conversion on/off when separate consent is useful
- selected provider
- selected model or `Auto`
- whether the selected provider credential is configured
- locally normalized count
- AI-generated canonical count
- pending count
- failed count
- current conversion activity
- retry failed

If OpenAI is selected and the existing OpenAI API key is already configured, the UI should show it as ready. Do not show another API-key field inside Adaptive icons.

Appearance controls remain outside the AI page.

## provider prompt versioning

Treat the AI prompt as part of the canonical format contract.

Give it a version/revision id.

A material prompt change that affects semantic geometry should be able to invalidate or selectively regenerate canonical assets.

Minor wording changes that do not alter output semantics should not force a whole-library regeneration.

The prompt should reference internal contract versions rather than embedding undocumented magic values that drift away from `APPLE-ICON-MODEL.md` or `APPLE-ICON-AUTOFIT.md`.

## validation after AI

Remote structured output is only a proposal.

Before activation, run all existing canonical validation plus AI-specific checks:

- response JSON schema valid
- SVG XML valid
- no scripts/events
- no external resources
- no embedded raster payload
- correct canonical canvas
- expected layer/semantic structure
- no baked final enclosure
- no generated external shadow/glow/specular in canonical artwork
- identity similarity acceptable
- local/AI silhouette classification not materially contradictory
- optical fit valid against the Apple-derived grid
- default readable
- mono readable
- dark mapping valid when supplied
- small-size previews readable
- Clear/Tinted derivation remains recognizable

Never write raw model output directly into the active freedesktop theme.

## observability

Machine-readable status should distinguish how each icon reached canonical state:

```text
canonical-local
canonical-ai-openai
legacy-auto-fit
original-fallback
failed
```

Store provider/model provenance without secrets.

Useful status metadata includes:

- provider id
- model id/family
- prompt revision
- source fingerprint prefix
- generation timestamp
- validation score
- retry count

Do not expose raw API request payloads in normal logs.

## acceptance criteria

The AI integration is not complete until all of these are true:

1. an OpenAI API key configured in the existing Vesper AI settings is detected automatically by adaptive icons
2. enabling adaptive icons does not ask for the same OpenAI key a second time
3. a raster icon can be sent as image input to a configured vision-capable GPT model without using image generation
4. an SVG source can provide a sanitized rendered preview and sanitized vector text when useful
5. model output is schema-constrained structured data rather than conversational prose
6. the model produces canonical artwork/metadata rather than a flattened shiny icon
7. local geometry analysis runs independently of the model
8. local measurements and AI semantic analysis are reconciled before activation
9. contradictory or low-confidence output falls back safely
10. no raw AI output becomes the active icon before local validation
11. icon artwork is the only app-specific content sent remotely unless additional data is strictly required
12. `.desktop` contents, paths, usage history and unrelated system metadata are not uploaded
13. changing accent, wallpaper, appearance or material performs zero AI requests for valid cached canonical assets
14. provider outage leaves existing icons and theme switching functional
15. a clean official SVG can complete through a fully local path
16. AI provenance and failure status are visible without leaking API keys
17. provider/model selection remains capability-driven rather than permanently tied to one exact model version
18. all other Apple model, auto-fit and validation contracts continue to apply

## implementation order

Implement the AI portion in this order after local icon discovery/normalization and canonical validation exist:

1. define the structured canonical response schema
2. expose provider capability detection through the existing Vesper AI control plane
3. reuse existing Secret Service credential execution
4. implement the OpenAI image-input + structured-output adapter
5. implement sanitized SVG preview/source payloads
6. add the versioned canonicalization prompt
7. add local/AI reconciliation
8. add AI-specific validation and identity checks
9. add cache/provenance/failure state
10. wire provider/model/status controls into the existing AI page
11. add bounded retries and offline fallback tests
12. verify palette/theme changes never trigger remote regeneration

Do not implement this as an agent skill that must be manually invoked for normal operation. The adaptive icon service owns routine conversion. The skill can remain useful for diagnostics and explicit repair.
