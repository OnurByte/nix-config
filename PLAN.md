# Vesper — AI Control Plane + Production App Icon Plan

This file is the implementation plan for the current Vesper control-plane work in PR #18.

It supersedes older notes/prompts where Adaptive Icons were experimental or lived under `Settings → Apps`.

The target is a production Vesper feature controlled from `Settings → AI`, with Apple-style Light/Dark/Tinted/Clear appearance modes, repository-owned curated SVGs for Vesper's default applications, automatic AI SVG generation for every non-curated application, one shared provider usage/quota model used by both the Dashboard AI surface and Settings, and a production `Settings → Apps` control plane with Flatseal-style permission management, per-application notification policy and a global Do Not Disturb mode.

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

9. **`Settings → Apps` becomes a real Flatseal-style application permission center.**
   - The current Network/Home-only Flatpak subset is not enough.
   - For Flatpak applications, expose the full useful set of effective permission/override categories supported by the underlying Flatpak model instead of arbitrarily hardcoding two toggles.
   - The interaction model should feel like Flatseal: select app → inspect effective permissions → change overrides → reset individual/all overrides.
   - Reuse one canonical app identity/registry across permissions, Wellbeing, notifications and AI icons.
   - Do not create fake security switches for native Nix applications when Vesper cannot actually enforce the requested restriction.

10. **Notifications are a first-class per-application permission/policy.**
   - Every resolvable application can expose an `Allow notifications` control in its Apps detail page.
   - Notification blocking must be enforced by Vesper's notification handling layer, not represented as decorative state.
   - Notification policy is separate from Flatpak filesystem/network overrides so native applications can also be controlled at the Vesper notification layer.

11. **Do Not Disturb is a global Vesper notification mode.**
   - DND is not modeled as a per-app permission.
   - It must be available from Settings and from a fast shell/quick-settings action.
   - DND suppresses interruption UI globally while preserving a coherent notification history policy.
   - Per-app notification deny still means that app is blocked independently of DND.

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

# 15. Settings → Apps — Flatseal-style application control plane

`Settings → Apps` remains application-centric, but it must become substantially more capable than the current Network/Home-only permission subset.

Target structure:

```text
Apps
├── Installed applications
├── Global notification / DND status
└── <selected application>
    ├── Overview
    ├── Permissions
    ├── Notifications
    └── Wellbeing
```

App Icons configuration does **not** return here; it remains under `Settings → AI → App Icons`.

## Application list

The Apps page should provide one canonical searchable application inventory with useful identity/state such as:

```text
name
icon
desktop/application id
packaging/sandbox type
permission status
notification status
foreground usage today
```

Selecting an application opens its controls without creating another registry.

## Flatseal parity for Flatpak applications

For Flatpak apps, aim for the same practical permission-management model as Flatseal rather than only exposing Network and Home.

The backend should inspect the effective application permissions plus user overrides and expose the useful categories that the underlying Flatpak permission model can actually control. This includes, where supported by the installed Flatpak/runtime:

```text
network
filesystem access
home/host/XDG directory access
custom filesystem paths
devices
sockets / display integration
IPC
session/system bus access
environment overrides
other effective Flatpak override categories
```

Do not hardcode a fake fixed capability list if Flatpak can expose the effective state dynamically.

The UI should support:

```text
current packaged/default permission
effective permission after overrides
user override state
change override
reset one override/category
reset all overrides
```

A user should be able to open a Flatpak app and manage it with roughly the same mental model as Flatseal without leaving Vesper Settings.

## Enforcement/ownership labels

Be explicit about what Vesper is actually controlling.

Possible states/labels:

```text
Flatpak-enforced
portal-mediated
Vesper notification policy
native / unrestricted
informational
unsupported
```

For native Nix applications, do not show a working-looking filesystem/network toggle unless Vesper has a real sandbox/enforcement mechanism behind it.

Native apps may still have Vesper-enforceable controls such as notification policy and local Wellbeing because those are enforced by Vesper-owned layers.

## Reset behavior

Flatpak permission changes must be reversible.

Provide:

```text
Reset this permission
Reset category
Reset all Flatpak overrides
```

Reset must return the application to its packaged/default Flatpak permissions, not to arbitrary Vesper defaults.

---

# 16. Per-application notification permissions

Notification control is part of each application's Apps detail page.

Minimum required control:

```text
Notifications                     [Allow / Block]
```

This must work for both Flatpak and native applications whenever Vesper can resolve the notification sender to a canonical application identity.

The notification permission is **not** merely a Flatpak override. It is a Vesper notification-server policy so that the desktop can consistently control notifications regardless of packaging type.

Conceptual policy:

```text
AppNotificationPolicy
├── appId
├── allowed: bool
├── popup: bool
├── sound: bool
├── keepInHistory: bool
└── updatedAt
```

Only expose finer-grained `popup`, `sound`, or history controls where the notification stack really supports them. `Allow / Block` is mandatory; unsupported granularity must not be faked.

Expected behavior:

```text
allowed = false
→ notifications from that app are suppressed by the Vesper notification layer
→ no popup/banner
→ no notification sound
→ history behavior follows the explicit block policy

allowed = true
→ notification proceeds through normal Vesper notification/DND policy
```

The Apps list should make blocked notification state visible at a glance where useful.

Notification identity must use the canonical app registry as far as possible. Avoid maintaining a disconnected list keyed only by arbitrary notification text/app-name strings.

If sender identity cannot be resolved safely, treat it as unknown/unmapped rather than applying policy to the wrong application.

---

# 17. Do Not Disturb

Add a global Vesper Do Not Disturb mode.

DND is distinct from per-app notification permission:

```text
Per-app Block
→ that application is not allowed to notify

Do Not Disturb
→ temporarily suppress interruption UI globally
```

## Required entry points

DND should be available from:

```text
Settings → Apps / Notifications
Shell quick settings / notification surface
```

A visible shell indicator should make it obvious when DND is active.

## Default DND semantics

When DND is enabled:

```text
notification popups/banners → suppressed
notification sounds         → suppressed
notification history        → retained by default
```

This allows DND to stop interruptions without silently destroying all incoming notification history.

Per-app `Allow notifications = false` remains a stronger independent policy and should still block that application according to its configured block/history behavior.

## Critical/system exceptions

Do not silently let arbitrary apps bypass DND by claiming importance.

If Vesper later supports critical exceptions, they must be explicit and bounded, for example:

```text
system-critical Vesper alerts
explicit per-app DND bypass allowlist
```

Normal applications do not bypass DND by default.

## State model

Conceptually:

```text
NotificationState
├── doNotDisturb: bool
├── appPolicies[]
├── updatedAt
└── optional temporaryUntil
```

The first implementation only requires reliable manual On/Off. A timed `for 1 hour` / `until tomorrow` control may be added later without changing the core model.

The notification service/state must survive shell UI recreation cleanly and must not lose per-app policy merely because Caelestia restarts.

---

# 18. AI Overview

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

# 19. Providers, API keys, Agents, Skills, MCP and Hermes

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

# 20. Failure isolation

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

## Apps permissions

```text
Flatpak permission inspection fails
→ show permission state unavailable
→ do not invent toggles/state

one Flatpak override fails
→ report that change as failed
→ do not corrupt unrelated overrides

native app has no enforcement backend
→ display native/unrestricted or unsupported
→ no fake security guarantee
```

## Notifications / DND

```text
app identity unresolved
→ do not apply another app's policy
→ treat sender as unknown/unmapped

notification policy backend unavailable
→ surface degraded state
→ shell remains usable

DND UI restarts
→ persisted DND/app policy remains coherent
```

None of these failures may prevent graphical login or make the desktop unusable.

---

# 21. Implementation order

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

## Phase H — move App Icon UI into AI

1. Add `Settings → AI → App Icons`.
2. Add global toggle.
3. Add appearance selector.
4. Add tint picker/presets.
5. Show curated/generated/pending/failed/fallback state.
6. Add preview/regenerate/original/reset repair controls.
7. Remove the Apps experimental icon section.

## Phase I — full Apps permission surface

1. Keep one canonical application registry.
2. Expand Flatpak permission inspection beyond Network/Home.
3. Represent packaged/default, override and effective permission state.
4. Add Flatseal-style permission categories based on what the installed Flatpak backend actually exposes.
5. Support individual/category/all override reset.
6. Label Flatpak-enforced, portal-mediated, native/unrestricted, informational and unsupported states honestly.
7. Remove fake/unenforceable native permission switches.

## Phase J — notifications and Do Not Disturb

1. Add canonical per-app notification policy state.
2. Add `Allow notifications` to app details.
3. Enforce that policy in the Vesper notification handling layer for both resolvable Flatpak and native app identities.
4. Add optional popup/sound/history controls only where actually supported.
5. Add global DND state.
6. Add Settings DND control.
7. Add quick-settings/notification-surface DND toggle and active indicator.
8. Suppress popups and sounds while DND is active while retaining history by default.
9. Keep per-app notification blocking independent from DND.
10. Persist notification/DND policy across shell restarts.

## Phase K — validation

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
14. Compare at least one Flatpak app's effective permissions/overrides against Flatpak tooling and confirm Vesper reports/changes the same underlying state.
15. Test reset-one/reset-category/reset-all Flatpak override behavior.
16. Verify unsupported native restrictions are not presented as enforceable permissions.
17. Block one Flatpak app's notifications and one native app's notifications and verify both are suppressed at the Vesper notification layer.
18. Enable DND and verify popups/sounds stop while notification history remains available.
19. Restart/reload the shell and verify notification policy/DND state remains coherent.
20. Confirm no icon operation leaks credentials or executes unsafe SVG content.

---

# 22. Definition of done

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
23. `Settings → Apps` provides a Flatseal-style permission-management experience for Flatpak applications rather than only Network/Home toggles.
24. Flatpak packaged/default, override and effective permission state are represented accurately enough to match the underlying Flatpak tooling.
25. Flatpak permissions can be changed and reset individually/by category/all where supported.
26. Native applications are not given fake unenforceable filesystem/network security toggles.
27. Every resolvable app can have Vesper notification delivery allowed or blocked independently of packaging type.
28. Per-app notification blocking is enforced by the Vesper notification layer.
29. Global Do Not Disturb can be toggled from Settings and a fast shell surface.
30. DND suppresses notification popups/sounds while retaining history by default.
31. Per-app notification deny remains independent of DND.
32. Notification/DND state survives shell UI restart without becoming inconsistent.
33. Existing API-key-only credential architecture remains intact.
34. No OAuth broker/token harvesting is introduced.
35. API keys remain outside QML, argv, logs, Git and the Nix store.
36. Failures remain isolated and cannot prevent graphical login.
37. PR #18 finishes with passing Rust/QML/Home Manager/NixOS validation plus a real-desktop visual pass.

---

# 23. Final target

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

```text
                     Settings → Apps
                          │
             ┌────────────┼─────────────┐
             ↓            ↓             ↓
          App list     Global DND    Wellbeing
             │
             ↓
        Selected app
             │
     ┌───────┼────────────┐
     ↓       ↓            ↓
 Permissions Notifications Usage
     │       │
     │       └── Allow/Block → Vesper notification policy
     │
     ├── Flatpak → Flatseal-style effective overrides
     └── Native  → only controls Vesper can really enforce

Shell quick settings
        │
        └── Do Not Disturb On/Off
```

Core principle:

```text
Default Vesper apps are curated.
Everything else is AI-SVG'd automatically when App Icons is on.
AI prepares the semantic icon once.
Vesper renders appearance modes many times without AI.
Dashboard shows quota at a glance.
Settings shows quota in full detail.
Apps provides real Flatpak permission control instead of fake toggles.
Notification permission is enforceable per app through Vesper's notification layer.
Do Not Disturb is global and quickly accessible.
Secrets stay centralized.
```