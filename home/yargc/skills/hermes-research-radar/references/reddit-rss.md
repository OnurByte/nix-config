# Reddit RSS Research Playbook

Treat Reddit RSS/Atom as a cheap intake sensor that feeds an AI ranking and deep-reading stage.

RSS is not the final research product. It is the mechanism that makes broad daily coverage affordable.

## Feed surfaces

Useful public Reddit feed shapes include:

- `/r/<subreddit>/new.rss`
- `/r/<subreddit>/top.rss?t=day`
- `/r/<subreddit>/comments.rss`
- `/r/<a>+<b>+<c>/new.rss`
- `/r/<subreddit>/search.rss?q=<query>&restrict_sr=1`
- `/search.rss?q=<query>&sort=new`
- `/user/<name>/.rss`

Reddit currently returns Atom-compatible feeds on these surfaces. Treat feed availability as an implementation detail that can change: fail softly and keep an HTML/search fallback path.

## Why RSS first

RSS gives a compact stream of titles, canonical links, timestamps and summaries without spending a model call on every item.

Use it to:

1. pull hundreds of fresh candidates cheaply;
2. canonicalize and deduplicate;
3. compare against previously seen URLs;
4. classify likely relevance with cheap heuristics/model triage;
5. deep-read only the strongest threads and linked artifacts.

Do not paste hundreds of complete Reddit threads into one model context.

## Feed mix

A healthy daily Reddit intake should mix:

- combined `new` feeds across seed subreddits for breadth;
- `comments.rss` on high-signal communities to catch useful details buried outside top posts;
- narrow search feeds for active questions/terms;
- newly discovered subreddits on probation.

Seed examples are only starting points. Discover new communities from `r/<name>` mentions, crossposts, repeated authors, linked repositories and related-community edges.

## Comment-aware research

Posts are not enough. Many reproducible fixes and workflows live in comments.

RSS comment feeds are a cheap detector. When a comment candidate scores highly:

1. open the parent thread;
2. read the relevant comment branch;
3. capture code/config/error strings exactly enough to verify;
4. follow any linked repo/docs/issue;
5. check whether another commenter contradicts or corrects it.

Do not equate upvotes with correctness.

## Canonicalization and dedupe

Normalize Reddit item URLs before counting coverage. Collapse:

- old/new/www Reddit hosts when they identify the same post;
- tracking query parameters;
- the same external URL reposted into multiple feeds, while retaining subreddit provenance as metadata.

Keep both `reddit_url` and `external_url` when a submission is a link post. The external artifact may be the real primary source.

## Source graph learning

Extract and remember edges such as:

- subreddit -> subreddit mention
- subreddit -> repeated repository/domain
- author -> useful subreddit
- thread -> linked issue/PR/repo/paper

New sources start on probation. Increase their intake budget only when they repeatedly survive deep-read and synthesis.

Decrease budget for feeds that repeatedly produce duplicates, memes, unsupported claims or stale content.

## Request discipline

Use a descriptive User-Agent, bounded concurrency, caching and backoff. Prefer a small number of combined feeds over a burst of many equivalent requests.

Do not repeatedly request the same feed during one run. Persist intake metadata so retries can distinguish network failure from an actually empty feed.

## Coverage accounting

Count unique canonical feed entries/items inspected. Track separately:

- raw feed entries
- canonical candidates after dedupe
- comment candidates
- deep-read threads
- linked primary artifacts verified
- feed failures

If feeds are blocked or incomplete, say so. Do not invent a 200+ coverage number because the run was supposed to reach a target.
