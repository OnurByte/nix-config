# Hermes automations

Status: **current**

Vesper uses Hermes cron as the only recurring scheduler. Vesper-owned scheduling, state, dispatch, watchdog, communications triage and briefing logic stays in the existing Vesper control plane; reliability checks may call `vesper-doctor` as the workstation diagnostic backend rather than creating a second scheduler.

```text
Hermes cron
    ↓
~/.hermes/scripts/vesper-<job>.sh
    ↓
vesper-hermes-automations trigger <job>
    ├─ watchdog → local checks → edge-triggered stdout
    └─ dispatch → systemd-run --user → vesper-hermes-automations execute <job>
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
├── communications.rs
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
runtime state    ~/.local/state/vesper/research/ + ~/.local/state/vesper/communications/ + Hermes runtime state
durable reports  ~/.local/share/vesper/briefings/
semantic memory  Hermes/runtime memory when configured
long context      Obsidian second brain
procedures        ~/.agents/skills/
```

A controller timeout, shell exit or agent claim is not automatically the task result. Durable state/artifacts and postconditions are the source of truth for resumable work.

The shared `agent-operations` skill owns the general reliability/governance contract. `hermes-research-radar` owns research-specific discovery/evidence behavior. `vesper-communications-intelligence` owns read-only communications triage, evidence-backed person/group context and the no-outbound-message boundary. `vesper-obsidian-second-brain` owns durable promotion into Obsidian.

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

Daily lanes currently use a 36-hour window, communications uses a 90-minute freshness window, and weekly lanes an 8-day window. These are conservative operational defaults derived from the actual schedules, not universal constants. Tune them only from observed scheduling/availability behavior.

## scheduled jobs

| time | job | behavior |
|---|---|---|
| `08:30` | `unknown-frontier-github` | GitHub coding-agent + Monero/privacy scout |
| `08:35` | `unknown-frontier-reddit` | Reddit RSS/web + comment/thread scout |
| `08:40` | `unknown-frontier-x` | X/FxTwitter direct frontier scout |
| every 3 min | `xpatla-scan` | dynamic Turkish news source intake through FxTwitter, SQLite state and media provenance |
| `08:45` | `unknown-frontier-web` | clearnet + Tor/onion scout |
| `08:50` | `free-ai-radar` | legitimate free/cheap coding-agent capability radar |
| `09:10` | `unknown-frontier-synthesis` | scout synthesis + counter-review |
| `09:30` | `agenda` | compact current agenda |
| `10:00` | `morning-check` | local projects + research + useful communications → Telegram |
| `15:00` | `upstream-edge-radar` | upstream change radar |
| every 15 min, `:04/:19/:34/:49` | `communications-radar` | Agent Messenger read-only delta → isolated analysis; local alert only for validated high/critical signal |
| every 15 min, offset | `vesper-health-watch` | internal health/freshness + optional external dead-man ping |
| every 6 h | `cron-skill-integrity-watch` | scheduler/registry/script integrity |
| `23:30` | `second-brain-dream` | durable research + communications/person context consolidation |

Sunday also runs `user-pain-miner`, `project-archaeologist`, `skill-evolution-review` and `ai-usage-economist`.

## communications intelligence

The communications feature is an inbox-intelligence pipeline, **not** an autonomous correspondent.

The hard boundary is:

```text
read → normalize → analyze → brief → remember → local alert

never send / reply / react / draft / mark-read
```

Agent Messenger is the **single transport** for WhatsApp, Telegram, Discord and Instagram. There is no alternate connector and no failover path. If one network is unconfigured, degraded or unavailable, the batch records that source state and continues only with sources that are actually readable.

The source messaging networks remain the message-history authority. Agent Messenger owns the local account/session material needed to talk to them. Vesper keeps only bounded operational state and derived briefings; it does not create a second full chat archive.

### capability boundary

Upstream Agent Messenger exposes both read and mutation commands. Vesper exposes only two narrowed wrappers:

```text
vesper-agent-messenger-auth  human authentication/setup only
vesper-agent-messenger-read  scheduled communications intake only
```

The unrestricted upstream `agent-messenger` executable is not installed in the normal Vesper user PATH.

`vesper-agent-messenger-auth` accepts only the four communications platforms and always routes into that platform's `auth` command family. It can create, inspect, switch or remove local authentication state when the human explicitly uses it, but it cannot route to message send/edit/delete/react commands.

`vesper-agent-messenger-read` has a hard command allowlist. The scheduled Rust control plane can query only:

```text
<platform> auth status
whatsapp|telegram|instagram chat list
whatsapp|telegram|instagram|discord message list
discord dm unread
discord mention unread
```

Send, reply, react, edit, delete, acknowledge/mark-read and other messaging mutations are outside that executable grammar.

Agent Messenger is also not added to Vesper's shared MCP registry. Communications intelligence therefore does not grant Codex, Claude Code or OpenCode a messaging MCP merely because the scheduled radar exists.

### analysis capability boundary

The normalized bounded batch is already included in the communications prompt, so the analysis model does not need shell, browser, MCP, messaging or file tools.

The isolation is enforced inside the Rust `run_communications_radar` call path. Its Hermes invocation includes:

```text
--safe-mode -t context_engine
```

This does not depend on PATH shadowing or on being launched by a particular cron wrapper, so direct/manual execution of `communications-radar` receives the same capability boundary. For the pinned Hermes release, safe mode suppresses user plugins/MCP/rules/customizations before agent startup and the built-in `context_engine` toolset is empty. This prevents a configured plugin context engine from re-introducing recovery tools into the communications lane. Other Hermes research jobs keep their normal tool surfaces.

The provider and model remain explicit command-line arguments from the Vesper Rust control plane. Do not replace this enforcement with a prompt-only "never send" instruction. If upstream Hermes gains a first-class explicit no-tools mode, prefer that primitive.

### package/runtime boundary

Vesper selects Agent Messenger `2.36.0` exactly. The wrappers run that package through Bun's package runner and keep its package cache under:

```text
~/.cache/vesper-agent-messenger/bun
```

This is version-pinned but not an offline Nix-store package: a cold cache can require registry access. Do not describe it as fully Nix-reproducible until Agent Messenger is packaged into the Nix build graph.

Agent Messenger account/session state lives under:

```text
~/.config/agent-messenger/
```

or the path selected by `AGENT_MESSENGER_CONFIG_DIR`.

### setup

After `nh os switch`, authenticate the networks you want observed through the auth-only Vesper wrapper:

```bash
vesper-agent-messenger-auth whatsapp login --qr
vesper-agent-messenger-auth telegram login
vesper-agent-messenger-auth discord extract
vesper-agent-messenger-auth instagram extract
```

Authentication is interactive and may require the corresponding logged-in app/browser/session or platform-specific credentials. Do not put extracted credentials, cookies, tokens or session files in Git or Nix source.

Check the Vesper-side intake state with:

```bash
vesper-hermes comms-status
vesper-doctor --json
```

`unconfigured`, `unavailable`, `degraded`, `ready` and a ready-but-empty delta are different states. One failed network is not silently replaced by another connector.

### delta and crash semantics

The first configured run bootstraps a bounded recent window, default `6h`. Later runs overlap the last committed watermark by 10 minutes and deduplicate with recent platform-prefixed message IDs.

Canonical message IDs are transport-independent inside Vesper's current model:

```text
whatsapp:<source-message-id>
telegram:<source-message-id>
instagram:<source-message-id>
discord:<source-message-id>
```

This prevents cross-network ID collisions.

Current Vesper intake bounds are:

```text
recent chats per network       80 by default
messages fetched per chat      50 by default
per-analysis batch             200 messages by default
recent dedupe IDs              5000
```

`VESPER_COMMS_CHAT_LIMIT`, `VESPER_COMMS_MESSAGES_PER_CHAT`, `VESPER_COMMS_BATCH_MESSAGES` and `VESPER_COMMS_BOOTSTRAP_HOURS` can tune the bounded fetch within the clamps enforced by `communications.rs`.

For WhatsApp, Telegram and Instagram, the intake selects recent/unread chats then reads bounded message windows. Discord uses unread DM and unread mention discovery, then reads bounded DM message windows. A source that reports incomplete unread discovery or partial read failures marks the batch degraded rather than pretending coverage was complete.

This is an inbox radar, not a guaranteed historical export. Per-chat and upstream query bounds mean an arbitrarily large offline backlog may require wider explicit limits or separate archival tooling; Vesper must never claim it observed messages that were outside the selected windows.

The watermark advances only after Hermes produces a report, that report passes the evidence sanitizer and the report is persisted. The current batch is held in one crash-recoverable `pending.json`; after a successful commit that staging file is removed.

Pending state is transport-tagged. A stale batch from a previous communications transport is discarded instead of being replayed as Agent Messenger data.

### evidence and alert gate

Model output is not trusted merely because it is valid JSON.

Before persistence/notification, the Rust sanitizer:

- accepts evidence message IDs only if they exist in the current `messages + contextMessages` set
- drops evidence-bound findings that are left without real source IDs
- requires every high/critical alert to contain a valid source message ID
- requires a high/critical alert to carry an explicit semantic ground such as a credential request, payment request, impersonation, coercion, threat, deadline, sensitive account action or material decision
- downgrades report priority when no validated high/critical alert survives

Presentation hints such as invisible Unicode or punycode can remain a review signal, but they do not create a high/critical alert by themselves.

### analysis model

`vesper-communications-intelligence` separates:

- salience / what actually deserves attention
- direct requests and decisions
- commitments and open loops (`me`, `them`, `shared`)
- person/source identity context
- group decisions and topic changes
- security/social-engineering/manipulation risk indicators

Message volume is not importance.

High-value risk analysis is evidence-bound. Valid signals include credential or payment requests, unusual urgency, impersonation/identity inconsistency, coercion, suspicious-link pressure, meaningful contradiction and boundary pressure. Findings must retain source message IDs and distinguish observation from inference.

The system does not infer protected/sensitive traits and does not diagnose people as narcissists, psychopaths, mentally ill, etc. Person notes describe concrete evidenced behavior and can be corrected by later evidence.

Only validated `high` and `critical` alerts can trigger immediate desktop `notify-send`. Exact duplicate alert bodies are suppressed. Communications code never routes those alerts back through WhatsApp/Telegram/Discord/Instagram.

### provider privacy boundary

Local Agent Messenger collection does not make inference local. The normalized bounded batch becomes part of the Hermes model request.

If the configured Hermes provider/model is remote, the message text and bounded identity/context fields in that batch leave the machine for inference. Do not call this workflow fully local unless the chosen inference provider is itself local.

### second-brain fan-in

Communications reports use the same durable briefing store as other Hermes lanes:

```text
~/.local/share/vesper/briefings/
```

At `23:30`, `second-brain-dream` inspects recent `lane=communications-radar` reports and promotes only durable knowledge. It may update compact existing notes such as:

```text
Hermes/Communications/Briefings/
Hermes/Communications/Groups/
Hermes/Communications/Topics/
Hermes/People/
```

It does not dump transcripts into Obsidian. Person notes keep aliases/source identities, useful facts, open loops, meaningful changes and dated risk/trust-boundary observations with evidence references. Uncertain cross-platform identities remain separate until evidence supports a merge.

Morning Check may surface a Communications section when there is a real action, commitment, risk or important change; routine chatter is omitted.

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

Reddit scouts may use bounded RSS/API intake before expensive deep reading. X
scouts use FxTwitter's direct profile/status API and canonicalize every result
back to its x.com status identity. Seed communities/accounts are bootstrap
hints rather than an allowlist. X mirrors are not a fallback for this workflow.

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

Communications operational state:

```text
~/.local/state/vesper/communications/
```

Briefings:

```text
~/.local/share/vesper/briefings/
```

The communications state contains watermarks, bounded dedupe IDs, per-source intake status, at most one pending batch and duplicate-alert state. It is not a transcript archive.

The Rust control plane writes atomic JSON records, Markdown briefings, run status and a rebuilt briefing index. Caelestia consumes the briefing index through the Rust AI Hub rather than invoking a Python runtime.

Scheduled job success/error receipts live under:

```text
~/.local/state/vesper/research/runs/<task>/latest.json
```

Freshness monitoring reads these receipts. A successful scheduler configuration without a recent successful receipt is not treated as proof that the job actually ran.

Long future workflows that contain many independently resumable units should additionally use a durable per-unit manifest from `agent-operations`; the existing latest-run record is a task receipt, not a general-purpose batch manifest.

## adaptive source state

Useful evidence-bearing research report URLs reinforce one shared source registry:

```text
~/.local/state/vesper/research/unknown-frontier-ai/source-registry.json
```

A source starts at `probation`, becomes `trusted` after repeated useful hits and `promoted` after further repeated evidence. A mention in a prompt or candidate list is not enough; the URL must survive into the final report evidence.

Communications reports use source message IDs rather than pretending private chats are public research URLs, so they do not reinforce this research source registry.

Inspect the research registry with:

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

Communications intake is deliberately read-only, so its postcondition is a persisted sanitized analysis report plus a committed local watermark; a model claim that it "checked messages" is not enough.

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
vesper-hermes comms-status
vesper-hermes list
vesper-hermes read <id>
vesper-hermes inbox
vesper-hermes run unknown-frontier-ai
vesper-hermes run communications-radar

vesper-agent-messenger-auth whatsapp login --qr
vesper-agent-messenger-auth telegram login
vesper-agent-messenger-auth discord extract
vesper-agent-messenger-auth instagram extract

vesper-hermes-automations jobs
vesper-hermes-automations validate-registry
vesper-hermes-automations sync-cron --prune
vesper-hermes-automations trigger agenda
vesper-hermes-automations trigger communications-radar
vesper-hermes-automations execute agenda
vesper-hermes-automations links
vesper-hermes-automations tor-fetch 'http://example.onion/'

vesper-research "query" --pages 600
vesper-research sources
vesper-doctor --json
```

## CI

The smoke workflow rejects first-party `.py` files before evaluation. It compiles the Rust control plane (including the communications module), validates the Hermes job registry, parses Nix and Hyprland Lua, evaluates Home Manager, builds the Caelestia surface before the complete Vesper system, then builds the separately exposed packages.
