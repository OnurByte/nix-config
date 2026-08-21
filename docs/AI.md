# AI

Status: **partial**

This document is the single product-boundary contract for Vesper AI.
It contains both implemented behavior and target behavior. Current code must be checked before claiming a capability exists.

`AI-ANALYTICS.md` owns analytics measurement semantics. `ADAPTIVE-ICONS.md` owns the adaptive icon pipeline. `SETTINGS.md` owns where AI controls appear in the wider Settings information architecture.

## current implementation

Current Vesper AI already has:

- a native Caelestia AI settings area
- API-key-only shared provider credentials through `vesper-control`
- freedesktop Secret Service storage via `secret-tool`
- provider/status snapshots and health state
- live Agent Cockpit process state
- canonical skills inventory
- shared MCP inventory
- Hermes integration
- adaptive-icon AI controls and provider readiness

Current implementation is still incomplete relative to the target contract below.
In particular, detailed analytics/history, enforceable per-agent capability policy and the backend-neutral Agent Teams orchestration surface are not complete product surfaces yet.

## product boundary

Use this split consistently:

```text
Vesper Hub
├── compact AI health summary
├── most constrained provider
├── current usage percentage
├── nearest reset
├── warnings / critical state
├── live-agent count
└── Hermes unread summary

AI
├── Overview
├── Providers
├── Usage & Analytics
├── Agents
├── Agent Teams
├── Skills
├── MCP
├── Automations
└── AI feature controls
```

Vesper Hub stays glanceable. It must not become a second AI settings application.

The AI section owns the detailed control plane.

### compact quota pressure widget

Vesper should expose a tiny bar-level quota-pressure surface for the providers that matter right now. This is a presentation layer over the existing AI Hub/CodexBar pipeline, not a new quota backend.

Target compact form may look like:

```text
Codex 63% · Claude 28%
```

The percentage shown in the compact surface represents the normalized `usedPercent` for the selected quota window. The tooltip or expanded Hub must make the semantic explicit and show reset/freshness details when available.

Rules:

- read only the normalized Vesper AI Hub snapshot; do not add provider-specific parsing in the bar
- keep CodexBar authoritative for supported live provider/account quota facts
- order providers by quota pressure so the most constrained reliable provider is surfaced first
- keep the compact surface bounded; normally show at most the one or two most useful provider summaries rather than every configured provider
- use the existing health/pressure semantics from `AI-ANALYTICS.md`; do not invent a second warning threshold system for the bar
- clicking the widget opens the existing Vesper Hub rather than a parallel settings popup
- manual refresh should reuse the existing AI Hub refresh path
- if data is stale, preserve the last valid values and visibly mark the widget stale/dimmed
- if no trustworthy quota value exists, show an unknown/unavailable state rather than `0%`
- provider failure and quota pressure remain separate facts; a critical provider must not cause another provider's quota label to inherit that failure
- do not add another analytics database, daemon, subscription scraper or provider credential store for this widget

The bar is therefore a glanceable entry point into the existing control plane:

```text
CodexBar / provider sources
          │
          ▼
   Vesper AI Hub
   normalized snapshot
          │
     ┌────┴────┐
     ▼         ▼
quota widget  Vesper Hub
  glance       detail
```

This follows the same product principle as the rest of Vesper: authoritative sources own facts, Vesper owns normalization and semantics, and lightweight surfaces only render that normalized state.

## credentials

The credential manager is API-key only. It does not implement OAuth.

Shared provider keys are stored through freedesktop Secret Service with `secret-tool`.
They are not written into Nix source, Home Manager session variables, shell history or process arguments.

Supported shared key slots:

- OpenAI
- Anthropic
- xAI
- OpenRouter
- Google AI

Check configured providers:

```bash
vesper-control ai-status
```

Run one command with a provider key scoped to that child process:

```bash
vesper-control credential exec openai your-command --args
```

Do not configure these shared Vesper provider slots through sops-nix. `SECRETS.md` defines the repository-wide secret ownership split.

Credential availability and usage observability are separate capabilities. A provider can be configured while quota, reset, credit or cost information remains unavailable.

## provider usage and health

Provider adapters normalize reliable source data into backend-neutral state.

A provider may expose multiple independent windows such as:

```text
5-hour window
weekly allowance
monthly credits
monetary budget
```

Never collapse independent windows into one fake quota.

Normalized quota windows use fields equivalent to:

```text
kind
label
usedPercent
remainingPercent
resetAt
updatedAt
source
```

Health classification follows `AI-ANALYTICS.md`:

- `critical` — provider critical/error state or <= 10% reliable remaining
- `warning` — provider warning state or <= 25% reliable remaining
- `ok` — reliable status/quota evidence is healthy
- `unknown` — no trustworthy health/quota evidence

`unknown` is not `ok`.

Provider health and quota pressure are separate facts. The Hub must keep `overallHealth` separate from `mostConstrainedQuota` rather than making an unrelated quota provider appear to own another provider's failure.

If refresh fails:

- keep the last valid snapshot
- mark it stale
- expose the refresh/backend error separately
- never replace known quota data with invented zeroes

Detailed history, tokens, cost and activity semantics are defined only in `AI-ANALYTICS.md`.

## agents

The Agents view owns individual running coding/research processes.

When available, expose:

- runtime
- project/repository
- working directory
- process state
- PID for diagnostics
- branch
- dirty/clean state
- elapsed runtime
- current task
- model/provider

Agent process state and provider quota state may be linked in the UI without coupling their implementations.

Persistent Agent Cockpit snapshots belong under:

```text
~/.local/state/vesper/agents/
```

### Agent Cockpit collection contract

Current Agent Cockpit status collection inspects live processes and Git working-tree state. It writes only bounded diagnostic metadata, keeps process identity separate from raw argv, and reuses a schema-versioned status snapshot for a short bounded TTL. The UI may refresh frequently, but that must not force every expensive process and Git probe to rerun at the same cadence.

Remaining remediation for the desktop polling path:

- separate cheap liveness refresh from slower repository/dirty-state inspection where useful
- prefer event-driven updates when the source already exposes them
- make QML consume normalized state rather than independently repeating process or Git commands
- keep persisted snapshots bounded and diagnostic; AgentsView remains the durable activity archive
- do not persist secrets or unsanitized command arguments merely to make the cockpit observable

A five-second visual refresh target is not permission to run a full process/Git scan every five seconds.

## capability policy

Vesper should grow from inventory into an enforceable AI capability control plane.

Target policy dimensions include:

- default model/router
- provider priority and fallback
- model/provider selection per agent or runtime
- usage/budget policy
- MCP access
- skill access
- shared-secret access
- filesystem scope
- browser/network access
- shell capability
- privileged/root capability
- permission to modify `nix-config`
- context/memory controls

Conceptual policy:

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

The exact capability vocabulary should remain backend-neutral. A runtime/backend can map these concepts to its own enforceable primitives.

Rules:

- a visible permission must correspond to a real enforcement path
- `deny` must not mean "hide the button but the agent can still do it"
- `ask` must gate the capability before use, not merely log it afterward
- default to least privilege for dangerous capabilities such as root, secret access and declarative system mutation
- capability decisions should be attributable to the agent/runtime and action
- do not pass long-lived secrets through broad environment state merely to implement permission checks
- an orchestration backend must not silently widen capabilities beyond Vesper policy

Until an enforcement backend exists for a capability, show it as unavailable/unenforced rather than presenting a fake security toggle.

`SETTINGS.md` owns how this policy is presented alongside the rest of Vesper Settings. This document owns the AI permission semantics.

## Agent Teams and orchestration

Agent Teams is the user-facing orchestration feature.

Vesper owns the product boundary. An orchestration backend is an implementation detail.

```text
Vesper
└── AI Control Plane
    ├── providers and credentials
    ├── analytics
    ├── skills
    ├── MCP
    ├── live agents
    ├── Hermes
    └── AgentOrchestrator
        ├── native Vesper backend
        └── optional external backend
```

The orchestration contract must stay backend-neutral.

A backend may provide capabilities such as:

- list runtimes
- list/create/delete teams
- start/stop teams
- add/restart agents
- assign tasks
- inspect task state
- send messages
- expose bounded diagnostics

Do not leak backend-specific actor IDs, ledgers, daemon details or transport concepts into the primary UI.

### CCCC

CCCC (`ChesterRa/cccc`) is an optional replaceable orchestration backend and a useful development-time coordination tool.

CCCC is not:

- the Vesper product model
- a mandatory runtime dependency
- the owner of Vesper credentials, skills, analytics or MCP
- justification for duplicating Vesper's AI control plane

If CCCC is absent, Agent Teams should report that no orchestration backend is available rather than making unrelated AI settings disappear.

Installation must follow Vesper/Nix packaging rules rather than an arbitrary mutable `pip install` path.

Rule: **CCCC may help build Vesper; Vesper must not be built around CCCC.**

## skills and MCP

The AI page reads skills from the canonical:

```text
~/.agents/skills
```

Agent-specific skill directories remain links into that tree.

The MCP list comes from the Home Manager `programs.mcp.servers` registry so Codex, Claude Code and OpenCode see the same configured servers.

Skills and MCP configuration remain Vesper-owned even when an orchestration backend references them.

Capability policy may restrict an individual agent's access to a configured skill or MCP server without deleting it from the shared registry.
Configuration and permission are separate facts.

## Hermes

Hermes keeps its own recurring research/scheduling contract.
The AI page may surface Hermes status and relevant controls without creating a second scheduler.

Recurring Hermes jobs are governed by `HERMES.md`.
The target Settings presentation is defined in `SETTINGS.md`.

## adaptive icons

Adaptive icons use the Rust-owned automatic pipeline defined only in `ADAPTIVE-ICONS.md`.

The old Apps -> Experimental manual request/review model is obsolete.

Ownership split:

```text
AI
  provider readiness
  remote-analysis consent
  generation/provider controls

Appearance / Theme
  icon appearance
  material/rendering controls

Apps
  per-app retry
  exclusion/original controls
  diagnostics
  per-app export
```

Remote semantic conversion requires explicit **Allow remote icon analysis** consent and a configured provider key.
Consent is off by default.

Palette, wallpaper, appearance and renderer changes are local recompiles and must not consume another AI request for an already-valid canonical package.

Vesper intentionally has no bulk icon export UI or bulk export backend command.

## implementation rule

When implementing this document:

1. inspect current code first
2. preserve the Vesper-owned backend-neutral boundary
3. reuse existing provider/Agent Cockpit/skills/MCP/Hermes data instead of adding parallel parsers
4. keep detailed analytics semantics in `AI-ANALYTICS.md`
5. do not present unenforced capability controls as security boundaries
6. update this document's `current implementation` section when a target surface becomes real
