---
name: vesper-skill-authoring
description: Write or revise Vesper skills and agent-facing instruction documents with clear triggers, progressive disclosure, completion criteria and one authoritative owner. Use when creating or editing a Vesper SKILL.md, AGENTS.md, or an authoritative agent-consumed document.
---

# Vesper Skill Authoring

Write the smallest durable instruction that changes agent behavior for a real Vesper workflow.

## before writing

1. Read `AGENTS.md`, `docs/README.md` and the authoritative subsystem document.
2. Search `docs/SKILLS.md` and `home/yargc/skills/` for an existing owner. Extend it when the procedural boundary is the same.
3. Decide whether the skill is model-invoked or user-invoked. Keep descriptions specific to the branches that should trigger autonomous use.
4. Separate current behavior from target behavior. Never make a plan or draft sound implemented.

## write

- Put ordered actions in the body and end each meaningful step with a checkable completion condition.
- Keep the description as the context pointer: name the job and the concrete situations that should trigger it.
- Put rules needed on every path in `SKILL.md`; move branch-specific detail to one-level-deep references only when that reduces attention load.
- Keep one meaning in one authoritative file. Point to `AGENTS.md` and subsystem docs instead of copying their rules.
- Prefer existing Vesper commands, schemas and ownership boundaries over new wrappers or parallel control planes.
- State evidence, approval, secret and rollback boundaries where the procedure can cause mutation or external side effects.
- Use imperative language and plain names. Remove no-op advice, repeated explanations and stale examples.

## Vesper boundaries

- Local source files belong under `home/yargc/skills/` and are wired through `home/yargc/skills.nix`.
- `~/.agents/skills` is the canonical active tree; Codex, Claude Code and OpenCode paths link back to it.
- Hermes links are a curated subset. Add a skill there only when it is required by a scheduled or Hermes-owned workflow.
- Hermes research may draft changes under `~/.local/share/vesper/skill-drafts/`; drafts are inactive until reviewed and promoted.
- A skill may guide an agent but cannot grant credentials, approval, root, network or capability enforcement that the backend does not implement.

## completion checks

- frontmatter has a lowercase hyphenated `name` and a trigger-complete `description`
- the body is concise, under 500 lines, and has no duplicated repository-wide policy
- every referenced file or command exists or is explicitly marked conditional
- the skill is present in `skillSources` and `docs/SKILLS.md` describes its ownership
- run the repository checks required by the changed files; use the skill validator when the environment provides it
