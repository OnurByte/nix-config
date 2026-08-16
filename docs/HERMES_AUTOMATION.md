# Hermes automation fleet

Vesper uses Hermes cron as the single recurring automation scheduler.

The design separates deterministic collection, bounded agent reasoning, fan-in synthesis, notifications, and long-term knowledge. It deliberately avoids a second systemd/GitHub Actions schedule for the same work.

## Why the fleet is split

Hermes cron agent runs are isolated sessions and do not receive normal built-in memory. They also have a bounded agent execution window. Large recurring research therefore should not be implemented as one prompt that tries to browse hundreds of pages sequentially.

Vesper uses this pattern instead:

```text
cheap deterministic collection
        ↓
source-specific agent triage
        ↓
independent local reports
        ↓
context_from fan-in
        ↓
Morning Check / Weekly Review
        ↓
Obsidian second-brain consolidation
```

The collectors can inspect hundreds of candidates cheaply. The agent spends its time on verification, ranking and synthesis.

## Daily schedule

| Time | Job | Delivery | Purpose |
|---|---|---|---|
| 07:50 | Hermes Skill Integrity Watch | notify on change | detect enabled jobs whose attached skills disappeared |
| 08:20 | Unknown Frontier AI — GitHub Scout | local | low-attention repos/issues and deep verification |
| 08:30 | Unknown Frontier AI — Reddit Scout | local | low-score posts/comments and niche technical findings |
| 08:40 | Unknown Frontier AI — X Scout | local | low-engagement builder/researcher signals |
| 08:55 | Free AI Radar | local | Linux.do-first legitimate free AI discovery |
| 09:05 | Unknown Frontier AI — Synthesis | local | fan-in of three frontier scouts |
| 09:20 | Daily Agenda | local | important current developments, independent of obscurity |
| 10:00 | Morning Check | notify | final daily briefing: projects, todos, agenda, frontier, free AI |
| 19:15 | Upstream Edge Radar | notify on material change | cheap upstream-head gate, then PR/issue/commit analysis only after a change |
| 23:30 | Second Brain Reflection | local | Obsidian consolidation and research-derived skill candidates |

`Vesper Health Watch` runs every three hours as a no-agent watchdog and emits only on a state transition. `Hermes Cron Retention` runs Monday at 03:15 and prunes ended cron-source sessions and cron output older than 30 days.

## Weekly schedule

Sunday:

| Time | Job | Delivery |
|---|---|---|
| 10:30 | User Pain Miner | local |
| 10:40 | Project Archaeologist | local |
| 10:50 | AI Usage Economist | local |
| 11:00 | Skill Evolution Review | local |
| 12:00 | Weekly Intelligence Review | notify |

The final review consumes the latest completed weekly outputs through `context_from` and turns them into decisions rather than another long digest.

## Discovery lanes

### Unknown Frontier AI

Core question:

```text
what useful AI thing exists outside the current knowledge map?
```

GitHub, Reddit and X are separate scouts so one platform's ranking/popularity bias does not dominate the result. GitHub and Reddit have broad deterministic pre-collectors. X uses Hermes `x_search` directly.

The frontier score rewards unknown-to-user value, utility, novelty, evidence, technical density, early-signal value and information gain while penalizing hype and duplication.

### Daily Agenda

Core question:

```text
what important thing happened or changed today?
```

Mainstream importance is allowed. Hidden-gem scoring is not applied.

### Free AI Radar

Linux.do is a first-class discovery source. Findings are verified outward against primary repositories/docs/providers and must state the real quota/catch.

Unauthorized access, stolen/shared credentials, leaked keys, payment bypass, abusive account creation and service-restriction evasion are excluded.

## Deterministic collectors and gates

`home/yargc/hermes/automation-support.py` is copied under several script names. Its behavior is selected from its filename.

Shared support modes:

- `frontier-github-collect.py`
- `frontier-reddit-collect.py`
- `free-ai-linuxdo-collect.py`
- `vesper-health-watch.py`
- `vesper-skill-integrity-watch.py`
- `project-inventory.py`
- `ai-usage-snapshot.py`

Dedicated scripts:

- `upstream-edge-monitor.py` — stores the last tracked upstream-head snapshot and emits `wakeAgent=false` when nothing changed, so the Upstream Edge Radar spends no model tokens on unchanged ticks
- `vesper-cron-retention.py` — deletes old cron output and runs `hermes sessions prune --older-than 30 --source cron --yes`

Watchdogs keep small state under `$VESPER_RESEARCH_STATE_DIR` so unchanged failures do not repeatedly notify.

### Why scripts are copied instead of linked

Hermes resolves cron script paths with `realpath` and requires the resolved path to stay under `~/.hermes/scripts/`. A normal Home Manager `home.file.source` becomes a symlink into `/nix/store`, which Hermes correctly treats as escaping its script sandbox.

For that reason the Nix modules use Home Manager activation steps to copy Nix-owned scripts as real regular files into `~/.hermes/scripts/`. The desired fleet Python file itself is not executed through the Hermes script sandbox and may remain a normal Home Manager source link under `~/.config/vesper/hermes/`.

## Runtime state vs declarative state

The Nix repository defines desired jobs in:

```text
home/yargc/hermes/automation-fleet.py
```

Hermes still owns mutable runtime state in:

```text
~/.hermes/cron/jobs.json
~/.hermes/cron/output/
```

Do not make `jobs.json` a Home Manager file. Hermes updates run counters, next-run times, errors, delivery state and internal metadata there.

The reconciler uses Hermes' own `cron.jobs` API so locking, schedule parsing and derived fields remain canonical.

## Apply/reconcile

After changing this repository:

```bash
nh os switch
vesper-hermes-cron-sync
vesper-hermes-cron-sync --apply
hermes cron list
```

The first sync command is a dry-run.

The reconciler:

- matches managed jobs by English name
- recognizes the legacy `Sabah check` alias
- preserves the existing job ID during that migration
- discovers the existing notification destination without committing a Telegram chat ID to the public repository
- creates missing jobs
- updates changed prompts/schedules/toolsets/scripts
- resolves fan-in names to canonical upstream job IDs before storing `context_from`
- preserves an operator-paused job instead of silently re-enabling it

If no messaging origin can be found, notification jobs fall back to local delivery. Override locally with:

```bash
export VESPER_HERMES_DELIVER='telegram:<chat-id>'
vesper-hermes-cron-sync --apply
```

Optional model pinning:

```bash
export VESPER_HERMES_CRON_PROVIDER='xai-oauth'
export VESPER_HERMES_CRON_MODEL='grok-4.5'
vesper-hermes-cron-sync --apply
```

When unset, jobs use Hermes' configured cron/default inference routing.

## Memory and second brain

Cron sessions must not rely on conversational memory from earlier runs.

Use:

- Vesper research state for deduplication/source/heuristic state
- Hermes cron output for immediate pipeline fan-in
- Obsidian for durable long-form knowledge and relationships
- Hermes normal memory for small cross-session facts outside cron
- skill drafts for reusable procedures awaiting review

Hermes' bundled `obsidian` skill and Vesper's `vesper-obsidian-second-brain` skill are loaded together for nightly consolidation.

## Noise and cost controls

- Raw scouts and intermediate synthesis are local-only.
- Only final daily/weekly reports and changed alarms notify.
- No-agent watchdogs consume zero model tokens.
- Upstream Edge Radar uses a `wakeAgent` pre-run gate and spends zero model tokens when tracked upstream heads are unchanged.
- Source-specific `enabled_toolsets` avoid loading every tool schema into every cron turn.
- Collector scripts front-load breadth so the agent does not waste the bounded reasoning window on mechanical enumeration.
- Old cron sessions and output are pruned after 30 days.

## Validation

Useful checks:

```bash
vesper-hermes-cron-sync
hermes cron list
hermes cron status
hermes skills list
vesper-doctor --json
```

A healthy fleet has one job per managed name, no duplicate `Sabah check`, valid skills, local scout delivery, correct fan-in IDs, silent healthy watchdogs, sandbox-valid regular script files and a single Hermes scheduler owner.
