# Hermes automation fleet

Vesper uses Hermes cron as the single recurring automation scheduler.

The design separates deterministic collection, bounded agent reasoning, fan-in synthesis, notifications, and long-term knowledge. It deliberately avoids a second systemd/GitHub Actions schedule for the same work.

## Why the fleet is split

Hermes cron agent runs are isolated sessions and do not receive normal built-in memory. They also have a bounded agent execution window. Large recurring research therefore should not be implemented as one prompt that tries to browse hundreds of pages sequentially.

Vesper uses this pattern instead:

```text
parallel deterministic collection
        ↓
full candidate pool on disk
        ↓
bounded prompt sample
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

GitHub, Reddit and Linux.do collectors fan out network requests in parallel with short per-request deadlines. They may inspect hundreds of candidates while emitting at most roughly 80k characters of valid JSON into the agent prompt. The complete pool remains under `$VESPER_RESEARCH_STATE_DIR/candidate-pools/` for deeper inspection when necessary.

This keeps the agent's bounded cron turn focused on verification, ranking and synthesis rather than mechanical enumeration or oversized context.

## Daily schedule

| Time | Job | Delivery | Purpose |
|---|---|---|---|
| 07:50 | Hermes Skill Integrity Watch | notify on change | detect enabled jobs whose attached skills disappeared |
| 08:20 | Unknown Frontier AI — GitHub Scout | local | low-attention repos/issues and deep verification |
| 08:30 | Unknown Frontier AI — Reddit Scout | local | low-score posts/comments and niche technical findings |
| 08:40 | Unknown Frontier AI — X Scout | local | low-engagement builder/researcher signals |
| 08:55 | Free AI Radar | local | Linux.do-first legitimate free AI discovery |
| 09:05 | Unknown Frontier AI — Synthesis | local | fan-in of three frontier scouts + learned discovery seeds |
| 09:20 | Daily Agenda | local | important current developments, independent of obscurity |
| 10:00 | Morning Check | notify | final daily briefing: projects, todos, agenda, frontier, free AI |
| 19:15 | Upstream Edge Radar | notify on material change | native `monitor_script` gate, then PR/issue/commit analysis only after snapshot changes |
| 23:30 | Second Brain Reflection | local | direct daily fan-in + Obsidian consolidation + research-derived skill candidates |

`Vesper Health Watch` runs every three hours as a no-agent watchdog and emits only on a state transition. `Hermes Cron Retention` runs Monday at 03:15 and prunes ended cron-source sessions, cron output and saved candidate pools older than 30 days.

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

### Adaptive discovery loop

Unknown Frontier is not limited to a permanently hard-coded query list. The synthesis job maintains a bounded learned-search file:

```text
$VESPER_RESEARCH_STATE_DIR/frontier-discovery-seeds.json
```

It may contain compact lists such as:

```json
{
  "githubQueries": [],
  "githubIssueQueries": [],
  "redditQueries": [],
  "redditSubreddits": [],
  "linuxdoQueries": [],
  "xQueries": [],
  "updatedAt": "..."
}
```

Only search routes that produced real downstream value or expose a promising adjacent frontier should survive. Strong existing seeds are retained, duplicate/low-signal routes decay, and each list stays bounded. This file is inert research state, not executable code, cron state, or an active skill.

The next day's GitHub, Reddit and Linux.do collectors consume the relevant learned seeds automatically. X uses `xQueries` as optional expansion hints while preserving explicit exploration budget for completely new vocabulary/authors/communities.

```text
useful discovery
      ↓
learned query/source edge
      ↓
bounded seed state
      ↓
wider next-day collector
      ↓
measured downstream value
```

GitHub search breadth is deliberately rate-limit aware: use fewer high-yield searches with up to 100 results each, then spend agent budget on graph expansion and verification instead of firing dozens of near-duplicate Search API calls.

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

High-volume research uses three layers:

- `research-collectors.py` — parallel source-specific fan-out for GitHub, Reddit and Linux.do
- `bounded-collector.py` — saves the full JSON pool and emits a bounded sample plus `fullPoolPath`
- the Hermes scout agent — filters, opens, verifies and ranks the strongest candidates

Low-volume deterministic jobs continue to use `automation-support.py` directly.

Installed high-volume entrypoints:

- `frontier-github-collect.py`
- `frontier-reddit-collect.py`
- `free-ai-linuxdo-collect.py`

Low-volume modes:

- `vesper-health-watch.py`
- `vesper-skill-integrity-watch.py`
- `project-inventory.py`
- `ai-usage-snapshot.py`

Dedicated scripts:

- `upstream-edge-monitor.py` — emits a stable, timestamp-free upstream-head snapshot; Hermes stores/hashes the monitor output itself and skips the agent when the exact snapshot is unchanged
- `vesper-cron-retention.py` — deletes cron output and candidate-pool JSON older than 30 days, then runs `hermes sessions prune --older-than 30 --source cron --yes`

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

The reconciler uses Hermes' own `cron.jobs` API so locking, schedule parsing and derived fields remain canonical. It also uses Hermes' native `monitor_script` field instead of inventing a second change-detection protocol.

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
- updates changed prompts/schedules/toolsets/scripts/monitor scripts
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

- Vesper research state for deduplication/source/heuristic/adaptive-search state
- Hermes cron output for immediate pipeline fan-in
- Obsidian for durable long-form knowledge and relationships
- Hermes normal memory for small cross-session facts outside cron
- skill drafts for reusable procedures awaiting review

`Second Brain Reflection` receives the latest Frontier synthesis, Free AI Radar, Daily Agenda, Morning Check and Upstream Edge output directly via `context_from`. This avoids asking an isolated nightly cron session to rediscover its own daily inputs from conversational memory.

Hermes' bundled `obsidian` skill and Vesper's `vesper-obsidian-second-brain` skill are loaded together for nightly consolidation.

## Noise and cost controls

- Raw scouts and intermediate synthesis are local-only.
- Only final daily/weekly reports and changed alarms notify.
- No-agent watchdogs consume zero model tokens.
- Upstream Edge Radar uses Hermes' native `monitor_script` hash gate and spends zero model tokens when tracked upstream heads are byte-identical to the previous tick.
- Source-specific `enabled_toolsets` avoid loading every tool schema into every cron turn.
- Parallel collectors front-load breadth while bounded prompt samples protect the reasoning context.
- Full candidate pools, old cron sessions and cron output are pruned after 30 days.
- Adaptive seeds stay bounded so discovery can evolve without turning the crawler into an unbounded self-expanding query storm.

## Validation

Useful checks:

```bash
vesper-hermes-cron-sync
hermes cron list
hermes cron status
hermes skills list
vesper-doctor --json
```

A healthy fleet has one job per managed name, no duplicate `Sabah check`, valid skills, local scout delivery, correct fan-in IDs, silent healthy watchdogs, a native monitor-gated upstream radar, sandbox-valid regular script files and a single Hermes scheduler owner.
