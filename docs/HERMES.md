# Hermes automations

Status: **current**

Vesper uses Hermes cron as the only recurring scheduler. Vesper-owned scheduling, state, dispatch, watchdog and briefing logic lives in one native Rust control plane:

```text
Hermes cron
    ↓
~/.hermes/scripts/vesper-<job>.sh
    ↓
vesper-hermes-automations trigger <job>
    ├─ watchdog → local Rust checks → edge-triggered stdout
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

## watchdogs

`vesper-health-watch` checks:

- `vesper-doctor`
- failed user/system units
- root filesystem usage threshold

`cron-skill-integrity-watch` checks:

- registry validity
- missing Hermes jobs
- enabled-state drift
- script drift
- `no_agent=true`
- physical script presence
- Hermes cron stalled state

Watchdog output is edge-triggered. An unchanged fault stays silent; a changed fault emits once; recovery emits once.

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
```

## CI

The smoke workflow now rejects any first-party `.py` file before evaluation. It compiles both Rust control planes, validates the Hermes job registry, parses Nix and Hyprland Lua, evaluates Home Manager, builds the Caelestia surface before the complete Vesper system, then builds the separately exposed packages.
