# Ad-hoc measured research

Use this procedure when the user asks for a specific research question with an explicit breadth target such as `3000 pages`.

## Meaning of the page budget

`--pages N` is a target for **distinct canonical candidate items/pages inspected** across all research surfaces. It is not permission to claim N full deep reads.

Keep breadth and depth separate:

`candidate inspection -> dedupe -> triage -> selective deep read -> verification -> synthesis`

Report actual coverage and shortfall. Never turn a requested number into a fabricated metric.

## Mandatory surfaces

Every normal ad-hoc run attempts all four lanes:

1. GitHub: repositories, issues, PRs, commits, discussions, releases, forks, authors and dependencies.
2. Reddit: RSS/Atom intake, posts, comment branches, discovered subreddits and linked primary evidence.
3. X/Twitter: direct X when possible, XCancel/Nitter-compatible fallback, accounts, posts, replies/quotes and linked technical evidence.
4. Web: ordinary sites/forums plus Tor onion services using locally prefetched Tor content.

A failed surface is a reported limitation, not a reason to pretend it was covered.

## Large runs

Do not ask one model call to inspect thousands of items. Split large budgets into bounded waves (normally around 250 candidates). Keep waves for one surface sequential so later waves can avoid URLs already surfaced; different surfaces may run concurrently with conservative worker limits.

For a 3000-candidate run, each surface receives a non-zero budget. Default allocation is approximately GitHub 30%, Reddit 25%, X 25%, web/onion 20%. This is a planning allocation, not a quality score.

## State isolation

Store wave artifacts under a run-specific `adhoc-research/<run-id>/` directory. Read the durable frontier source graph for routing, but do not dump raw ad-hoc context into daily frontier state.

Only evidence-bearing source discoveries should reinforce the shared source registry. Final synthesis can enter the briefing/second-brain pipeline.

## OPSEC discovery seed

`r/opsec` is an explicit Reddit discovery/comment seed alongside the existing privacy/Tor/Monero sources. It is a seed, not proof and not an allowlist.
