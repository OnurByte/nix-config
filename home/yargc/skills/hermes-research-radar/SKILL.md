---
name: hermes-research-radar
description: Run Vesper's persistent research system for coding agents, Monero/privacy/OPSEC and adjacent Nix/Linux/security with broad discovery, evidence-ranked deep reading, primary-source verification, adaptive evidence-backed sources and honest coverage reporting.
platforms: [linux]
---

# Hermes Research Radar

Treat research as a measured funnel, not one giant search prompt and not a generic AI-news feed.

The goal is to inspect a broad frontier cheaply, discover sources outside the current map, spend expensive context only on promising material, verify important claims and retain only state that improves later runs.

## priorities

In order:

1. **coding agents / vibe coding / agentic software engineering** — Codex, Claude Code, OpenCode, Hermes, harnesses, skills, MCP, context engineering, evals, orchestration and practical developer workflows
2. **Monero / privacy / OPSEC** — Monero, Cuprate, wallets, swaps, private payments, Tor/onion, SimpleX and privacy engineering
3. **Nix/Linux/security/open source** when it improves Vesper or the two priorities above

Do not spend routine frontier budget on generic local-model benchmarking, quantization hobby content, token prices, trading chatter, engagement bait or repeated mainstream launch summaries unless there is a concrete workflow/privacy consequence.

## focused references

Load only what the task needs:

- `references/research-pipeline.md` — research funnel
- `references/source-governance.md` — evidence and final selection
- `references/central-sources.md` — bootstrap source map
- `references/reddit-rss.md` — Reddit discovery
- `references/x-research.md` — X and mirror fallback
- `references/web-tor.md` — clearnet/onion research
- `references/research-evolution.md` — procedure/eval evolution

Do not dump every reference into every run.

## daily lanes

Keep these questions separate:

- `unknown-frontier-*`: what useful capability, technique or change exists outside the current map?
- `agenda`: what consequential current development should not be missed?
- `free-ai-radar`: what legitimate free/cheap capability materially improves coding-agent work?

The scheduled frontier uses four scouts:

1. GitHub
2. Reddit
3. X
4. web/onion

Their normal combined target is about `600` canonical candidate inspections and `48` deeper reads. These are targets, not numbers to fabricate.

## discovery contract

Seeds are bootstrap hints, never an allowlist.

Always preserve exploration outside known subreddits, accounts, repositories, sites, onion services and vocabulary. A productive known source may receive more attention, but it must not consume all discovery budget.

Low engagement is a discovery hint, not a quality score.

Reward:

- relevance
- novelty / information gain
- reproducibility
- technical density
- evidence potential
- early-signal value
- practical workflow impact
- source independence

Penalize:

- duplicates and familiar unchanged findings
- hype without technical payload
- unsupported claims
- obsolete methods
- price/trading noise
- generic model chatter
- mirrors presented as independent sources

## Reddit

Reddit RSS/Atom is useful as a cheap first pass. Use shell/curl when appropriate, then deep-read only promising threads and comment branches.

Configured subreddit/comment seeds are starting points only. Discover adjacent communities and vocabulary when evidence suggests a better route.

Community claims are not automatically proof. Follow important claims to repositories, docs, issues, PRs, releases, specifications or papers when possible.

## X

X remains a required discovery surface when relevant.

Prefer direct X when accessible. If blocked, XCancel or Nitter-compatible mirrors may be used through web or shell/curl. Canonicalize copies conceptually to the original `x.com/<user>/status/<id>` identity.

A mirror is transport, not corroboration.

Inspect replies, quotes, demos and linked artifacts when they materially change the claim.

## web and onion

Clearnet uses normal web/HTTP tools.

For `.onion` content use the local Vesper helper through shell access:

```bash
vesper-hermes-automations tor-fetch 'http://example.onion/path/'
```

Do not claim a normal clearnet web tool reached an onion service. If Tor fails, record the failure as a limitation.

Tor is transport, not an independent source.

## deep reading and verification

For a strong candidate:

1. open enough context to understand the real claim
2. inspect replies/issues/commits that can correct it
3. follow one or two evidence-bearing links
4. prefer source code, commits, PRs, issues, official docs, specifications, advisories, releases or papers for important technical claims
5. keep contradictory evidence and caveats
6. lower confidence instead of inventing verification

Final synthesis should work from distilled evidence, not an unbounded raw-search dump.

## coverage

Every scout/report should expose actual coverage when measurable:

- candidate target
- candidates inspected
- canonical candidates
- deep reads
- primary verifications
- surfaces used
- limitations

If access failures prevent the target, report the shortfall. Never manufacture coverage counts.

## durable state

Preserve compact state for:

- delivered/known findings
- recent scout reports
- unresolved useful questions
- evidence-backed source URLs
- research heuristics and counterexamples
- recent coverage and failures

Raw discovery output is disposable unless it changes a future decision.

## adaptive source registry

Vesper's Rust control plane maintains a compact registry at:

```text
~/.local/state/vesper/research/unknown-frontier-ai/source-registry.json
```

Inspect it with:

```bash
vesper-research sources
vesper-hermes-automations links
```

Current reinforcement is deliberately evidence-gated:

```text
first useful final-report source -> probation
second useful hit               -> trusted
fourth useful hit               -> promoted
```

A mention in `candidateSources`, a feed result or a prompt does not earn a hit. The URL must survive into final evidence.

Do not assume removed Python-era commands such as `links --prune` or `links --all` exist. Do not invent hidden source-GC state.

The registry is guidance, not an allowlist. Exploration remains mandatory even when promoted sources are productive.

## self-evolution

Runtime evidence may improve source choices, queries and heuristics, but reusable instruction changes require a slower loop:

`trajectory evidence -> draft -> representative eval -> compare -> review -> promote/reject -> monitor`

Stage proposed procedures under `$VESPER_SKILL_DRAFT_DIR`. Never rewrite the active Nix-owned skill automatically because one run suggested a clever rule.

Representative evals live under `evals/`.

Judge procedure changes by useful verified findings, unsupported rate, duplicate rate, source diversity, coverage honesty, access-failure reporting and token/time cost when measurable.

## reporting

Prefer a few dense discoveries over a long weak list.

For each strong finding make clear:

- what changed / what the technique is
- why it is new or useful
- what evidence supports it
- uncertainty or access limitations
- primary source URL when available

If nothing meaningful was found, say so rather than filling quotas.

## free-AI boundary

Free-AI research may recommend legitimate free tiers, promotions, open-source/self-hosted alternatives and compatibility layers.

Do not recommend leaked/shared credentials, stolen accounts, payment bypasses, abusive mass-account creation or service-restriction evasion.
