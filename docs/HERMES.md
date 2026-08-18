# Hermes automations

Status: **current**

Vesper uses Hermes cron as the only recurring scheduler. Vesper-owned scheduling, state, dispatch, watchdog and briefing logic stays in the existing Vesper control plane; reliability checks may call `vesper-doctor` as the workstation diagnostic backend rather than creating a second scheduler.

```text
Hermes cron
    ↓
~/.hermes/scripts/vesper-<job>.sh
    ↓
vesper-hermes-automations trigger <job>
    ├─ watchdog → local checks → edge-triggered stdout
    └─ research → systemd-run --user → vesper-hermes-automations execute <job>
                                      ↓
                                  Hermes agent
                                      ↓
                           persistent state + briefing
```

`systemd-run` is execution containment only, not another scheduler. A fixed unit name is used per task so the same job is not dispatched twice while it is already active.

## implementation

First-party Vesper Hermes code is Rust:

```text
home/yargc/packages/hermes-rs/
├── main.rs
├── cron.rs
├── prompts.rs
├── state.rs
└── util.rs
```

`home/yargc/packages/hermes-core.nix` builds one binary and exposes three command names:

```text
vesper-hermes
vesper-hermes-automations
vesper-research
```

Vesper does not carry first-party Python Hermes code. Upstream Hermes may internally use Python; that implementation stays upstream and is not vendored into this repository.

## operational model

Hermes is treated as an operational agent, not a long-lived chat session.

The persistence layers are distinct:

```text
runtime state    ~/.local/state/vesper/research/ and Hermes runtime state
durable reports  ~/.local/share/vesper/briefings/
semantic memory  Hermes/runtime memory when configured
long context      Obsidian second brain
procedures        ~/.agents/skills/
```

A controller timeout, shell exit or agent claim is not automatically the task result. Durable state/artifacts and postconditions are the source of truth for resumable work.

The shared `agent-operations` skill owns the general reliability/governance contract. `hermes-research-radar` owns research-specific discovery and evidence behavior.

## declarative registry

Schedules live in `home/yargc/hermes-jobs.nix`. Home Manager writes `~/.config/vesper/hermes-jobs.json`, installs physical scripts under `~/.hermes/scripts/`, then reconciles Vesper-owned records with:

```bash
vesper-hermes-automations sync-cron --prune
```

Only `vesper:*` records are pruned. A legacy Sabah/Morning Check record is migrated into the declarative `vesper:morning-check` job instead of creating a duplicate.

Validate the registry without running research:

```bash
vesper-hermes-automations validate-registry
```

Dispatch jobs may declare `freshnessMinutes`. This is an **absence detector**, not a schedule: `vesper-doctor` checks the durable latest run record and warns when a declared job has never produced a record, its latest run ended in `error`, or the last successful record exceeds the declared window.

Daily lanes currently use a 36-hour window and weekly lanes an 8-day window. These are conservative operational defaults derived from the actual daily/weekly schedules, not universal constants. Tune them only from observed scheduling/availability behavior.

## scheduled jobs

| time | job | behavior |
|---|---|---|
| `08:30` | `unknown-frontier-github` | GitHub coding-agent + Monero/privacy scout |
| `08:35` | `unknown-frontier-reddit` | Reddit RSS/web + comment/thread scout |
| `08:40` | `unknown-frontier-x` | X with direct/mirror fallback |
| `08:45` | `unknown-frontier-web` | clearnet + Tor/onion scout |
| `08:50` | `free-ai-radar` | legitimate free/cheap coding-agent capability radar |
| `09:10` | `unknown-frontier-synthesis` | scout synthesis + counter-review |
| `09:30` | `agenda` | compact current agenda |
| `10:00` | `morning-check` | local projects + research → Telegram |
| `15:00` | `upstream-edge-radar` | upstream change radar |
| every 15 min, offset | `vesper-health-watch` | internal health/freshness + optional external dead-man ping |
| every 6 h | `cron-skill-integrity-watch` | scheduler/registry/script integrity |
| `23:30` | `second-brain-dream` | durable knowledge consolidation |

Sunday also runs `user-pain-miner`, `project-archaeologist`, `skill-evolution-review` and `ai-usage-economist`.

## research profile

Priority order:

1. coding agents / vibe coding / developer-agent infrastructure
2. Monero / privacy / Tor / OPSEC
3. Nix/Linux/security/open source when it improves the first two or the workstation

Generic local-model benchmarking, price chatter and repeated mainstream launch coverage are not standing targets.

## frontier coverage

The normal daily frontier target is `600` canonical candidate inspections with `48` deeper reads, split approximately as:

| scout | candidates | deep reads |
|---|---:|---:|
| GitHub | 180 | 15 |
| Reddit | 150 | 12 |
| X | 150 | 12 |
| web/onion | 120 | 9 |

These are targets, not fabricated success metrics. Each scout must report actual coverage and limitations. Synthesis works from persisted scout reports and should call out missing/stale input rather than inventing coverage.

Research intake prefers deterministic RSS/API/metadata collection and normalization where possible before spending model context. Missing, empty, zero, stale and blocked are different states. Filters should leave observable exclusion reasons when practical so research quality can be calibrated later.

## Reddit, X and Tor

Reddit and X scouts are allowed shell access because cheap RSS/mirror intake is useful before expensive deep reading. Seed communities/accounts are bootstrap hints rather than an allowlist.

X mirror copies are one evidentiary identity. A mirror is transport, not corroboration.

For onion pages the web scout can call:

```bash
vesper-hermes-automations tor-fetch 'http://example.onion/path/'
```

The helper only accepts an actual `.onion` authority and fetches through the local Tor SOCKS endpoint at `127.0.0.1:9050` with remote hostname resolution. Normal clearnet web tooling must not pretend it reached an onion service.

## durable state

Research state:

```text
~/.local/state/vesper/research/
```

Briefings:

```text
~/.local/share/vesper/briefings/
```

The Rust control plane writes atomic JSON records, Markdown briefings, run status and a rebuilt briefing index. Caelestia consumes the briefing index through the Rust AI Hub rather than invoking a Python runtime.

Scheduled job success/error receipts live under:

```text
~/.local/state/vesper/research/runs/<task>/latest.json
```

Freshness monitoring reads these receipts. A successful scheduler configuration without a recent successful receipt is not treated as proof that the job actually ran.

Long future workflows that contain many independently resumable units should additionally use a durable per-unit manifest from `agent-operations`; the existing research latest-run record is a task receipt, not a general-purpose batch manifest.

## adaptive source state

Useful evidence-bearing report URLs reinforce one shared source registry:

```text
~/.local/state/vesper/research/unknown-frontier-ai/source-registry.json
```

A source starts at `probation`, becomes `trusted` after repeated useful hits and `promoted` after further repeated evidence. A mention in a prompt or candidate list is not enough; the URL must survive into the final report evidence.

Inspect it with:

```bash
vesper-research sources
vesper-hermes-automations links
```

The current Rust registry intentionally keeps this mechanism small. It does not expose the removed Python-era `links --prune`, `links --all`, wave-local intake databases or 84-hour GC API.

## watchdogs and liveness

Internal watchdogs and external liveness solve different failures.

`vesper-health-watch` checks through `vesper-doctor`:

- Btrfs/system health already covered by the doctor
- failed units
- disk/backup/local workstation signals already covered by the doctor
- declared Hermes dispatch-job latest-run status and freshness

The Rust health watcher also keeps its existing failed-unit/disk checks. Watchdog output is edge-triggered: an unchanged fault stays silent, a changed fault emits once and recovery emits once.

`cron-skill-integrity-watch` checks:

- registry validity
- missing Hermes jobs
- enabled-state drift
- script drift
- `no_agent=true`
- physical script presence
- Hermes cron stalled state

### optional external dead-man

The health-watch wrapper can also ping an external dead-man/heartbeat endpoint after a successful local trigger. This catches a different failure class: if the laptop, network path or Hermes scheduler stops running the wrapper, the external service sees a missing ping even though the dead machine cannot alert on its own.

Provision the endpoint outside Git at:

```text
~/.config/vesper/hermes-deadman.url
```

or point `VESPER_DEADMAN_URL_FILE` at another file-backed secret/config path.

The file should contain one `http://` or `https://` URL. The URL is read at runtime and sent to `curl` through stdin/config, not as a command-line argument. Do not put it in Nix source, Git or shell history. If the service treats the URL as a bearer secret, manage the file with an appropriate Vesper secret mechanism such as `sops-nix`.

When the file is absent the dead-man integration is disabled. A configured ping failure emits a local watchdog message, while sustained missing pings are expected to be alerted by the independent external service.

The dead-man is **not** the same as a full active external probe. A stronger remote monitor can periodically exercise the actual agent/model path from another machine and require an exact response. That monitor must remain outside the system it is judging; Vesper does not pretend an internal cron task is independent.

## evidence and postconditions

For critical automation paths, distinguish action receipts from outcomes:

```text
cron exists / service active      -> component state
latest run status = ok            -> execution receipt
fresh artifact / expected remote state -> outcome evidence
```

An API or provider call that mutates remote state should be followed by a read of the remote object when the integration supports it. Ambiguous timeout/retry paths must not blindly repeat externally visible side effects.

## skill evolution

`skill-evolution-review` is review, not unattended self-rewrite.

Reusable rules move through:

```text
observation -> repeated evidence -> draft -> representative eval -> review -> promote/reject -> monitor
```

Canonical skills remain Nix/Home Manager owned. Approval for a promotion must be tied to the reviewed draft and canonical target pre-image; if either changes before application, the approval is stale and review must be repeated. See `agent-operations/references/lifecycle-evals.md`.

## commands

```bash
vesper-hermes status --json
vesper-hermes list
vesper-hermes read <id>
vesper-hermes inbox
vesper-hermes run unknown-frontier-ai

vesper-hermes-automations jobs
vesper-hermes-automations validate-registry
vesper-hermes-automations sync-cron --prune
vesper-hermes-automations trigger agenda
vesper-hermes-automations execute agenda
vesper-hermes-automations links
vesper-hermes-automations tor-fetch 'http://example.onion/'

vesper-research "query" --pages 600
vesper-research sources
vesper-doctor --json
```

## CI

The smoke workflow rejects first-party `.py` files before evaluation. It compiles both Rust control planes, validates the Hermes job registry, parses Nix and Hyprland Lua, evaluates Home Manager, builds the Caelestia surface before the complete Vesper system, then builds the separately exposed packages.
