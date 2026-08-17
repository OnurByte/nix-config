# Research Evolution

Use this reference when changing how the researcher searches, ranks, learns or promotes reusable procedures.

## two-speed improvement

### Runtime adaptation

Runtime state may change quickly when evidence supports it:

- evidence-backed source tiers and scores
- candidate query/source paths
- mirror or access observations recorded in reports
- heuristic confidence
- known/duplicate state
- open questions

These changes should be driven by downstream usefulness, not raw volume or popularity.

### Skill and procedure changes

Reusable instruction changes affect every future run, so use a slower loop:

```text
trajectory evidence
  -> candidate rule
  -> draft
  -> representative eval
  -> comparison
  -> promote or reject
  -> monitor
```

Do not promote a rule from one lucky run.

## candidate rule record

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

## representative evals

Maintain realistic tests for the user's actual goals:

- discover a novel coding-agent workflow without filling the result with mainstream launch news
- find a Monero/privacy development and trace community discovery to primary technical evidence
- keep X in the research plan when direct access or one mirror fails
- use Reddit RSS for broad intake while deep-reading only selected threads
- avoid generic local-model/inference material when it is not relevant
- preserve exploration even when known sources alone can fill the candidate target
- keep newly useful sources on probation until repeated final-report evidence supports promotion

Canonical cases live in `evals/evals.json`.

## comparison metrics

Compare a draft against the current procedure or a baseline. Measure:

- relevant findings
- unsupported findings
- primary-source verification rate
- duplicate/familiar finding rate
- candidate coverage
- source diversity
- known-source vs exploration balance
- access-failure reporting
- token/time cost when available
- instruction compliance

Prefer deterministic checks where practical. Use qualitative comparison only where judgment is necessary.

Do not promote a change that only increases verbosity, token use or source count without increasing useful verified findings.

## promotion gate

A draft is promotion-ready only when:

1. the problem is repeated or clearly structural
2. scope and counterexamples are explicit
3. representative evals show no important regression
4. quality measurably improves or a serious failure mode disappears
5. source diversity and exploration remain healthy
6. rollback is straightforward

If evidence is mixed, keep testing.

## current source lifecycle

The Rust source registry uses a deliberately small positive-evidence lifecycle:

```text
first final-report evidence -> probation
second useful hit          -> trusted
fourth useful hit          -> promoted
```

Rules:

- discovery mention alone gives no useful-hit credit
- feed/mirror/candidate presence alone gives no useful-hit credit
- repeated useful final-report evidence raises tier and score
- promoted sources remain hints, never an allowlist
- exploration must continue outside the registry

Do not assume the removed Python-era automatic demotion/84-hour GC/tombstone machinery exists. If negative-evidence source retirement is reintroduced later, add it as explicit Rust state logic and tests before documenting it as active behavior.

## query and heuristic lifecycle

Reward a query or heuristic for producing candidates that survive deep reading and synthesis, not for returning many results.

Downweight paths that repeatedly produce duplicates, generic launch chatter, irrelevant model benchmarks, memes, price/trading noise, dead pages or claims that fail verification.

Never let optimization reduce exploration to zero.

## counter-review

Before changing research policy ask:

- Is the problem repeated or one unlucky run?
- Would the new rule overfit one platform or source?
- Could it suppress useful unknown sources?
- Does it reward volume instead of usefulness?
- Does it create a self-confirming feedback loop?
- Can deterministic Rust/state logic handle the problem better than prompt text?

Keep deterministic code focused on state, scheduling, safety, canonical interfaces and accounting. Keep the skill focused on judgment and research procedure.

## weekly evolution review

The weekly review should inspect:

- active research skill and references
- `evals/evals.json`
- `$VESPER_SKILL_DRAFT_DIR`
- accumulated heuristics/open questions
- source registry tiers and useful hits
- recent run/coverage data when available

Classify proposals as `promote`, `keep-testing`, `merge`, `narrow-scope`, `retire` or `rollback` and explain the evidence.
