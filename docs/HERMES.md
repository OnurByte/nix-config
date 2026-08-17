# Hermes automations

Vesper uses **Hermes cron as the only recurring scheduler** and `vesper-hermes-automations` as the only research execution owner.

The recurring path is:

```text
Hermes cron
    ↓
short no-agent wrapper
    ↓
vesper-hermes-automations trigger <job>
    ├─ watchdog → deterministic check → edge-triggered stdout
    ├─ monitor  → deterministic change gate → dispatch only when changed
    └─ dispatch → systemd-run --user → vesper-hermes-automations execute <job>
                                      ↓
                              long Hermes one-shot
                                      ↓
                     persistent state + briefing inbox
                                      ↓
                              Morning Check / Obsidian
```

`systemd-run --user` is a transient execution container, not another timer. It lets research run longer than Hermes cron's bounded agent execution window without creating a second scheduler.

`vesper-hermes` is the briefing/inbox client. Its legacy `run` and `daily` commands are compatibility shims that delegate to `vesper-hermes-automations`; there is no second research engine.

## declarative registry

Schedules live in `home/yargc/hermes-jobs.nix` and are rendered to:

```text
~/.config/vesper/hermes-jobs.json
```

Home Manager installs one physical wrapper per job under:

```text
~/.hermes/scripts/vesper-<job>.sh
```

They are real files, not Home Manager symlinks, because Hermes resolves script paths before enforcing containment under `~/.hermes/scripts/`; a symlink into `/nix/store` would be rejected as a script-directory escape.

After activation Vesper runs:

```bash
vesper-hermes-automations sync-cron --prune
```

The reconciler mutates only machine-owned `vesper:*` jobs through Hermes' own CLI. It preserves unrelated user-created cron jobs and migrates the legacy Morning Check aliases.

## daily jobs

| time | job | mode | purpose |
|---|---|---|---|
| `08:30` | `frontier-daily` | dispatch | wide Unknown Frontier AI discovery and synthesis |
| `08:45` | `free-ai-radar` | dispatch | Linux.do-first legitimate free AI radar |
| `09:30` | `agenda` | dispatch | important current agenda, independent of obscurity |
| `10:00` | `morning-check` | dispatch | final Telegram brief from local state + persistent reports |
| `15:00` | `upstream-edge-radar` | monitor | zero-token upstream head gate; research only after change |
| `23:30` | `second-brain-dream` | dispatch | Obsidian consolidation and skill-draft learning |

`vesper-health-watch` runs every three hours and `cron-integrity-watch` every six hours. Both are zero-token watchdogs and stay silent while their state is unchanged and healthy.

## Unknown Frontier AI

The core question is:

```text
what useful AI thing exists outside the user's current knowledge map?
```

This is intentionally not a popularity feed and not the Daily Agenda.

### broad deterministic funnel

Before the expensive scout reasoning begins, Vesper performs cheap parallel collection:

- **GitHub** — recent small repositories plus active issues, with up to hundreds of candidates
- **Reddit** — recent/low-attention posts and learned niche subreddit/query routes
- **X** — uses Hermes' native `x_search` toolset directly rather than pretending normal web search is X search

GitHub and Reddit full candidate pools are persisted under:

```text
~/.local/state/vesper/research/candidate-pools/
```

Only a bounded ~80k JSON sample is injected into a scout prompt. The complete pool remains on disk for deeper inspection, and old pools are pruned after 30 days.

The agent therefore spends its long worker window on **triage, verification, graph expansion and synthesis**, not mechanical enumeration.

### scout + synthesis

GitHub, Reddit and X scouts run in parallel inside one `frontier-daily` worker. The synthesis stage waits for the scouts that actually completed, tolerates partial failures, deduplicates overlapping claims and ranks findings by:

- information gain
- usefulness
- novelty
- evidence
- technical density
- early-signal value
- low-visibility context
- independence / corroboration

Low engagement is only a discovery hint. Obscurity without utility is discarded.

### adaptive discovery seeds

Synthesis may update this inert state file:

```text
~/.local/state/vesper/research/unknown-frontier-ai/discovery-seeds.json
```

Supported bounded seed classes are:

```text
githubQueries
githubIssueQueries
redditQueries
redditSubreddits
linuxdoQueries
xQueries
```

Only routes that produced downstream value or expose a promising adjacent frontier should persist. Duplicate/hype-heavy routes should decay. Seeds are data only: no credentials, shell commands or executable payloads are accepted, and they never auto-modify cron or active skills.

## Free AI Radar

Linux.do is a first-class discovery surface rather than a single search result.

The deterministic collector inspects multiple latest pages plus a bounded set of Chinese/English AI/free/API queries, including learned query seeds. The agent then verifies promising findings outward against official providers, repositories, releases, docs or authors.

Useful findings must explain what is actually free, quota/limit/catch, compute requirement, expiry/uncertainty and why it matters.

Unauthorized credential/account/payment/service-restriction abuse is excluded.

## Daily Agenda

Agenda answers a different question:

```text
what important thing happened or changed today?
```

Mainstream importance is allowed. It ranks by importance, recency, consequence and relevance rather than hidden-gem obscurity.

Morning Check keeps `Agenda`, `Unknown Frontier AI` and `Free AI Radar` as separate sections instead of mixing their scoring philosophies.

## Upstream Edge Radar

The 15:00 cron tick performs a deterministic GitHub head snapshot for the Vesper upstream set. If enough repositories cannot be checked, it fails quiet instead of spending model tokens on a network outage.

If the healthy snapshot is byte-equivalent to the previous one, nothing is dispatched. When tracked heads move, Vesper records the changed repositories under the upstream lane state and launches the long research worker.

Tracked projects include Hermes Agent, llm-agents.nix, nixpkgs, Home Manager, Hyprland, Caelestia, Zen/Helium integration flakes, CodexBar, Monero and Cuprate.

## skills and toolsets

Research prompts do not merely *mention* skills. The one-shot CLI explicitly preloads them:

```text
research jobs      → --skills hermes-research-radar
second brain       → --skills obsidian,vesper-obsidian-second-brain
```

Toolsets are also explicit:

```text
GitHub/Reddit/general web research → web
X scout / social corroboration     → web,x_search
local project/skill analysis       → file,terminal
```

This keeps every worker's available capabilities aligned with its actual task and avoids silently assuming a named skill or X search capability was loaded.

## second brain and learning

Cron runs do not rely on conversational memory for continuity. Durable state lives outside the session.

```text
research state  → ~/.local/state/vesper/research/
briefings       → ~/.local/share/vesper/briefings/
long knowledge  → Obsidian
procedures      → ~/.local/share/vesper/skill-drafts/
```

The nightly `second-brain-dream` job preloads Hermes' bundled `obsidian` skill together with `vesper-obsidian-second-brain`, deduplicates against existing notes, records durable facts/relationships/corrections/open questions and stages repeated procedures as skill drafts.

A draft is never promoted directly into the active skill tree. The weekly `skill-evolution-review` evaluates evidence and recommends retain, narrow, merge, draft or retire.

## weekly jobs

Sunday jobs are staggered so they do not all compete for provider quota or the laptop at once.

| time | job |
|---|---|
| `11:00` | `user-pain-miner` |
| `12:30` | `project-archaeologist` |
| `14:00` | `skill-evolution-review` |
| `15:30` | `ai-usage-economist` |
| `17:00` | `weekly-intelligence-review` |

`weekly-intelligence-review` does not repeat every report. It selects the highest-leverage discoveries, recurring problems, project opportunities, upstream changes, AI-cost optimizations and skill-learning decisions, then ends with the top three actions for the next week.

## watchdogs

`vesper-health-watch` reads `vesper-doctor --json` and emits only when workstation health becomes bad or materially changes.

`cron-integrity-watch` verifies:

- every declarative Vesper cron job exists
- schedule/script/enabled state did not drift
- skill references resolve through nested/symlinked Hermes/Vesper skill trees
- Hermes cron/gateway status is healthy

Watch state is fingerprinted under `~/.local/state/vesper/research/watches/`; unchanged warnings do not spam Telegram and recovery produces one recovery message.

## commands

```bash
# inspect the desired registry
vesper-hermes-automations jobs

# reconcile machine-owned Hermes cron jobs
vesper-hermes-automations sync-cron --prune

# launch a long task through a transient user service
vesper-hermes-automations dispatch frontier-daily

# run synchronously for debugging
vesper-hermes-automations execute free-ai-radar

# briefing inbox / UI
vesper-hermes status
vesper-hermes list
vesper-hermes inbox

# Hermes scheduler state
hermes cron status
hermes cron list
```

## design decisions

### why not one giant cron-agent prompt

Wide research wants many candidates and more wall-clock time than a bounded scheduler agent turn. Mechanical discovery is cheaper outside the model, while verification/synthesis benefits from the model. The split funnel gives both breadth and reasoning quality.

### why not `context_from` for frontier joining

`context_from` reads the most recent completed output; it is useful for pipelines but is not a true same-run join. Frontier scouts therefore run as futures inside one worker and synthesis waits for those futures directly.

### why not a second timer

Recurrence belongs only to Hermes cron. systemd provides transient execution isolation after a cron trigger but owns no timer and cannot independently schedule the research fleet.

### why not two research runtimes

`vesper-hermes-automations` owns execution. `vesper-hermes` owns the briefing/inbox interface and forwards legacy execution commands to the automation owner.

### why not automatic skill promotion

Research evidence is noisy. Hermes may learn heuristics and draft reusable procedures, but active skills remain reviewed Nix-owned configuration.
