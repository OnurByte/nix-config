# Research Pipeline

Use this reference for every Hermes research lane.

The pipeline borrows the strongest patterns from mature deep-research skills: broad exploration before deep reading, lead/scout separation, distilled notes instead of raw-context accumulation, source governance, explicit freshness and a counter-review before final synthesis.

## P0 — orient

1. Read lane state and recent delivered findings.
2. Set an `AS_OF` timestamp from the actual current date/time.
3. State the research objective in one sentence.
4. Identify 3-6 discovery angles instead of relying on one query vocabulary.
5. Determine the lane's candidate budget and deep-read budget.

For unknown-frontier daily research the combined candidate budget must normally remain between 200 and 1000 distinct canonical items/URLs.

## P1 — broad intake

Search cheaply and broadly first.

Useful cheap intake includes:

- RSS/Atom feeds
- search-result metadata/snippets
- repository/issue/PR/commit metadata
- timeline/profile/search mirrors
- author/account/repository neighborhoods
- package/release indexes

Do not spend a full deep-read on every intake item.

Normalize each candidate to a canonical identity. Deduplicate tracking parameters, mirror URLs that represent the same X post, repeated Reddit links, reposts and duplicate repository references.

Each candidate record should carry enough metadata for triage:

- canonical URL/id
- source surface
- title/summary/snippet
- author/account/repository/community when known
- publication/update time when known
- engagement only as weak context
- discovery query/feed/edge
- whether it is already known

## P2 — triage

Score candidates for expected information gain, not popularity.

A useful conceptual score is:

`novelty + relevance + utility + technical density + evidence potential + early-signal value + independence - duplication - hype - familiarity`

Use cheap deterministic filters before LLM judgement when possible:

- remove already-seen canonical URLs
- remove stale material outside the lane's horizon unless historically relevant
- remove empty/meme/promotional items
- collapse reposts/duplicate mirrors
- retain low-engagement items when the content itself has technical signal

The model should see compact candidate records, not full HTML for hundreds of pages.

## P3 — deep read

Deep-read the strongest subset, normally 24-60 items across the daily unknown-frontier bundle.

For each strong candidate:

1. open the complete thread/page/repository context;
2. follow one or two evidence-bearing links when they materially improve understanding;
3. inspect comments/issues/commits/source code when the claim depends on them;
4. extract concrete claims, limitations and reproducible details;
5. identify the primary source that can verify the claim.

Stop spending budget on a branch when additional pages become repetitive.

## P4 — evidence registry

Build a small evidence registry from deep-read notes, not from every intake item.

For each evidence source retain:

- URL
- source type (`official`, `code`, `academic`, `journalism`, `community`, `other`)
- accessibility (`public`, `semi-public`, `user-provided`)
- publication/update date when known
- authority for the specific claim
- which finding/claim it supports
- whether it is primary or corroborating

Important technical claims should prefer official docs, source code, commits, issues/PRs owned by the project, papers or first-party announcements.

## P5 — counter-review

Before final synthesis, actively test the strongest findings:

- Is this actually new, or merely new wording for a known capability?
- Is a mirror/aggregator being mistaken for independent corroboration?
- Is the claim contradicted by source code/docs?
- Is the item obsolete, rate-limited, discontinued or no longer free?
- Is a benchmark missing important caveats?
- Did engagement/popularity bias the ranking?
- Is a single-source rumor being presented as fact?

Downgrade confidence or drop the item when the answer is unfavorable.

## P6 — synthesis

Write from distilled findings and evidence registry only.

Prefer a few technically dense discoveries over a long list of weak links. Explain why each surviving item is new/useful and what evidence supports it.

Always include coverage metadata:

- candidates inspected
- canonical candidates after dedupe
- deep reads performed
- source surfaces used
- important failures/blocked surfaces
- primary-source verification count where practical

Never claim a numeric coverage value that was not actually tracked.

## P7 — learn

After delivery:

- add delivered items to known state;
- update source hit/failure history;
- retain useful newly discovered accounts/subreddits/repos/sites on probation;
- update candidate heuristics only when evidence supports them;
- record dead ends and access failures so tomorrow does not repeat them blindly.

Raw intake should remain disposable. Durable state should contain compact facts, source graph edges, heuristics, coverage and unresolved high-value questions.
