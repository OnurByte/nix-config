# Skill lifecycle, bounded self-improvement, QA and model evals

Use this reference when an agent wants to change a reusable rule, promote a skill draft, review its own work, or compare models/routes.

## self-improvement is governed maintenance

Do not let every correction immediately rewrite active behavior.

Use this progression:

```text
feedback / failure
-> observation
-> repeated evidence
-> candidate heuristic
-> skill draft or bounded patch proposal
-> representative eval
-> review
-> promote / reject / keep testing
-> monitor for regression
```

One occurrence is usually evidence, not a universal rule. A repeated operator correction or repeated failure is a stronger candidate for procedural memory.

Self-improvement happens **after the main task**, not in the middle of delivering it. The main job must not be starved because the agent became fascinated with maintaining its own instructions.

Put explicit budgets around maintenance. Exact numbers are local, but the system needs limits for:

- patches proposed per task/round
- self-improvement rounds per day
- new skill creation
- automatic lifecycle transitions

A new reusable skill or a broad behavioral rewrite requires review. Human-authored canonical rules are read-only to unattended self-improvement unless the operator explicitly stages a change.

If the same active skill needs constant patches, that is evidence the skill is too broad, poorly written or missing a better deterministic mechanism.

## read-before-write and stale approval protection

A background reviewer must read the exact current target before proposing a mutation. A cached summary, old prompt copy or earlier review is not authorization to patch the current file.

When a skill change is staged for approval, bind approval to the state that was actually reviewed.

A review record should conceptually retain:

```json
{
  "target": "skill/file",
  "targetPreimageSha256": "hash at review time",
  "draftSha256": "hash of proposed content",
  "origin": "who/what proposed it",
  "decision": "allow|deny|defer"
}
```

Immediately before applying an approved write:

1. re-read the live target
2. recompute the pre-image hash
3. recompute/confirm the draft hash
4. reject the approval as stale if either reviewed identity changed
5. preserve intervening human/agent edits and require a fresh review

Never replay an old approval onto a target that changed after staging.

If multiple lifecycle governors participate, combine decisions conservatively:

```text
deny > defer > allow
```

When lifecycle governance is configured as required, a missing, crashed or malformed governor response fails closed to `defer`; it never silently grants authority.

Lifecycle transitions should have stable IDs, explicit from/to states, reason and origin so decisions can be audited without guessing from logs.

## skill provenance and ownership

Keep provenance for reusable rules:

```text
human-authored / Nix-owned canonical skill
agent-created draft
background review proposal
explicit operator promotion
```

Automatic agents may inspect all of them but must not silently make a human-owned canonical file their mutable scratchpad.

A promotion should preserve the evidence that justified it: repeated examples, eval result, known counterexamples and the reason it is better than the previous behavior.

## QA convergence

QA exists to block material defects, not to discover an endless sequence of stylistic improvements.

Deterministic checks run first for properties code can decide: schema, lengths, empty sections, forbidden tokens, hashes, counts, formatting and hard policy constraints.

Model QA runs in a clean context with:

- source/requirements
- produced artifact
- written quality standard/checklist

Do not give the reviewer the author's persuasive reasoning as evidence that the output is correct.

Use explicit severities:

```text
error    -> blocks; a required correction
warning  -> does not block; useful operator note/preference
reject   -> stop/escalate; unusable or unsafe artifact
```

The verdict should be derived mechanically from findings, not left to a model that can always imagine one more improvement.

```text
any reject -> reject
else any error -> revise
else -> pass
```

Bound iterative QA. Default operational pattern:

```text
round 1 -> inspect full artifact
round 2 -> inspect only required changes from round 1
round 3 -> no automatic polish loop; escalate if blocking errors remain
```

Round 2 may not reopen already-passed stylistic questions unless it discovers a genuinely blocking correctness/safety error.

Name artifacts by factual round/state (`qa-round-1`) rather than optimistic names such as `final-approved` whose contents may still say revise.

Measure QA convergence over time. If almost every artifact needs the maximum number of rounds, fix the producer instructions or deterministic prechecks instead of making QA more aggressive.

## blind model and route evaluation

Model routing should eventually be backed by representative local evals, not vendor reputation or one memorable success.

For comparative tests:

- use the same prompt/input/constraints for each route
- hide model/provider identity from the evaluator when practical
- retain raw untouched outputs
- measure quality together with latency and real access-channel cost/limits when available
- include known-good material and deliberate false-positive traps
- score unsupported claims and fabricated defects, not only true positives
- keep a separate corrected/fixed artifact if humans repair a broken output; never overwrite the raw model result

A model that confidently invents a nonexistent bug can be more dangerous than one that misses a minor bug because the false positive becomes an unnecessary destructive change downstream.

Useful dimensions vary by task but may include:

```text
correctness
instruction adherence
evidence fidelity
false-positive rate
unsupported-claim rate
completion/acceptance success
latency
cost or quota pressure
```

One run is a sample, not a universal ranking. Repeat representative tests when changing a default route and keep uncertainty visible.

The evaluator should not know which result came from the preferred/fashionable model if that knowledge can bias scoring.

## context refinery eval boundary

A deep research/reasoning model may be valuable as a **context refinery** even if it is not the best executor.

Prefer:

```text
deep reviewer/refinery
-> verified findings / hypotheses / contradictions / unknowns
-> planner converts evidence to executable steps
-> agent workers implement
```

Do not allow hypothesis wording to harden into fact as it moves downstream. Keep uncertainty labels until live verification resolves them.

## session pressure

A giant long-lived session is not procedural memory. If context repeatedly compresses, latency grows or accuracy visibly degrades, checkpoint durable state and start a fresh session from the continuity bridge.

Continuity should come from state/context files, not from keeping one conversation alive forever.

## thresholds are measured, not inherited

Timeouts, QA round budgets, stale windows, concurrency and maintenance caps from another system are starting hypotheses only.

Choose Vesper defaults conservatively, instrument actual behavior, then tune from evidence. Preserve the failure-derived invariant even when the numeric threshold changes.
