---
name: x-algorithm-news
description: "Run the Vesper Turkish X news workflow: read enabled source accounts through FxTwitter, cluster and rank fresh posts, choose provenance-safe photo/video media, stage a local BPT-style draft, and inspect the XPatla analytics panel. Use when the user asks to scan X news sources, evaluate an opportunity, select source media, draft locally, inspect X algorithm signals, or reconcile a previously approved publication."
---

# x algorithm news

Use Vesper's `vesper-xpatla` state and the configured source list. The source
count is dynamic. Never assume 35 accounts or invent a missing account.

Before changing ranking or publication heuristics, read the repository's
`x-algorithm-news-account-analysis.md` and verify implementation claims against
the pinned public `x-algorithm` checkout. Treat proprietary weights as unknown.

## workflow

1. Read the local source configuration and run `vesper-xpatla sources`. The
   command output is the authority for the enabled account count.
2. Read source timelines only through FxTwitter:
   `https://api.fxtwitter.com/2/profile/<handle>/statuses`.
   Hydrate important posts through `https://api.fxtwitter.com/2/status/<id>`.
   Do not use XCancel, Nitter, generic web search or generic news RSS as a
   fallback for this workflow.
3. Keep source post ID, x.com URL, author, created timestamp, engagement
   counters, raw text, and media publisher as provenance. A mirror or embed is
   not an independent source.
4. Cluster posts that describe the same event before drafting. Require a
   primary or independently corroborated source for disputed claims. Preserve
   `confirmed`, `inferred`, and `unknown` states.
5. Rank by freshness, engagement velocity, source-relative lift, evidence,
   public interest, originality and topic diversity. Do not rank raw likes as
   truth or treat a viral source as automatically correct.
6. Read [media-contract.md](references/media-contract.md) whenever a source
   post contains a photo, video, GIF or external media.
7. Stage one original Turkish news draft locally. Use the factual payload, not
   the source wording. Keep the target voice short, direct and BPT-like. State
   attribution when the claim is not independently verified.
8. Keep this collector read-only. It never calls x-use, posts, replies, quotes,
   reposts, likes, follows, bookmarks or DMs. A separate future publisher must
   receive explicit approval, validated local media and the same reconciliation
   contract before it can write.
9. After a write, do not infer success from a browser click or x-use response.
   Re-read the account timeline, identify the exact post, hydrate it through
   FxTwitter, and mark it `confirmed` only when text, author and media agree.
   Keep an ambiguous publication for reconciliation and never blind-retry it.

## target voice

- Turkish, concise and factual.
- Put the event in the first sentence.
- Avoid copied phrases, drama, unsupported certainty, engagement bait,
  excessive hashtags and default emojis.
- Do not add a source URL to the post merely to prove provenance. Keep the
  source URL and media provenance in XPatla state and the panel.
- Do not claim a native video guarantees reach. Public x-algorithm code exposes
  predicted action surfaces, not the private model or a guaranteed boost.

## references

- [algorithm-map.md](references/algorithm-map.md) — public x-algorithm code
  mapping and confidence boundaries.
- [media-contract.md](references/media-contract.md) — media selection,
  validation, deduplication, cache and provenance rules.
- [source-profiles.md](references/source-profiles.md) — dynamic account
  configuration and editorial profile fields.
- [operations.md](references/operations.md) — scan, draft, publish and
  reconciliation commands plus evidence requirements.

Completion means the state record, provenance and validation evidence exist;
an agent's claim that it posted is not completion evidence.
