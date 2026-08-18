# AI

Vesper exposes AI as a native Caelestia Nexus settings area.

The AI control plane owns provider credentials, usage and quota analytics, live agents, agent teams, skills, MCP inventory, Hermes integration and AI-backed desktop features. Vesper Hub is only the compact operational summary; detailed AI inspection belongs here.

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

The Hub must remain glanceable. It should not become a second AI settings application and it should not contain long provider lists, historical charts, account-management controls or orchestration configuration.

The AI section is the detailed source of truth.

## API keys

The credential manager is API-key only. It does not implement OAuth.

Keys are stored through freedesktop Secret Service with `secret-tool`. They are not written into Nix source, Home Manager session variables, shell history or process arguments.

Check configured providers:

```bash
vesper-control ai-status
```

Run one command with a single provider key scoped to that child process:

```bash
vesper-control credential exec openai your-command --args
```

Supported shared key slots are OpenAI, Anthropic, xAI, OpenRouter and Google AI.

Credential availability and usage observability are separate capabilities. A provider may be configured and usable while plan, reset-window, credit or cost information is unavailable.

## Vesper Hub AI summary

Vesper Hub shows only the information required to decide whether the user needs to open AI settings.

Recommended summary:

```text
AI
Codex                72% used
weekly reset         21 Aug 11:54
2 agents active      1 warning
```

If several providers are active, the Hub highlights the most constrained provider/window and can show a compact aggregate such as `1 critical · 2 warning`.

The Hub may expose a refresh action, but detailed rows and historical analysis stay in AI.

The current implementation already has a normalised usage backend and provider health model. The existing detailed provider cards in `AiHub.qml` should be treated as functionality to migrate/reuse in the AI settings surface; Hub should then be reduced to the compact summary above. Do not create a second quota parser during that migration.

## usage and analytics

Usage, quotas and resets are first-class AI data, not a connected/disconnected badge.

The AI section should provide detailed analytics for every provider that can expose them.

### provider details

Each provider view should support, when data exists:

- provider name and backend source;
- account identity;
- subscription or plan;
- provider health;
- last successful refresh;
- stale/fresh state;
- backend errors;
- one or more independent quota windows;
- used percentage;
- remaining percentage;
- reset timestamp;
- credits remaining;
- API/provider cost data;
- current-day cost;
- rolling 7-day and 30-day usage/cost when the source can provide it.

Do not collapse several provider limits into one fake quota. A service may expose a short rolling window, weekly allowance, monthly credits and monetary budget independently.

Example:

```text
Codex
Plan: Plus
Status: healthy

5-hour window
42% used                 58% remaining
Reset: 18 Aug 19:40

Weekly allowance
71% used                 29% remaining
Reset: 21 Aug 11:54

Credits
12.40 remaining

Cost
Today                    $1.26
Last 7 days              $6.91
Last 30 days             $22.18
```

Exact labels and reset timestamps must come from the provider/backend. Vesper must not invent reset times, limits or costs.

### statistics and history

The detailed AI analytics view should persist bounded local snapshots so it can show trends instead of only the latest value.

Useful views:

- usage over time by provider;
- remaining quota over time;
- reset events;
- daily request/token/cost totals when exposed;
- 7-day and 30-day comparisons;
- provider availability/error history;
- active-agent runtime usage when attributable;
- model usage split when attributable;
- peak usage periods;
- time-to-reset;
- estimated exhaustion time when enough recent samples exist.

Any forecast must be clearly labelled as an estimate and derived only from observed local history. Vesper must never present a prediction as a provider-issued limit.

Suggested periods:

```text
24h · 7d · 30d · 90d
```

Long-term history must be bounded and cheap. This is telemetry about the user's own local AI usage, not a general analytics service.

### quota windows

The normalised provider model already supports independent windows with:

```text
kind
label
usedPercent
remainingPercent
resetAt
```

Keep this model backend-neutral. Provider adapters translate source-specific data into the common representation.

Health rules can use the most constrained available window:

- critical: provider error/critical state or <= 10% remaining;
- warning: <= 25% remaining;
- healthy: otherwise;
- unknown: no reliable quota data.

Unknown must remain distinct from healthy.

### refresh and stale data

Passive polling may use cached snapshots. Every displayed snapshot must know when it was generated.

Manual refresh requests fresh provider state where possible.

If refresh fails:

- keep the last valid snapshot;
- mark it stale;
- show the refresh/backend error separately;
- never replace a known quota with fake zeroes.

## agents

The Agents view owns individual running AI/coding processes.

Show at minimum:

- runtime;
- project/repository;
- working directory;
- process state;
- PID when useful for diagnostics;
- current branch;
- dirty/clean working-tree state;
- elapsed runtime;
- current task when known;
- model/provider when discoverable.

Agent process state and provider quota state should be linkable without coupling their implementations. For example, the UI may show that three Codex workers are active while the Codex weekly allowance is nearing its limit.

## agent orchestration and CCCC

CCCC (`ChesterRa/cccc`) is useful to Vesper as an optional agent-orchestration backend, but it must not become a core Vesper dependency or the implementation foundation of the AI control plane.

The architectural boundary is deliberate:

```text
Vesper
└── AI Control Plane
    ├── providers and shared credentials
    ├── usage / quotas / reset windows
    ├── analytics history
    ├── skills
    ├── MCP registry and permissions
    ├── live agents
    ├── Hermes integration
    └── AgentOrchestrator
        ├── native Vesper
        └── CCCC (optional)
            ├── ChatGPT Web
            ├── Codex
            ├── Claude Code
            ├── Hermes
            ├── OpenCode
            └── other supported runtimes
```

Vesper remains the owner of provider configuration, secrets, permissions, skills, usage analytics, desktop integration and user-facing AI settings. CCCC may be used behind that boundary for agent lifecycle, persistent coordination, foreman/worker teams, task/message state, nudging and cross-runtime orchestration.

Do not expose CCCC as the product model in the primary UI. The user-facing feature is **Agent Teams**. CCCC is an implementation choice behind that interface.

### Agent Teams

`Agent Teams` manages groups of cooperating coding/research agents.

Example:

```text
Vesper Development                     running

Coordinator    Claude Code             working
Coder 1        Codex                   implementing Vesper Store
Coder 2        Codex                   idle
Research       Hermes                  researching
Reviewer       Claude Code             reviewing
```

A team should expose:

- team name;
- repository/working directory;
- coordinator/foreman;
- worker roles;
- runtime per agent;
- model per agent when discoverable;
- running/idle/working/error state;
- current task;
- tracked tasks and expected outcomes;
- task assignment/reassignment;
- activity/message stream;
- start, stop and restart controls;
- bounded logs and diagnostics.

CCCC-specific actor IDs, ledger files, daemon details and transport internals stay behind the adapter unless shown in an explicit diagnostics view.

### orchestration interface

Vesper must own a backend-neutral contract rather than import CCCC concepts throughout the codebase.

The exact Rust API may evolve, but the capability boundary should resemble:

```rust
trait AgentOrchestrator {
    fn available(&self) -> bool;
    fn list_runtimes(&self) -> Result<Vec<Runtime>>;
    fn list_teams(&self) -> Result<Vec<Team>>;
    fn create_team(&self, spec: TeamSpec) -> Result<TeamId>;
    fn delete_team(&self, id: TeamId) -> Result<()>;
    fn start_team(&self, id: TeamId) -> Result<()>;
    fn stop_team(&self, id: TeamId) -> Result<()>;
    fn add_agent(&self, team: TeamId, spec: AgentSpec) -> Result<AgentId>;
    fn restart_agent(&self, agent: AgentId) -> Result<()>;
    fn assign_task(&self, agent: AgentId, task: TaskSpec) -> Result<TaskId>;
    fn task_status(&self, task: TaskId) -> Result<TaskState>;
    fn send_message(&self, target: AgentId, message: &str) -> Result<()>;
}
```

The first external adapter may call CCCC. A later native implementation can replace it without redesigning the AI UI.

### optional installation

CCCC must remain optional.

If no orchestration backend is available, `Agent Teams` should still exist and explain the state rather than disappearing mysteriously:

```text
Agent Teams

No orchestration backend installed.

[ Install CCCC ]
```

Installation should follow Vesper/Nix packaging rules rather than arbitrary runtime `pip install` mutations. The UI can request the declarative package/configuration change and let the normal Vesper activation path apply it.

Removing CCCC must not remove Vesper credentials, skills, provider history or unrelated AI settings.

### use CCCC to develop Vesper

CCCC may also be used directly as a development tool without product integration.

Useful topology:

```text
ChatGPT Web / strong reasoning model   -> coordinator / reviewer
Codex                                  -> implementation workers
Hermes                                 -> research / long-running worker
CCCC                                   -> coordination and persistent hand-offs
Vesper repository                      -> shared workspace
```

Typical loop:

```text
inspect repository
-> plan bounded changes
-> delegate independent work
-> implement
-> test / typecheck / build
-> review diff
-> return failures to the responsible worker
-> checkpoint
-> continue
```

Near-term rule: **use CCCC to build Vesper; do not build Vesper on CCCC**.

Do not duplicate CCCC wholesale inside Vesper. Reuse it first to validate which orchestration features are actually valuable. Only migrate proven Vesper-specific primitives into the native Rust control plane when there is a concrete reason to own them.

## skills and MCP

The AI page reads skills from the canonical `~/.agents/skills` tree. Agent-specific skill directories stay links into that tree.

The MCP list is generated from `programs.mcp.servers`, so the settings page reports the same registry that Home Manager exposes to Codex, Claude Code and OpenCode.

Skills and MCP availability may be referenced by Agents and Agent Teams, but their canonical configuration remains owned by Vesper rather than CCCC.

## adaptive icons

Adaptive icons use the automatic Rust-owned pipeline defined in `ADAPTIVE-ICONS.md`; the old Apps -> Experimental manual request/review queue is obsolete.

The engine discovers effective `.desktop` applications, resolves trustworthy packaged icon sources, fingerprints and deduplicates canonical work, persists conversion jobs, reuses accepted `.vicon` packages, and updates the generated Vesper freedesktop icon theme. Provider outages or missing credentials leave existing/original fallback icons usable instead of breaking the desktop.

Generation/provider controls live under AI. Appearance and material controls stay under Appearance/Theme, while application-specific retry/exclusion/original/diagnostic actions stay under Apps.

Remote semantic conversion requires explicit **Allow remote icon analysis** consent. Consent is off by default. With consent off, local vector handling, accepted canonical packages and fallback rendering continue to work, while jobs that require a provider remain `blocked-no-consent` and the worker does not claim new remote work. Enabling consent allows eligible jobs to resume automatically; a provider key is still required separately.

A configured shared provider key is reused automatically. Palette, wallpaper, appearance and renderer changes are local recompiles and must not consume another AI request for an already valid canonical package.

Per-app diagnostic export is local-only. Vesper intentionally has no bulk icon export UI or bulk export backend command.
