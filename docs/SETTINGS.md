# Vesper Settings

Status: **spec**

This document is the single source of truth for the Vesper Settings information architecture, cross-page UX rules and the runtime-to-declarative control model.

It does not replace subsystem documents. It defines where a capability appears in Settings and how settings surfaces interact. Operational details remain with their owning subsystem documents.

Current code must be inspected before claiming any target surface below is implemented.

## current implementation

Vesper extends Caelestia Nexus instead of shipping a second general settings application.

Current Vesper-specific Settings integration includes:

- `Appearance` integration over the upstream wallpaper/style surface
- extended `Network` controls
- `AI` settings and credentials
- installed-app controls for permissions, wellbeing and adaptive icons
- Vesper Store handoff through `Settings -> Apps`

Caelestia still owns the ordinary desktop settings surfaces such as network, Bluetooth/devices, audio, updates, panels, apps, services, language/region and about.

Vesper already has system capabilities outside Settings that should become first-class control-plane surfaces over time, including declarative NixOS generations, Btrfs/Snapper recovery, Restic backup, `vesper-doctor`, Tor/Zapret2, power profiles, hybrid AMD/NVIDIA state and Hermes automation.

## product direction

Vesper Settings is not a KDE System Settings clone.

Classic desktop controls still belong here when needed, but Vesper's distinguishing control-plane surfaces are:

```text
NixOS state
recovery
privacy
AI agents
automation
system health
```

The goal is to expose capabilities Vesper already owns rather than create pages only to increase the number of settings.

Use native Caelestia/Nexus UI and Vesper Rust backends where Vesper-specific system logic is required.
Do not spawn GTK control applications as the primary Settings experience when the capability can be represented natively in QML.

## ownership map

Use these boundaries consistently:

| surface | Settings owns | canonical subsystem document |
|---|---|---|
| System / NixOS | generation/config UX, declarative-change flow | this document + current Nix code |
| Display | output/profile UX | this document + current Hyprland config |
| Power & Performance | power/GPU UX | this document + current system modules |
| Recovery | unified recovery UX | `BACKUP.md`, `INSTALL.md` |
| System Health | doctor result presentation/remediation routing | this document + `vesper-doctor` implementation |
| Network & Privacy | navigation and combined privacy/network UX | `NETWORK-SETTINGS.md` + current privacy modules |
| Input | keyboard/pointer/gesture UX | this document + Hyprland config |
| Shortcuts | keybind editor/search/conflicts | this document + Hyprland config |
| AI | AI control-plane UX | `AI.md`, `AI-ANALYTICS.md` |
| Automations | Hermes job management UX | `HERMES.md` |
| Apps | installed-app inspector and wellbeing UX | `APPS-SETTINGS.md` |
| Appearance | desktop visual controls | this document; adaptive icons remain `ADAPTIVE-ICONS.md` |
| Services & Startup | curated Vesper service UX | this document + owning Nix modules |
| Store | discovery/install application | `MARKETPLACE.md` |

Do not duplicate subsystem implementation rules in this file. Link to the canonical owner instead.

## target information architecture

The target sidebar should be hierarchical rather than a flat collection of pages.
Exact grouping can follow Caelestia's navigation primitives, but semantic ownership should stay equivalent to:

```text
System
├── System / NixOS
├── Display
├── Power & Performance
├── Storage & Recovery
└── System Health

Connectivity & privacy
├── Network & Privacy
└── Bluetooth / Devices

Personalization
├── Appearance
├── Input
└── Shortcuts

Applications
├── Apps
├── Wellbeing
└── Services & Startup

AI
├── Overview
├── Providers
├── Usage & Analytics
├── Agents
├── Agent Teams
├── Skills
├── MCP
└── Automations

System integration
├── Updates
├── Language & region
└── About
```

Do not create duplicate pages solely because upstream Caelestia and Vesper use different names for the same system concern.
Prefer extending or regrouping the native page.

## System / NixOS

This is the highest-priority Vesper-specific Settings surface.

Target read-only state includes, when reliably available:

- running generation
- booted generation
- last known successful generation
- current `nix-config` revision
- dirty/clean repository state
- pending configuration changes
- `flake.lock` age/change state
- whether a reboot is required for the intended state transition

Target actions include:

- test current configuration
- switch to tested configuration
- list generations
- inspect a generation diff
- roll back to a selected generation
- clean old generations through the existing Vesper/Nix ownership path

Do not turn arbitrary shell strings into QML actions. The backend owns allowed operations and structured results.

### runtime to declarative bridge

Vesper should distinguish runtime changes from declarative ownership.

For settings that support both modes, the UX may offer:

```text
Temporary
Persist in nix-config
```

`Temporary` changes runtime state only.

`Persist in nix-config` must use a guarded transaction:

```text
requested setting
    ↓
structured Nix/config edit
    ↓
show diff
    ↓
nh os test
    ↓ success
explicit apply
    ↓
nh os switch
```

Rules:

- never silently edit arbitrary Nix text from QML
- generate a bounded structured change through a Vesper backend
- show the exact diff before testing
- a failed test must not switch the system
- do not auto-commit unless a separate explicit workflow is designed
- preserve rollback information
- clearly label settings that are runtime-only or declarative-only

This bridge is a Vesper control-plane feature, not a requirement that every upstream Caelestia toggle become a Nix editor.

## Display

Target Display support should fill the current desktop-control gap natively in Nexus.

Target controls:

- output enable/disable
- resolution
- refresh rate
- scale
- orientation
- primary display semantics where useful
- VRR when supported
- mirror/extend
- workspace to monitor assignment
- saved monitor profiles

The UI should derive available modes from the compositor/system rather than hard-code the laptop panel.
The known Vesper panel state may be used by diagnostics, but the page must remain truthful when an external monitor is connected.

`nwg-displays` is a useful behavior/reference point for Hyprland multi-display workflows such as saved output configuration and workspace assignment. Vesper should implement the intended surface natively rather than launch `nwg-displays` as the normal UI.

Persistent monitor configuration should eventually use the runtime-to-declarative bridge when the underlying Vesper config supports a structured edit.

## Power & Performance

Target controls and status include:

- Power Saver / Balanced / Performance profiles
- battery percentage and health
- cycle count when exposed
- charge threshold/conservation mode only when hardware support exists
- display-off and suspend behavior
- lid-close behavior
- idle timeout
- AC vs battery policy where the underlying system can represent it reliably

Hybrid-GPU status belongs here:

- AMD iGPU activity
- NVIDIA active/asleep state
- PRIME/offload state
- applications currently using the dGPU when attributable

Do not invent charge-threshold or GPU controls when the hardware/backend cannot enforce them.

## Storage & Recovery

Settings should present Nix generations, Snapper and Restic as one **Recovery Center** without collapsing their different semantics.

Target overview:

- filesystem/disk usage
- Nix store size
- local snapshot status/timeline
- Btrfs scrub state
- last backup
- next scheduled backup
- backup target availability
- retention summary

Target actions, where supported by the owning backend:

- create local snapshot
- inspect snapshots
- restore selected files
- run backup now
- verify backup repository
- inspect Nix generations
- roll back a generation

`BACKUP.md` remains authoritative for Restic/Snapper operational behavior and credentials.
`INSTALL.md` remains authoritative for the verified storage topology.

Never describe a Snapper snapshot as an off-machine backup.
Never expose Restic secrets in Settings.

## System Health

`vesper-doctor --json` should be the primary structured substrate for a native System Health page rather than reimplementing the same checks in QML.

Target summary:

```text
Healthy
2 warnings
```

A check may expose:

- title
- state
- concise evidence
- explanation
- related Settings location
- safe remediation action when one exists

The page may cover existing doctor domains such as storage/Btrfs, scrub, power state, NVIDIA/PRIME, displays, Tor, local web stack, Restic/timers, failed units and suspend capability as the current doctor implementation exposes them.

Target actions may include:

```text
Fix
Explain
Open related setting
Fix with AI
```

`Fix` must be a known bounded backend action, not arbitrary model-generated shell execution.

`Fix with AI` is an escalation path into the Vesper AI control plane. It must show the diagnosis and proposed mutation before privileged/declarative changes are applied.

## Network & Privacy

Do not create separate top-level pages for VPN, Tor, Zapret and DNS.
Use one coherent `Network & Privacy` architecture with sub-sections where needed.

Current network behavior remains owned by `NETWORK-SETTINGS.md`.

Target privacy/network additions may include:

- Tor service status
- new circuit action where technically meaningful
- SOCKS endpoint visibility
- explicit per-app Tor launch profiles when a real enforcement path exists
- DNS mode
- DNS leak-test shortcut/result surface
- MAC-randomization policy
- microphone/camera activity
- clipboard-history retention controls
- screenshot/screen-share indicators
- firewall status
- a composable privacy mode

A privacy mode must be a documented composition of real actions, not a magic toggle whose side effects are hidden.

Privacy-sensitive local state should stay local unless a specific external test is intentionally invoked.

## Input

Target Input exposes existing desktop input policy rather than leaving it buried in Hyprland Lua.

Target controls include:

- keyboard layouts and ordering
- default layout
- repeat delay/rate
- Caps/Ctrl remapping where supported
- pointer sensitivity/acceleration
- natural scrolling
- tap-to-click
- scroll factor
- supported Hyprland gesture configuration

The current Turkish-Q-first layout policy remains the default until explicitly changed.

Persistent changes should use structured Vesper configuration rather than ad-hoc text editing.

## Shortcuts

Target Shortcuts turns the existing Vesper key vocabulary into an editable searchable surface.

Required capabilities:

- list actions and current chords
- search by action, application and chord
- record shortcut input
- conflict detection before apply
- restore the Vesper default for one action
- distinguish Vesper-owned bindings from upstream/application bindings where attribution is known

Do not maintain a second hard-coded shortcut registry only for Settings. The UI and the existing keybind sheet/command surfaces should converge on one structured source over time.

## AI control plane

The Settings AI surface remains governed by `AI.md`.

The target control plane should go beyond inventory and expose explicit capability policy such as:

- default model/router
- provider priority and fallback
- model selection per agent/runtime
- usage/budget policy
- agent -> MCP permission matrix
- agent -> skill permission matrix
- secret access policy
- shell capability
- privileged/root capability
- network capability
- permission to modify `nix-config`
- context/memory controls

Example conceptual view:

```text
Codex
  GitHub       allow
  filesystem   allow
  browser      allow
  nix-config   ask
  root         deny

Hermes
  research     allow
  network      allow
  nix-config   deny
  root         deny
```

This is a policy surface. It must be backed by enforceable capabilities before a toggle claims to restrict an agent.

Do not show fake security controls that are only labels.

## Automations

Hermes remains the only recurring scheduler for Hermes research jobs.
Settings must not create a second scheduler.

Target Automation UI may expose the existing declarative Hermes registry as:

- enabled state
- schedule
- next run
- last run
- duration
- model/provider when known
- tokens/cost when attributable
- delivery target
- last result
- failure streak
- run now
- pause/resume through the real declarative/runtime contract
- edit schedule through a structured registry change
- logs/briefing link

A dependency view may visualize known workflow relationships such as:

```text
scouts -> synthesis -> agenda -> morning-check
```

The graph must represent real dependencies/data flow, not infer an execution DAG merely from close cron times.

`HERMES.md` remains authoritative for the actual scheduler, registry, jobs and durable state.

## Apps and App Inspector

`APPS-SETTINGS.md` owns the installed-app surface.

The target direction is an App Inspector rather than fake native-app permission toggles.

Useful inspectable state includes, when attributable:

- executable/package owner
- desktop entry
- source/installation owner
- Wayland/XWayland state
- active processes
- CPU/RAM usage
- GPU usage
- network connections
- autostart state
- file associations
- wellbeing
- adaptive-icon state

Flatpak permissions may be editable where the backend can enforce them.
Native Nix applications remain native/unsandboxed unless Vesper introduces a real sandbox launch profile.

A future Vesper sandbox profile may use a real isolation backend such as bubblewrap/systemd sandboxing, but permission toggles must not appear before enforcement exists.

## Wellbeing

Wellbeing remains local-only.

Target UX may include:

- daily/weekly activity graphs
- app categories
- focus mode
- app timers
- break reminders
- category distribution such as coding/browser/social

The existing foreground-usage collector can remain the source where appropriate.
Do not upload wellbeing history merely to generate charts or summaries.

## Appearance

Appearance should expose system-wide visual controls, not adaptive-icon AI generation internals.

Target controls may include:

- palette/theme mode
- Material variant
- transparency
- window rounding
- gaps
- blur strength/passes
- shadow parameters
- animation speed
- reduced motion
- adaptive-icon global appearance/material mode

Ownership split remains:

```text
Appearance
  global visual/icon appearance

AI
  remote icon analysis/generation controls

Apps
  per-app regenerate/retry/exclude/original/diagnostics
```

Adaptive-icon rendering and appearance semantics remain owned only by `ADAPTIVE-ICONS.md`.

Hyprland visual values currently stored in Lua must not be described as editable until a structured settings backend exists.

## Services & Startup

Separate desktop-shell services from curated Vesper system services.

```text
Desktop Services
  Caelestia-owned polling/integration controls

System Services
  explicitly whitelisted Vesper services/targets
```

Possible Vesper system entries include services already owned by Vesper such as Tor, backup, Hermes, the opt-in local web target and optional node services.

Do not dump every systemd unit into the UI.
The page is a curated control plane, not a generic systemd browser.

Every writable entry must state whether it is runtime-only, declarative or both.

## global Settings search

As Settings grows, sidebar navigation alone is insufficient.

Target search should index individual settings semantically rather than only page titles.

Examples:

```text
165 hz
Tor
natural scroll
rollback
OpenAI
backup
```

A result should navigate directly to the relevant page/row and focus or highlight it.

The same settings registry should be reusable by the Caelestia command palette so Settings does not become an isolated navigation system.

A structured search entry should carry at least:

```text
id
page
section
label
keywords
capability
navigation target
```

Do not maintain separate hand-written search keyword lists in multiple UIs.

## action safety

Settings actions fall into four classes:

```text
read-only
runtime mutation
declarative mutation
privileged/destructive mutation
```

The UI must make the class clear through interaction design.

Rules:

- prefer structured backend calls over shell interpolation
- require explicit confirmation for destructive rollback/remove/restore operations
- show planned Nix diffs before declarative mutation
- test before switch when changing system configuration
- never expose secrets in logs, diffs or diagnostics
- keep privileged actions narrow and auditable
- do not claim a restriction/control exists unless it can be enforced

## implementation priority

Recommended first implementation sprint:

1. System / NixOS
2. Display
3. Power & Performance
4. Storage & Recovery
5. System Health
6. Hermes Automations

This sequence turns Settings into a Vesper control plane before expanding secondary convenience pages.

Second layer:

1. Network & Privacy
2. AI capability permissions
3. Input
4. Shortcuts
5. App Inspector / Wellbeing
6. Services & Startup
7. global Settings search and command-palette integration

Implementation order is a priority plan, not proof of current capability.

## implementation rules

When implementing Settings work:

1. inspect the current Caelestia Nexus page before replacing or extending it
2. inspect the owning Vesper subsystem and canonical document
3. extend native Caelestia surfaces when the concern already exists there
4. keep first-party system/control logic in Rust rather than QML shell commands
5. use structured machine-readable backend responses
6. distinguish runtime and declarative ownership
7. do not add fake controls for unsupported hardware or unenforced permissions
8. keep Settings local-first and avoid external telemetry
9. update this document's current implementation section when target surfaces become real
10. update the owning subsystem document when its operational contract changes

Core rule: **Vesper Settings is the UI control plane over Vesper-owned system state. It should expose real NixOS, recovery, privacy, AI and automation capabilities without duplicating their underlying subsystem ownership.**
