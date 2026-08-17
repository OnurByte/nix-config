# Source Governance

Apply this reference during synthesis and final reporting.

## Source classes

Classify evidence by what it actually is:

- `official` — first-party docs, release notes, status pages, announcements
- `code` — repository source, commits, tags, issues/PRs owned by the project
- `academic` — papers, proceedings, datasets
- `journalism` — reported secondary coverage
- `community` — Reddit, X, forums, comments, personal posts
- `other` — mirrors, aggregators, indexes and uncategorized material

A mirror inherits the evidentiary identity of the content it mirrors. It is not a second source.

## Primary-source rule

Community and social sources are discovery surfaces. For important technical claims, follow the claim back to the source that owns it whenever possible.

Examples:

- feature exists -> docs/source/commit/release
- bug fixed -> issue/PR/commit/release containing the fix
- project supports protocol/model -> code/docs/tests
- free tier changed -> official pricing/docs/terms
- benchmark -> original benchmark methodology/data
- research result -> paper/preprint/project page

If primary verification is unavailable, label the claim accordingly and lower confidence.

## Freshness

Every time-sensitive run has an `AS_OF` date.

Prefer the newest authoritative material for claims about current support, pricing, availability, releases, security posture and service behavior. Older sources may explain history but should not override a newer first-party state without evidence.

## Independence

Two pages copying the same claim are not two independent confirmations.

Watch for:

- mirrors of the same X post
- news articles sourcing the same press release
- forks copying the same README
- Reddit posts linking the same unverified tweet
- aggregator pages repeating one upstream announcement

Track provenance so synthesis can distinguish independent evidence from repetition.

## Confidence

Use confidence based on evidence quality, recency and independence rather than source count alone.

A useful qualitative mapping:

- high — current primary source plus successful technical verification/corroboration
- medium — strong first-party statement or multiple independent credible sources with limited verification
- low — single community source, stale evidence, blocked primary source or unresolved contradiction

Never make confidence high merely because many mirrors repeat the same content.

## Negative evidence

Record important failed verification attempts:

- docs do not mention claimed capability
- repository/tag does not contain claimed release
- endpoint is unavailable
- free method appears discontinued
- benchmark cannot be reproduced from available information

Absence is not automatically proof of falsehood, but it matters to confidence and should survive into the counter-review.

## Final selection

A final report should optimize for information gain and decision usefulness, not source volume.

Drop a candidate when it is:

- already known and materially unchanged
- pure hype with no technical payload
- duplicate coverage
- unsupported after reasonable verification
- obsolete for the current question
- outside the lane objective

Keep a weaker-evidence item only when its early-signal value is high, and label the limitation clearly.
