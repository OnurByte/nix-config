# Hermes automations

Vesper keeps Hermes cron as the only recurring scheduler. Long research never runs inside the cron gateway: each `vesper:*` cron entry is a short `no_agent` script that dispatches a transient user service, which runs Hermes one-shot research and persists state/briefings.

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

`systemd-run` is execution containment only, not another scheduler.

## declarative registry

Schedules live in `home/yargc/hermes-jobs.nix`. Home Manager writes `~/.config/vesper/hermes-jobs.json`, installs physical wrappers under `~/.hermes/scripts/`, then runs:

```bash
vesper-hermes-automations sync-cron --prune
```

Only machine-owned `vesper:*` jobs are reconciled/pruned. Dispatch jobs use `deliver=local`; Morning Check sends its completed message explicitly through Hermes Telegram delivery. Watchdogs use Telegram and remain zero-token while healthy.

## daily pipeline

| time | job | behavior |
|---|---|---|
| `08:30` | `unknown-frontier-github` | GitHub coding-agent + Monero/privacy frontier scout |
| `08:35` | `unknown-frontier-reddit` | Reddit RSS/Atom intake + selective deep research |
| `08:40` | `unknown-frontier-x` | X profiles/search with XCancel/Nitter-compatible fallback |
| `08:45` | `free-ai-radar` | Linux.do-first legitimate coding-agent cost/free-tier radar |
| `09:00` | `unknown-frontier-synthesis` | fresh scout fan-in + counter-reviewed synthesis |
| `09:30` | `agenda` | compact important current agenda |
| `10:00` | `morning-check` | projects/todos/research → Telegram |
| `15:00` | `upstream-edge-radar` | early warning for coding-agent/Vesper/privacy upstreams |
| `23:30` | `second-brain-dream` | durable knowledge consolidation |

## research profile

The frontier is intentionally opinionated.

Priority 1 is **vibe coding / agentic software engineering**: Codex, Claude Code, OpenCode, Hermes, agent harnesses, skills, MCP, context engineering, orchestration, evals, practical workflows and overlooked developer tooling.

Priority 2 is **Monero / privacy**: Monero, Cuprate, wallets, atomic swaps, private payments, Tor/onion, SimpleX, GrapheneOS/privacy engineering and adjacent infrastructure.

Priority 3 is **Nix/Linux/security/open source** where it improves the workstation or the two priorities above.

Generic local-LLM/model-quantization/inference hobby material is not a standing target. `r/LocalLLaMA` is explicitly excluded from the default source graph. Model/inference material survives only when it materially changes coding-agent quality/cost/privacy/deployment.

## high-volume research funnel

The default frontier target is `600` distinct canonical candidate items/URLs, configurable through `VESPER_FRONTIER_CANDIDATE_TARGET` and clamped to `200..1000`. Default deep-read target is `48`, configurable and clamped to `24..60`.

```text
RSS/search/metadata/mirror intake
        ↓
canonicalize + dedupe
        ↓
anchor / learned / exploration candidate pools
        ↓
cheap relevance + novelty triage
        ↓
24-60 strongest deep reads
        ↓
primary-source verification
        ↓
counter-review
        ↓
synthesis + durable learning
```

Hundreds of candidate inspections therefore do not mean hundreds of full pages dumped into one model context.

## central sources without source lock-in

Protected central sources are guaranteed inspection seeds, not an allowlist.

### Reddit anchors

Vibe coding / agentic development:

- `r/vibecoding`
- `r/ClaudeCode`
- `r/codex`
- `r/opencodeCLI`
- `r/cursor`

Monero/privacy/security:

- `r/MoneroMeansMoney`
- `r/Monero`
- `r/privacy`
- `r/Tor`
- `r/netsec`

Workstation:

- `r/NixOS`

`r/LocalLLaMA` is a user-excluded source and is retired during source-registry migration rather than rediscovered into probation.

### X anchors

Coding/dev: `@Teknium`, `@thdxr`, `@XOpenSource`, `@ZixuanLi_`.

Monero/privacy/payments: `@eigenwallet`, `@kyc_rip`, `@XBToshi`, `@schmidt1024`, `@XMRHub_org`, `@CR1337`, `@linuxuser1996`, `@Examare1`, `@ZcashLabs`.

Threat/privacy communications: `@akaclandestine`, `@DailyDarkWeb`, `@SimpleXChat`.

### GitHub anchor neighborhoods

Coding-agent starting points:

- `NousResearch/hermes-agent`
- `openai/codex`
- `anthropics/claude-code`
- `anomalyco/opencode`

Monero/privacy starting points:

- `monero-project/monero`
- `Cuprate/cuprate`

The GitHub scout expands outward through issues, PRs, commits, forks, authors, dependencies and small adjacent repositories.

## adaptive candidate allocation

Social intake separates final candidates into pools instead of concatenating anchor feeds and truncating them:

- about **45% anchors**
- about **30% learned dynamic sources**
- about **25% exploration/query tail**

Unused quota is redistributed if a pool is blocked/empty. Selection inside each pool is round-robin across source identities/queries, preventing one prolific subreddit/account from monopolizing the candidate budget.

This fixes the failure mode where central feeds alone could fill the daily target and starve autonomous discovery.

## Reddit: RSS + AI

Reddit RSS/Atom is the cheap sensor layer. Anchor `new.rss` and selected `comments.rss` feeds are fetched separately, learned sources get their own intake, and general exploration can use combined feeds.

After canonicalization and pool selection, Hermes deep-reads only promising threads/comment branches. Community claims are followed to repositories/docs/issues/PRs/papers when they matter.

Latest deterministic intake:

```text
~/.local/state/vesper/research/unknown-frontier-ai/intake/reddit-latest.json
```

## X: mandatory surface + mirrors

X remains mandatory. The playbook prefers direct X when available; deterministic intake uses XCancel/Nitter-compatible profile/search RSS, falling back to HTML on the same mirror and then the next configured mirror.

```text
VESPER_X_MIRRORS=https://xcancel.com,https://nitter.net
```

X/Twitter/XCancel/Nitter copies of one status normalize to one canonical `x.com/<user>/status/<id>` identity. A mirror is transport, not independent corroboration.

Latest intake:

```text
~/.local/state/vesper/research/unknown-frontier-ai/intake/x-latest.json
```

## self-evolving source graph

Runtime state lives at:

```text
~/.local/state/vesper/research/unknown-frontier-ai/source-registry.json
```

Protected anchors remain anchors. New sources begin at `probation`. Merely appearing in `candidateSources` does **not** earn a useful hit. A source gets hit/score credit only when its URL survives research as an evidence-bearing candidate/source.

Normal lifecycle:

```text
discovered → probation → trusted → promoted → decay/review → probation/retired
```

Repeated zero-value failures and long periods without useful output can retire learned sources. Explicitly user-excluded sources stay retired. Retired non-excluded sources may later be rediscovered, but only at probation.

## skill evolution is eval-gated

The researcher has two speeds of self-improvement.

Fast/reversible state can change automatically: source tiers, scores, mirror health, query candidates, heuristic confidence and dead-end state.

Nix-owned active skill instructions use a slower process:

```text
trajectory evidence
    ↓
candidate rule / skill draft
    ↓
representative evals
    ↓
with-skill vs current/baseline comparison
    ↓
promote / keep-testing / reject / rollback
```

The official pinned Anthropic `skill-creator` is exposed to Hermes for this evaluation-oriented workflow. The research skill also ships a representative eval set under:

```text
home/yargc/skills/hermes-research-radar/evals/evals.json
```

Eval cases cover vibe-coding novelty, Monero community→primary-source verification, X mirror failure, Reddit RSS breadth, LocalLLaMA noise rejection, anchor saturation/exploration preservation and evidence-gated source promotion.

Research skill structure:

```text
home/yargc/skills/hermes-research-radar/
├── SKILL.md
├── evals/
│   └── evals.json
└── references/
    ├── research-pipeline.md
    ├── source-governance.md
    ├── central-sources.md
    ├── reddit-rss.md
    ├── x-research.md
    └── research-evolution.md
```

The weekly `skill-evolution-review` reads the active research skill, evals, adaptive source registry, recent frontier run state, heuristics and skill drafts. It produces an evidence-backed promotion/testing/retirement queue but does not mutate the active Nix-owned skill automatically.

## frontier fan-in

Scouts are separate cron entries and write timestamped envelopes under:

```text
~/.local/state/vesper/research/unknown-frontier-ai/scouts/
```

Synthesis uses only fresh envelopes, waits a bounded interval for missing scouts, then reports missing/stale sources explicitly. If no fresh scout exists, synthesis fails instead of silently recycling yesterday's research.

## watchdogs

`vesper-health-watch` checks Vesper doctor state, failed user/system units, disk utilization and Restic state when present.

`cron-skill-integrity-watch` checks declarative cron records, enabled state, schedule, physical script paths, `no_agent=true`, duplicate names, skill references and Hermes scheduler health.

Both are edge-triggered and silent while healthy.

## weekly jobs

| time Sunday | job |
|---|---|
| `11:00` | `user-pain-miner` |
| `12:30` | `project-archaeologist` |
| `14:00` | `skill-evolution-review` |
| `15:30` | `ai-usage-economist` |

`user-pain-miner` requires recurrence evidence. `project-archaeologist` scans bounded local Git roots. `ai-usage-economist` separates measured usage from model-routing suggestions.

## validation and CI

```bash
vesper-hermes-automations validate-registry
```

GitHub Actions evaluates the Nix registry and runs the Python contract suite. Contracts cover schedule/task wiring, 200–1000 coverage bounds, source interests/exclusions, source-registry promotion behavior, old LocalLLaMA migration, 45/30/25 candidate-pool preservation, skill/reference/eval presence, X/Reddit canonicalization and Hermes no-agent script mode.

## commands

```bash
vesper-hermes-automations jobs
vesper-hermes-automations validate-registry
vesper-hermes-automations sync-cron --prune
vesper-hermes-automations dispatch frontier-daily
vesper-hermes-automations execute unknown-frontier-github
vesper-hermes-automations execute unknown-frontier-reddit
vesper-hermes-automations execute unknown-frontier-x
vesper-hermes-automations execute unknown-frontier-synthesis

vesper-hermes status
vesper-hermes list
vesper-hermes inbox

hermes cron status
hermes cron list
hermes cron run <job>
```
