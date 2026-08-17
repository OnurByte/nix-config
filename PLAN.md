# Vesper — AI Control Plane + Production App Icon Plan

This file is the implementation plan for the current Vesper control-plane work in PR #18.

It supersedes older notes/prompts where Adaptive Icons were experimental or lived under `Settings → Apps`.

The target is a production Vesper feature controlled from `Settings → AI`, with Apple-style Light/Dark/Tinted/Clear appearance modes, repository-owned curated SVGs for Vesper's default applications, automatic AI SVG generation for every non-curated application, and one shared provider usage/quota model used by both the Dashboard AI surface and Settings.

---

## 0. Non-negotiable decisions

1. **App Icons are production, not experimental.**
   - Remove the `Experimental` label and experimental positioning.
   - Failure must degrade cleanly to the original application icon.

2. **App Icon controls live under `Settings → AI → App Icons`.**
   - Remove the user-facing icon toggle from `Settings → Apps`.
   - The shared app registry can still be reused internally by Apps, Wellbeing, Permissions and AI.

3. **Appearance modes are deterministic runtime rendering states.**
   - `Light`
   - `Dark`
   - `Tinted`
   - `Clear`
   - Tinted includes a user-selectable color.
   - Switching mode or tint must never call AI.

4. **Default Vesper applications use curated repository SVGs.**
   - Known/default applications should have prepared semantic SVG assets in the repository.
   - Those assets win over generated assets.
   - The curated SVG set can be manually populated/refined after booting Vesper and visually checking the real desktop.

5. **Every non-curated application is AI-SVG'd automatically when App Icons are enabled.**
   - Do not put deterministic icon conversion in front of AI for unknown/new applications.
   - If a canonical curated Vesper SVG does not exist for an application, the AI icon curator is the normal preparation path.
   - The AI receives the original application icon plus canonical app identity and produces a semantic SVG.
   - The user must not press `Generate` for each application.
   - This applies both to already-installed non-curated applications when the feature is first enabled and to applications installed later.

6. **Deterministic rendering happens after semantic SVG preparation.**
   - AI/curated source produces one semantic SVG representation.
   - Vesper then renders Light/Dark/Tinted/Clear from that semantic source without further AI calls.

7. **Usage & Quotas exists in two surfaces with one data source.**
   - Dashboard/AI keeps a compact glanceable quota view.
   - `Settings → AI → Usage & Quotas` contains the detailed view.
   - Both consume the same normalized `@ai@ status` model.
   - Dashboard provides a button/action into detailed AI settings.

8. **The API-key-only architecture remains unchanged.**
   - No OAuth broker.
   - No refresh-token manager.
   - No CLI/session-token harvesting.
   - Secrets never enter QML, argv, logs, Git or the Nix store.

---

# 1. Target Settings → AI structure

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

`API Keys / Credentials` is one backend system. The UI may use the friendlier `API Keys` label while the backend continues to use logical credential aliases.

---

# 2. Usage & Quotas — one backend, two presentations

The existing `@ai@ status` model remains the source of truth.

```text
@ai@ status
     │
     ├── Dashboard / AI surface → compact
     │
     └── Settings → AI → Usage & Quotas → detailed
```

Do not create a second quota backend.

## Dashboard / AI compact view

Keep the current AI/dashboard surface, but allow it to be less detailed than Settings.

Prioritize:

```text
provider count
most constrained provider
used / remaining percentage
important reset time
warning / critical state
stale / error state
```

Example:

```text
AI
Codex · 91% used · 9% remaining
Weekly resets Aug 18 11:54
CRITICAL

[Usage details]
```

or:

```text
5 providers · 1 critical · 1 warning
Most constrained: Codex · 9% remaining
[Open AI settings]
```

The compact surface does not need to show every quota window, account, credit and cost field at once.

Add a direct action such as:

```text
Usage details
Open AI settings
Manage AI
```

Prefer direct navigation to:

```text
Settings → AI → Usage & Quotas
```

If the Settings navigation cannot deep-link to the subpage, open `Settings → AI` with Usage & Quotas immediately visible/selectable.

## Settings → AI → Usage & Quotas detailed view

Each provider card should show fields actually supplied by that provider:

```text
provider name
source/status
plan
account
health
max used percentage
quota windows
credits
cost
provider error
```

Each quota window supports:

```text
label
usedPercent
remainingPercent
resetAt
status
```

Example:

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

When available also show:

```text
credits remaining
$ spent today
$ spent in last 30 days
provider/account error
```

Preserve the normalized health rules:

```text
remaining <= 10%  → critical
remaining <= 25%  → warning
otherwise          → healthy/normal
```

Unknown provider data stays unknown; never fabricate zero values.

## Shared quota UI

Do not maintain drifting Dashboard and Settings implementations.

Extract shared components conceptually like:

```text
ProviderQuotaCard.qml
QuotaWindowRow.qml
ProviderHealthBadge.qml
```

or equivalent repository-style components.

They should support presentation density, conceptually:

```text
compact: true   → Dashboard
compact: false  → Settings
```

Both consume exactly the same normalized provider schema.

---

# 3. Production App Icons UI

Location:

```text
Settings → AI → App Icons
```

Top-level controls:

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

The tint control only affects Tinted mode.

Status should expose:

```text
Curated icons
AI-generated icons
Pending generations
Failed generations
Original fallbacks
Last reconciliation
```

Per-app repair/override controls may include:

```text
Preview
Regenerate with AI
Use curated icon
Use original icon
Reset generated icon
```

These are repair/override tools. Normal operation is automatic.

---

# 4. Automatic AI SVG generation

This is the core behavior.

When:

```text
Settings → AI → App Icons = On
```

Vesper reconciles the canonical application registry against the semantic icon inventory.

For each application:

```text
app identity
↓
curated Vesper SVG exists?
├── yes → validate → use curated semantic SVG
└── no
    ↓
AI icon curator
    ↓
generate semantic SVG from original app icon + app identity
    ↓
sanitize
    ↓
validate
    ↓
store generated semantic SVG
    ↓
deterministic Light/Dark/Tinted/Clear renderer
```

There is intentionally **no `symbolic icon → deterministic conversion → maybe AI` bypass** for non-curated applications.

The rule is simply:

```text
curated default icon exists → curated SVG
otherwise                   → AI generates SVG
```

The deterministic renderer is for appearance transformation after semantic SVG preparation, not for avoiding AI generation of unknown apps.

---

# 5. Existing apps when the feature is enabled

Turning App Icons on must not only affect future installs.

On first enable/re-enable:

```text
current app registry
↓
find every app without a valid semantic SVG
↓
curated asset?
├── yes → use it
└── no  → queue AI generation
```

Do not require reinstalling applications or manually opening each app entry.

If many apps require generation, process them through a bounded background queue rather than firing uncontrolled parallel requests.

---

# 6. Newly installed apps

While App Icons remains enabled, newly available applications should automatically enter the AI generation queue.

The implementation may detect this through the cleanest existing integration point, for example app-registry reconciliation after Home Manager/Nix activation or session/app-registry changes.

The user-facing behavior must be:

```text
install/add app
→ Vesper sees that the app has no curated semantic SVG
→ AI generates the SVG automatically
→ sanitized/validated SVG becomes the semantic source
→ current Light/Dark/Tinted/Clear rendering is applied
```

No manual Generate step.

Reconciliation is important so an app added while Vesper/session services were unavailable is processed on the next successful reconciliation.

---

# 7. AI generation queue

Automatic generation must be controlled.

States:

```text
pending
processing
prepared
failed
waiting-for-provider
fallback
```

Requirements:

- deduplicate by canonical app identity + original source icon hash
- do not regenerate unchanged prepared apps
- persist enough state across session/reboot to avoid repeated paid requests
- bounded concurrency
- bounded retry/backoff
- one failed app must not block the queue
- manual Regenerate remains available
- provider/quota failure keeps the original icon active

If the source icon changes after an application update, its source hash changes and the app can be queued for AI regeneration.

---

# 8. AI icon curator contract

`icon-curator` is a production Vesper capability.

Conceptually:

```text
icon-curator
├── input
│   ├── canonical app identity
│   ├── app/display name
│   └── original application icon
├── output
│   └── semantic SVG
├── provider
│   └── selected Vesper AI provider
└── credential
    └── logical Vesper credential alias
```

AI should preserve recognizable product identity and important icon geometry while converting it into a representation suitable for the Vesper renderer.

AI must not choose the current tint color or current desktop appearance mode.

No secret may be embedded in:

- skill files
- QML
- generated SVG
- SVG metadata
- cache metadata
- logs
- argv

---

# 9. Curated default SVGs

Create one repository-owned source of truth, for example:

```text
home/yargc/assets/app-icons/
├── manifest.nix / equivalent mapping
├── firefox.svg
├── terminal.svg
├── vesper-ai.svg
└── ...
```

Map assets by canonical desktop/application identity, not loose filename guessing in QML.

Contract:

```text
known Vesper/default app
→ curated semantic SVG
→ validate
→ deterministic renderer
```

These SVGs are intentionally easy to replace/refine manually after Vesper is booted and visually inspected.

Curated assets always override generated ones unless the user explicitly chooses another per-app override.

---

# 10. Semantic SVG model

The semantic SVG is the stable prepared source.

Conceptually:

```text
AppIconAsset
├── appId
├── desktopId
├── sourceType: curated | ai-generated
├── sourceIconHash
├── semanticSvg
├── generatorVersion
├── generatedAt
├── validationStatus
└── fallbackOriginalIcon
```

Do not store a separate AI-generated source just because the user changes theme or tint.

---

# 11. Appearance modes

The renderer consumes the validated semantic SVG.

## Light

- optimized for light surfaces
- maintain semantic silhouette/hierarchy
- use Vesper light palette rules
- zero AI calls

## Dark

- optimized for dark surfaces
- adapt foreground/background treatment
- preserve readability
- zero AI calls

## Tinted

Apple-style single-accent treatment.

- user chooses a tint color
- current Vesper accent may be offered as a preset
- derive foreground/background shades from the chosen tint
- preserve silhouette and semantic hierarchy
- all prepared icons change together
- no AI call on tint change

Do not bake a chosen tint into the semantic SVG.

## Clear

- transparent/low-fill visual treatment
- retain recognizability and contrast
- renderer may use current surface/wallpaper contrast information where appropriate
- no separate AI generation
- zero AI calls when switching mode

---

# 12. Global App Icon state

Conceptually:

```text
AppIconSettings
├── enabled: bool
├── mode: light | dark | tinted | clear
├── tintColor
├── selectedAiProvider
├── selectedCredentialRef
├── rendererVersion
└── updatedAt
```

When `enabled = true`, automatic AI generation/reconciliation is implied. A second "auto-generate new apps" user toggle is unnecessary unless later explicitly wanted.

QML displays backend state and invokes explicit backend actions.

QML must not:

- rewrite SVG internals
- perform AI calls
- parse untrusted SVG
- read provider secrets
- shell-interpolate untrusted app identities

Conceptual backend control commands may include:

```text
vesper-control icon status
vesper-control icon on
vesper-control icon off
vesper-control icon mode light
vesper-control icon mode dark
vesper-control icon mode tinted
vesper-control icon mode clear
vesper-control icon tint <validated-color>
vesper-control icon reconcile
vesper-control icon list
vesper-control icon regenerate <app-id>
vesper-control icon reset <app-id>
```

Exact names should fit the existing Rust command structure.

---

# 13. SVG security and validation

Treat curated/imported/AI SVG data as untrusted input to the renderer boundary.

Reject or remove dangerous constructs including:

- scripts
- event handlers
- external network references
- external file references
- embedded executable content
- `foreignObject` or similarly unnecessary active content
- unsafe/pathological filters
- pathological document size or complexity

Prefer a deliberately restricted SVG subset suitable for application icons.

A generated icon must not execute code or fetch resources.

AI output only becomes active after successful sanitization and validation.

On validation failure:

```text
reject generated SVG
keep original app icon
mark generation failed
allow Regenerate
```

---

# 14. Cache and mutable state

Curated icons belong to repository/Nix-managed source.

AI-generated semantic SVGs and queue state belong outside the Nix store under the established Vesper XDG state/cache convention, conceptually:

```text
$XDG_CACHE_HOME/vesper/app-icons/
$XDG_STATE_HOME/vesper/app-icons/
```

Use the correct existing repository convention during implementation.

Cache keys/state should include:

```text
canonical app identity
original icon hash
generator version
semantic format version
renderer version where relevant
```

Changing Light/Dark/Tinted/Clear or tint color must not invalidate the semantic AI-generated SVG.

---

# 15. Settings → Apps relationship

`Settings → Apps` continues to own app-centric controls such as:

```text
installed app inventory
permissions
wellbeing / foreground usage
```

App Icons configuration lives only under AI.

Remove the old:

```text
Experimental
→ AI adaptive icons
```

UI from `VesperAppsSettings.qml` after the AI App Icons page is wired.

Do not maintain independent icon-enabled state in both Apps and AI.

---

# 16. AI Overview

`Settings → AI → Overview` should stay concise and summarize:

```text
provider count
configured API credentials
most constrained provider
active agents
skills
MCP
Hermes unread/attention
App Icons enabled/mode
curated/generated/pending/failed icon counts
```

Detailed data belongs in the dedicated pages.

---

# 17. Providers, API keys, Agents, Skills, MCP and Hermes

The broader AI control-plane architecture from PR #18 remains in force.

## API Keys / Credentials

- Secret Service-backed interactive API keys
- Nix/sops-managed credentials remain declarative/read-only from the interactive side
- logical aliases reusable by multiple consumers
- scoped credential injection
- no secret values in QML/argv/logs/Nix store

## Providers

- one provider registry/adaptor source
- configured credential alias
- models/capabilities when available
- health
- usage/quota model
- consumers
- safe Test action where supported

## Agents

- show live agent state/inventory
- integrate assignments where supported

## Skills

- keep the existing canonical Vesper skill tree
- do not create a duplicate skill architecture
- generated/downloaded executable skills remain reviewable/untrusted

## MCP

- one MCP inventory
- credentials via logical Vesper aliases
- agent assignment
- tool inventory
- allow/ask/deny only where enforcement is real
- no OAuth broker

## Hermes

- retain Hermes status/briefing/job-registry integration
- expose it from the same AI settings control plane

---

# 18. Failure isolation

## App Icons

```text
AI provider unavailable
→ original icons remain active for unresolved apps
→ queue enters waiting/retryable state

quota exhausted
→ do not hammer provider
→ unresolved apps keep original icons

invalid AI SVG
→ reject SVG
→ keep original icon

renderer failure
→ original icon fallback

tint invalid
→ retain previous valid tint

one app generation fails
→ remaining queue continues
```

## Usage & Quotas

```text
one provider status failure
→ only that provider shows error

usage snapshot stale
→ stale is visible in Dashboard and Settings
→ other AI settings continue working
```

None of these failures may prevent graphical login or make the desktop unusable.

---

# 19. Implementation order

## Phase A — document/current-state cleanup

1. Treat this `PLAN.md` as source of truth for PR #18 follow-up.
2. Remove outdated Experimental Adaptive Icons wording as implementation migrates.

## Phase B — shared quota UI

1. Extract/reuse common provider quota components.
2. Keep compact quota information in Dashboard AI.
3. Add Dashboard → detailed AI settings action.
4. Build `Settings → AI → Usage & Quotas` detailed view.
5. Show windows, used/remaining percentages, resets, plan/account/credits/cost/error when available.

## Phase C — production App Icon state

1. Replace experimental boolean with production settings.
2. Add Light/Dark/Tinted/Clear.
3. Add validated tint color.
4. Add AI provider/credential reference for icon-curator.
5. Preserve original fallback.

## Phase D — curated default semantic SVGs

1. Add repository-owned asset location and canonical manifest.
2. Map Vesper/default apps by canonical identity.
3. Keep the files easy to manually replace/refine after real-desktop inspection.

## Phase E — automatic AI generation

1. Reconcile the full app registry when App Icons is enabled.
2. For each app without a curated semantic SVG, enqueue AI generation.
3. Automatically enqueue later-installed apps.
4. Automatically regenerate when the original source icon hash changes.
5. Deduplicate/persist queue state.
6. Use bounded concurrency and retry/backoff.

## Phase F — secure SVG pipeline

1. Feed original app icon + identity to icon-curator.
2. Generate semantic SVG.
3. Sanitize.
4. Validate against restricted SVG profile.
5. Store generated semantic source in mutable Vesper state/cache.
6. Activate only after validation succeeds.

## Phase G — deterministic appearance renderer

1. Implement Light.
2. Implement Dark.
3. Implement Tinted with chosen tint.
4. Implement Clear.
5. Ensure switching any appearance option is AI-free.

## Phase H — move UI into AI

1. Add `Settings → AI → App Icons`.
2. Add global toggle.
3. Add appearance selector.
4. Add tint picker/presets.
5. Show curated/generated/pending/failed/fallback state.
6. Add preview/regenerate/original/reset repair controls.
7. Remove the Apps experimental icon section.

## Phase I — validation

1. Compile Rust control-plane code.
2. Validate QML and Caelestia patches.
3. Validate Home Manager.
4. Validate complete NixOS configuration.
5. Boot Vesper.
6. Turn App Icons on with existing non-curated apps and verify automatic AI generation.
7. Install at least one previously unknown app and verify automatic AI SVG generation without manual interaction.
8. Confirm a missed app is reconciled after restart/session activation.
9. Change tint repeatedly and verify there are zero AI requests.
10. Verify curated defaults beat generated assets.
11. Visually inspect/refine curated SVGs.
12. Verify Dashboard compact quota and Settings detailed quota stay consistent.
13. Verify Dashboard button opens detailed AI settings.
14. Confirm no icon operation leaks credentials or executes unsafe SVG content.

---

# 20. Definition of done

The plan is complete when:

1. App Icons are no longer experimental.
2. App Icons live under `Settings → AI`.
3. The feature has one global On/Off control.
4. Appearance modes are Light/Dark/Tinted/Clear.
5. Tinted has a user-selectable color.
6. Mode/tint changes never invoke AI.
7. Default/known Vesper apps use repository-owned curated semantic SVGs when available.
8. Every non-curated installed app is automatically sent through AI semantic-SVG generation when App Icons is enabled.
9. Every newly installed non-curated app is automatically AI-SVG'd without a manual Generate action.
10. No deterministic pre-conversion path silently bypasses AI for unknown/non-curated apps.
11. AI receives canonical app identity + original icon and returns one semantic SVG source.
12. AI output is sanitized and validated before activation.
13. Original app icons always remain a fallback.
14. Generation is queued, deduplicated, bounded and persistent enough to avoid repeated paid calls.
15. Source-icon changes can trigger regeneration.
16. Curated semantic SVGs remain manually replaceable/refinable in the repository.
17. Dashboard AI continues to show compact quota information.
18. Dashboard has a direct action into detailed AI settings/quota information.
19. `Settings → AI → Usage & Quotas` shows full normalized provider quota data.
20. Dashboard and Settings use one quota schema/source of truth.
21. Provider cards show real plan/account/windows/reset/credits/cost/error fields only where available.
22. Warning/critical semantics remain consistent across surfaces.
23. Existing API-key-only credential architecture remains intact.
24. No OAuth broker/token harvesting is introduced.
25. API keys remain outside QML, argv, logs, Git and the Nix store.
26. Failures remain isolated and cannot prevent graphical login.
27. PR #18 finishes with passing Rust/QML/Home Manager/NixOS validation plus a real-desktop visual pass.

---

# 21. Final target

```text
                     Dashboard / AI
                    compact quotas
                         │
                   [Usage details]
                         │
                         ↓
                     Settings → AI
                          │
        ┌────────────┬────┼───────────────┐
        ↓            ↓    ↓               ↓
 Usage & Quotas  App Icons API Keys    Providers
        │            │    │               │
        │            │    └──────┬────────┘
        │            │           ↓
        │            │     Credential Vault
        │            │           │
        │            ↓           ↓
        │      App Icon AI   AI providers
        │            │
        │    ┌───────┴────────┐
        │    ↓                ↓
        │ curated default   non-curated app
        │    SVG                │
        │                       ↓
        │                 AI icon-curator
        │                       │
        │                       ↓
        │                  semantic SVG
        │                       │
        │             sanitize + validate
        │                       │
        │             deterministic renderer
        │                       │
        │          ┌────────┬───┼───────┐
        │          ↓        ↓   ↓       ↓
        │        Light     Dark Tinted Clear
        │
        └── one normalized quota model
```

Core principle:

```text
Default Vesper apps are curated.
Everything else is AI-SVG'd automatically when App Icons is on.
AI prepares the semantic icon once.
Vesper renders appearance modes many times without AI.
Dashboard shows quota at a glance.
Settings shows quota in full detail.
Secrets stay centralized.
```