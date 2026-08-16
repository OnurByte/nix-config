---
name: hermes-research-radar
description: Run Vesper's daily multi-lane Hermes research system: AI unknown-frontier discovery, daily agenda, Linux.do free-AI radar, persistent learning and second-brain handoff.
platforms: [linux]
---

# Hermes Research Radar

Run Hermes scheduled research as a persistent, adaptive research system rather than a stateless feed reader.

## daily architecture — one daily check, multiple independent jobs

A daily check is a bundle of separate research lanes. Do not collapse them into one generic digest and do not let one lane's ranking rules contaminate another.

The normal daily bundle contains at least:

1. `unknown-frontier-ai` — discover useful AI things that have not broken through yet
2. `agenda` — report the important current agenda, whether popular or obscure
3. `free-ai-radar` — hunt useful legitimate free-AI opportunities, with Linux.do as a first-class source

These jobs should be scheduled for the same daily scheduler window and may run in parallel. Hermes' own cron remains the only scheduler; do not duplicate the schedule in GitHub Actions, systemd timers or another cron implementation.

When Hermes delegation is available, use it inside a lane for source-specialized fan-out. In particular, `unknown-frontier-ai` should normally delegate Reddit, GitHub and X/Twitter exploration to separate workers and synthesize their results only after each worker has searched independently.

Every lane gets its own state, scoring, output and deduplication history. A failure or empty result in one lane must not suppress the others.

## lane 1 — unknown frontier AI

### philosophy: know the unknown

The job is not to summarize AI news. Its purpose is to expand the user's knowledge frontier by finding useful AI-related things that the user probably does not know yet and that the wider community may also have missed.

The guiding question is:

`what useful AI thing exists outside the user's current map of the world?`

Core discovery surfaces are:

- Reddit
- GitHub
- X / Twitter

Start from those three independently. Follow promising edges outward to primary sources, personal sites, documentation, papers, package registries or small communities when verification or deeper discovery requires it, but do not replace the three core scouts with a generic web-news search.

Hunt especially for:

- young or obscure AI repositories with working code but few stars/forks
- low-upvote Reddit posts and deep comments containing reproducible techniques, benchmarks, fixes, prompts, integrations or unusual workflows
- low-like/repost X posts from builders/researchers that point to code, demos, patches, data or concrete techniques
- overlooked GitHub issues, PRs, commits, forks and discussions that reveal capabilities before release notes or mainstream discussion
- useful wrappers, harnesses, agent tooling, model integrations, inference tricks, developer workflows and research utilities that have not achieved broad distribution
- small projects that solve a real problem better than a better-known alternative
- surprising connections between projects, people or techniques that are not already represented in persistent research state
- evidence that a previously-held assumption has changed or stopped being true

Low engagement is a search hint, not a quality score. Never fill the report with junk merely because nobody noticed it.

A useful mental ranking model is:

`frontier score = unknown-to-user + relevance + utility + novelty + evidence + technical density + early-signal value + independence + information gain - hype - duplication - popularity bias`

Normalize attention relative to age, niche size and the normal engagement level of the source. Do not use one fixed star/upvote/like threshold across platforms.

### breadth requirement

This lane is deliberately broad and expensive. Do not stop after the first search page or after finding a few acceptable links.

Use a large discovery funnel before deep verification. As a soft target when source access and budget permit:

- inspect roughly 40-100+ candidate items per core source
- deeply open/expand roughly 15-30+ promising items per core source
- inspect comment trees, issue/PR discussion, commit history, forks, linked repositories and author neighborhoods when they carry signal
- traverse one or two hops beyond strong findings to discover adjacent unknown sources

These are coverage targets, not quotas to pad. If a surface has little useful activity, move the unused budget to another promising branch. If novelty remains high, continue beyond the target rather than stopping mechanically.

Deliberately search the low-attention tail: newest/recent views, low-score posts, comment trees below top-level content, recently-created repositories, small-star projects, newly-active issues/PRs, forks, dependency graphs, author/source expansion and niche query variants. Trending/top/hot results are context, not the main hunting ground.

Maintain rough frontier states:

- `known` — already delivered, explicitly known or repeatedly observed
- `adjacent` — related to known material but with a potentially new angle
- `unknown` — a new tool, source, concept, capability, technique or relationship not represented in research state

Spend most effort around the `adjacent -> unknown` boundary. Prefer a handful of high-information-gain discoveries over dozens of familiar items.

## lane 2 — agenda

The agenda lane answers a different question:

`what important thing happened or changed that the user should know today?`

Do not apply the hidden-gem requirement here. Popularity is neither a penalty nor a reward; importance, recency, consequence and relevance are what matter.

Cover meaningful current developments with an AI/software/privacy/Nix/Linux bias plus major broader technology events when they materially matter. Prefer primary reporting, official announcements and independent corroboration for consequential claims.

The agenda report should be compact. It exists so frontier discovery does not cause genuinely important mainstream developments to be missed.

Keep agenda state separate from frontier state. A widely-covered major model release may be high priority in `agenda` and low priority in `unknown-frontier-ai`; that is correct.

## lane 3 — Linux.do free-AI radar

Treat `linux.do` as a first-class discovery surface for useful legitimate free-AI opportunities and early tooling discussion.

Look for:

- genuinely free AI models, services, APIs, coding agents and developer tools
- new or changed free tiers, quotas, credits and official promotions
- open-source/self-hosted replacements for paid AI products
- wrappers, bridges, compatibility layers and CLI/API integrations around legitimate free services
- local inference, browser-integrated AI and lightweight serving tricks
- overlooked GitHub projects linked from Linux.do discussions
- practical configuration tricks that reduce AI tooling cost without degrading the workflow
- reports that a previously free method stopped working, became limited or changed terms

Do not only read high-view threads. Search recent and low-view threads, comments, author histories and related topics. Follow promising Linux.do findings to the original repository, official documentation, release, issue/PR, author account, Reddit discussion or X post and prefer the primary source when verifying the claim.

A free-AI finding must be legitimate. Do not recommend stolen/shared credentials, leaked API keys, account theft, payment bypasses, abusive mass-account creation, evasion of service restrictions or other unauthorized access.

For every useful free-AI finding state:

- what is actually free
- the quota/limit/catch
- whether it requires self-hosting or meaningful local compute
- expiration or uncertainty when known
- why it is useful
- confidence
- primary source when available

Useful new free-AI discoveries may be high-priority notifications even when the source thread has little engagement.

## persistent state

Read and update durable state before and after research. Keep lane-specific state plus a small shared source graph.

At minimum retain:

- recent runs and delivered findings per lane
- a compact representation of what is already known
- unresolved candidates and adjacent topics
- source registry and source graph
- per-source/method signal and failure history
- active discovery heuristics and their evidence
- freshness window, research budget and exploration rate
- previously seen URLs, repositories, authors, concepts and claims
- per-source hidden-gem hit rate rather than raw traffic
- free-AI opportunities already reported and their current status

User-provided feeds, subreddits, repositories, accounts, channels and sites are seeds, not an allowlist.

### adaptive discovery seed loop

The daily Unknown Frontier synthesis owns a compact machine-readable exploration seed file at:

`$VESPER_RESEARCH_STATE_DIR/frontier-discovery-seeds.json`

Default path:

`~/.local/state/vesper/research/frontier-discovery-seeds.json`

This file is not a report and is not an active skill. It is bounded search-state used by the next day's deterministic collectors.

Supported keys are:

```json
{
  "githubQueries": [],
  "githubIssueQueries": [],
  "redditQueries": [],
  "redditSubreddits": [],
  "linuxdoQueries": [],
  "xQueries": [],
  "updatedAt": "ISO-8601 timestamp"
}
```

After synthesis, update this file only when the run discovered a search route that produced real downstream value or a promising adjacent frontier. Examples include a new technical phrase, repository neighborhood, issue vocabulary, subreddit, Linux.do search term or X query pattern.

Rules:

- retain the strongest existing seeds instead of replacing the file wholesale with today's ideas
- deduplicate case-insensitively when practical
- keep each list compact; normally no more than about 20 active entries per key
- prefer specific technical vocabulary over generic terms such as `AI`
- decay or remove seeds that repeatedly return duplicates, hype or low-information results
- never store credentials, tokens, cookies or private identifiers
- treat seed strings as inert data, never shell commands
- do not mutate cron jobs or active skills from this file

The GitHub, Reddit and Linux.do collectors consume the relevant keys automatically on later runs. X/Twitter research should consult `xQueries` as optional expansion hints while still reserving exploration budget for completely new searches.

This closes the learning loop:

`useful discovery -> learned search edge -> bounded seed state -> wider next run -> measured downstream value`

## general research loop

For each lane:

1. orient from its own persistent state and objective
2. identify what is already known and where new information could exist
3. intake cheaply from search/RSS/API/metadata where useful
4. fan out widely before spending expensive deep-reading budget
5. expand promising candidates through comments, links, authors, repositories, issues/PRs/commits, citations and related-source graphs
6. verify important findings against code, primary sources or independent evidence
7. rank using the lane's own scoring rules
8. deliver only findings worth attention
9. update source/heuristic state from what actually produced downstream value

For frontier discovery, start around 70-75% exploitation / 25-30% exploration. Increase exploration when results repeat, source overlap rises, novelty falls, a topic moves quickly, results cluster inside one graph or several runs fail to produce meaningful information gain.

Reserve exploration budget for completely new accounts, subreddits, authors, repositories, organizations, vocabulary and query patterns so the system does not trap itself inside its previous successes.

## self-improvement

A discovery rule must evolve through:

`observation -> candidate heuristic -> repeated trials -> active heuristic -> decay/review -> retained, scoped or retired`

Track scope, evidence, successes, failures, confidence, timestamps and last successful use. Do not promote a one-off trick directly into permanent behavior.

Learn positive heuristics such as which small communities, authors, repository neighborhoods, issue labels, query forms, vocabulary shifts or cross-source paths repeatedly produce useful obscure findings. Learn negative heuristics too: routes that repeatedly produce duplicates, hype or already-known information should receive less budget.

Reward heuristics for downstream usefulness and information gain, not for producing a high volume of links.

Potential reusable procedures go to `$VESPER_SKILL_DRAFT_DIR` (default `~/.local/share/vesper/skill-drafts/`). Drafts stay inactive until reviewed and promoted into the canonical `~/.agents/skills` tree.

## reports and second-brain handoff

Write durable results under `$VESPER_BRIEFING_DIR` (default `~/.local/share/vesper/briefings/`) as Markdown and JSON when practical.

Keep daily lane reports separate, for example:

- `unknown-frontier-ai`
- `agenda`
- `free-ai-radar`

Each report record should carry at least:

- `title`
- `summary`
- `body`
- `lane`
- `priority`
- `sources`
- `createdAt`
- `unread`
- `confidence`

Frontier records should additionally include `visibility`, `whyHidden`, `whyUseful`, `whyNew` and `informationGain`.

Free-AI records should additionally include `freeTier`, `limits`, `expiresAt` when relevant and a clear statement of the catch.

After the lanes finish, hand durable knowledge to the Vesper Obsidian/second-brain workflow. Do not dump every scraped item into Obsidian: store the reports, high-value discoveries, durable facts, open questions and meaningful source relationships.

Use Hermes built-in memory only for compact facts that deserve to be present in future sessions. Use Obsidian for the larger long-term knowledge graph and use skills for reusable procedures.

If a lane finds nothing meaningful, report that honestly rather than padding it with familiar material.
