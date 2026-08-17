# Vesper — Production Control Plane Master Plan

This file is the source of truth for the current Vesper production-control-plane work in PR #18.

It supersedes older notes where Adaptive Icons were experimental, Apps only exposed a tiny permission subset, AI Settings were mostly inventory, or control-plane capabilities existed only as backend/CLI features.

The target is a cohesive Caelestia-native workstation control plane covering AI, applications, permissions, notifications, wellbeing, networking, privacy, system health and recovery while preserving Vesper's Nix-first, local-first and inspectable design.

---

# 0. Non-negotiable principles

1. **Do not expose fake controls.** A UI switch is shown as enforceable only when a real backend can enforce it.
2. **One canonical app registry.** Permissions, notifications, wellbeing and AI icons share the same application identity model.
3. **One AI provider/quota model.** Dashboard and Settings use the same normalized backend data.
4. **One credential vault.** API credentials are referenced by logical aliases and injected only into intended consumers.
5. **API-key-only Vesper credential management.** No OAuth broker, refresh-token manager, browser-login broker or CLI token harvesting.
6. **Secrets never enter QML, argv, logs, Git or the Nix store.**
7. **Failure isolation is mandatory.** A broken provider, icon, MCP, proxy, permission backend or health check must not prevent graphical login.
8. **Mutable runtime state stays outside the Nix store.** Declarative packages/defaults remain in Nix; runtime state belongs under appropriate XDG state/cache locations.
9. **Caelestia UI remains thin.** QML renders structured backend state and invokes explicit actions; business logic belongs in Rust/backend modules.
10. **Production features need behavior tests, not only compile/eval tests.**

---

# 1. Target Settings information architecture

```text
Settings
├── Appearance
├── Network
│   ├── Connections
│   ├── DPI / Zapret
│   ├── Proxy
│   └── Privacy
├── Apps
│   ├── Installed Apps
│   ├── Permissions
│   ├── Notifications
│   ├── Do Not Disturb
│   └── Wellbeing
├── AI
│   ├── Overview
│   ├── Usage & Quotas
│   ├── App Icons
│   ├── API Keys / Credentials
│   ├── Providers
│   ├── Agents
│   ├── Skills
│   ├── MCP
│   └── Hermes
├── Privacy
│   ├── Tor
│   ├── DPI / Zapret status
│   ├── Metadata Sanitizer
│   ├── Monero / node status
│   ├── Cuprate / monerod selection
│   └── OnionShare
├── System Health
│   └── vesper-doctor
└── Backup & Recovery
    ├── Restic
    ├── Snapper
    ├── Btrfs scrub
    └── Restore / verification state
```

Do not create duplicate independent registries or settings applications for these surfaces.

---

# 2. Production gap backlog — priority order

The following backlog is ordered by product impact and should remain visible until each item is actually complete.

## P1 — Production AI App Icons

The current queue/review-style adaptive-icon foundation must become a real production system.

Required end state:

- move App Icons out of `Settings → Apps → Experimental`
- place the feature at `Settings → AI → App Icons`
- global On/Off
- appearance modes: `Original`, `Light`, `Dark`, `Tinted`, `Clear`
- user-selectable tint/accent color
- repository-owned canonical SVGs for default Vesper applications
- when App Icons is enabled, every non-curated installed application is automatically sent through AI semantic-SVG generation
- newly installed non-curated applications are automatically AI-SVG'd without a manual Generate action
- generated SVG is sanitized and validated before activation
- generated → active pipeline is automatic after validation
- semantic SVG is cached and reused
- source-icon hash changes can trigger regeneration
- rollback/reset to original icon
- broken/generated-icon fallback to original icon
- mode/tint changes are deterministic and do not call AI

The rule for source preparation is deliberately simple:

```text
curated Vesper SVG exists → curated semantic SVG
otherwise                  → AI generates semantic SVG
```

Do not place a deterministic icon-conversion bypass in front of AI for unknown/non-curated apps.

## P2 — Flatseal-level Apps permission manager

`Settings → Apps` must become a real application permission control plane rather than Network/Home-only Flatpak overrides.

For Flatpak applications, inspect and manage the real effective permission model exposed by Flatpak/runtime tooling, including where supported:

- network
- filesystem access
- home/host/XDG paths
- custom filesystem paths
- camera
- microphone
- location/portal-mediated location state where enforceable
- devices / USB exposure
- Bluetooth where a real backend exists
- printing where a real backend exists
- Wayland/X11/display sockets
- IPC
- session/system D-Bus access
- portals
- background execution where enforceable
- environment overrides
- other actual Flatpak override categories

The UI should distinguish:

```text
packaged/default permission
user override
effective permission
enforcement backend
```

Support reset-one, reset-category and reset-all overrides.

For native Nix apps, do **not** pretend Flatpak overrides exist. If stronger native restrictions are wanted, investigate a separate enforceable model such as bubblewrap/systemd sandboxing and portal-aware controls. Until then label native apps honestly as unrestricted/informational for capabilities Vesper cannot enforce.

## P3 — Full notification management

Per-app notification policy must become first-class and packaging-independent when notification sender identity can be mapped safely to the canonical app registry.

Target per-app model:

```text
Notifications
├── state: allowed | silent | disabled | priority
├── popup/banner
├── sound
├── badge
├── keep in history
└── DND bypass / priority exception
```

Only expose a field if the notification stack can really enforce it. At minimum `Allow/Block` must be real.

Notification policy is enforced by Vesper's notification layer, not represented as decorative state.

## P4 — Wellbeing becomes a real screen-time system

The current foreground-time counter is only the foundation.

Add:

- daily history
- weekly history
- monthly history
- screen-time graphs
- per-app history
- categories
- app exclusion
- per-app daily limits
- category limits where useful
- Focus mode
- daily goals
- idle/locked/suspended filtering
- reset controls
- local export
- explicit behavior for `limit reached`

Possible limit behavior must be technically honest, for example:

```text
notify only
hide/disable launcher entry where safely reversible
request confirmation before launch
stronger blocking only when an enforceable mechanism exists
```

Do not claim hard blocking if Vesper only has advisory UI control.

## P5 — Central AI credentials become a real runtime control plane

The Secret Service-backed key vault is only part of the final architecture.

Move beyond manual-only patterns such as:

```text
vesper-control credential exec <provider> -- <command>
```

Build consumer adapters/wrappers so Vesper-managed tools can receive selected provider credentials automatically without copying plaintext secrets into config files.

Target consumers include where technically compatible:

- OpenCode
- Hermes
- custom Vesper agents/tools
- MCP servers
- icon-curator
- future AI consumers

Codex/Claude Code/Gemini CLI native login systems remain untouched when they rely on their own subscription/auth systems.

The central runtime must support explicit provider/credential selection and scoped child-process injection without harvesting official-client auth files.

## P6 — Correct AI Dashboard vs Settings responsibility

Keep both surfaces, but separate density correctly.

```text
Dashboard / AI
→ compact, glanceable status

Settings → AI
→ detailed control plane
```

Dashboard should prioritize:

- provider count
- most constrained provider
- critical/warning state
- important used/remaining percentage
- nearest/relevant reset
- stale/error state
- button to open detailed AI settings

`Settings → AI → Usage & Quotas` should show the full provider model:

- provider/source
- plan
- account
- health
- all real quota windows
- used percentage
- remaining percentage
- reset time
- credits
- today cost
- 30-day cost
- provider/account errors

Use one shared normalized `@ai@ status` schema and reusable UI components; do not fork two independent implementations.

## P7 — Dynamic AI provider management

Replace the static-provider ceiling with a provider registry/adaptor system supporting where applicable:

- built-in providers
- custom OpenAI-compatible endpoints
- configurable base URL
- model inventory
- default provider
- default model
- per-agent provider/model mapping
- multiple credentials per provider
- credential aliases
- safe key validation/test
- central key rotation
- fallback routing
- provider health
- latency
- quota/usage
- per-provider budget policy
- which agents/consumers may use which credential

Unknown capability must remain unknown rather than fabricated.

## P8 — Skills and MCP become writable control-plane surfaces

The AI page must move beyond read-only inventories.

### Skills

Add where consistent with the canonical skill architecture:

- enable/disable
- source
- managed-by state
- update status
- dependencies
- validation/security state
- generated skill drafts
- review
- promotion
- remove where ownership permits

Nix-managed skills must not be silently mutated as if they were runtime-owned.

### MCP

Add:

- installed server inventory
- enable/disable
- start/stop where Vesper owns the process
- health
- logs
- source/version
- tool inventory
- credential alias
- per-agent enablement
- permissions
- test/reconnect
- install/remove/configure where ownership permits

Use allow/ask/deny only where enforcement is real.

## P9 — Hermes gets a real Settings control plane

Expose Hermes as more than registry-present/unread counters.

Target UI:

- cron/job list
- enabled/disabled state
- run now
- last run
- next run
- last result
- last error
- briefing history
- source registry
- job detail
- learned heuristic review
- skill-draft review/promotion flow

Keep Hermes backend behavior separate from UI rendering; Settings should consume structured state/actions.

## P10 — Network DPI / Zapret tuning

Turn the current DPI status surface into an actual control/diagnostic page where the underlying Zapret configuration supports runtime/declarative changes.

Potential controls:

- active profile/preset
- host list
- auto-host learning state
- TCP/UDP scope
- target ports
- desync technique
- fake/fakedsplit parameters
- packet count/range
- test domain
- diagnostics
- last test/result

Do not expose knobs that the installed Zapret version/config does not support.

Declarative Nix ownership must remain visible; mutable runtime tuning must not silently rewrite Nix-owned configuration.

## P11 — Production proxy management

Expand the current global environment proxy mechanism.

Target capabilities:

- separate HTTP proxy
- separate HTTPS proxy
- SOCKS proxy
- no-proxy/bypass list
- authenticated proxy credentials stored through secret storage
- PAC support where practical
- per-app proxy strategy where enforceable
- Flatpak propagation where applicable
- Tor shortcut/profile
- connection test
- external IP test
- effective-proxy status
- explicit indication when session restart/relaunch is needed

Do not pretend already-running processes were updated if the mechanism only affects newly launched processes.

## P12 — Airplane mode gets a real state model

Do not infer airplane mode merely from both Wi-Fi and Bluetooth currently being off.

Maintain explicit airplane state and/or previous radio state.

Desired behavior:

```text
Wi-Fi off manually + Bluetooth off manually
≠ automatically claim Airplane Mode is active

Airplane Mode on
→ remember previous radio states
→ disable radios

Airplane Mode off
→ restore intended previous states where safe
```

Add behavior tests for these transitions.

## P13 — Network privacy layer

Add a Vesper-oriented privacy section for networking.

Potential capabilities:

- DNS mode/provider
- custom DNS
- DNSSEC status/control where supported
- DoT selection/status where supported
- MAC randomization policy
- per-network privacy profile
- VPN status
- kill switch where a real VPN backend supports enforcement
- DNS/IP leak test
- Tor routing status
- proxy/Tor interaction visibility

Avoid fake leak-protection claims when no enforcement path exists.

## P14 — Unified Privacy Settings

Create `Settings → Privacy` as a common status/control surface for existing Vesper privacy tools rather than forcing the user to inspect each application separately.

Surface where available:

- Tor service/status/ports
- Zapret/DPI status
- metadata sanitizer
- Monero node/wallet tooling status
- Cuprate status
- monerod status
- Cuprate/monerod selected/default node backend
- OnionShare status
- relevant health/errors

This page is a control/status aggregator; do not unnecessarily reimplement the underlying tools.

## P15 — System Health powered by `vesper-doctor`

Add `Settings → System Health` using structured `vesper-doctor --json` output as the primary backend where possible.

Candidate cards/checks:

- failed services
- Tor
- backup health
- Btrfs scrub
- GPU / PRIME
- display refresh
- local web stack
- other checks already emitted by vesper-doctor

Provide:

- overall state
- per-check status
- useful detail
- refresh
- copy/export diagnostic summary where safe

Do not duplicate health-check logic in QML.

## P16 — Backup & Recovery in Settings

Expose existing backup/recovery infrastructure instead of leaving it CLI-only.

Target status/actions where supported:

- last Restic backup
- last backup result
- last repository check
- snapshot count
- latest Snapper snapshots
- Btrfs scrub last/next/result
- disk/repository usage
- restore readiness
- backup test/verification state
- run safe check actions

High-risk restore/destructive actions require explicit confirmation and should not be casually surfaced as one-click operations.

## P17 — Harden Caelestia integration

Reduce fragility of large upstream patches and numeric navigation.

Required direction:

- split major feature patches by domain where practical: AI / Apps / Network / Appearance / Notifications / Health
- minimize patch overlap
- avoid hardcoded `openSubPage(<number>)` style routing where upstream changes can shift page indices
- introduce symbolic route/page IDs or another stable navigation contract
- smoke-test patch application against the pinned Caelestia source
- fail clearly when an upstream patch no longer applies

Do not allow an upstream subpage insertion to silently route Vesper buttons to the wrong page.

## P18 — Production Rust control-plane architecture

Move first-party control-plane code away from indefinitely growing standalone `rustc` single-file binaries.

Target a Cargo workspace or equivalent maintainable Rust architecture, conceptually:

```text
vesper-core/
├── Cargo.toml
├── crates/ or src/
│   ├── credentials
│   ├── apps
│   ├── permissions
│   ├── notifications
│   ├── wellbeing
│   ├── network
│   ├── proxy
│   ├── icons
│   ├── ai
│   ├── privacy
│   ├── health
│   └── recovery
```

Share common:

- JSON models
- command execution helpers
- error types
- app identity
- persistence helpers
- secret-safe process spawning

Use appropriate maintained Rust crates such as serde/zbus/etc. when they solve a real problem; do not add dependencies only for architectural aesthetics.

Keep the frontend/backend contract structured and versionable.

## P19 — Behavior-focused CI and tests

Compile/eval/build checks remain necessary but are insufficient for a production Settings control plane.

Add tests with mocked/fake backends for at least:

- `nmcli`
- Flatpak permission inspection and overrides
- `secret-tool` / credential flows
- wellbeing accounting
- idle/locked filtering
- notification allow/block
- DND semantics
- proxy state
- airplane-mode transitions
- AI quota normalization
- adaptive/app-icon queue
- icon cache invalidation
- generated SVG sanitizer/validator
- provider failure isolation

Also add tests/smoke coverage for:

- QML navigation routes
- Dashboard → AI Settings deep-link
- Settings subpage routing
- Caelestia patch application
- important QML interaction/state bindings where feasible

---

# 3. AI App Icons detailed design

## 3.1 User-facing location

```text
Settings → AI → App Icons
```

The Apps page must not retain an `Experimental → Adaptive Icons` control after migration.

## 3.2 Global controls

```text
App Icons                         [On/Off]

Appearance
○ Original
○ Light
○ Dark
○ Tinted
○ Clear

Tint color                        [picker]
                                  [presets/current accent]

Provider                           [selected AI provider]
Credential                         [logical credential alias]
```

`Original` disables Vesper appearance replacement without deleting prepared assets.

## 3.3 Automatic generation/reconciliation

When App Icons is enabled:

```text
canonical app registry
↓
semantic icon inventory
↓
for each app
    curated SVG?
    ├── yes → validate/use curated
    └── no  → enqueue AI generation
```

This applies to:

- already-installed apps when the feature is first enabled
- newly installed apps
- apps missed while Vesper/session services were unavailable
- apps whose original source icon changes after update

No per-app Generate click is required for normal operation.

## 3.4 Queue states

```text
pending
processing
prepared
failed
waiting-for-provider
waiting-for-quota
fallback
```

Queue rules:

- deduplicate by canonical app identity + source icon hash
- persist enough state to avoid repeated paid calls after reboot
- bounded concurrency
- bounded retry/backoff
- quota/provider failure must not hammer APIs
- failure of one app does not block other jobs
- manual Regenerate remains a repair tool

## 3.5 AI icon-curator contract

Input:

```text
canonical app ID
display name
original application icon
semantic format/version request
```

Output:

```text
semantic SVG
```

AI preserves recognizable product identity and important geometry while producing a representation suitable for Vesper's deterministic renderer.

AI does not choose the current tint/theme mode.

## 3.6 Curated default assets

Create one repository-owned asset source, conceptually:

```text
home/yargc/assets/app-icons/
├── manifest.nix / equivalent canonical mapping
├── firefox.svg
├── terminal.svg
├── vesper-ai.svg
└── ...
```

The files should be easy to replace/refine manually after Vesper is booted and visually inspected.

Curated assets override generated ones by default.

## 3.7 Semantic icon model

```text
AppIconAsset
├── appId
├── desktopId
├── sourceType: curated | ai-generated
├── sourceIconHash
├── semanticFormatVersion
├── generatorVersion
├── generatedAt
├── validationStatus
└── fallbackOriginalIcon
```

Mutable generated assets stay outside the Nix store.

## 3.8 Appearance rendering

The semantic source is prepared once; Vesper renders it many times.

### Light
- optimized for light surfaces
- no AI call

### Dark
- optimized for dark surfaces
- no AI call

### Tinted
- Apple-style single-accent treatment
- user-selected tint/accent
- deterministic derived foreground/background shades
- no AI call when tint changes

### Clear
- transparent/low-fill appearance
- preserve recognizability/contrast
- no separate AI generation

### Original
- use original application icon
- retain prepared semantic asset for later re-enable

## 3.9 SVG security

Treat all SVG as untrusted at the renderer boundary.

Reject/remove:

- scripts
- event handlers
- external network references
- external file references
- executable/active content
- unnecessary `foreignObject`
- pathological filters
- pathological size/complexity

Prefer a restricted icon-safe SVG subset.

Generated assets only become active after sanitize + validation succeeds.

---

# 4. Apps, permissions, notifications and DND

## 4.1 App detail structure

```text
Apps
└── Selected App
    ├── Overview
    ├── Permissions
    ├── Notifications
    └── Wellbeing
```

Overview should show:

- app name/icon
- canonical app/desktop ID
- packaging/sandbox type
- permission summary
- notification state
- foreground time today

## 4.2 Permission enforcement labels

Use explicit backend labels such as:

```text
Flatpak-enforced
portal-mediated
Vesper-enforced
sandbox-managed
native/unrestricted
informational
unsupported
```

Never render an unsupported native restriction as if it were enabled/disabled security enforcement.

## 4.3 Notification policy model

Conceptually:

```text
AppNotificationPolicy
├── appId
├── mode: allowed | silent | disabled | priority
├── popup
├── sound
├── badge
├── keepInHistory
├── dndBypass
└── updatedAt
```

Unknown/unmapped notification sender identity must remain unmapped rather than receiving another app's policy accidentally.

## 4.4 Global Do Not Disturb

DND is global and distinct from per-app deny.

Required entry points:

- Settings
- shell quick settings / notification surface
- visible active indicator

Default behavior:

```text
popups/banners → suppressed
sounds          → suppressed
history         → retained
```

Normal apps do not bypass DND by default. Priority/system exceptions must be explicit.

Support manual On/Off first; timed variants such as `1 hour` or `until tomorrow` may build on an `optional temporaryUntil` field later.

---

# 5. Wellbeing detailed design

Use foreground application activity, not merely process existence.

Pause accounting while:

- idle
- locked
- suspended

Do not collect:

- keystrokes
- clipboard contents
- browser URLs
- document contents
- prompt text
- window titles by default

Target data/features:

```text
Today
Week
Month
Per-app history
Categories
Limits
Focus
Goals
Exclusions
Export
Reset
```

Keep all Wellbeing data local by default.

Agent access, if retained, is read-only structured summary access rather than unrestricted raw history mutation.

---

# 6. AI control-plane detailed design

## 6.1 Usage & Quotas

One source of truth:

```text
@ai@ status
     ├── Dashboard compact
     └── Settings detailed
```

Health semantics remain consistent, including existing warning/critical thresholds unless intentionally revised in one shared backend.

## 6.2 Credential vault

Supported credential ownership/state:

```text
Managed by Vesper
Managed by Nix/sops
External environment
Missing
```

Interactive keys use a proper secret store. Nix-managed keys are detected but not silently rewritten.

One credential alias may serve multiple consumers.

Rotation updates the secret behind an alias rather than rewriting every consumer config.

## 6.3 Scoped runtime injection

Consumers receive credentials only for their process scope.

Conceptually:

```text
Credential Vault
      ↓
consumer adapter / launcher
      ↓
child environment only
      ↓
OpenCode / Hermes / MCP / icon-curator / tool
```

No plaintext temporary credential files should remain after process exit.

## 6.4 Provider registry

Conceptually:

```text
AiProvider
├── id
├── name
├── baseUrl
├── apiType
├── credentialRefs[]
├── models[]
├── health
├── latency
├── usage
├── budget
└── consumers[]
```

Provider adapters declare only capabilities they actually support.

## 6.5 Agents

Settings should support current agent inventory/state and, where backend support exists:

- assigned provider/model
- credential access policy
- MCP assignment
- skill assignment

Do not invent control over official-client subscription auth that Vesper does not own.

---

# 7. Skills, MCP and Hermes detailed design

## Skills

Keep one canonical skill architecture. Show:

- name
- description
- source
- managed-by
- compatible agents
- enabled agents
- dependencies
- files
- security/review status

Downloaded/generated executable skills are untrusted until reviewed.

Agent-generated skill drafts require review before promotion.

## MCP

Conceptual model:

```text
McpServer
├── id
├── name
├── source
├── version
├── transport
├── command/url
├── credentialRef
├── tools
├── assignedAgents
├── permissions
├── health
├── logs/status
└── managedBy
```

Differentiate global Vesper, project, agent-specific and external/manual MCP scopes.

Do not overwrite project-owned configuration silently.

## Hermes

Treat Hermes jobs/briefings as first-class AI control-plane entities and expose structured job state instead of making Settings parse arbitrary log text.

---

# 8. Network, proxy and privacy design

## 8.1 DPI / Zapret

Clearly distinguish:

- Nix/declarative profile
- runtime status
- supported mutable controls

Provide diagnostics before adding dozens of knobs blindly.

## 8.2 Proxy

Model effective proxy state explicitly:

```text
ProxyState
├── http
├── https
├── socks
├── noProxy[]
├── pac
├── credentialRef
├── scope
├── requiresRelaunch
└── lastTest
```

Proxy authentication secrets use the secret store.

## 8.3 Airplane mode

Use explicit state rather than deriving it from radios coincidentally being off.

## 8.4 Network privacy

Expose actual backend capability and status for DNS/MAC/VPN/Tor-related protections.

---

# 9. Unified Privacy Settings

`Settings → Privacy` aggregates existing Vesper privacy infrastructure without replacing it.

A concise overview can show:

```text
Tor            active / inactive / error
DPI/Zapret     active / profile / error
Metadata       sanitizer available/status
Monero node    backend/status/height if available
OnionShare     available/status
Network        DNS/privacy profile summary
```

Deep links may open dedicated subsections or the underlying application where appropriate.

---

# 10. System Health and Recovery

## 10.1 System Health

Prefer `vesper-doctor --json` or a shared Rust health model as the backend.

QML must not reimplement checks.

## 10.2 Backup & Recovery

Expose status first. Mutating/destructive recovery actions require stronger confirmation and clear ownership.

Show stale timestamps so an old successful backup is not mistaken for current protection.

---

# 11. Caelestia integration architecture

Replace fragile numeric navigation contracts with stable semantic routes.

Conceptually:

```text
settings.open("ai/usage")
settings.open("ai/icons")
settings.open("apps/notifications")
settings.open("privacy")
settings.open("system-health")
```

Exact IPC syntax may differ, but routes should be symbolic and testable.

Keep upstream patch scope small enough to diagnose failures.

---

# 12. Rust control-plane architecture

Move toward a Cargo-managed shared backend rather than an indefinitely growing `vesper-control.rs` plus separate single-file utilities.

A reasonable target:

```text
vesper-core
├── credentials
├── apps
├── permissions
├── notifications
├── wellbeing
├── icons
├── ai
├── skills
├── mcp
├── hermes
├── network
├── proxy
├── privacy
├── health
└── recovery
```

Potential binaries can remain thin frontends over shared libraries:

```text
vesper-control
vesper-ai
vesper-doctor
```

Do not force a daemon where a command is enough. Long-running services are justified only for stateful/event-driven features such as Wellbeing or notification policy when required.

---

# 13. Failure isolation

Examples:

```text
AI provider unavailable
→ unresolved icons keep originals
→ other desktop features work

quota exhausted
→ icon queue waits/backoffs
→ provider is marked constrained

invalid AI SVG
→ reject generated asset
→ original icon remains active

Flatpak permission inspection fails
→ show unavailable
→ do not invent state

native app has no sandbox enforcement
→ label unsupported/unrestricted
→ do not show fake toggle

notification identity unresolved
→ do not apply another app's policy

notification backend unavailable
→ surface degraded state
→ shell remains usable

proxy test fails
→ show failed test
→ do not silently rewrite unrelated settings

vesper-doctor check fails
→ one health card fails
→ settings still loads

backup status unavailable
→ show unknown/stale
→ do not imply backup safety
```

None of these failures may block graphical login.

---

# 14. Implementation phases

The priority order above should guide scheduling. Keep changes staged and buildable.

## Phase A — architecture cleanup and contracts

1. Treat `PLAN.md` as source of truth.
2. Define canonical app identity model.
3. Define stable symbolic Settings routes.
4. Define shared structured backend schemas.
5. Begin splitting first-party Rust control-plane code into Cargo-managed modules without breaking current commands.

## Phase B — AI quota/UI consolidation

1. Extract shared provider quota components.
2. Keep Dashboard compact.
3. Add detailed `Settings → AI → Usage & Quotas`.
4. Add Dashboard deep-link/button into AI Settings.
5. Preserve one normalized quota schema.

## Phase C — production App Icons

1. Remove experimental positioning.
2. Add `Original/Light/Dark/Tinted/Clear`.
3. Add tint picker.
4. Add canonical curated SVG manifest/assets.
5. Add automatic full-registry reconciliation.
6. Auto-queue non-curated apps for AI semantic-SVG generation.
7. Add sanitize/validate/cache/activate pipeline.
8. Add fallback/rollback/regenerate.
9. Verify new-app automatic generation.

## Phase D — Apps permission expansion

1. Expand Flatpak inspection beyond Network/Home.
2. Show packaged/override/effective state.
3. Add granular categories based on actual backend support.
4. Add reset-one/category/all.
5. Add honest native-app enforcement labels.
6. Research/implement native sandbox controls only where a real backend can enforce them.

## Phase E — Notifications and DND

1. Add canonical notification identity mapping.
2. Add allowed/silent/disabled/priority model where enforceable.
3. Add popup/sound/badge/history controls where supported.
4. Add DND exception policy.
5. Add global DND Settings control.
6. Add quick-settings toggle/indicator.
7. Persist state across shell restarts.

## Phase F — Wellbeing expansion

1. Persist daily/weekly/monthly history.
2. Add graphs.
3. Add per-app/category data.
4. Add exclusions.
5. Add limits/goals/focus mode.
6. Add reset/export.
7. Implement only enforceable limit actions honestly.

## Phase G — AI runtime/provider management

1. Generalize provider registry.
2. Add custom OpenAI-compatible endpoints.
3. Add model inventory/defaults.
4. Add multiple credentials/rotation/test.
5. Add agent/provider/model/credential mapping.
6. Add scoped automatic consumer adapters.
7. Add health/latency/budget/fallback policy.

## Phase H — Skills/MCP/Hermes control planes

1. Add writable ownership-aware Skills controls.
2. Add MCP lifecycle/health/logs/tools/permissions/agent mapping.
3. Add Hermes jobs/run-now/history/error/next-run UI.
4. Add learned heuristic and skill-draft review flows.

## Phase I — Network production controls

1. DPI/Zapret diagnostics and supported tuning.
2. Proxy expansion and tests.
3. Correct airplane state model.
4. Network privacy controls/status.

## Phase J — Privacy, System Health and Backup UI

1. Add unified Privacy page.
2. Add `vesper-doctor`-backed System Health.
3. Add Backup & Recovery status/control surface.
4. Reuse existing infrastructure rather than duplicating services.

## Phase K — Caelestia hardening

1. Split oversized patches by feature where practical.
2. Remove numeric subpage assumptions.
3. Add symbolic routes.
4. Add patch/application/navigation smoke tests.

## Phase L — behavior-test expansion

Add mocked/integration tests for permissions, notifications, DND, wellbeing, proxy, airplane, AI quotas, icon generation/cache, credentials and navigation.

## Phase M — complete validation

1. Cargo/Rust tests and compilation.
2. QML/patch checks.
3. Nix parse/eval.
4. Home Manager eval/build.
5. full NixOS build.
6. real desktop boot.
7. real interaction tests for each production surface.

---

# 15. Required validation scenarios

At minimum test these end-to-end cases:

1. Enable App Icons with several existing non-curated apps; AI generates semantic SVGs automatically.
2. Install a previously unknown app; no manual Generate action is needed.
3. Change tint repeatedly; zero AI generation requests occur.
4. Corrupt/reject a generated SVG; original icon remains usable.
5. Change source icon hash; only affected app regenerates.
6. Compare Flatpak permission state with underlying Flatpak tooling.
7. Toggle/reset granular Flatpak overrides and verify actual effective state.
8. Confirm unsupported native restrictions are not shown as enforceable.
9. Block notifications for a Flatpak app and a native app; both are suppressed when identity mapping succeeds.
10. Enable DND; popup/sound stop while history remains according to policy.
11. Restart Caelestia; DND/per-app notification state remains coherent.
12. Accumulate Wellbeing history across idle/lock transitions; idle/locked periods are excluded.
13. Rotate a credential alias; mapped Vesper-managed consumers continue working without config copies.
14. Fail one provider; other provider cards/consumers remain usable.
15. Dashboard quota and Settings quota report the same normalized underlying state.
16. Dashboard details button opens the correct symbolic AI Usage route.
17. Toggle airplane mode from mixed radio states and verify intended state restoration.
18. Configure proxy and verify effective state plus external connectivity test.
19. Break one vesper-doctor check; System Health renders the failure without breaking Settings.
20. Verify backup timestamps/status accurately distinguish healthy, failed and stale states.

---

# 16. Definition of done

Vesper's production control-plane work is complete only when all of the following are true:

1. App Icons are production and live under AI.
2. `Original/Light/Dark/Tinted/Clear` exist as global appearance modes.
3. Default Vesper apps can use repository-owned curated SVGs.
4. Every non-curated app is automatically AI-SVG'd when App Icons is enabled.
5. New apps are automatically processed.
6. AI icon output is sanitized/validated before activation.
7. Cache, regeneration, rollback and original-icon fallback work.
8. Theme/tint changes never invoke AI.
9. Apps provides practical Flatseal-level Flatpak permission management rather than only Network/Home.
10. Native apps never receive fake unenforceable permission switches.
11. Per-app notifications can be allowed/blocked and richer states are exposed only where enforceable.
12. Global DND works from Settings and a fast shell surface.
13. Wellbeing has usable history/graphs and local limits/focus controls with honest enforcement semantics.
14. Dashboard retains compact AI quota status.
15. Settings contains the detailed Usage & Quotas control plane.
16. Both use one quota backend/schema.
17. The credential vault is usable through Vesper-managed runtime adapters, not only manual CLI injection.
18. Providers support extensible registry/adapters, custom compatible endpoints, multiple keys and mapping where supported.
19. Skills, MCP and Hermes have ownership-aware interactive control surfaces.
20. DPI/Zapret, proxy and airplane mode are real control planes rather than misleading status-only abstractions.
21. Network privacy controls expose real backend capability/state.
22. A unified Privacy page exists.
23. `vesper-doctor` powers a System Health UI.
24. Backup/recovery status is visible in Settings.
25. Caelestia navigation uses stable symbolic routing instead of fragile numeric page indexes.
26. Large Vesper Caelestia changes are split/tested enough that upstream drift fails clearly.
27. First-party Rust control-plane code has a maintainable Cargo/module architecture.
28. Behavior tests cover critical state-changing features in addition to Nix/build tests.
29. Secrets remain outside QML, argv, logs, Git and the Nix store.
30. No OAuth broker/token harvesting is introduced.
31. No control-plane subsystem failure can prevent graphical login.
32. Final Rust/QML/Home Manager/NixOS builds and real-desktop interaction checks pass.

---

# 17. Final architecture

```text
                         Vesper Settings
                               │
        ┌──────────────┬───────┼───────────┬──────────────┐
        ↓              ↓       ↓           ↓              ↓
       AI             Apps   Network     Privacy      System Health
        │              │       │           │              │
        │              │       │           │          vesper-doctor
        │              │       │           │
        │              │       │           └── Tor / Monero / OnionShare / metadata
        │              │       │
        │              │       ├── DPI / Zapret
        │              │       ├── Proxy
        │              │       ├── Airplane
        │              │       └── Network Privacy
        │              │
        │              ├── Permissions
        │              ├── Notifications
        │              ├── DND
        │              └── Wellbeing
        │
        ├── Usage & Quotas ← same normalized model → Dashboard compact AI
        ├── App Icons
        │      ├── curated defaults
        │      └── non-curated apps → AI icon-curator → safe semantic SVG
        │                                     ↓
        │                        Original/Light/Dark/Tinted/Clear renderer
        ├── Credential Vault
        │      ↓ scoped injection
        │   Vesper-managed consumers
        ├── Providers
        ├── Agents
        ├── Skills
        ├── MCP
        └── Hermes

                         Backup & Recovery
                         Restic / Snapper / Btrfs
```

Core direction:

```text
Vesper should expose one coherent workstation control plane.
If Vesper can enforce a setting, Settings should make it usable.
If Vesper cannot enforce it, Settings should say so instead of faking control.
Default app icons are curated; non-curated app icons are AI-generated automatically.
AI prepares semantic icons once; deterministic rendering handles appearance changes.
Dashboard is glanceable; Settings is detailed.
Credentials are centralized and scoped.
Privacy, health and recovery reuse existing backend infrastructure.
Production means behavior tests, stable routes and failure isolation—not only passing builds.
```