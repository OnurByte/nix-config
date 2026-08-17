# X / Twitter Research Playbook

Use X as a first-class discovery surface. Do not silently replace it with generic web search when direct access is inconvenient.

## Access ladder

Use the first healthy path that provides the needed data:

1. direct `x.com` / `twitter.com` pages or search when accessible;
2. XCancel;
3. another configured Nitter-compatible mirror;
4. search-engine discovery constrained to X/Twitter URLs;
5. linked primary artifacts (GitHub, docs, paper, demo, release) for verification.

Mirrors are transport fallbacks, not independent evidence. An XCancel page and the corresponding X page represent the same source.

## Nitter-compatible routes

Current Nitter source supports profile/timeline/search routes and RSS for user/search surfaces when enabled by the instance. Useful shapes include:

- `/<user>`
- `/<user>/rss`
- `/<user>/with_replies`
- `/<user>/with_replies/rss`
- `/search?f=tweets&q=<query>`
- `/search/rss?f=tweets&q=<query>`

Do not assume every public instance enables RSS. RSS and search may be blocked, challenged or temporarily broken even when ordinary HTML pages work.

## Mirror fallback rules

Keep a configurable mirror list. For every mirror remember:

- last success
- last failure
- supported surface (`html`, `rss`, `search`, `profile`)
- failure reason when useful
- temporary backoff/circuit-breaker state

Prefer XCancel first only while it remains healthy. If RSS fails, fall back to HTML on the same instance before abandoning the mirror. If the instance fails repeatedly, move to the next configured mirror rather than retrying aggressively.

Never hammer public mirrors. Space requests, keep bounded concurrency and reuse already fetched results.

## Discovery strategy

Explore at least four X dimensions when budget permits:

1. query/search tail — niche vocabulary, new terminology, exact error strings, project names;
2. builder/researcher accounts — low-attention posts that link code or demos;
3. conversation graph — replies, quotes and nearby accounts that add technical detail;
4. external-link graph — GitHub repos, docs, papers, demos and release notes referenced by posts.

Trending/high-like posts are context, not the primary frontier. Search newest/recent results and low-attention builders deliberately.

Seed accounts are not an allowlist. Discover adjacent accounts from replies, mentions, quoted posts, repository authors and repeated external links. Put new accounts on probation and keep them only when they produce downstream value.

## Canonicalization

Normalize mirror URLs to canonical X identities before dedupe.

For example, these should become one candidate:

- `https://x.com/alice/status/123`
- `https://twitter.com/alice/status/123`
- `https://xcancel.com/alice/status/123`
- `https://<nitter-mirror>/alice/status/123`

Use the canonical post id plus username when available. Strip mirror-only query parameters.

## Deep reading

A promising X post should trigger deeper inspection when it contains or points to:

- code/repository
- patch/commit/issue/PR
- benchmark or reproducible measurement
- demo
- paper/data
- concrete integration/workflow
- a surprising technical claim that can be verified

Read enough replies/quotes to catch corrections or missing caveats. Then follow the strongest external artifact to its primary source.

## Evidence rules

X is excellent for early discovery but weak as sole proof for consequential technical claims.

For important findings:

- builder says feature exists -> verify in repo/docs/commit when possible;
- benchmark claim -> inspect methodology/source data when available;
- release claim -> verify release/tag/changelog;
- outage/deprecation/free-tier change -> verify first-party status/docs/terms if available.

If no primary verification exists, keep the finding but label it single-source/community evidence and lower confidence.

## Coverage accounting

Count canonical posts/items actually inspected. Do not count the same post again merely because it was reached through another mirror or query.

Report direct-X and mirror failures explicitly so synthesis can distinguish lack of signal from lack of access.
