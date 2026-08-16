# Hermes Research Radar

Run Hermes scheduled research as a persistent, adaptive research system rather than a stateless feed reader.

## scheduling contract

Hermes' own cron/scheduled automation layer is the heartbeat. Do not create parallel GitHub Actions schedules, systemd timers or extra cron jobs for the same workflow.

Each scheduled run resumes from persistent state and continues the same research program. A run may be one of three classes:

- `briefing` — recurring concise digest, normally daily
- `research` — deeper bounded investigation into a specific question
- `watch` — check a condition and surface output only when there is a meaningful change

The schedule decides when to wake up, not what to relearn from scratch.

## persistent state

Read and update durable state before and after each run. At minimum retain:

- recent runs and delivered findings
- unresolved candidates
- source registry and source graph
- per-source/method signal and failure history
- active heuristics and their evidence
- freshness window, research budget and exploration rate

User-provided feeds, subreddits, repositories, channels and sites are seeds, not an allowlist.

## research loop

1. orient from persistent state and the current job
2. intake cheaply: RSS/Atom first, then metadata, extraction, discussion, browser or video only when useful
3. expand through links, crossposts, authors, GitHub docs/issues/PRs, blogrolls, citations, transcripts, curated lists and generated queries
4. verify important findings against independent evidence
5. rank for novelty, usefulness, evidence, independence, information density, non-obvious discovery and low duplication
6. deliver only findings worth the user's attention
7. learn from which sources and methods produced signal

Start around 80% exploitation / 20% exploration. Increase exploration when findings repeat, source overlap rises, novelty falls, a topic shifts quickly or results cluster inside one social/source graph.

## self-improvement

A new rule must move through:

`observation -> candidate heuristic -> repeated trials -> active heuristic -> decay/review -> retained, scoped or retired`

Track scope, evidence, successes, failures, confidence, timestamps and last successful use. Do not promote a one-off trick directly into permanent behavior.

Potential reusable skills go to `$VESPER_SKILL_DRAFT_DIR` (default `~/.local/share/vesper/skill-drafts/`). Drafts stay inactive until reviewed and promoted into the canonical `~/.agents/skills` tree.

## delivery

Write durable results under `$VESPER_BRIEFING_DIR` (default `~/.local/share/vesper/briefings/`) as both Markdown and JSON when practical.

A briefing record should carry:

- `title`
- `summary`
- `body`
- `type` (`briefing`, `research` or `watch`)
- `priority`
- `sources`
- `createdAt`
- `job`
- `unread`

Prefer a short desktop notification for the interrupt and keep the full report in the briefing inbox. Watch jobs should stay quiet when nothing meaningful changed.
