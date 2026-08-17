---
name: hermes-research-radar
description: Run Vesper's high-volume daily research system for vibe coding/coding agents, Monero/privacy and adjacent Nix/Linux/security: 200-1000-item discovery, protected central sources, self-evolving source graphs, GitHub/Reddit/X scouts, evidence-ranked deep reading, primary-source verification, eval-gated research-policy evolution, Linux.do free-AI radar and second-brain handoff.
platforms: [linux]
---

# Hermes Research Radar

Treat research as a measured pipeline, not as one giant search prompt or a generic AI-news feed.

The objective is to inspect a broad frontier cheaply, preserve the user's highest-value communities, discover new source neighborhoods, spend expensive model/context budget only on promising material, verify important claims against primary evidence, and learn from which routes actually produced useful findings.

## Research profile

Prioritize, in this order:

1. **vibe coding / agentic software engineering** — Codex, Claude Code, OpenCode, Hermes, coding-agent harnesses, skills, MCP, context engineering, agent orchestration, evals, workflows, prompt/program structures, automation and overlooked developer tooling;
2. **Monero / privacy** — Monero protocol and ecosystem, Cuprate, wallets, atomic swaps, private payments, privacy tooling, Tor/onion, SimpleX, GrapheneOS and adjacent privacy engineering;
3. **Nix/Linux/security/open source** — especially when it improves Vesper, developer workflows, privacy or the two priorities above.

Do **not** spend routine frontier budget on generic local-LLM/model-quantization/inference hobby content. Model/inference material is relevant only when it materially changes coding-agent quality/cost, privacy, deployment or a concrete workflow.

Also deprioritize price charts, trading chatter, generic AI headlines, engagement bait, product-launch repetition and mainstream model benchmark noise with no actionable technical angle.

## Progressive disclosure

Keep this file as the routing and invariant layer. Load focused references only when needed:

- `references/research-pipeline.md` — every research lane
- `references/source-governance.md` — synthesis/final reporting
- `references/central-sources.md` — frontier anchors and domain source map
- `references/reddit-rss.md` — Reddit intake/deep reading
- `references/x-research.md` — X/Twitter and mirror fallback
- `references/research-evolution.md` — source/heuristic/skill evolution and eval gates

Do not dump every reference into context for every task. Scripts should do deterministic intake/canonicalization/measurement without loading their source code into model context.

## Daily architecture

The daily bundle keeps separate lanes and separate ranking rules:

1. `unknown-frontier-github`
2. `unknown-frontier-reddit`
3. `unknown-frontier-x`
4. `unknown-frontier-synthesis`
5. `free-ai-radar`
6. `agenda`

Frontier scouts ask:

`what useful technique, tool, workflow, capability or ecosystem change exists outside the user's current map?`

Agenda asks:

`what important current development should not be missed today?`

Free-AI radar asks:

`what legitimate free/cheap capability materially improves the user's coding-agent workflow?`

Do not collapse those questions into one score.

## Central anchors + autonomous expansion

The source graph has simultaneous obligations:

1. **protected central anchors** receive guaranteed inspection budget;
2. **trusted/promoted learned sources** receive adaptive exploitation budget;
3. **exploration** always keeps meaningful budget for new subreddits, accounts, repositories, authors, sites and vocabulary.

Anchors are a floor, not a ceiling. Learned sources must never crowd exploration to zero.

Candidate selection should normally reserve roughly:

- 45% central-anchor candidates
- 30% trusted/promoted/probation source candidates
- 25% query-tail/new-source exploration

If one pool cannot fill its quota, redistribute unused budget rather than padding. Within a pool, diversify across accounts/subreddits/queries so one prolific source cannot dominate merely by publishing more items.

## Hard coverage contract

A normal unknown-frontier bundle targets **200-1000 distinct canonical candidate items/URLs total** across GitHub, Reddit and X. Default runtime target is around the middle of the range.

This is candidate inspection, not 200-1000 full LLM page reads.

Use the funnel:

`cheap intake -> canonicalize/dedupe -> relevance/novelty triage -> deep read -> primary verification -> counter-review -> synthesis -> durable learning`

Deep-read only the strongest subset, normally **24-60 total**. A candidate can be an RSS entry, X post, Reddit post/comment, repository, issue, PR, commit, discussion, paper, documentation page or linked primary artifact.

Count an item once after canonicalization. Mirror copies of one X post are one source identity.

If access failure prevents the target, report actual coverage and the failure. Never manufacture numeric coverage.

## Frontier ranking

Low attention is a discovery hint, not a quality score.

Reward:

- user relevance
- novelty/information gain
- reproducibility
- technical density
- evidence potential
- early-signal value
- practical workflow impact
- source independence

Penalize:

- duplication/familiarity
- hype and engagement-only popularity
- unsupported claims
- stale/dead methods
- generic model chatter
- sources that repeatedly fail to survive deep reading

A tiny coding-agent repo with one useful primitive may outrank a major launch. A Monero issue/PR with real operational consequences may outrank a high-engagement crypto post.

## Deep reading and evidence

Community/social sources are discovery surfaces, not automatic proof.

For a strong candidate:

1. open enough context to understand the actual claim;
2. inspect relevant replies/comments/issues/commits when they can correct it;
3. follow one or two evidence-bearing links;
4. prefer source code, commit/PR/issue, official docs, release notes or papers for verification;
5. record contradictory evidence and caveats;
6. lower confidence instead of inventing corroboration.

Final synthesis should work from distilled candidate/evidence notes, not from an unbounded raw-search dump.

## Persistent adaptive state

Keep compact durable state for:

- delivered/known findings and canonical URLs
- unresolved high-value candidates
- protected anchors
- discovered source registry with tier/score/hits/failures/freshness/provenance
- mirror/feed health
- useful query/source paths
- research heuristics and counterexamples
- recent coverage and verification statistics
- open questions

Raw intake is disposable. Durable state should preserve only information that changes tomorrow's decisions.

## Self-evolution

The researcher may autonomously change **runtime research policy data**: source tiers, exploration candidates, source weights, mirror health, candidate queries, heuristic confidence and retired dead ends.

Do not equate discovery with usefulness. A newly mentioned source starts on probation; it earns credit only when a later candidate survives deep reading and contributes evidence-bearing value.

Use the lifecycle:

`observe -> probation -> repeated useful evidence -> trusted -> promoted -> decay/review -> demote/retire`

Protected anchors cannot be auto-demoted. User-excluded sources remain excluded unless the user changes the preference.

For reusable instruction/skill changes, use a stricter lifecycle:

`trajectory evidence -> draft -> representative evals -> with-skill vs current/baseline comparison -> review -> promote or reject -> rollback if regression appears`

Do not directly rewrite the Nix-owned active skill because a single run suggested a clever rule. Keep proposed procedures under `$VESPER_SKILL_DRAFT_DIR` until evaluation supports promotion. See `references/research-evolution.md`.

## Evaluation contract

Research quality is not just whether a report exists. Track and review:

- candidate coverage and shortfall
- pool diversity (anchor/dynamic/explore)
- deep-read count
- primary-verification count
- useful findings delivered
- duplicate/familiar finding rate
- source hit rate and failure rate
- access failures
- token/time cost when measurable

The skill's representative eval cases live under `evals/`. Skill evolution should compare the candidate procedure against the current one rather than judging a draft only by how convincing it sounds.

## Reporting

Every scout should expose real coverage, limitations and evidence quality. Prefer a few dense discoveries over a long weak list.

If nothing meaningful was found, say so. Do not fill quotas with familiar or irrelevant material.

## Free-AI safety boundary

Free-AI research may recommend legitimate free tiers, promotions, open-source/self-hosted alternatives, local inference when relevant, and compatibility layers.

Do not recommend leaked/shared credentials, stolen accounts, payment bypasses, abusive mass-account creation or evasion of service restrictions.
