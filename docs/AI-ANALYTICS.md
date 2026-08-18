# AI Analytics

This document defines the telemetry, quota, usage-history and coding-activity semantics used by the Vesper AI settings area.

`AI.md` remains authoritative for the AI control-plane product boundary, credentials, agents, Agent Teams, CCCC, skills, MCP and AI-backed desktop features. This document is authoritative only for AI usage/analytics data and how that data is measured and presented.

## product boundary

Vesper Hub is a compact operational summary. Detailed analytics belong under **AI -> Usage & Analytics**.

```text
Vesper Hub
├── most constrained provider
├── current quota usage
├── nearest reset
├── warning / critical state
├── active-agent count
└── compact today's activity

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

The Hub must not duplicate the detailed analytics application. It should answer only: **is something constrained, when does it reset, and how active is AI work right now/today?**

## canonical data sources

Vesper must reuse the specialised tools already installed instead of inventing competing parsers.

### CodexBar

Primary source for live provider/account limit state when supported by the provider.

Use it for:

- provider/account identity;
- subscription/plan metadata;
- quota windows;
- used percentage;
- remaining percentage;
- reset timestamps;
- provider health/status;
- credits when exposed;
- cost/budget values when exposed;
- fresh/stale provider snapshots.

CodexBar is the source for provider-issued quota/reset facts. Vesper must not infer a provider reset time when CodexBar or the underlying provider did not supply one.

### ccusage

Primary source for broad historical accounting where supported.

Use it for:

- historical token consumption;
- daily and longer-period usage totals;
- historical cost/accounting;
- session/day aggregation already available from ccusage;
- cross-checking Vesper's own bounded local history.

Do not reimplement ccusage calculations unless a Vesper-specific metric is genuinely missing.

### TurnLens

Primary source for fine-grained per-turn telemetry for supported Codex and Claude Code activity.

Use it for:

- per-turn input/output token counts;
- per-turn API-equivalent cost;
- turn timestamps;
- model/runtime attribution when exposed;
- session-level fine-grained activity derived from turns.

TurnLens is especially important for Vibe Coding Activity because process uptime alone cannot tell whether useful coding interaction occurred.

### Agent Cockpit

Primary source for live coding-agent process context.

Use it for:

- runtime/agent identity;
- repository/project;
- working directory;
- process start/stop state;
- PID where useful;
- branch;
- dirty/clean state;
- elapsed runtime;
- live process inventory.

Agent Cockpit process uptime is **not** itself human active coding time.

### CCCC / Agent Teams

When the optional CCCC backend is active, orchestration events may enrich analytics with:

- team;
- role;
- task assignment;
- task start/completion;
- hand-offs;
- agent start/stop/restart events;
- coordinator/worker relationships.

CCCC is an enrichment source, not the canonical provider quota source.

### Git repositories

Git metadata may be used only as supporting project activity evidence:

- commit timestamps;
- branch/repository attribution;
- changed-file counts;
- diff statistics where useful.

A commit is not equivalent to a unit of coding time. Git history must never be used alone to claim hours worked.

## normalization layer

The AI analytics backend should merge all available sources into a backend-neutral event/history model rather than expose CodexBar, ccusage or TurnLens structures directly to QML.

Conceptual event model:

```text
AiActivityEvent
├── timestamp
├── source                 codexbar | ccusage | turnlens | agent-cockpit | cccc | git
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

Missing attribution remains unknown. Do not fabricate a model, project, agent, token value or cost to make a chart complete.

Deduplicate overlapping observations. For example, the same Codex turn seen by TurnLens and ccusage must not be counted twice merely because two collectors observed it.

## quota and reset analytics

Each provider may expose multiple independent windows. Keep all of them.

```text
Codex
5-hour window       42% used    reset 18 Aug 19:40
Weekly allowance    71% used    reset 21 Aug 11:54
Credits             12.40 remaining
```

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

Do not collapse a short rolling window, weekly allowance, monthly credits and monetary budget into one fake percentage.

Health classification may use the most constrained reliable window:

- `critical`: provider reports critical/error or <= 10% reliable remaining;
- `warning`: <= 25% reliable remaining;
- `healthy`: otherwise;
- `unknown`: no trustworthy quota information.

`unknown` is not `healthy`.

## Vibe Coding Activity

The AI page should include a GitHub-contribution-style activity heatmap showing how much AI-assisted coding activity occurred each day.

The default heatmap period is the most recent 12 months, arranged as weeks x weekdays in the familiar contribution-graph layout.

Each cell represents one local calendar day. Intensity defaults to **observed active coding minutes**, with optional display modes for:

- active coding time;
- agent-hours;
- tokens;
- turns;
- estimated/API-equivalent cost;
- completed agent tasks.

Example concept:

```text
Vibe Coding Activity                         Last 12 months

Mon  ░ ░ ▒ ▒ ░ ░ ▓ ▓ ░ ░ ░ ▒ ▒ ▒ ░ ...
Tue  ░ ▒ ▒ ░ ░ ░ █ ▓ ░ ▒ ░ ░ ▓ ▒ ░ ...
Wed  ░ ░ ▓ ▒ ░ ▒ █ █ ▒ ▒ ░ ▓ ▓ ░ ░ ...
Thu  ▒ ▒ ▒ ░ ░ ░ ▓ █ ▒ ░ ░ ▒ ▒ ░ ░ ...
Fri  ░ ░ ▒ ░ ░ ░ ▒ ▓ ░ ░ ░ ░ ▒ ░ ░ ...
Sat  ░ ░ ░ ░ ░ ░ ░ ▒ ░ ░ ░ ░ ░ ░ ░ ...
Sun  ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ...

Today
3h 18m active · 5.7 agent-hours · 84 turns · 1.9M tokens
```

Selecting a day opens the daily drill-down instead of treating the heatmap as decoration.

### daily drill-down

A day should be able to show, when attributable:

```text
18 August 2026

Observed active coding       3h 18m
Agent wall-clock             4h 06m
Agent-hours                  5h 43m
Turns                        84
Tokens                       1.9M
API-equivalent cost          $12.48
Commits                      7
Repositories                 3

By model
GPT-5.x / Codex              1h 42m
Claude                       58m
Hermes model/runtime         38m

By agent
Codex worker 1               1h 31m
Codex worker 2               1h 02m
Claude reviewer              45m
Hermes research              20m

By project
vesper                       2h 14m
dreadFetch                   49m
other                        15m
```

These values are examples of layout only. The implementation must display measured values, not hard-coded sample numbers.

## time semantics

Vesper must distinguish three different time concepts.

### observed active coding time

This is the closest metric to "how many hours did I vibe code today?"

It measures periods with observed interactive coding activity, primarily from TurnLens/ccusage turn timestamps and attributable agent interaction events.

Do not count the entire lifetime of a terminal or agent process as active coding.

Recommended sessionization:

1. order attributable interaction/turn events by timestamp;
2. group events into a session while gaps remain below an idle threshold;
3. close the active interval after prolonged inactivity;
4. cap contribution from large silent gaps;
5. merge overlapping active intervals before calculating human-facing active time.

Default idle threshold: **15 minutes**. Keep it configurable.

This metric should be labelled **Observed active coding** or **Vibe coding activity**, because it is inferred from observed activity and is not a biometric timesheet.

### agent wall-clock

Elapsed wall time during which at least one coding agent was running/working.

Overlapping agents are unioned. Two agents running from 10:00 to 11:00 produce one hour of wall-clock activity, not two.

This is useful for answering "how long was the AI work session running?"

### agent-hours

Sum of attributable runtime across agents. Parallel work counts independently.

Two agents working from 10:00 to 11:00 produce **2 agent-hours**.

This is useful for comparing orchestration intensity and team parallelism, but must never be presented as human hours worked.

## model and agent usage time

For each session/event with reliable attribution, aggregate active/runtime duration by:

- provider;
- model;
- runtime;
- agent;
- Agent Team role;
- project/repository.

Useful questions the UI should answer:

- How many hours did I actively vibe code today/this week/this month?
- Which model did I use most?
- How many agent-hours did Codex consume?
- How long did Claude spend reviewing versus implementing?
- Which repository received the most AI-assisted work?
- Which Agent Team generated the most parallel runtime?
- Which model produced the most turns/tokens/cost?
- How much usage happened before the current quota reset?

Model time must be marked `unknown` when the underlying collector cannot reliably determine the model.

## charts and statistics

The detailed analytics view should support:

### activity

- contribution-style 12-month heatmap;
- active coding minutes by day;
- agent-hours by day;
- turns by day;
- tokens by day;
- cost by day;
- weekday/time-of-day activity distribution;
- current streak;
- longest streak;
- most active day;
- average active time per active day.

### model breakdown

- active time by model;
- turns by model;
- tokens by model;
- cost by model;
- model share over 7d / 30d / 90d;
- model usage trend over time.

### agent breakdown

- runtime by agent;
- active/working/idle/error time when state is observable;
- tasks completed;
- average task duration;
- reviewer versus implementer time;
- parallel-agent peak;
- agent-hours by team.

### project breakdown

- active coding time by repository;
- turns/tokens/cost by repository;
- agent-hours by repository;
- commits as supporting activity metadata;
- most active branches;
- per-project daily/weekly trend.

### quota correlation

The UI may correlate local activity with provider quota consumption, for example:

```text
Codex weekly quota
71% used

Since previous reset
Observed active coding       12h 34m
Codex agent-hours            19h 12m
TurnLens turns               463
Tokens                       8.7M
```

Correlation does not imply that Vesper knows the provider's internal quota formula. Do not claim a direct token-to-quota conversion unless the provider explicitly defines one.

## day, week and month summaries

Provide fast presets:

```text
Today · 7d · 30d · 90d · 1y · All
```

Example summary cards:

```text
This week
14h 22m observed active coding
26h 10m agent-hours
412 turns
7.8M tokens
$34.20 equivalent/API cost
3 projects
4 models
```

Costs must preserve their source semantics. Provider-billed cost and TurnLens API-equivalent cost are not necessarily the same and must not be silently added together.

## cost semantics

Maintain distinct fields for:

- provider-reported billed/credit cost;
- ccusage historical cost/accounting;
- TurnLens API-equivalent cost estimate.

The UI must label estimates explicitly, e.g. **API-equivalent cost** or **estimated cost**, and must not present them as an invoice.

## local history store

Vesper may maintain a bounded local analytics store to join sources and render long-term trends efficiently.

The store should persist normalized observations/events, not raw secret-bearing provider responses.

Requirements:

- local-only by default;
- no API keys or prompts stored merely for analytics;
- bounded retention or compaction;
- deterministic daily aggregation;
- timezone-aware local-day boundaries;
- deduplication across collectors;
- source provenance;
- schema versioning;
- rebuildable aggregates where source history still exists.

Suggested rollups:

```text
raw/fine events       short retention
hourly aggregates     medium retention
daily aggregates      long retention
```

The contribution heatmap should read daily aggregates rather than repeatedly scanning raw turn history.

## attribution and confidence

Every derived metric should retain enough provenance to explain itself.

Suggested confidence classes:

- `exact`: explicit source timestamps/identity;
- `attributed`: joined reliably across sources;
- `estimated`: inferred from activity/sessionization;
- `unknown`: insufficient data.

Examples:

- CodexBar reset timestamp supplied by provider: `exact`;
- TurnLens turn linked to a repository by matching an active Agent Cockpit session: `attributed`;
- observed active coding duration produced by 15-minute idle sessionization: `estimated`;
- model not exposed by collector: `unknown`.

The UI should not drown the normal view in confidence labels, but diagnostics must make provenance inspectable.

## concurrency rules

Parallel agents create several useful metrics and one common trap.

```text
10:00-11:00 Codex worker A
10:00-11:00 Codex worker B
```

Correct results:

```text
agent wall-clock   1h
agent-hours        2h
```

Human/observed active coding time is calculated independently from interaction activity and cannot exceed wall time simply because several agents ran in parallel.

## privacy

Analytics remain local unless a specific provider query is required to refresh provider status.

Do not upload:

- repository paths;
- filenames;
- prompts;
- task text;
- commit content;
- TurnLens history;
- ccusage history;
- local activity aggregates;

to an external analytics service.

The feature is a personal local control-plane dashboard, not telemetry collection for Vesper maintainers.

## implementation priority

1. Keep CodexBar as the existing live quota/reset source.
2. Add ccusage and TurnLens adapters to the Vesper AI analytics backend.
3. Normalize/deduplicate the three usage sources.
4. Move detailed quota/provider cards from Hub into AI -> Usage & Analytics.
5. Reduce Hub to compact quota/reset/activity summary.
6. Persist daily/hourly local aggregates.
7. Add the 12-month Vibe Coding Activity heatmap.
8. Add daily drill-down and model/agent/project breakdowns.
9. Enrich attribution with Agent Cockpit and optional CCCC events.
10. Add estimates/trends only after source provenance and deduplication tests are reliable.

The core rule is: **CodexBar tells Vesper about live provider limits, ccusage provides broad historical accounting, TurnLens provides fine-grained per-turn telemetry, and Vesper joins them into one local analytics model without pretending that process uptime equals coding time.**
