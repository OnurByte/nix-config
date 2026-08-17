# Hermes automations

Vesper keeps **Hermes cron as the only recurring scheduler** and `vesper-hermes-automations` as the only research execution owner.

Cron never performs long research inside the gateway process. Every Vesper cron entry is a short `no_agent` script:

```text
Hermes cron
    ↓
~/.hermes/scripts/vesper-<job>.sh
    ↓
vesper-hermes-automations trigger <job>
    ├─ watchdog → deterministic local checks → edge-triggered stdout
    └─ dispatch → systemd-run --user → vesper-hermes-automations execute <job>
                                      ↓
                         long Hermes one-shot when needed
                                      ↓
                    persistent state + briefing + second brain
```

`systemd-run --user` is only a transient execution container after a Hermes cron trigger. It has no timer and does not own recurrence.

`vesper-hermes` is the briefing/inbox client. Its legacy `run` and `daily` commands delegate to `vesper-hermes-automations`; there is no second research engine.

## declarative registry

Schedules live in `home/yargc/hermes-jobs.nix`.

Home Manager writes the registry to `~/.config/vesper/hermes-jobs.json` and installs one physical wrapper per job under `~/.hermes/scripts/`.

The wrappers are deliberately real files, not Home Manager symlinks. Hermes resolves script paths before enforcing containment under `~/.hermes/scripts`; a Nix-store symlink would therefore be rejected as a script-directory escape at fire time.

After the wrappers are installed, Home Manager runs:

```bash
vesper-hermes-automations sync-cron --prune
```

Before mutating Hermes state, `sync-cron` validates the declarative registry against the installed task/watchdog set. It reconciles only machine-owned `vesper:*` records through Hermes' own CLI. Unrelated user-created Hermes jobs are not pruned.

The legacy `sabah-check-deliver.sh` and `morning-check-deliver.sh` paths remain physical compatibility aliases that dispatch the new Morning Check worker.

## delivery policy

Dispatch jobs use `deliver=local` at the Hermes cron layer because the cron tick launches a transient worker and intentionally prints nothing.

Morning Check explicitly sends the finished message through `hermes send`. It first discovers the preserved Telegram `origin` on the existing Morning Check/legacy Sabah Check cron record, so the personal chat ID does not need to be committed to this public repository. Override locally with:

```bash
export VESPER_HERMES_MORNING_TARGET='telegram:<chat-id>'
```

Watchdogs use `deliver=telegram` because their stdout is the alert itself. Healthy/unchanged ticks emit no stdout and use no model.

## daily pipeline

| time | job | behavior |
|---|---|---|
| `08:30` | `unknown-frontier-github` | wide GitHub hidden-gem collection + verification |
| `08:35` | `unknown-frontier-reddit` | wide Reddit recent/niche collection + verification |
| `08:40` | `unknown-frontier-x` | native `x_search` hidden-gem scout |
| `08:45` | `free-ai-radar` | Linux.do-first legitimate free AI radar |
| `09:00` | `unknown-frontier-synthesis` | fresh scout fan-in and frontier synthesis |
| `09:30` | `agenda` | important current agenda, separate from obscurity scoring |
| `10:00` | `morning-check` | projects/todos + separate Agenda / Frontier / Free AI sections, sent to Telegram |
| `15:00` | `upstream-edge-radar` | deterministic upstream change gate; model research only after movement |
| `23:30` | `second-brain-dream` | Obsidian consolidation and research-derived skill learning |

## Unknown Frontier AI

The guiding question is:

```text
what useful AI thing exists outside the user's current knowledge map?
```

This is deliberately not a popularity feed and not the Daily Agenda.

### wide deterministic funnel

GitHub and Reddit scouts do not ask the model to mechanically enumerate the web from scratch. A cheap parallel collector builds a large candidate pool first.

GitHub collection includes recent small/young repositories plus active issues across AI-agent, coding-agent, MCP, inference, compatibility, local-AI and LLM CLI neighborhoods. The funnel can retain hundreds of repository and issue candidates.

Reddit collection searches multiple recent AI/tooling terms plus learned niche query/subreddit routes and retains low-attention posts, metadata, comments count, excerpts and outbound links for triage.

Full candidate pools live under:

```text
~/.local/state/vesper/research/candidate-pools/
```

Only a bounded ~80k valid-JSON sample is injected into the scout prompt. The full pool remains on disk for deeper inspection. Candidate pools older than 30 days are pruned.

X does not pretend ordinary web search is equivalent to X search. The X scout explicitly loads Hermes' native `x_search` toolset and expands from useful builders/researchers into replies, quote-posts, repositories and neighboring accounts.

### staggered fan-out / fan-in

The three source scouts are independent cron entries so each receives its own long worker budget and they are not launched at exactly the same moment.

Each scout writes a fresh envelope under:

```text
~/.local/state/vesper/research/unknown-frontier-ai/scouts/
```

At 09:00 synthesis waits a bounded interval for missing scouts, accepts only fresh envelopes, explicitly records missing/stale sources and fails rather than silently recycling an old day if no scout is fresh.

The model sees the verified scout results and lightweight candidate-pool metadata, not the full raw corpus a second time.

The synthesis ranking rewards information gain, usefulness, novelty, evidence, technical density, early-signal value, independence/corroboration and meaningful low-visibility context. Low engagement is a discovery hint, never a quality score.

## adaptive discovery seeds

Useful search/source routes can feed the next run through:

```text
~/.local/state/vesper/research/unknown-frontier-ai/discovery-seeds.json
```

Supported bounded classes:

```text
githubQueries
githubIssueQueries
redditQueries
redditSubreddits
linuxdoQueries
xQueries
```

Only routes that produced downstream value or expose a promising adjacent frontier should persist. Generic, duplicate or hype-heavy routes should decay.

Seeds are inert data only. They never contain credentials, shell commands or executable payloads, never create new cron jobs and never auto-promote a skill.

## Free AI Radar

Linux.do is a first-class discovery surface rather than a single web-search result.

The deterministic collector samples multiple latest pages plus Chinese/English AI/free/API queries and bounded learned query routes. Promising threads/comments are then verified outward against the original provider, repository, release, docs or author.

A useful finding must state what is actually free, quota/limit/catch, local compute requirement, expiry/uncertainty and why it matters.

Unauthorized credential/account/payment/service-restriction abuse is excluded.

## Daily Agenda

Agenda answers a different question:

```text
what important thing happened or changed today?
```

Mainstream importance is allowed. It ranks by importance, recency, consequence and relevance rather than hidden-gem obscurity.

Morning Check keeps `Agenda`, `Unknown Frontier AI` and `Free AI Radar` as separate sections instead of mixing their ranking philosophies.

## Upstream Edge Radar

The 15:00 worker starts with a deterministic GitHub HEAD snapshot of the tracked Vesper upstream set.

If fewer than a healthy quorum can be checked, it exits without calling the model rather than turning a network/auth outage into an expensive false investigation.

If the healthy snapshot is unchanged, the worker exits with `modelInvoked=false`. When tracked heads move, it records the changed repositories under the upstream lane state and only then launches the research one-shot.

Tracked projects include Hermes Agent, llm-agents.nix, nixpkgs, Home Manager, Hyprland, Caelestia, Zen/Helium integration flakes, CodexBar, Monero and Cuprate.

## explicit skills and toolsets

Research workers do not merely mention a skill in prose. The Hermes one-shot CLI explicitly preloads it:

```text
research jobs  → --skills hermes-research-radar
second brain   → --skills obsidian,vesper-obsidian-second-brain
```

Toolsets are explicit too:

```text
GitHub/Reddit/general research → web
X/social research              → web,x_search
local project/skill analysis   → file,terminal
```

The contract test verifies that requested skill/toolset lists actually become CLI flags, so a future refactor cannot silently downgrade X back to web-only or stop loading the research skill.

## second brain and skill learning

Cron runs do not rely on conversational memory for continuity.

```text
research state  → ~/.local/state/vesper/research/
briefings       → ~/.local/share/vesper/briefings/
long knowledge  → Obsidian
procedures      → ~/.local/share/vesper/skill-drafts/
```

The nightly `second-brain-dream` job explicitly preloads Hermes' bundled `obsidian` skill together with `vesper-obsidian-second-brain`. It deduplicates against existing notes, promotes durable facts/relationships/corrections/open questions into the long-form graph and stages repeated procedures as drafts.

Drafts never become active skills automatically. `skill-evolution-review` evaluates evidence and recommends retain, narrow, merge, draft or retire.

The Obsidian vault resolver prefers `OBSIDIAN_VAULT_PATH` and known locations, then performs only a bounded-depth scan. It never recursively walks the entire home directory.

## watchdogs

`vesper-health-watch` runs every three hours. It checks `vesper-doctor --json`, failed user/system systemd units, root/home disk utilization and discovered Restic timer state when Restic timers exist.

`cron-skill-integrity-watch` runs every six hours. It checks desired jobs, enabled/paused state, schedules, physical script paths, `no_agent=true`, duplicate job names, referenced skills and Hermes cron/gateway health.

Both watchdogs are edge-triggered under `~/.local/state/vesper/research/watches/`: unchanged warnings are not resent and recovery emits one recovery message.

## retention

Monday 03:15 runs `cron-retention` without an LLM.

It removes:

- Hermes cron output files older than 30 days
- deterministic candidate pools older than 30 days
- worker run records older than 90 days
- ended Hermes cron-source sessions older than 30 days through `hermes sessions prune`

It does **not** delete durable briefing history or Obsidian knowledge.

## weekly jobs

Sunday jobs are staggered:

| time | job |
|---|---|
| `11:00` | `user-pain-miner` |
| `12:30` | `project-archaeologist` |
| `14:00` | `skill-evolution-review` |
| `15:30` | `ai-usage-economist` |
| `17:00` | `weekly-intelligence-review` |

`user-pain-miner` requires recurrence evidence instead of turning isolated complaints into fake trends.

`project-archaeologist` scans bounded local Git roots for unfinished work worth revisiting.

`skill-evolution-review` reviews research heuristics and `skill-drafts`; it never edits active skills automatically and does not replace Hermes Curator.

`ai-usage-economist` uses local accounting surfaces (`ccusage`, CodexBar and TurnLens) and keeps measured usage separate from routing suggestions.

`weekly-intelligence-review` consumes the week's durable briefings and selects the highest-leverage discoveries, recurring problems, project opportunities, upstream changes, AI-cost optimizations and skill decisions, ending with the top three actions for the next week.

## validation and CI

The runtime exposes:

```bash
vesper-hermes-automations validate-registry
```

The validator rejects unknown tasks/watchdogs, invalid schedule shape, duplicate cron/script identities and incorrect delivery policy before cron reconciliation.

GitHub Actions:

- parses every Nix file
- compiles every Hermes automation Python module
- evaluates `hermes-jobs.nix`
- runs the Python contract tests
- verifies scout/synthesis staggering
- verifies watchdog delivery
- verifies script-only Hermes cron wrappers
- verifies explicit `--skills` and `--toolsets` translation
- evaluates Home Manager and the Vesper module graph
- builds the complete Vesper system and key packages

## commands

```bash
vesper-hermes-automations jobs
vesper-hermes-automations validate-registry
vesper-hermes-automations sync-cron --prune
vesper-hermes-automations dispatch frontier-daily
vesper-hermes-automations execute unknown-frontier-github
vesper-hermes-automations execute unknown-frontier-synthesis
vesper-hermes-automations execute free-ai-radar

vesper-hermes status
vesper-hermes list
vesper-hermes inbox
```

Scheduler-level inspection remains native Hermes:

```bash
hermes cron status
hermes cron list
hermes cron run <job>
```

## why this shape

### why not one giant cron-agent prompt

Wide research wants many candidates and more wall-clock time than a bounded scheduler agent turn. Mechanical discovery is cheaper outside the model; verification and synthesis benefit from the model.

### why separate source scouts

GitHub, Reddit and X each get an independent worker budget and start five minutes apart. A slow or failed source does not erase the other two, and the fan-in explicitly tracks freshness.

### why not `context_from` as the join

These cron entries are script-only dispatch wrappers and long work lives outside the cron session. Fresh timestamped scout state is a more explicit same-day join primitive.

### why not a second timer

Recurrence belongs only to Hermes cron. systemd provides transient execution isolation after a cron trigger but owns no timer.

### why not two research runtimes

`vesper-hermes-automations` owns execution. `vesper-hermes` owns briefing/inbox UX and forwards legacy execution commands to the automation owner.

### why not automatic skill promotion

Research evidence is noisy. Hermes may learn heuristics and draft reusable procedures, but active skills remain reviewed Nix-owned configuration.
