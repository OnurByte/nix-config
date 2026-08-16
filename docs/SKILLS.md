# agent skills

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

- `vesper-maintainer`
- `hermes-research-radar`
- `hermes-automation-fleet`
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

Vesper's local workflow skills are also exposed to Hermes under:

```text
~/.hermes/skills/vesper/<skill> -> ~/.agents/skills/<skill>
```

Hermes keeps its own bundled skills alongside these links. In particular, Hermes already ships its upstream `obsidian` skill for reading, searching, creating and editing Obsidian notes. `vesper-obsidian-second-brain` complements that skill with Vesper's research-ingestion, reflection and skill-promotion policy rather than replacing it.

The active shared tree is Home Manager owned. Do not edit generated links directly.
Local skill source files live under `home/yargc/skills/` in this repository.

## Hermes automation fleet

`hermes-automation-fleet` defines how Vesper operates recurring Hermes work as one system rather than a pile of unrelated prompts.

The execution model is:

```text
cheap deterministic collection
        -> source-specific agent triage
        -> independent reports
        -> context_from synthesis
        -> final daily/weekly briefing
        -> Obsidian consolidation
```

The fleet deliberately assigns different objectives to different jobs:

```text
Unknown Frontier AI   discover useful overlooked AI
Daily Agenda          capture important current developments
Free AI Radar         discover legitimate usable free-AI opportunities
Watchdogs             deterministic silent-until-change checks
Second Brain          retain durable knowledge and relationships
Weekly Intelligence   turn accumulated evidence into decisions
```

The desired cron catalog lives in `home/yargc/hermes/automation-fleet.py` and is reconciled with Hermes runtime state through:

```bash
vesper-hermes-cron-sync
vesper-hermes-cron-sync --apply
```

The dry-run is intentional. Hermes owns mutable execution state in `~/.hermes/cron/jobs.json`; Nix owns the desired fleet definition, scripts and skills. Do not turn `jobs.json` itself into a Home Manager file.

See [`HERMES_AUTOMATION.md`](HERMES_AUTOMATION.md) for the schedule, collectors, fan-in graph, notification policy and operational commands.

## Hermes daily research lanes

`hermes-research-radar` defines separate research objectives rather than one generic daily digest.

`Unknown Frontier AI` is the broad discovery lane for overlooked AI findings across Reddit, GitHub and X/Twitter. Its GitHub and Reddit scouts use large deterministic candidate funnels before bounded agent verification; the X scout uses Hermes search directly.

`Daily Agenda` is a separate current-events lane ranked by importance, recency and consequence rather than obscurity.

`Free AI Radar` treats Linux.do as a first-class source for legitimate free AI tools, tiers, self-hosted alternatives and cost-saving workflows, then verifies promising claims outward against primary sources.

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

Hermes normal built-in memory is the compact hot memory for facts that should remain available in future interactive sessions.
Obsidian is the larger long-term knowledge graph.
Skills are procedural memory.
Scheduled research keeps its own durable Vesper state because cron sessions are isolated rather than depending on prior conversation context.

The Vesper second-brain workflow promotes only durable findings, useful relationships, important corrections, open questions and high-value source knowledge into Obsidian instead of dumping the entire scrape corpus into the vault.

A nightly reflection/consolidation pass connects the day's research, updates durable notes and stages reusable procedures as skill drafts when repeated evidence exists.

## use them

Agents discover their normal compatibility paths automatically. You can also name a skill explicitly:

```text
use frontend-design for this page
use webapp-testing to test the local app
use mcp-builder to design this MCP server
use vesper-maintainer to diagnose and repair this workstation issue
use hermes-research-radar for this research program
use hermes-automation-fleet to operate or evolve the scheduled automation system
use vesper-obsidian-second-brain to consolidate durable research into Obsidian
```

## update

The Anthropic pin and active skill mapping live in `home/yargc/skills.nix`.
Local skills live under `home/yargc/skills/`.

After changing either:

```bash
nh os switch
```

After changing the automation fleet itself:

```bash
vesper-hermes-cron-sync
vesper-hermes-cron-sync --apply
```

Keep the active set useful and reviewed. New Hermes discoveries belong in `skill-drafts` until they have repeated evidence behind them.
