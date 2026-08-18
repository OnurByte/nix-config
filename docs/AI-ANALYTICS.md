# AI Analytics

Status: **spec**

This document defines target telemetry, quota, usage-history and coding-activity semantics for Vesper AI.
It is not proof that every adapter, history store or UI described here is implemented.

Current state:

- CodexBar is already used for live provider/quota state
- Agent Cockpit already provides live process context
- `ccusage` and TurnLens are installed
- full ccusage/TurnLens normalization, local long-term history and the detailed analytics UI remain incomplete

`AI.md` owns the AI product/control-plane boundary. This document owns only analytics source normalization, measurement semantics and derived metrics.

## product boundary

Detailed analytics belong under **AI -> Usage & Analytics**.
Vesper Hub stays a compact operational summary.

```text
AI -> Usage & Analytics
├── Quotas & Resets
├── Usage History
├── Vibe Coding Activity
├── Models
├── Agents
├── Projects
├── Tokens
├── Costs
└── Raw / Diagnostics
```

## canonical sources

Vesper should reuse specialised tools rather than invent competing parsers.

### CodexBar

Canonical source for live provider/account limit state when supported.

Use it for:

- provider/account identity
- subscription/plan metadata
- quota windows
- used/remaining percentage
- reset timestamps
- provider health
- credits/cost/budget when exposed
- fresh/stale snapshots

Do not infer a provider reset time when the source did not supply one.

### ccusage

Canonical source for broad historical accounting where supported.

Use it for:

- historical token consumption
- daily and longer-period totals
- historical cost/accounting
- existing session/day aggregation

Do not reimplement ccusage calculations unless a Vesper-specific metric is missing.

### TurnLens

Canonical source for fine-grained per-turn telemetry where supported.

Use it for:

- turn timestamps
- input/output tokens
- API-equivalent cost
- model/runtime attribution when exposed
- fine-grained session activity

TurnLens is preferred over process uptime for interaction-based activity measurement.

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

### optional orchestration events

Agent Teams backends may enrich analytics with:

- team and role
- task assignment/start/completion
- hand-offs
- agent lifecycle events
- coordinator/worker relationships

They are enrichment sources, not quota authorities.

### Git

Git metadata may support attribution:

- commit timestamps
- branch/repository identity
- changed-file counts
- diff statistics

Git history alone must never be used to claim hours worked.

## normalization

Expose one backend-neutral model to QML rather than source-specific structures.

Conceptual event:

```text
AiActivityEvent
├── timestamp
├── source
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
├── cost_usd
├── event_kind
└── confidence
```

Missing attribution stays unknown.
Never fabricate model, project, agent, token or cost values to complete a chart.

Deduplicate overlapping observations. The same turn observed by TurnLens and ccusage must not be counted twice.

## quota and reset semantics

Keep independent provider windows independent.

Required normalized fields when available:

```text
kind
label
usedPercent
remainingPercent
resetAt
updatedAt
source
```

Health classification:

- `critical` — provider critical/error or <= 10% reliable remaining
- `warning` — <= 25% reliable remaining
- `healthy` — reliable data above warning threshold
- `unknown` — no trustworthy quota data

`unknown` is not `healthy`.

## time semantics

Vesper distinguishes three time concepts.

### observed active coding

Closest metric to "how long did I actively vibe code?"

Derive it from interaction/turn events and attributable activity, not full process lifetime.

Default sessionization rule:

1. order attributable events by time
2. keep events in one session while gaps stay below the idle threshold
3. close the interval after prolonged inactivity
4. cap silent gaps
5. merge overlapping active intervals

Default idle threshold: **15 minutes**.

This metric is estimated from observed activity and must not be presented as a biometric timesheet.

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

Default intensity: observed active coding minutes.
Optional modes may include:

- active coding time
- agent-hours
- tokens
- turns
- API-equivalent cost
- completed agent tasks

Selecting a day should open a drill-down instead of making the heatmap decorative.

A daily drill-down may show, when attributable:

- observed active coding
- agent wall-clock
- agent-hours
- turns
- tokens
- API-equivalent cost
- commits as supporting metadata
- repository count
- model breakdown
- agent breakdown
- project breakdown

## breakdowns

Aggregate reliable data by:

- provider
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

- active coding by day
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

Keep these concepts separate:

- provider-reported billed/credit cost
- ccusage historical cost/accounting
- TurnLens API-equivalent estimate

Never silently add incompatible cost semantics.
Estimates must be labelled as estimates.

## local history store

Vesper may maintain a bounded local analytics store for joins and trends.

Requirements:

- local-only by default
- no API keys or prompts stored merely for analytics
- bounded retention or compaction
- deterministic aggregation
- timezone-aware local-day boundaries
- deduplication
- source provenance
- schema versioning
- rebuildable aggregates when source history still exists

Suggested retention shape:

```text
fine events       short retention
hourly aggregates medium retention
daily aggregates  long retention
```

The heatmap should read daily aggregates rather than repeatedly scanning raw turn history.

## attribution and confidence

Derived metrics should retain provenance.

Confidence classes:

- `exact` — explicit source value/timestamp/identity
- `attributed` — reliably joined across sources
- `estimated` — inferred from sessionization or another bounded heuristic
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
- TurnLens history
- ccusage history
- local activity aggregates

to an external analytics service.

## implementation order

1. keep CodexBar as the live quota/reset source
2. add ccusage and TurnLens adapters
3. normalize and deduplicate sources
4. move detailed quota/provider UI into AI -> Usage & Analytics
5. keep Hub compact
6. persist bounded hourly/daily aggregates
7. add the 12-month activity heatmap
8. add daily and model/agent/project drill-downs
9. enrich attribution with Agent Cockpit and optional Agent Teams events
10. add forecasts only after provenance and deduplication are reliable

Core rule: **CodexBar owns live provider-limit facts, ccusage owns broad historical accounting, TurnLens owns fine-grained turn telemetry, and Vesper joins them without pretending process uptime equals coding time.**
