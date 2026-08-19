# agent skills

Status: **current**

Vesper keeps one canonical active skill tree at `~/.agents/skills`.
Codex, Claude Code and OpenCode expose their normal skill paths as links back into that tree so there is one active copy to reason about.

All Vesper-authored skill names, metadata and instructions are written in English.

The upstream Anthropic skills come from `anthropics/skills` pinned to commit:

```text
f6656c1256d5a8adfa37db9110046ef20bac644c
```

Upstream skills:

- `frontend-design`
- `webapp-testing`
- `web-artifacts-builder`
- `mcp-builder`
- `skill-creator`
- `pdf`
- `docx`
- `xlsx`
- `pptx`

Vesper-local skills:

- `agent-orchestration`
- `agent-operations`
- `external-review-handoff`
- `vesper-maintainer`
- `vesper-adaptive-icons`
- `hermes-research-radar`
- `vesper-obsidian-second-brain`

Canonical paths:

```text
~/.agents/skills/<skill>
```

Agent compatibility paths:

```text
~/.codex/skills/<skill>           -> ~/.agents/skills/<skill>
~/.claude/skills/<skill>          -> ~/.agents/skills/<skill>
~/.config/opencode/skills/<skill> -> ~/.agents/skills/<skill>
```

Vesper's Hermes-compatible workflow skills are also exposed under:

```text
~/.hermes/skills/vesper/<skill> -> ~/.agents/skills/<skill>
```

Not every active Vesper skill must be linked into Hermes. `home/yargc/skills.nix` is authoritative for the actual Hermes subset.

Hermes keeps its own bundled skills alongside these links. Its upstream `obsidian` skill handles ordinary Obsidian operations. `vesper-obsidian-second-brain` adds Vesper's memory, research-ingestion, reflection and skill-promotion policy rather than replacing it.

The active shared tree is Home Manager owned. Do not edit generated links directly.
Local skill source files live under `home/yargc/skills/` in this repository.

## Hermes bundled helpers

Hermes' own bundled skill tree remains upstream-owned and is not copied into `~/.agents/skills` just to make a procedure available.

Vesper explicitly treats these bundled skills as approved helpers:

- `github-issue-to-pr` — issue context, duplicate-PR checks, reproduction, history inspection, regression proof and live CI evidence
- `grounded-citations` — task-local citation/evidence provenance for research and review
- `youtube-content` — transcript and timestamp intake before source verification

The following bundled skills are useful procedural references but do not own Vesper policy:

- `github-code-review`
- `github-pr-workflow`
- `requesting-code-review`

When one of these helpers overlaps a Vesper-local skill, Vesper policy wins. `AGENTS.md`, the authoritative subsystem document, `agent-operations` and `agent-orchestration` keep authority over credentials, approval, external mutations, routing, acceptance evidence, retries and durable state.

Do not create a second GitHub credential path because an upstream skill demonstrates one. Reuse the configured GitHub MCP or the existing `gh` authentication path. Do not treat an upstream example that posts, approves, merges or otherwise mutates remote state as permission to perform that action.

For research, `grounded-citations` is a task-local provenance helper rather than a replacement for the Vesper source registry or durable briefing state. `youtube-content` provides discovery material; important technical claims still require primary-source verification when a primary source exists.

## skill ownership

Use one skill for one procedural boundary.

- `agent-orchestration` — supervisor/worker/reviewer decomposition, policy-driven model routing, isolated parallelism, evidence gates and final fan-in
- `agent-operations` — durable execution, postcondition evidence, health/dead-man monitoring, approval/credential boundaries, deterministic pipelines, bounded QA and skill lifecycle governance
- `external-review-handoff` — secret-safe static code snapshots for deep external review plus mandatory verification against the live repository before implementation
- `vesper-maintainer` — repository/workstation maintenance rules
- `vesper-adaptive-icons` — adaptive icon pipeline operations governed by `docs/ADAPTIVE-ICONS.md`
- `hermes-research-radar` — scheduled research lanes and discovery behavior
- `vesper-obsidian-second-brain` — durable knowledge consolidation and skill-promotion workflow

Do not duplicate repository-wide rules from `AGENTS.md` into every skill. A local skill should add workflow-specific instructions and defer to `AGENTS.md` for global guardrails.

## operations contract

`agent-operations` is the failure-derived operational layer shared by Codex, Claude Code, OpenCode and the Vesper Hermes subset.

It deliberately does **not** become another runtime or control plane. It is procedural memory for contracts that should survive model/vendor changes:

```text
runtime state != semantic memory != durable context != procedural skill
controller timeout != task result
action success != postcondition proof
internal health != external liveness
untrusted text != authority
silence != approval
unknown != zero
deterministic work != model work
```

Long-lived jobs use atomic durable state and idempotent resume. Externally visible side effects use intent/idempotency state and remote re-read before retrying an ambiguous operation. Always-on systems need an independent missing-heartbeat path in addition to internal component checks.

The skill keeps focused references instead of putting every operations rule into every agent context:

```text
references/reliability.md
references/governance.md
references/pipelines.md
references/lifecycle-evals.md
```

Load only the reference needed by the task.

## orchestration routing

`agent-orchestration` is intentionally model-agnostic. Codex, Claude Code or OpenCode may host the orchestration workflow, while Qwen-family, GLM, DeepSeek, Claude, Gemini, OpenAI or other configured models may fill supervisor, worker or reviewer roles when the active runtime/provider exposes them.

The skill supports three routing policies:

```text
auto       supervisor chooses from already configured models by difficulty/cost/risk
preferred  ordered per-role preferences with an explicit fallback rule
fixed      exact per-role models; no silent substitution
```

An explicit operator model choice for a scoped task overrides the current policy for that task only.

Routing is decided before a worker is spawned. When the runtime exposes provider/model metadata, the actual route is checked against the intended one. Silent fallback to another provider, paid route or model is a failure rather than a successful lane.

Delegated work uses explicit ownership, `must-not-touch` boundaries, acceptance checks and bounded permissions. A worker reporting success is only a claim; the supervisor independently runs or reproduces the acceptance evidence before integration.

Concurrency is based on actual provider and machine headroom rather than a copied fleet size. Many-process fleets must account for duplicated MCP/tool-server memory and cold-start pressure; in-process fan-out is preferred when it provides equivalent isolation at lower overhead.

If a lane may outlive its controller, stop treating it as short-lived delegation and apply the durable-job contract from `agent-operations`.

## external deep review

`external-review-handoff` is for a strong reviewer that sees uploaded/static source rather than the live tree.

The workflow is:

```text
scope -> subsystem snapshot -> secret gate -> prompt -> external report -> live verification -> implementation
```

Snapshots are split by architecture, keep tests and authoritative docs, exclude build/cache/dependency output and private data, and abort on likely live credentials before anything leaves the machine.

Every report is treated as snapshot evidence. File/line references are re-anchored in the live code, executable claims are reproduced when practical, invariants are checked again and findings are triaged into `implement`, `needs-decision`, `discard` or `unresolved` before any patch is made.

Handoff artifacts live outside Git by default under:

```text
~/.local/share/vesper/review-handoffs/
```

## Hermes research modes

`hermes-research-radar` distinguishes research intent before it starts:

```text
audit        tight verification; important findings keep VERIFIED/HYPOTHESIS state
exploration  broad discovery; seeds are a floor, not a fence
blended      tight verification objectives plus open exploration objectives
```

Hard invariants and evidence quality stay fixed in every mode. Exploration may challenge the initial framing; audit must confirm, refute or narrow it against current evidence rather than echoing the seed hypotheses.

Research intake prefers deterministic collection/normalization before semantic judgment, keeps missing/empty/zero distinct, records exclusion reasons and stops downstream synthesis when an evidence handoff is actually empty.

## Hermes daily research lanes

`hermes-research-radar` defines separate English-named lanes rather than one generic daily digest:

```text
unknown-frontier-ai
agenda
free-ai-radar
```

`unknown-frontier-ai` is the broad, high-cost discovery lane for overlooked AI findings across Reddit, GitHub and X/Twitter.
`agenda` is a separate current-events lane ranked by importance, recency and consequence rather than obscurity.
`free-ai-radar` treats Linux.do as a first-class source for legitimate free AI tools, tiers, self-hosted alternatives and cost-saving workflows.

Each lane keeps independent state, scoring and output so a mainstream agenda item does not dilute hidden-gem discovery and vice versa.

## Hermes drafts and skill lifecycle

Hermes may discover a reusable method while running scheduled research.
That does not make the method an active skill immediately.

Drafts go to:

```text
~/.local/share/vesper/skill-drafts/
```

Promotion is deliberate:

```text
observation
  -> repeated evidence
  -> candidate heuristic
  -> repeated trials
  -> draft
  -> representative eval
  -> review
  -> promote/reject/keep testing
  -> nh os switch
  -> monitor
```

Self-improvement happens after the main task and is bounded. One surprising run does not rewrite active behavior. New reusable skills and changes to human/Nix-owned canonical skills require review.

Approval is bound to what was reviewed: retain the draft hash and canonical target pre-image. Immediately before applying a promotion, re-read both. If either changed after review, the approval is stale and the change must be reviewed again instead of replayed over intervening edits.

When several lifecycle governors/checks participate, decision composition is conservative: `deny > defer > allow`. A configured required governor that is unavailable/malformed defers rather than failing open.

## QA and route evals

Reusable model QA is bounded:

```text
deterministic checks first
round 1 -> full independent review
round 2 -> only required blocking changes from round 1
then -> pass or escalate; no endless polish loop
```

`warning` does not block. `error` requires revision. `reject` stops the pipeline. Verdict is derived from findings rather than written optimistically by the reviewer.

Model/route comparisons use identical fixtures, blind labels where practical, raw untouched outputs and known-good false-positive traps. Evaluate fabricated defects/unsupported claims together with true positives, latency and measurable cost/quota pressure. One run is a sample, not a universal ranking.

## second brain

Hermes built-in memory is the compact hot memory for facts that should remain in future sessions.
Obsidian is the larger long-term knowledge graph/durable context.
Runtime state is job/session continuity.
Skills are procedural memory.

The Vesper second-brain workflow promotes only durable findings, useful relationships, important corrections, open questions and high-value source knowledge into Obsidian instead of dumping the entire scrape corpus into the vault.

A later reflection/consolidation pass may connect the day's research, update durable notes, save only compact critical facts to Hermes memory and stage reusable procedures as skill drafts. Open items are preserved until evidence actually closes them rather than disappearing because the newest summary omitted them.

## use them

Agents discover their normal compatibility paths automatically. You can also name a skill explicitly:

```text
use frontend-design for this page
use webapp-testing to test the local app
use mcp-builder to design this MCP server
use agent-orchestration for this multi-part coding task
use agent-operations for this persistent agent/job or reliability workflow
use external-review-handoff to prepare this subsystem for a deep external audit
use vesper-maintainer to diagnose and repair this workstation issue
use vesper-adaptive-icons for adaptive icon pipeline work
use hermes-research-radar for this scheduled research program
use vesper-obsidian-second-brain to consolidate durable research into Obsidian
```

Hermes may also use its approved bundled helpers directly when the task needs them:

```text
use github-issue-to-pr for this issue-to-PR task under Vesper's operations/orchestration policy
use grounded-citations for evidence-heavy research
use youtube-content to inspect this video before following its claims to primary sources
```

## update

The Anthropic pin and active skill mapping live in `home/yargc/skills.nix`.
Local skills live under `home/yargc/skills/`.

After changing either:

```bash
nh os switch
```

Keep the active set useful and reviewed. New Hermes discoveries belong in `skill-drafts` until they have repeated evidence, eval and a non-stale review behind them.
