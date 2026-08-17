# Vesper — AI Control Plane + Production App Icon Plan

This file is the implementation plan for the current Vesper control-plane work in PR #18.

It intentionally supersedes older notes/prompts where **Adaptive Icons** were described as an experimental feature under `Settings → Apps`.

The target is now a **production Vesper feature**, controlled from **Settings → AI**, with deterministic rendering, safe AI-assisted preparation, curated SVG assets for default applications, and Apple-style icon appearance modes.

---

## 0. Non-negotiable decisions

1. **AI App Icons are not experimental.**
   - Remove the `Experimental` label and experimental positioning.
   - Treat the feature as a normal supported Vesper capability.
   - Failure must degrade cleanly to the original application icon.

2. **App Icon controls live under Settings → AI.**
   - Remove the user-facing `AI adaptive icons` control from `Settings → Apps`.
   - Keep the shared app registry/backend model reusable by Apps and AI.
   - The AI page owns enable/disable, mode selection, tint configuration, generation state and repair/regenerate actions.

3. **The visual modes are production UI states, not AI prompts.**
   - `Light`
   - `Dark`
   - `Tinted`
   - `Clear`
   - Tinted mode includes a user-selectable tint color.
   - Switching mode/color must be deterministic and must never call an AI provider.

4. **AI is only an icon-preparation fallback.**
   - Curated semantic SVG available → use it.
   - Existing usable symbolic/vector icon available → normalize deterministically.
   - Deterministic conversion possible → use it.
   - Only difficult/unsupported icons go through the AI icon curator.

5. **Default Vesper application icons should have curated SVGs.**
   - Repository-owned default/known applications should resolve to prepared semantic SVG assets before runtime AI is considered.
   - Prepare a stable asset directory and manifest keyed by desktop/application identity.
   - The actual curated SVG files can be populated/adjusted manually after Vesper is booted and visually inspected.
   - Runtime AI must not waste quota regenerating icons that already have a curated Vesper SVG.

6. **Settings → AI must expose full provider usage/quota data.**
   - The existing `@ai@ status` provider model is the source of truth.
   - Reuse/refactor the existing Dashboard provider-card logic instead of creating a second quota backend.
   - Settings must show the actual provider windows and reset times, not just `Providers → N`.

7. **API-key-only architecture remains unchanged.**
   - No OAuth broker.
   - No refresh-token manager.
   - No CLI/session token harvesting.
   - Secrets never enter QML, argv, logs, SQLite/JSON snapshots, Git, or the Nix store.

---

# 1. Target Settings → AI information architecture

The AI settings surface should converge toward:

```text
AI
├── Overview
├── Usage & Quotas
├── App Icons
├── API Keys / Credentials
├── Providers
├── Agents
├── Skills
├── MCP
└── Hermes
```

`API Keys / Credentials` is one credential system, not two different stores. The UI can use the friendlier `API Keys` label while the backend continues to model logical credential aliases.

The current lightweight overview should remain concise and link into the detailed sections.

---

# 2. Usage & Quotas

## Current problem

`AiPage.qml` already invokes `@ai@ status`, but the Settings page currently reduces provider information to an aggregate provider count/status line.

The Dashboard AI surface already has detailed provider rendering and the backend already normalizes the useful data.

Do not build another quota service.

## Target

Add a first-class:

```text
Settings
→ AI
→ Usage & Quotas
```

Use the same normalized provider objects returned by `@ai@ status`.

Each provider card should support, where the provider actually supplies the data:

```text
provider name
provider/source status
plan
account
health
max used percent
quota windows
credits
cost
provider error
```

Each quota window should support:

```text
label/window name
usedPercent
remainingPercent
resetAt
status
```

Example presentation:

```text
OpenAI / Codex
Plus

5h window
73% used
27% remaining
resets 03:42

Weekly
91% used
9% remaining
resets Aug 18 11:54

CRITICAL
```

And another provider may look like:

```text
Claude
Max
Session 42% used
Weekly 68% used
resets …
```

If supported, also show:

```text
credits remaining
$ spent today
$ spent in last 30 days
provider/account error
```

## Health semantics

Preserve the current normalized health logic:

```text
remaining <= 10%  → critical
remaining <= 25%  → warning
otherwise          → healthy/normal
```

Do not invent quota windows for providers that do not expose them.

## UI implementation rule

Do not maintain two separate ProviderCard implementations that drift apart.

Extract/reuse common provider quota components so both:

```text
Dashboard → AI
Settings → AI → Usage & Quotas
```

render the same normalized schema.

Settings may be more detailed, while Dashboard stays compact.

## Refresh behavior

- normal cached/status refresh for opening the page
- explicit manual refresh action
- stale snapshot indication
- provider errors isolated per provider
- one broken provider must not break the AI page

---

# 3. Production App Icons

## User-facing location

Move the feature from:

```text
Settings → Apps → Experimental → AI adaptive icons
```

to:

```text
Settings → AI → App Icons
```

No `Experimental` heading should remain for this feature.

## Top-level controls

The App Icons page/section should expose:

```text
App Icons                         [On/Off]

Appearance
○ Light
○ Dark
○ Tinted
○ Clear

Tint color                        [color picker]
                                  [presets / current accent]
```

The tint selector is relevant to `Tinted` mode and may be visually disabled/hidden in the other modes.

Also expose useful status:

```text
Prepared icons
Curated icons
Generated icons
Original fallbacks
Pending/failed conversions
Last rebuild
```

Optional per-app actions:

```text
Preview
Regenerate
Use curated icon
Use original icon
Reset generated icon
```

---

# 4. Icon appearance semantics

The source asset is a validated semantic SVG/mask. Appearance is applied later by a deterministic renderer.

## Light

Goal:

- clean icon intended for light desktop surfaces
- preserve semantic shape
- use the light palette/foreground treatment
- no AI call

## Dark

Goal:

- dark-mode counterpart
- adapt fill/stroke/background to Vesper dark palette
- preserve semantic shape and legibility
- no AI call

## Tinted

Goal: Apple-style single-accent icon treatment.

Behavior:

- user selects one tint/accent color
- renderer derives foreground/background tones from that tint plus the active Vesper palette
- preserve icon silhouette and semantic hierarchy
- retain sufficient contrast
- all installed prepared icons update together
- changing tint color is immediate/deterministic
- changing tint color never calls AI

Do not bake a tint into every generated SVG. Store semantic source once and render the selected tint from it.

## Clear

Goal: a transparent/clear appearance using the semantic icon mask.

Behavior:

- favor transparent/low-fill surfaces and clean strokes/highlights
- remain readable on the current wallpaper/panel/surface
- use Vesper palette/contrast information
- do not require a separate AI-generated asset
- no AI call when switching into or out of Clear

The exact visual recipe should be tuned after booting Vesper and inspecting real icons, but the data model must make Clear a normal first-class mode rather than an experimental special case.

---

# 5. Global App Icon state model

Use a backend-owned state similar to:

```text
AppIconSettings
├── enabled: bool
├── mode: light | dark | tinted | clear
├── tintColor
├── rendererVersion
└── updatedAt
```

QML should display state and invoke explicit backend actions.

QML must not:

- rewrite SVGs itself
- invoke arbitrary shell pipelines
- parse untrusted SVG internals
- hold provider credentials
- perform AI requests directly

Suggested CLI/control surface conceptually:

```text
vesper-control icon status
vesper-control icon on
vesper-control icon off
vesper-control icon mode light
vesper-control icon mode dark
vesper-control icon mode tinted
vesper-control icon mode clear
vesper-control icon tint <validated-color-value>
vesper-control icon list
vesper-control icon regenerate <app-id>
vesper-control icon reset <app-id>
```

Exact command names may be adapted to the existing Rust command structure.

---

# 6. App identity and curated SVG assets

Icons must key off the same central app identity used by permissions/wellbeing rather than filename guessing scattered across QML.

Conceptually:

```text
App
├── id
├── desktopId
├── name
├── originalIcon
└── appearance
    ├── source
    ├── semanticSvg
    ├── preparedState
    └── fallback
```

## Curated default icon source

Create one stable repository-owned location for hand-prepared semantic SVGs, for example:

```text
home/yargc/assets/app-icons/
```

with a manifest mapping application/desktop IDs to assets.

Example concept:

```text
home/yargc/assets/app-icons/
├── manifest.json or Nix-owned manifest
├── firefox.svg
├── terminal.svg
├── vesper-ai.svg
└── …
```

Prefer declarative metadata/Nix structures where practical instead of mutable JSON in the Nix store.

The important contract is:

```text
known default app
→ curated semantic SVG
→ deterministic renderer
→ Light/Dark/Tinted/Clear
```

The curated SVG set is expected to be visually refined manually after the real desktop is running.

---

# 7. Icon preparation pipeline

Target order:

```text
application icon
↓
curated Vesper semantic SVG exists?
├── yes → validate → cache/reference → renderer
└── no
    ↓
usable existing symbolic/vector icon?
├── yes → deterministic normalize → validate → renderer
└── no
    ↓
deterministic conversion possible?
├── yes → normalize → validate → renderer
└── no
    ↓
AI icon curator enabled and credential available?
├── yes → AI preparation → sanitize → validate → cache → renderer
└── no  → original icon fallback
```

This order is mandatory so AI quota is not consumed for ordinary known icons.

---

# 8. AI icon curator

Keep the `icon-curator` concept, but treat it as a production fallback capability rather than an experimental UI feature.

The AI is responsible only for converting a difficult source icon into a **semantic, renderable representation**.

It must not decide the current desktop tint or theme.

Conceptually:

```text
icon-curator
├── input: original icon + app identity
├── output: semantic SVG
├── provider: selected Vesper AI provider
└── credential: logical Vesper credential alias
```

The selected provider/credential comes from the shared Vesper AI credential system.

No API key may be embedded in:

- skill files
- QML
- generated SVG
- icon metadata
- cache metadata
- logs
- argv

## Production application of generated assets

When App Icons are enabled, a successfully generated icon may become active only after the backend completes strict sanitize/validation.

Validation failure means:

```text
keep original icon
mark conversion failed
show repair/regenerate option
```

There must always be a reversible path back to the original icon.

---

# 9. SVG security and validation

Treat all SVG input as untrusted, including AI output and icons discovered from applications.

The validator/sanitizer must reject or strip unsafe constructs such as:

- scripts
- event handlers
- external network references
- external file references
- embedded executable content
- unsupported/unsafe filters or foreign objects where appropriate
- pathological document sizes/complexity

Prefer a deliberately small supported SVG subset suitable for icon rendering.

The rendered icon must not be able to execute code or fetch external content.

---

# 10. Icon caches and generated state

Static curated assets belong in the repository/Nix-managed source.

Generated/prepared mutable assets belong outside the Nix store, under an appropriate XDG state/cache location.

Conceptually:

```text
$XDG_CACHE_HOME/vesper/app-icons/
```

or the existing Vesper cache/state convention discovered during implementation.

Cache keys should include enough information to invalidate correctly, such as:

```text
app identity
source icon hash
semantic conversion version
renderer version
```

Do not regenerate semantic SVGs merely because the tint color changed.

Tint/light/dark/clear output can be regenerated cheaply/deterministically from the same semantic source.

---

# 11. Relationship with Settings → Apps

`Settings → Apps` remains responsible for application-centric controls such as:

```text
installed app inventory
permissions
wellbeing/usage
```

The app registry can expose icon readiness/status internally, but App Icons configuration belongs to AI.

Remove the current user-facing `Experimental` section and `AI adaptive icons` toggle from `VesperAppsSettings.qml` once the AI App Icons UI is wired.

Do not duplicate icon state independently in Apps and AI.

---

# 12. AI page consolidation

The AI page should stop being only an inventory count surface.

Keep its overview, then expose real subpages/sections.

Target responsibilities:

## Overview

```text
provider count
credentials configured
most constrained provider
active agents
skills
MCP
Hermes unread/attention
App Icons status
```

## Usage & Quotas

Detailed normalized provider usage described above.

## App Icons

Production icon controls and status.

## API Keys / Credentials

Existing secure key management and aliases.

## Providers

Provider definitions, configured credential, models, health, consumers and test action where supported.

## Agents

Existing/live agent information plus assignments as implemented.

## Skills

Canonical skill registry and drafts/review model.

## MCP

Inventory, credentials, assignments, tools and enforceable permission policy.

## Hermes

Hermes registry/status/briefings/control-plane integration.

---

# 13. Shared provider/quota component refactor

Avoid Dashboard/Settings drift.

Extract the reusable visual/data pieces from the current Dashboard AI provider rendering, conceptually:

```text
ProviderQuotaCard.qml
QuotaWindowRow.qml
ProviderHealthBadge.qml
```

or equivalent local components consistent with the repository style.

Dashboard can use a compact configuration.

Settings can use the full configuration.

Both consume the same `@ai@ status` schema.

---

# 14. Backend rules for usage/quota data

Preserve one normalized schema.

Conceptually:

```text
AiProviderStatus
├── id
├── name
├── source
├── plan
├── account
├── health
├── maxUsedPercent
├── windows[]
│   ├── label
│   ├── usedPercent
│   ├── remainingPercent
│   └── resetAt
├── credits
├── cost
└── error
```

Do not force every provider adapter to return fields it cannot know.

Unknown means unknown, not fabricated zeroes.

---

# 15. Failure isolation

## App Icons

```text
AI unavailable
→ curated/deterministic/original icon path still works

invalid generated SVG
→ reject it
→ original icon stays active

renderer failure
→ original icon fallback

tint setting invalid
→ reject setting
→ retain previous valid configuration
```

## Usage & Quotas

```text
one provider API/status failure
→ only that provider card shows the error

usage backend stale
→ show stale state
→ credentials/skills/MCP/Hermes settings still work
```

None of these failures may prevent graphical login or break the Settings application.

---

# 16. Implementation order

## Phase A — document/current-state cleanup

1. Treat this `PLAN.md` as the current source of truth for PR #18 follow-up work.
2. Remove outdated `Experimental Adaptive Icons` language as code is migrated.
3. Keep older master prompts only as historical context where they conflict with this file.

## Phase B — shared quota UI

1. Extract/reuse Dashboard provider quota components.
2. Add `Settings → AI → Usage & Quotas`.
3. Render provider windows, remaining/used percentages and resets.
4. Render plan/account/credits/cost/error where available.
5. Preserve stale/manual-refresh behavior.

## Phase C — App Icon backend state

1. Promote icon state from an experimental boolean to production settings.
2. Add mode enum: Light/Dark/Tinted/Clear.
3. Add validated tint color state.
4. Add icon inventory/status data.
5. Preserve original icon fallback.

## Phase D — curated semantic SVG foundation

1. Add repository-owned asset location/manifest.
2. Resolve curated assets by canonical app identity.
3. Add default Vesper/installed-app mappings as assets become available.
4. Ensure curated SVG always wins over runtime AI generation.

## Phase E — deterministic renderer

1. Normalize semantic SVG/mask representation.
2. Implement Light.
3. Implement Dark.
4. Implement Tinted with user-selected color.
5. Implement Clear.
6. Ensure switching modes/colors is AI-free.

## Phase F — move UI from Apps to AI

1. Add `Settings → AI → App Icons`.
2. Add global toggle.
3. Add appearance mode selector.
4. Add tint color picker/presets.
5. Add prepared/generated/fallback state.
6. Add preview/regenerate/reset actions where practical.
7. Remove the old Apps experimental toggle/section.

## Phase G — AI curator production fallback

1. Keep provider/credential selection in the shared AI credential system.
2. Invoke AI only for icons not solvable by curated/deterministic paths.
3. Sanitize and validate generated SVG.
4. Cache semantic output.
5. Automatically fall back to original icon on any failure.

## Phase H — validation

1. Validate Rust control-plane compilation.
2. Validate QML/patch application.
3. Validate Home Manager evaluation/build.
4. Validate full NixOS configuration.
5. Boot Vesper.
6. Visually inspect curated default SVGs.
7. Tune Tinted and Clear rendering against real light/dark wallpapers/surfaces.
8. Confirm no icon operation leaks credentials or executes untrusted SVG content.

---

# 17. Definition of done

The work described by this plan is complete when all of the following are true:

1. `Settings → Apps` no longer presents App Icons as experimental.
2. `Settings → AI` contains a production `App Icons` section/page.
3. App Icons have a global enable/disable control.
4. The supported appearance modes are `Light`, `Dark`, `Tinted`, and `Clear`.
5. Tinted mode has a user-selectable tint color.
6. Changing appearance mode never calls AI.
7. Changing tint color never calls AI.
8. Known/default Vesper applications can use repository-owned curated semantic SVGs.
9. The curated SVG set can be manually refined after booting the real desktop.
10. Unknown icons first attempt deterministic preparation.
11. AI is used only when curated/deterministic preparation cannot produce a usable semantic icon.
12. AI-generated SVG is sanitized and validated before activation.
13. Unsafe/invalid SVG cannot execute code or load external content.
14. Original application icons always remain a fallback.
15. Users can revert/reset generated icon state.
16. Generated semantic icons are not regenerated merely because the palette/tint changed.
17. `Settings → AI → Usage & Quotas` shows detailed provider data from the existing `@ai@ status` model.
18. Quota windows expose used percentage, remaining percentage and reset time when available.
19. Provider cards expose plan/account/credits/cost/error when available.
20. `critical` and `warning` quota states remain consistent with the existing normalized health model.
21. Dashboard and Settings share provider quota rendering/data components instead of drifting copies.
22. No new OAuth management is introduced.
23. API keys remain outside QML, argv, logs, Git and the Nix store.
24. App Icon failures and provider failures remain isolated and cannot prevent graphical login.
25. PR #18 ends with passing patch/Rust/Home Manager/NixOS validation and a real-desktop visual pass for the icon modes.

---

# 18. Final target

```text
                         Settings → AI
                              │
        ┌───────────────┬─────┼───────────────┐
        ↓               ↓     ↓               ↓
 Usage & Quotas     App Icons API Keys      Providers
        │               │     │               │
        │               │     └──────┬────────┘
        │               │            ↓
        │               │      Credential Vault
        │               │            │
        │               ↓            ↓
        │        semantic icon    AI providers
        │             source          │
        │               │             │
        │      ┌────────┼───────┐     │
        │      ↓        ↓       ↓     │
        │    Light     Dark   Tinted  │
        │                    / Clear  │
        │                      ↑      │
        │                deterministic│
        │                   renderer  │
        │                      ↑      │
        │       curated/deterministic │
        │                or AI curator┘
        │
        └── same normalized provider quota model as Dashboard AI
```

Core principle:

```text
AI prepares difficult icons once.
Vesper renders them many times.
Theme/tint changes are deterministic.
Known default icons are curated.
Usage/quota data has one source of truth.
Secrets stay centralized.
```
