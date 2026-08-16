# Hermes automations

Vesper keeps Hermes cron as the only recurring scheduler.

Cron does not perform long research inside the gateway process. Every Vesper cron entry is a short `no_agent` script:

```text
Hermes cron
    ↓
~/.hermes/scripts/vesper-<job>.sh
    ↓
vesper-hermes-automations trigger <job>
    ├─ watchdog → inspect local state → empty stdout when healthy
    └─ research → systemd-run --user → vesper-hermes-automations execute <job>
                                      ↓
                                 Hermes one-shot
                                      ↓
                           persistent state + briefing
```

This keeps the cron tick cheap and bounded while long research runs outside the gateway. `systemd-run` is only an execution container for a triggered run. It is not another timer and does not own recurrence.

## declarative registry

Schedules live in `home/yargc/hermes-jobs.nix`.

Home Manager writes the registry to:

```text
~/.config/vesper/hermes-jobs.json
```

and generates one short wrapper per job under:

```text
~/.hermes/scripts/
```

During Home Manager activation:

```bash
vesper-hermes-automations sync-cron --prune
```

reconciles `vesper:*` jobs through Hermes' own CLI. It does not delete unrelated user-created Hermes jobs.

The old `sabah-check-deliver.sh` and `morning-check-deliver.sh` paths remain as tiny compatibility aliases. They dispatch the new Morning Check runner instead of running a long model call inside Hermes' script timeout.

## daily jobs

| time | job | behavior |
|---|---|---|
| `08:30` | `frontier-daily` | GitHub, Reddit and X scouts in parallel, then one verified unknown-frontier synthesis |
| `08:45` | `free-ai-radar` | legitimate free AI, free tiers, self-hosted alternatives and Linux.do findings |
| `09:30` | `agenda` | compact important current agenda |
| `10:00` | `morning-check` | local projects + todos + persistent research, delivered to Telegram |
| `15:00` | `upstream-edge-radar` | early breaking changes and capabilities in Vesper upstreams |
| `23:30` | `second-brain-dream` | deduplicate and promote durable knowledge into the Obsidian workflow |

The frontier scouts are one orchestration job instead of three independent cron entries. Synthesis waits for the scouts that actually finished, so there is no `context_from` same-tick race and no arbitrary five-minute timing dependency.

## watchdogs

`vesper-health-watch` runs every three hours.

It reads `vesper-doctor --json` and stays completely silent while healthy. It only emits the warning checks when something is wrong.

`cron-integrity-watch` runs every six hours.

It verifies:

- every declarative Vesper cron job exists and is enabled
- schedules and script paths did not drift
- referenced skills still resolve through Hermes/Vesper skill roots
- Hermes cron/gateway status does not report a stopped or stalled scheduler

Both are `no_agent` jobs. Healthy ticks use no model.

## weekly jobs

Sunday jobs are staggered so they do not all compete for the provider or laptop at once.

| time | job |
|---|---|
| `11:00` | `user-pain-miner` |
| `12:30` | `project-archaeologist` |
| `14:00` | `skill-evolution-review` |
| `15:30` | `ai-usage-economist` |

`user-pain-miner` clusters recurring complaints across agent/Linux tooling and turns strong clusters into concrete project or skill opportunities.

`project-archaeologist` scans local Git repositories and surfaces forgotten branches, dirty work and abandoned experiments that are actually worth revisiting.

`skill-evolution-review` reads research heuristics and `skill-drafts`. It only recommends promotion when there is repeated evidence. It never edits active skills automatically.

`ai-usage-economist` uses whatever local accounting surfaces are available (`ccusage`, CodexBar and TurnLens) and separates measured usage from model-routing suggestions.

## state

Research state remains outside Hermes cron sessions:

```text
~/.local/state/vesper/research/
├── unknown-frontier-ai/
│   ├── scouts/
│   ├── known.json
│   ├── candidateSources.json
│   ├── heuristics.json
│   └── openQuestions.json
├── agenda/
├── free-ai-radar/
├── upstream-edge-radar/
├── user-pain-miner/
├── project-archaeologist/
├── skill-evolution-review/
├── ai-usage-economist/
├── second-brain-dream/
├── locks/
└── runs/
```

Briefings remain under:

```text
~/.local/share/vesper/briefings/
```

and feed the Caelestia Hermes badge/inbox.

## commands

```bash
# inspect the declarative schedule
vesper-hermes-automations jobs

# reconcile Hermes jobs.json now
vesper-hermes-automations sync-cron --prune

# dispatch a long job through a transient user service
vesper-hermes-automations dispatch frontier-daily

# run synchronously for debugging
vesper-hermes-automations execute free-ai-radar

# existing briefing inbox runtime
vesper-hermes status
vesper-hermes list
vesper-hermes inbox
```

For scheduler-level status and manual cron firing use Hermes directly:

```bash
hermes cron status
hermes cron list
hermes cron run <job>
```

## why not `context_from`

Hermes supports `context_from`, but it reads the most recent completed upstream output and does not wait for another job that started in the same tick.

The unknown-frontier pipeline needs a real join. Vesper therefore starts the three scouts inside one `frontier-daily` run and synthesizes only after the scout futures complete. Persistent scout JSON stays available for inspection and recovery.

## why not automation blueprints here

Automation Blueprints are useful for portable opt-in skills that a user chooses to schedule.

These jobs are machine-owned Vesper configuration. Their schedule, wrapper and state paths belong in Nix so a rebuild can reproduce them. A mature Vesper workflow can still be exported later as a blueprint for sharing, but blueprints are not used as a second source of truth for this workstation.
