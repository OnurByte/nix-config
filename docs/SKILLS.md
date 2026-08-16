# agent skills

Vesper keeps a small pinned skill catalog and exposes the same skills globally through Home Manager.

The catalog uses immutable commits from maintained upstream repositories rather than mutable installers.

## installed

General agent work from `anthropics/skills`:

- `frontend-design`
- `webapp-testing`
- `web-artifacts-builder`
- `mcp-builder`
- `skill-creator`
- `pdf`
- `docx`
- `xlsx`
- `pptx`

Discovery and frontend stack:

- `find-skills` — Vercel Labs skill discovery through the open skills ecosystem
- `shadcn` — official shadcn skill including CLI, registry and customization guidance
- `impeccable` — design review and UI quality workflow
- `tailwindcss-development` — Laravel Boost's Tailwind CSS 4 skill

PHP and Laravel stack:

- `laravel-best-practices` — Laravel Boost's Laravel workflow and conventions
- `php-development` — modern PHP and PSR-oriented development guidance
- `mysql-best-practices` — schema, indexing, EXPLAIN and MySQL operational guidance

## global paths

Home Manager exposes every skill under the ecosystem-standard path plus explicit agent compatibility paths:

```text
~/.agents/skills/<skill>
~/.codex/skills/<skill>
~/.claude/skills/<skill>
~/.config/opencode/skills/<skill>
```

These are links into immutable Nix store sources. Do not edit the generated paths directly.

## use them

Agents discover skills automatically. You can name one when you want a specific workflow:

```text
find a maintained skill for Redis
use shadcn to build this settings form
use impeccable to review and improve this UI
use tailwindcss-development for these Tailwind v4 styles
use laravel-best-practices for this Laravel change
use php-development to refactor this PHP service
use mysql-best-practices to review this schema and query plan
```

`find-skills` can search the public skill ecosystem when the installed catalog does not cover a task. Installing a discovered skill manually is not persistent Vesper state; add useful long-term skills to `home/yargc/skills.nix` and rebuild instead.

## update

Pins and source paths live in `home/yargc/skills.nix`.

After changing them:

```bash
nh os switch
```

Keep the catalog selective. MCPs handle live services and current documentation; skills should add reusable workflows or domain judgment rather than duplicate shell access.
