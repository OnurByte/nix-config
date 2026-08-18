# AI Analytics

Status: **spec**

This document defines target telemetry, quota, usage-history and coding-activity semantics for Vesper AI.
It is not proof that every adapter or detailed UI described here is implemented.

Current state:

- CodexBar is already used for live provider/quota state
- Agent Cockpit already provides live process context
- AgentsView, `ccusage` and TurnLens are installed
- AI Hub already normalizes the live CodexBar snapshot
- the detailed AI -> Usage & Analytics UI and full cross-source adaptation remain incomplete

`AI.md` owns the AI product/control-plane boundary. This document owns only analytics source normalization, measurement semantics and derived metrics.

## product boundary

Detailed analytics belong under **AI -> Usage & Analytics**.
Vesper Hub stays a compact operational summary.

```text
Vesper Hub
├── overall provider health
├── most constrained reliable quota
├── current reset/freshness data
├── active-agent count
└── tiny current/today summary

AI -> Usage & Analytics
├── Quotas & Resets
├── Activity History
├── Vibe Coding Activity
├── Sessions
├── Models
├── Agents
├── Projects
├── Tokens
├── Costs
└── Raw / Diagnostics
```

Provider health and quota pressure are separate facts. A failing provider must not make an unrelated provider's quota label look critical.

## canonical sources

Vesper should reuse specialised tools rather than invent competing parsers or a second general-purpose session archive.

### CodexBar

Canonical source for live provider/account limit state when supported.

Use it for:

- provider/account identity
- subscription/plan metadata
- quota windows
- used/remaining percentage
- reset timestamps
- provider health/status
- credits/cost/budget when exposed
- top-level `generatedAt` and `staleAfterSeconds`
- provider/account `updatedAt` when exposed
- multi-account `accounts[]` and account quota windows
- pace data when exposed

Do not infer a provider reset time when the source did not supply one.
Do not discard account or freshness data while normalizing the dashboard snapshot.

The CodexBar flake remains intentionally pinned. Treat updates as explicit flake maintenance rather than replacing the pin with a mutable install.

### AgentsView

Canonical source for durable local coding-agent session/activity history.

Use it for supported historical views such as:

- sessions and session-duration statistics
- temporal activity and heatmap substrate
- model breakdowns
- agent breakdowns
- project breakdowns
- tool activity
- Git outcomes when available
- live session updates when available

AgentsView is the primary history substrate. Vesper should adapt and present its data instead of re-ingesting the same agent history into a competing analytics database.

A session duration is not automatically human active coding time.

### ccusage

Accounting cross-check and broad-source fallback where supported.

Use it for:

- historical token consumption
- daily and longer-period totals
- historical accounting
- existing session/day aggregation
- agent-aware accounting when exposed

Do not reimplement ccusage calculations unless a Vesper-specific metric is missing.
Do not assume normal ccusage JSON is a complete raw per-turn timeline.

Scheduled analysis must keep history requests bounded rather than dumping all available local history into an AI prompt.

### TurnLens

Fine-grained microscope for supported Codex and Claude activity.

Use it for:

- per-turn token measurements when captured
- duration measurements when captured
- model/runtime attribution when exposed
- API-equivalent cost inspection
- watched-session diagnostics

TurnLens is not the global session-history authority and must not be the only source for multi-agent `how many hours did I vibe code?` metrics.
Coverage can differ from the complete set of parallel agents, subagents and historical sessions.

A retrospective report and a cost captured while a turn/session was observed may use different pricing semantics. Preserve that distinction.

### Agent Cockpit

Canonical source for live coding-agent process context.

Use it for:

- runtime/agent identity
- repository/project
- working directory
- process state
- PID when useful
- branch
- dirty/clean state
- elapsed runtime

Process uptime is not human active coding time.
Agent Cockpit's bounded live/process snapshots are not a replacement for the primary historical session archive.

### optional orchestration events

Agent Teams backends may enrich analytics with:

- team and role
- task assignment/start/completion
- hand-offs
- agent lifecycle events
- coordinator/worker relationships

They are enrichment sources, not quota or session-history authorities.

When CCCC is the backend, application integration should use its supported SDK/IPC surface. Do not parse its append-only ledger or human-readable CLI output as Vesper's application API.

### Hermes

Hermes owns scheduled research and automation.
Keep the current pattern where Hermes cron is the heartbeat and long-running work is dispatched outside the scheduler through the existing Vesper runtime path.

The AI usage economist should gather bounded machine-readable snapshots:

```text
codexbar dashboard --identity redacted
ccusage daily --last 7 --by-agent --json
turnlens report --last 7d --json
```

If a command cannot start or exits non-zero, the snapshot must show that failure instead of silently returning an empty section.

### Git

Git metadata may support attribution:

- commit timestamps
- branch/repository identity
- changed-file counts
- diff statistics

Git history alone must never be used to claim hours worked.

## provider surfaces

API credentials and subscription/account quota surfaces are different identities.

Examples:

```text
OpenAI API key      != ChatGPT / Codex subscription
Anthropic API key   != Claude subscription
xAI API key         != Grok subscription/session surface
```

The credential manager can use company/provider identifiers while analytics keeps a separate source surface.
Do not merge records only because they share a company name.

Conceptual identity:

```text
ProviderSurface
├── provider
├── surface_kind       api | subscription | account | local | unknown
├── source             oauth | api | web | local | cli | ...
├── account_id
├── account_label
├── plan
└── source_identity
```

Keep `surface_kind` unknown when the source does not expose enough evidence. Do not guess it from display text.

## normalization

Expose one backend-neutral view/semantics layer to QML rather than source-specific structures.
Vesper owns the joins and user-facing meaning, not a second authoritative session archive.

Conceptual observation:

```text
AiActivityObservation
├── timestamp
├── source
├── source_event_id
├── provider_surface_id
├── provider
├── model
├── runtime
├── agent
├── team
├── project
├── repository
├── branch
├── session_id
├── task_id
├── input_tokens
├── output_tokens
├── cached_tokens
├── cost_amount
├── currency
├── cost_kind
├── pricing_version
├── event_kind
└── confidence
```

Missing attribution stays unknown.
Never fabricate model, project, agent, token, duration or cost values to complete a chart.

Deduplicate overlapping observations. The same work observed by AgentsView, TurnLens and ccusage must not be counted repeatedly just because several collectors saw related records.

Use `source_event_id` and source-specific identity where available instead of heuristic text matching alone.

## quota and reset semantics

Keep independent provider windows and independent accounts independent.

Required normalized fields when available:

```text
kind
label
usedPercent
remainingPercent
resetAt
updatedAt
source
account
staleAfterSeconds
```

Health classification:

- `critical` — provider critical/error or <= 10% reliable remaining
- `warning` — provider warning or <= 25% reliable remaining
- `ok` — reliable status/quota evidence is healthy
- `unknown` — no trustworthy health/quota evidence

`unknown` is not `ok`.

The Hub summary keeps two independent results:

```text
overallHealth
mostConstrainedQuota
```

`overallHealth` is derived from provider health.
`mostConstrainedQuota` is selected only from reliable quota windows, including account-level windows.
Never reuse the quota provider's label as the identity of an unrelated provider failure.

## freshness

Preserve source freshness instead of replacing it with Vesper's local normalization timestamp.

Relevant fields include:

```text
CodexBar generatedAt
CodexBar staleAfterSeconds
provider updatedAt
account updatedAt when exposed
```

Vesper may also mark its own cached snapshot as stale after a refresh failure.
Source freshness and Vesper cache freshness are separate facts.

The UI should make provider update/freshness information inspectable rather than silently treating old provider data as current.

## time semantics

Vesper distinguishes four useful time concepts.

### observed active coding

Closest metric to "how long did I actively vibe code?"

Derive it only from source records that actually expose sufficiently granular attributable timestamps.
Do not turn daily/session aggregates into a fake raw event timeline.
Do not substitute process lifetime or session duration when the required activity evidence is unavailable.

When a source supports granular activity, a bounded sessionization rule may:

1. order attributable events by time
2. keep nearby events in one interval while gaps stay below the idle threshold
3. close the interval after prolonged inactivity
4. cap silent gaps
5. merge overlapping human-facing intervals

A **15 minute** idle threshold may be the default for a supported event source, but it is not a claim that every collector exposes the events required to apply it.

This metric is estimated from observed activity and must not be presented as a biometric timesheet.

### session duration

Duration of a recorded agent session, primarily from AgentsView history.

A session left open for four hours is not automatically four hours of observed active coding.

### agent wall-clock

Elapsed wall time during which at least one coding agent was running/working.
Overlapping agents are unioned.

Two agents from 10:00-11:00 produce:

```text
agent wall-clock = 1h
```

### agent-hours

Sum of attributable runtime across agents.
Parallel work counts independently.

Two agents from 10:00-11:00 produce:

```text
agent-hours = 2h
```

Agent-hours must never be presented as human hours worked.

## Vibe Coding Activity

Target UI includes a GitHub-contribution-style daily heatmap for the most recent 12 months.
AgentsView is the primary historical substrate for sessions and temporal activity.

Available intensity modes depend on source support and may include:

- observed active coding time
- session duration
- agent wall-clock
- agent-hours
- tokens
- turns/events
- cost by explicit cost kind
- completed agent tasks

Selecting a day should open a drill-down instead of making the heatmap decorative.

A daily drill-down may show, when attributable:

- observed active coding or unavailable/unknown
- session duration
- agent wall-clock
- agent-hours
- turns/events
- tokens
- current-rate API-equivalent cost
- as-observed cost estimate
- commits as supporting metadata
- repository count
- model breakdown
- agent breakdown
- project breakdown

## breakdowns

Aggregate reliable data by:

- provider surface
- model
- runtime
- agent
- Agent Team role
- project/repository

Useful periods:

```text
Today · 7d · 30d · 90d · 1y · All
```

Useful views include:

- session/activity duration by day
- observed active coding where supported
- agent-hours by day
- turns/tokens/cost by day
- model share and trend
- agent runtime/task statistics
- project/repository activity
- current/longest streak
- weekday/time-of-day distribution
- quota state correlated with local activity

Correlation must not be presented as knowledge of a provider's internal quota formula.

## cost semantics

Costs from different sources are not automatically additive.

Keep explicit cost kinds such as:

```text
provider_billed
provider_credit
local_log_estimate
api_equivalent_current_rate
api_equivalent_as_observed
```

A normalized cost should carry, when known:

```text
cost_kind
currency
source
source_event_id
pricing_version
```

CodexBar, ccusage and TurnLens may observe overlapping local usage or different billing surfaces.
Never silently add them into one `cost_usd` stream.

TurnLens retrospective reports may re-price historical usage with current API rates while an observation captured earlier may reflect rates at capture time.
Do not overwrite one semantic with the other.

Estimates must be labelled as estimates and must not be presented as subscription invoices.

## Vesper-owned state

AgentsView is the primary durable session/activity archive.
Vesper should not create a parallel general-purpose analytics database that re-ingests the same coding-agent history.

Vesper may keep small rebuildable state for things the upstream sources do not own, for example:

- normalized live quota cache
- source join keys
- small derived UI rollups
- Vesper-specific confidence/provenance metadata

Requirements:

- local-only by default
- no API keys or prompts stored merely for analytics
- bounded/rebuildable where practical
- timezone-aware local-day boundaries
- deduplication
- source provenance
- schema versioning

## attribution and confidence

Derived metrics should retain provenance.

Confidence classes:

- `exact` — explicit source value/timestamp/identity
- `attributed` — reliably joined across sources
- `estimated` — inferred from a documented bounded rule
- `unknown` — insufficient data

Diagnostics should make provenance inspectable without cluttering the normal UI.

## privacy

Analytics remain local unless a provider query is required to refresh provider status.

Do not upload local analytics such as:

- repository paths
- filenames
- prompts
- task text
- commit content
- AgentsView history
- TurnLens history
- ccusage history
- local derived activity

to an external Vesper analytics service.

## implementation order

1. keep CodexBar as the live quota/reset/account source
2. preserve CodexBar freshness, multi-account windows and pace through normalization
3. keep `overallHealth` separate from `mostConstrainedQuota` and preserve `unknown != ok`
4. use AgentsView as the primary session/activity history backend
5. keep ccusage as bounded accounting cross-check/fallback
6. keep TurnLens as the supported per-turn microscope rather than the global history source
7. keep Agent Cockpit as live process context
8. enrich with backend-neutral Agent Teams events when useful
9. build AI -> Usage & Analytics as a Vesper UI/adaptation layer over those sources
10. add Vesper-specific derived metrics only when source granularity and provenance support them
11. keep Hub compact
12. update fast-moving pinned inputs intentionally through normal flake maintenance

Core rule: **CodexBar owns live provider-limit facts, AgentsView owns session/activity history, ccusage cross-checks accounting, TurnLens inspects supported turns, Agent Cockpit owns live process context, and Vesper owns normalization plus honest user-facing semantics.**
