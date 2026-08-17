---
name: hermes-research-radar
description: Run Vesper's high-volume daily Hermes research system: broad 200-1000-item discovery, protected central sources, self-expanding source graphs, source-specialized GitHub/Reddit/X scouts, evidence-ranked deep reading, primary-source verification, durable learning, Linux.do free-AI radar and second-brain handoff.
platforms: [linux]
---

# Hermes Research Radar

Treat research as a pipeline, not as one large search prompt.

The objective is to inspect a broad information frontier cheaply, spend model/context budget only on promising material, verify important claims against primary evidence, then preserve only durable high-value findings.

## Load references before research

Read only the references relevant to the lane being executed:

- `references/research-pipeline.md` — mandatory for every research lane
- `references/source-governance.md` — mandatory before synthesis/final reporting
- `references/central-sources.md` — mandatory for frontier scouts; protected anchors plus source-evolution policy
- `references/reddit-rss.md` — mandatory for Reddit research
- `references/x-research.md` — mandatory for X/Twitter research

Do not dump every reference into the model context when a lane does not need it.

## Daily architecture

The normal bundle keeps independent lanes with independent state and ranking:

1. `unknown-frontier-github`
2. `unknown-frontier-reddit`
3. `unknown-frontier-x`
4. `unknown-frontier-synthesis`
5. `free-ai-radar`
6. `agenda`

The unknown-frontier scouts answer:

`what useful AI/software capability exists outside the user's current map of the world?`

The agenda lane answers:

`what important thing happened or changed that should not be missed today?`

The free-AI lane answers:

`what legitimate new free tier, free model, open-source replacement or cost-saving workflow became useful?`

Never collapse those questions into one ranking function.

## Central anchors + autonomous expansion

The source graph has two simultaneous obligations:

1. **protected central anchors** are inspected every normal run and cannot be silently optimized away;
2. **autonomous exploration** discovers adjacent accounts, subreddits, repositories, authors, sites and vocabulary, then feeds useful discoveries back into later runs.

Central sources are a floor, not a ceiling. They prevent the adaptive researcher from drifting away from the user's highest-value communities while leaving meaningful budget for discovering sources the user did not already know.

New sources begin on probation. Repeatedly useful sources can receive more future intake budget. Repeatedly noisy/dead sources lose adaptive budget, but explicit protected anchors remain anchors even when temporarily quiet.

## Hard coverage contract

For a normal unknown-frontier daily bundle, target **200-1000 distinct candidate items/URLs total** across GitHub, Reddit and X. The default runtime target is intentionally around the middle of that range and is split across the three scouts.

This is a discovery budget, not a requirement to fully inject 200-1000 pages into an LLM context.

Use a funnel:

`cheap intake -> normalize/dedupe -> heuristic/LLM triage -> deep read -> primary verification -> synthesis -> durable state`

A candidate may be an RSS entry, post, tweet, repository, issue, PR, commit, discussion, paper, documentation page or linked primary artifact. Count a URL once after canonicalization.

Deep-read only the strongest subset, normally **24-60 items total** unless the run remains unusually novel. Following one or two evidence-bearing links from a strong candidate is encouraged.

If source access prevents the target from being reached, never fabricate coverage. Report actual coverage and the limiting failure mode.

## Frontier philosophy

Low attention is a discovery hint, not a quality score.

Prefer:

- young/small repositories with working code
- overlooked issues, PRs, commits, forks and discussions
- low-score Reddit posts and deep comments with reproducible details
- low-like/repost X posts from builders/researchers that point to code, demos, patches, data or concrete techniques
- small communities and author neighborhoods that repeatedly produce useful early signals
- surprising cross-source connections
- evidence that a previously held assumption stopped being true

Penalize hype, duplicate coverage, generic news, engagement-only popularity and claims that cannot be traced to evidence.

## Persistent adaptive state

Keep lane-specific state plus a shared source registry. Retain at least:

- delivered/known findings
- unresolved candidates
- seen canonical URLs
- protected central sources
- discovered source/account/subreddit/repository registry
- source tier, hit/failure history and freshness
- active discovery heuristics with evidence
- open questions
- mirror/feed health where applicable
- coverage statistics from recent runs

User-provided feeds, accounts, subreddits and repositories are seeds/anchors, not an allowlist. Discover adjacent sources automatically, but keep new sources on probation until they demonstrate repeated downstream value.

A heuristic evolves through:

`observation -> candidate heuristic -> repeated trials -> active heuristic -> decay/review -> retained/scoped/retired`

Do not promote one lucky hit into permanent behavior.

## Reporting

Every scout should expose real coverage, limitations and evidence quality. Final reports should be written from distilled notes/evidence, never from an unbounded dump of raw search results.

Important claims must trace to the source that owns the claim whenever possible. Community/social sources are excellent discovery surfaces but are not automatically proof.

If nothing meaningful was found, report that honestly instead of padding the result.

## Safety for free-AI discovery

Free-AI research may recommend legitimate free tiers, official promotions, open-source/self-hosted alternatives, local inference and compatibility layers.

Do not recommend leaked/shared credentials, stolen accounts, payment bypasses, abusive mass-account creation or evasion of service restrictions.
