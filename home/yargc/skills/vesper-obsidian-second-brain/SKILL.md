---
name: vesper-obsidian-second-brain
description: Turn Hermes research, memories and recurring insights into a durable Obsidian second brain without bloating short-term memory.
platforms: [linux]
---

# Vesper Obsidian Second Brain

Use Obsidian as the durable knowledge layer behind Hermes research.

Do not collapse every persistence mechanism into `memory`:

```text
runtime state    -> jobs, retries, logs, temporary artifacts and resume state
semantic memory  -> compact associative recall when the runtime provides it
durable context  -> decisions, rationale, projects and evidence in human-readable notes
skills           -> reviewed procedural memory loaded for a class of work
```

Hermes built-in memory is the small hot cache for facts that should be present in future sessions. Session search is episodic recall. Obsidian is the long-term second brain/durable context. Skills are procedural memory. Runtime state stays outside the vault unless a result deserves promotion.

## vault resolution

Prefer a configured `OBSIDIAN_VAULT_PATH`.

If it is not set, look for an existing vault by locating a `.obsidian/` directory under sensible user document locations. Use `~/Documents/Obsidian Vault` only if that path actually exists. Never invent a new vault or silently write into an arbitrary directory.

If no vault can be resolved, keep research output in `$VESPER_BRIEFING_DIR` and report that second-brain ingestion is pending.

## Vesper knowledge layout

Within the resolved vault, prefer a compact Hermes namespace rather than scattering files across the user's notes:

```text
Hermes/
├── Inbox/
├── Research/
│   ├── Unknown Frontier/
│   ├── Agenda/
│   └── Free AI/
├── Sources/
├── Concepts/
├── Projects/
└── Memory/
    └── Dreams/
```

Reuse an equivalent existing structure if the vault already has one. Do not duplicate folders merely to satisfy this example.

## ingestion rules

Do not mirror the scrape corpus into Obsidian. Promote only information worth keeping:

- final daily lane reports
- genuinely useful unknown-frontier discoveries
- durable technical facts
- useful relationships between tools, people, repositories or ideas
- important changes to previously-known facts
- unresolved research questions worth revisiting
- high-signal source profiles that repeatedly produce discoveries
- procedures that may later become skills

For each promoted research note, preserve enough provenance to retrace the finding. Prefer YAML frontmatter for stable metadata and normal Markdown for the note body.

Suggested metadata when relevant:

```yaml
---
type: research
lane: unknown-frontier-ai
created: YYYY-MM-DD
confidence: medium
status: active
sources:
  - https://...
tags:
  - hermes
  - ai
---
```

Use `[[wikilinks]]` when a stable concept/project/person note already exists or deserves its own durable page. Avoid link spam.

## deduplication and evolution

Before creating a note, search the vault for the same URL, repository, title, concept and close semantic equivalents.

When new evidence extends an existing note, update it instead of creating another copy. Add a dated `Updates` section when the history matters.

When a claim becomes false or obsolete, keep the historical context but mark its current status clearly. Dirty/anomalous historical observations should be marked rather than silently deleted when the history explains later decisions.

## memory boundary

Do not put long reports into Hermes `MEMORY.md`.

Use built-in Hermes memory for compact durable facts such as:

- user preferences that affect future work
- stable project/environment facts
- a source or technique that has repeatedly proven valuable
- an unresolved objective that must survive between sessions

Use Obsidian for details, evidence, long explanations, research trails and knowledge graphs.

When saving a memory fact that has a richer Obsidian note, keep the memory concise and point conceptually to the second-brain topic rather than copying the entire note.

## continuity bridge

Continuity is a lifecycle, not a request to `remember`.

At the beginning of a meaningful new session, prefer compact continuity inputs: the last meaningful state, currently active threads and unresolved objectives. Do not keep one giant conversation alive merely to preserve history.

At the end of meaningful work, leave enough durable state to answer:

- what changed
- what was decided and why
- what remains open
- what evidence/artifacts were produced
- which item should be resumed next

If a session ends without a clean handoff, a later scheduled consolidation may recover it. Use an overlap wider than the scheduler interval when collecting recent sessions so timing jitter creates deduplicatable overlap rather than gaps. Preserve previously-open items that the newest window does not explicitly close.

Provenance matters when several agents/runtimes write context. Keep enough source metadata to distinguish operator statements from agent-produced synthesis.

## dream / reflection cycle

Vesper's `dream` behavior is a research-memory consolidation workflow. Do not assume Hermes has a built-in `dreaming=true` configuration flag.

After a meaningful daily research cycle, or during a later quiet scheduled run:

1. read that day's `unknown-frontier-ai`, `agenda` and `free-ai-radar` reports
2. compare them with existing Obsidian notes and durable Hermes memory
3. deduplicate repeated claims and sources
4. identify genuinely durable facts, connections, contradictions and open questions
5. update or create the smallest useful set of Obsidian notes
6. save only compact critical facts to Hermes memory
7. stage repeatable procedures as candidate skill drafts under `$VESPER_SKILL_DRAFT_DIR`
8. write a short reflection note under `Hermes/Memory/Dreams/` only when there was meaningful synthesis

A dream note should not be a transcript summary. It should capture higher-level synthesis such as:

- what new pattern emerged
- what changed in the knowledge graph
- what previous belief was corrected
- which source path became more valuable
- which question should be investigated next
- which repeated workflow may deserve promotion to a skill

If there is no meaningful synthesis, do not create filler.

## skill learning and governance

Do not immediately rewrite active skills because one run produced a clever rule.

Use:

`observation -> repeated evidence -> candidate heuristic -> repeated trials -> skill draft -> representative eval -> review -> promotion/rejection -> monitoring`

One occurrence normally becomes an observation. Repeated operator feedback or repeated failure is a stronger reason to propose a reusable rule.

Self-improvement is maintenance and must remain bounded:

- do it after the main task, never instead of finishing the task
- cap patches/proposals per review cycle
- new reusable skills require explicit review
- unattended agents do not silently rewrite human/Nix-owned canonical skills
- repeated patches to the same skill are evidence to narrow or redesign it

Write proposed procedures to `$VESPER_SKILL_DRAFT_DIR`. Active shared skills remain under `~/.agents/skills` and are Home Manager/Nix owned.

When a reviewed draft is promoted, bind approval to the exact state that was reviewed. Record or compare the draft hash and the live target pre-image. Immediately before applying, re-read the target; if the canonical target or draft changed after review, the approval is stale and a new review is required. Never replay an old approval over intervening edits.

If several lifecycle checks/governors participate, use conservative composition: `deny > defer > allow`. If governance is configured as required but unavailable or malformed, defer instead of failing open.

Use `agent-operations` / `references/lifecycle-evals.md` for the full operational contract.

## output quality

The second brain should compound knowledge rather than accumulate noise.

Prefer fewer notes with strong provenance, clear relationships and useful future retrieval over a large archive of near-duplicate summaries.
