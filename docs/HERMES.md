# Hermes automations

Vesper keeps Hermes cron as the only recurring scheduler.

Cron never performs long research inside the gateway process. Every Vesper cron entry is a short `no_agent` script:

```text
Hermes cron
    ↓
~/.hermes/scripts/vesper-<job>.sh
    ↓
vesper-hermes-automations trigger <job>
    ├─ watchdog → local checks → edge-triggered stdout
    └─ research → systemd-run --user → vesper-hermes-automations execute <job>
                                      ↓
                                 Hermes one-shot
                                      ↓
                           persistent state + briefing
```

`systemd-run` is only the execution container for a triggered run. It is not another timer and does not own recurrence. This keeps the Hermes cron tick cheap, zero-token for script-only triggers, and independent from long model runtime.

## declarative registry

Schedules live in `home/yargc/hermes-jobs.nix`.

Home Manager writes the registry to `~/.config/vesper/hermes-jobs.json` and installs one physical wrapper per job under `~/.hermes/scripts/`.

The wrappers are deliberately real files, not Home Manager symlinks. Hermes resolves script paths before enforcing containment under `~/.hermes/scripts`; a Nix-store symlink would therefore be rejected as a script-directory escape at fire time.

After the wrappers are installed, Home Manager runs:

```bash
vesper-hermes-automations sync-cron --prune
```

Before mutating Hermes state, `sync-cron` validates the declarative registry against the installed task/watchdog set. It then reconciles only machine-owned `vesper:*` records through Hermes' own CLI. Unrelated user-created Hermes jobs are never pruned.

The old `sabah-check-deliver.sh` and `morning-check-deliver.sh` paths remain physical compatibility aliases that dispatch the new Morning Check worker.

## delivery policy

Dispatch jobs use `deliver=local` at the Hermes cron layer because the cron tick only launches a transient worker and intentionally prints nothing. Finished research is persisted to the Vesper briefing/state layer. Morning Check explicitly sends its completed brief with `hermes send --to telegram`.

Watchdogs use `deliver=telegram` because their stdout is the alert itself. Healthy ticks emit no stdout and use no model.

## daily pipeline

| time | job | behavior |
|---|---|---|
| `08:30` | `unknown-frontier-github` | GitHub frontier scout |
| `08:35` | `unknown-frontier-reddit` | Reddit frontier scout |
| `08:40` | `unknown-frontier-x` | X frontier scout |
| `08:45` | `free-ai-radar` | Linux.do-first legitimate free AI / free-tier / self-hosted radar |
| `09:00` | `unknown-frontier-synthesis` | bounded fan-in over fresh scout state, then verified synthesis |
| `09:30` | `agenda` | compact important current agenda |
| `10:00` | `morning-check` | projects + todos + durable research, delivered to Telegram by the completed worker |
| `15:00` | `upstream-edge-radar` | early breaking changes and capabilities in Vesper upstreams |
| `23:30` | `second-brain-dream` | durable knowledge consolidation into the Obsidian workflow |

### frontier fan-out / fan-in

The three frontier scouts are independent cron entries so each has its own trigger and transient worker. This prevents three research calls from being launched at the same instant.

The synthesis job does **not** use Hermes `context_from` as its join primitive. Cron entries are deliberately `no_agent` dispatch wrappers, while expensive work runs outside Hermes cron sessions. Each scout writes a timestamped envelope under:

```text
~/.local/state/vesper/research/unknown-frontier-ai/scouts/
```

At `09:00`, synthesis reads only fresh envelopes. It waits for missing scouts for a bounded interval (`VESPER_FRONTIER_FANIN_WAIT_SECONDS`, default 300 seconds), then synthesizes the fresh subset and explicitly records missing/stale sources. If no fresh scout exists, synthesis fails instead of silently recycling old state.

`frontier-daily` remains available as a manual compatibility task. It uses bounded concurrency (`VESPER_FRONTIER_MAX_WORKERS`, default 2) and the same state fan-in.

This join is inspectable, freshness-aware, independent of Hermes' cron-session lifetime, and survives process boundaries.

## watchdogs

`vesper-health-watch` runs every three hours. It checks `vesper-doctor --json`, failed user/system systemd units, root/home disk utilization (`VESPER_DISK_ALERT_PERCENT`, default 90), and discovered Restic timer state when Restic timers are present.

`cron-skill-integrity-watch` runs every six hours. It checks desired jobs, enabled/paused state, schedules, physical script paths, `no_agent=true`, duplicate job names, referenced skills, and Hermes cron/gateway health.

Both watchdogs are edge-triggered through `~/.local/state/vesper/research/watches/`: unchanged warnings are not resent, and recovery emits one recovery message.

## weekly jobs

Sunday jobs are staggered:

| time | job |
|---|---|
| `11:00` | `user-pain-miner` |
| `12:30` | `project-archaeologist` |
| `14:00` | `skill-evolution-review` |
| `15:30` | `ai-usage-economist` |

`user-pain-miner` requires recurrence evidence instead of turning isolated complaints into fake trends.

`project-archaeologist` scans bounded local Git roots for unfinished work worth revisiting.

`skill-evolution-review` reviews research heuristics and `skill-drafts`; it never edits active skills automatically and does not replace Hermes Curator.

`ai-usage-economist` uses available local accounting surfaces (`ccusage`, CodexBar and TurnLens) and keeps measured usage separate from routing suggestions.

The nightly second-brain resolver prefers `OBSIDIAN_VAULT_PATH` and known locations, then performs only a bounded depth scan. It never recursively walks the entire home directory.

## validation and CI

The runtime exposes:

```bash
vesper-hermes-automations validate-registry
```

The validator rejects unknown tasks/watchdogs, invalid schedule shape, duplicate cron/script identities and incorrect delivery policy before cron reconciliation.

GitHub Actions evaluates `hermes-jobs.nix` to JSON and runs the Python contract suite. Tests assert the automation surface, scout/synthesis staggering, watchdog delivery policy and `--no-agent`/script flags used by the Hermes CLI integration.

## commands

```bash
vesper-hermes-automations jobs
vesper-hermes-automations validate-registry
vesper-hermes-automations sync-cron --prune
vesper-hermes-automations dispatch frontier-daily
vesper-hermes-automations execute unknown-frontier-github
vesper-hermes-automations execute unknown-frontier-synthesis

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

## why not automation blueprints here

Automation Blueprints are useful for portable opt-in skills. These jobs are machine-owned Vesper configuration, so schedule, wrapper and state paths stay in Nix as the single source of truth. A mature workflow can later be exported as a blueprint for sharing.
