# agent skills

Vesper exposes one declarative skill set to Codex, Claude Code and OpenCode through Home Manager. Every upstream repository is pinned to an immutable commit so a rebuild cannot silently change agent behavior.

## installed skills

| Skill | Source | Pin |
| --- | --- | --- |
| `frontend-design` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `webapp-testing` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `web-artifacts-builder` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `mcp-builder` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `skill-creator` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `pdf` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `docx` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `xlsx` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `pptx` | `anthropics/skills` | `f6656c1256d5a8adfa37db9110046ef20bac644c` |
| `shadcn` | `shadcn-ui/ui` | `d4fc45b1fbabfccb7a6a4333d8004cf19481caa9` |
| `tailwindcss-development` | `laravel/boost` (official Tailwind v4 skill) | `243045b97c4ea22f3838a35e79c496ed3df74cdb` |
| `laravel-best-practices` | `laravel/boost` | `243045b97c4ea22f3838a35e79c496ed3df74cdb` |
| `find-skills` | `vercel-labs/skills` | `c6f69c631292444cc541ac6d91e2226b0ff247da` |
| `mysql` | `planetscale/database-skills` | `af0ce0cfb65cca4cc21d18ca0d9cf270ca99d488` |
| `php-best-practices` | `AsyrafHussin/agent-skills` | `2631530e9d47c45f6d153ed9f245f073ccbbba30` |
| `impeccable` | `pbakaus/impeccable` | `9ce0350054b0199bfd0ebbde95d9fd70c7c91741` |

`laravel-best-practices` is Laravel's own Agent Skill and already covers Laravel backend PHP style, Eloquent, migrations and database performance. Laravel Boost also ships generic PHP guidance, but that guidance is a generated Blade template rather than a standalone `SKILL.md`; `php-best-practices` is therefore intentionally sourced separately. The MySQL skill is PlanetScale's dedicated MySQL/InnoDB skill.

Home Manager exposes every skill under:

```text
~/.codex/skills/<skill>
~/.claude/skills/<skill>
~/.config/opencode/skills/<skill>
```

These are links into immutable Nix store sources. Do not edit the generated paths directly.

## use them

Agents discover skills automatically. You can also name one explicitly when you want to force a workflow:

```text
use shadcn for this component
use tailwindcss-development for this layout
use impeccable to critique and polish this UI
use laravel-best-practices for this Laravel change
use php-best-practices to review this PHP code
use mysql to review this schema/query
use find-skills to find a reusable skill for this task
```

## update

Pins and skill paths live in `home/yargc/skills.nix`.

Update the relevant commit there, verify the upstream skill directory still contains `SKILL.md`, then rebuild:

```bash
nh os switch
```

Keep the set focused. Prefer official or well-maintained upstream skills over copying guidance into the config.
