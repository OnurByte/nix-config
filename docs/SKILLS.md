# agent skills

Vesper keeps one pinned skill source and exposes the same skills to Codex, Claude Code and OpenCode through Home Manager.

The source is Anthropic's public `anthropics/skills` repository pinned to commit:

```text
f6656c1256d5a8adfa37db9110046ef20bac644c
```

Installed skills:

- `frontend-design`
- `webapp-testing`
- `web-artifacts-builder`
- `mcp-builder`
- `skill-creator`
- `pdf`
- `docx`
- `xlsx`
- `pptx`

Home Manager exposes each skill under:

```text
~/.codex/skills/<skill>
~/.claude/skills/<skill>
~/.config/opencode/skills/<skill>
```

These are links to the same immutable Nix store source. Do not edit the generated paths directly.

## use them

Agents discover skills automatically. You can also name one explicitly when you want to force the workflow:

```text
use frontend-design for this page
use webapp-testing to test the local app
use mcp-builder to design this MCP server
use skill-creator to turn this repeated workflow into a skill
use pdf to inspect and modify this PDF
```

## update

The pin lives in `home/yargc/skills.nix`.

To update the skill set, change the pinned commit there and rebuild:

```bash
nh os switch
```

Keep the list small. Add a skill when it provides a reusable workflow that the agents do not already get from shell access, MCP tools or `AGENTS.md`.
