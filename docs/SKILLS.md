# agent skills

Vesper keeps one canonical active skill tree at `~/.agents/skills`.
Codex, Claude Code and OpenCode expose their normal skill paths as links back into that tree so there is one active copy to reason about.

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

The active tree is Home Manager owned. Do not edit generated links directly.
Local skill source files live under `home/yargc/skills/` in this repository.

## hermes drafts

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

## use them

Agents discover their normal compatibility paths automatically. You can also name a skill explicitly:

```text
use frontend-design for this page
use webapp-testing to test the local app
use mcp-builder to design this MCP server
use vesper-maintainer to diagnose and repair this workstation issue
use hermes-research-radar for this scheduled research program
```

## update

The Anthropic pin and active skill mapping live in `home/yargc/skills.nix`.
Local skills live under `home/yargc/skills/`.

After changing either:

```bash
nh os switch
```

Keep the active set useful and reviewed. New Hermes discoveries belong in `skill-drafts` until they have repeated evidence behind them.
