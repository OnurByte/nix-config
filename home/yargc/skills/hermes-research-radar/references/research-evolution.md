# Research Evolution

Use this reference when changing how the researcher searches, ranks, learns or promotes reusable procedures.

## Two-speed improvement

### Runtime adaptation

The researcher may update reversible policy data automatically:

- source tiers and scores
- source freshness and failure history
- mirror health
- candidate query/source paths
- exploration weights inside configured bounds
- heuristic confidence
- known/duplicate state
- retired dead ends

These changes should be driven by downstream usefulness, not raw volume or popularity.

### Skill and procedure changes

Reusable instruction changes affect every future run, so use a slower loop:

`trajectory evidence -> candidate rule -> draft -> representative eval -> comparison -> promote or reject -> monitor -> rollback`

Do not promote a rule from one lucky run.

## Candidate rule record

Keep:

- concise rule or procedure
- where it applies and does not apply
- originating run or trajectory
- positive examples
- counterexamples
- expected benefit
- expected token/time cost
- confidence
- observation count
- evaluation status

Store proposed procedures under `$VESPER_SKILL_DRAFT_DIR`. Keep the active skill Nix-owned.

## Representative evals

Maintain realistic tests for the user's actual goals:

- discover a novel coding-agent or vibe-coding workflow without filling the result with mainstream launch news;
- find a Monero/privacy development and trace community discovery to primary technical evidence;
- keep X in the research plan when one mirror fails;
- use Reddit RSS for broad intake while deep-reading only selected threads;
- avoid generic local-model/inference material when it is not relevant;
- preserve exploration even when anchors alone can fill the candidate target;
- keep a newly discovered noisy source on probation until repeated evidence supports promotion.

Canonical cases live in `evals/evals.json`.

## Comparison metrics

When testing a draft, compare it against the current procedure or a baseline. Measure:

- relevant findings
- unsupported findings
- primary-source verification rate
- duplicate/familiar finding rate
- candidate coverage
- source diversity
- anchor/dynamic/explore balance
- access-failure reporting
- token/time cost when available
- instruction compliance

Prefer deterministic checks when practical. Use blind qualitative comparison for the parts that require judgment.

Do not promote a change that only increases verbosity, token use or source count without increasing useful verified findings.

## Promotion gate

A draft is promotion-ready only when:

1. the problem is repeated or clearly structural;
2. scope and counterexamples are explicit;
3. representative evals show no important regression;
4. quality measurably improves or a serious failure mode disappears;
5. source diversity and exploration remain healthy;
6. rollback is straightforward.

If evidence is mixed, keep testing.

## Source lifecycle

Source learning can move faster:

`discovered -> probation -> trusted -> promoted -> decay/review -> probation/retired`

Rules:

- discovery mention alone gives no useful-hit credit;
- repeated useful evidence raises tier and score;
- repeated fetch failure with zero useful hits can retire a source;
- long periods without useful output can demote or retire learned sources;
- protected anchors are exempt from automatic demotion;
- user-excluded sources remain excluded;
- a rediscovered retired source re-enters at probation.

## Query and heuristic lifecycle

Reward a query or heuristic for producing candidates that survive deep reading and synthesis, not for returning many results.

Downweight paths that repeatedly produce duplicates, generic launch chatter, irrelevant model benchmarks, memes, price/trading noise, dead pages or claims that fail verification.

Never let optimization reduce exploration to zero.

## Counter-review

Before changing research policy ask:

- Is the problem repeated or just one unlucky run?
- Would the new rule overfit one platform or source?
- Could it suppress useful unknown sources?
- Does it reward volume instead of usefulness?
- Does it create a self-confirming feedback loop?
- Can deterministic code handle the problem better than prompt text?

Use code for canonicalization, quotas, dedupe, retries and state accounting. Keep the skill focused on judgment and research procedure.

## Weekly evolution review

The weekly review should inspect:

- the active research skill and references
- `evals/evals.json`
- `$VESPER_SKILL_DRAFT_DIR`
- accumulated heuristics
- source registry tiers, hits and failures
- recent run/coverage data when available

Classify proposals as `promote`, `keep-testing`, `merge`, `narrow-scope`, `retire` or `rollback` and explain the evidence.
