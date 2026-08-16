# Hermes Research Radar

Run Hermes scheduled research as a persistent, adaptive research system rather than a stateless feed reader.

## primary objective

The radar is not a popularity feed. Its main job is to surface useful things before they become widely known.

Actively hunt low-attention, high-signal findings on X/Twitter, Reddit, GitHub and Linux.do: posts, comments, repositories, issues, pull requests, commits, experiments, techniques and tools that have little engagement but unusually high practical or technical value.

Treat popularity as weak context, never as a quality score. Low engagement is also not automatically good. A hidden gem should combine low visibility with strong utility, novelty, evidence or technical density.

Prefer candidates that look like one or more of these:

- a young or obscure repository with working code and a clear useful idea despite few stars/forks
- a low-upvote Reddit post/comment containing reproducible technical detail, an unusual fix, benchmark, workflow or source
- a low-like/repost X/Twitter post from a credible builder/researcher that links to code, data, a patch, a demo or a concrete technique
- an overlooked GitHub issue/PR/commit that reveals an upcoming feature, fix, workaround or implementation detail before release notes or mainstream discussion catch up
- a Linux.do thread/comment exposing a useful new AI tool, free tier, open-source alternative, local/self-hosted workflow, API/CLI technique, compatibility layer, temporary official promotion or practical setup trick before it spreads elsewhere
- a small project solving a real problem more cleanly than a better-known tool
- an independent source that contradicts repeated social consensus with stronger evidence

Do not fill the briefing with merely unpopular content. The target is obscurity with value.

A useful mental ranking model is:

`hidden-gem score = relevance + utility + novelty + evidence + technical density + early-signal value + independence - hype - duplication - popularity bias`

Do not use fixed star/upvote/like thresholds across platforms. Normalize attention relative to the age of the item, the size of its community/account and the normal engagement level of that niche. A 20-star repository in a tiny new ecosystem can be more meaningful than a 2,000-star generic project.

## linux.do and free-ai discovery

Treat `linux.do` as a first-class seed/source for early AI discoveries, not as an occasional fallback.

Continuously look for newly posted or newly resurfaced information about:

- genuinely free AI models, services, APIs, coding agents and developer tools
- new or changed free tiers, quotas, credits and official promotions
- open-source/self-hosted replacements for paid AI products
- wrappers, bridges, compatibility layers and CLI/API integrations that make an existing legitimate free service easier to use
- local inference, browser-integrated AI and lightweight model-serving tricks
- overlooked GitHub projects linked from Linux.do discussions
- practical configuration tricks that reduce AI tooling cost without degrading the workflow
- reports that a previously free method stopped working, became limited or changed terms

Follow promising Linux.do discoveries outward to the original project, official documentation, repository, release, issue/PR, author account, Reddit discussion or X/Twitter post. Prefer the primary source when verifying the claim.

A "free AI trick" must be useful and legitimate. Do not recommend stolen/shared credentials, leaked API keys, account theft, payment bypasses, abusive mass-account creation, evasion of service restrictions or other unauthorized access. Separate official free tiers/promotions, open-source/self-hosted methods and ordinary product quirks from questionable claims.

When a genuinely useful new free-AI finding appears, give it elevated notification priority even when the source thread itself has little engagement. Include what is free, the important limit/catch, why it is useful, how confident the finding is and the primary source when available.

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
- previously seen URLs, repositories, authors and claims for deduplication
- per-source hit rate for hidden-gem discoveries rather than raw traffic
- free-AI opportunities already reported and their current status so expired or dead tricks are not repeatedly resurfaced

User-provided feeds, subreddits, repositories, accounts, channels and sites are seeds, not an allowlist.

## research loop

1. orient from persistent state and the current job
2. intake cheaply: RSS/Atom and APIs/search metadata first, then extraction, discussion, browser or video only when useful
3. deliberately inspect low-ranking and low-engagement result tails instead of only top/hot/trending results
4. expand through links, crossposts, authors, followers/following when useful, GitHub docs/issues/PRs/commits, Linux.do related threads, blogrolls, citations, transcripts, curated lists and generated queries
5. verify important findings against code, primary sources or independent evidence
6. rank for hidden-gem value, novelty, usefulness, evidence, independence, information density, non-obvious discovery and low duplication
7. deliver only findings worth the user's attention
8. learn from which sources and methods produced signal

Search strategies should intentionally escape popularity ranking. Examples include newest/recent result views, small subreddits, comment trees below top-level posts, low-view Linux.do threads/comments, GitHub repositories with low star counts, recently-created repositories, recently-active issues/PRs, niche topic queries and author/source expansion from previously successful discoveries.

Start around 80% exploitation / 20% exploration. Increase exploration when findings repeat, source overlap rises, novelty falls, a topic shifts quickly or results cluster inside one social/source graph.

Keep some exploration budget reserved for completely new accounts, subreddits, Linux.do authors/topics, repositories, organizations and query patterns so the system does not become trapped in its own successful source bubble.

## self-improvement

A new rule must move through:

`observation -> candidate heuristic -> repeated trials -> active heuristic -> decay/review -> retained, scoped or retired`

Track scope, evidence, successes, failures, confidence, timestamps and last successful use. Do not promote a one-off trick directly into permanent behavior.

Learn discovery heuristics such as which small communities, Linux.do topics/authors, maintainers, repository neighborhoods, issue labels, query forms or cross-source paths repeatedly produce useful obscure findings. Reward heuristics for downstream usefulness, not for producing a high volume of links.

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
- `visibility` or comparable attention estimate
- `whyHidden`
- `whyUseful`
- `confidence`
- `freeTier` / `limits` / `expiresAt` when relevant to a free-AI finding

For each hidden-gem item explain briefly why it was probably missed and why it is useful. Include the engagement/visibility context when available, but do not confuse those metrics with truth or quality.

For free-AI discoveries, explicitly state the catch: quota, model limits, region/account requirements, expiration, self-hosting cost or uncertainty. If a previously reported opportunity materially changes or dies, a `watch` job may notify about that change.

Prefer a short desktop notification for the interrupt and keep the full report in the briefing inbox. Useful new free-AI discoveries should be eligible for immediate/high-priority notification; watch jobs should otherwise stay quiet when nothing meaningful changed.
