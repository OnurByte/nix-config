# agent skills

Status: **current**

Vesper keeps one canonical active skill tree at `~/.agents/skills`.
Codex, Claude Code and OpenCode expose their normal skill paths as links back into that tree so there is one active copy to reason about.
Qwen Code receives reviewed compatible links from the same tree because its skill loader requires valid `SKILL.md` metadata.

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

Qwen Code uses its documented personal skill root. Vesper currently exposes the reviewed orchestration skill there:

```text
~/.qwen/skills/agent-orchestration -> ~/.agents/skills/agent-orchestration
```

Do not link the entire mixed-format skill tree into Qwen until every selected skill satisfies Qwen Code's `SKILL.md` metadata contract.

Vesper's Hermes-compatible workflow skills are also exposed under:

```text
~/.hermes/skills/vesper/<skill> -> ~/.agents/skills/<skill>
```

Not every active Vesper skill must be linked into Hermes. `home/yargc/skills.nix` is authoritative for the actual Hermes subset.

Hermes keeps its own bundled skills alongside these links. Its upstream `obsidian` skill handles ordinary Obsidian operations. `vesper-obsidian-second-brain` adds Vesper's memory, research-ingestion, reflection and skill-promotion policy rather than replacing it.

The active shared tree is Home Manager owned. Do not edit generated links directly.
Local skill source files live under `home/yargc/skills/` in this repository.

## skill ownership

Use one skill for one procedural boundary.

- `agent-orchestration` — supervisor/worker decomposition, model routing, isolated parallelism, fan-in and independent review across compatible agent runtimes
- `vesper-maintainer` — repository/workstation maintenance rules
- `vesper-adaptive-icons` — adaptive icon pipeline operations governed by `docs/ADAPTIVE-ICONS.md`
- `hermes-research-radar` — scheduled research lanes and discovery behavior
- `vesper-obsidian-second-brain` — durable knowledge consolidation and skill-promotion workflow

Do not duplicate repository-wide rules from `AGENTS.md` into every skill. A local skill should add workflow-specific instructions and defer to `AGENTS.md` for global guardrails.

`agent-orchestration` is intentionally model-agnostic. A strong Codex, Qwen-family, GLM, DeepSeek, Claude, Gemini or another compatible model may act as supervisor; cheaper/faster or specialized models may act as workers when the runtime exposes safe model routing. The skill keeps one accountable supervisor, bounded concurrency, isolated writing work and final independent verification rather than hard-coding a vendor hierarchy.

For Qwen Code, use native subagents, per-agent model selection/model grades and worktree isolation when appropriate. For Codex, use the native parallel-agent/worktree primitives exposed by the active Codex surface. Missing runtime capabilities must degrade safely rather than being emulated with uncontrolled nested processes.

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

## Hermes drafts

Hermes may discover a reusable method while running scheduled research.
That does not make the method an active skill immediately.

Drafts go to:

```text
~/.local/share/vesper/skill-drafts/
```

Promotion is deliberate:

```text
observation
  -> candidate heuristic
  -> repeated trials
  -> active skill candidate
  -> review
  -> home/yargc/skills/<name>/SKILL.md
  -> nh os switch
```

This keeps self-improvement possible without letting one noisy run mutate the active skill tree.

## second brain

Hermes built-in memory is the compact hot memory for facts that should remain in future sessions.
Obsidian is the larger long-term knowledge graph.
Skills are procedural memory.

The Vesper second-brain workflow promotes only durable findings, useful relationships, important corrections, open questions and high-value source knowledge into Obsidian instead of dumping the entire scrape corpus into the vault.

A later reflection/consolidation pass may connect the day's research, update durable notes, save only compact critical facts to Hermes memory and stage reusable procedures as skill drafts.

## use them

Agents discover their normal compatibility paths automatically. You can also name a skill explicitly:

```text
use frontend-design for this page
use webapp-testing to test the local app
use mcp-builder to design this MCP server
use agent-orchestration for this multi-part coding task
use vesper-maintainer to diagnose and repair this workstation issue
use vesper-adaptive-icons for adaptive icon pipeline work
use hermes-research-radar for this scheduled research program
use vesper-obsidian-second-brain to consolidate durable research into Obsidian
```

In Qwen Code the same skill can also be invoked directly as `/agent-orchestration` after Home Manager has created the compatibility link.

## update

The Anthropic pin and active skill mapping live in `home/yargc/skills.nix`.
Local skills live under `home/yargc/skills/`.

After changing either:

```bash
nh os switch
```

Keep the active set useful and reviewed. New Hermes discoveries belong in `skill-drafts` until they have repeated evidence behind them.
