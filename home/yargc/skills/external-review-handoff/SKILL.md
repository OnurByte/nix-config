---
name: external-review-handoff
description: Prepare and verify deep external-model reviews when a strong model sees only a static code snapshot or uploaded files instead of the live repository. Use for ChatGPT/Pro-style code audits, architecture reviews, bug hunts or optimization research: build a secret-safe subsystem snapshot, author a high-signal prompt, then verify every returned finding against the live tree before implementation.
platforms: [linux]
---

# External Review Handoff

Use a static-snapshot frontier model as a deep reviewer, not as the source of truth.

Core law: **the snapshot model proposes; the live repository decides.**

A model that cannot read the current working tree, run tests or observe the runtime may still produce unusually deep analysis. Its limitation is staleness and lack of execution evidence. This workflow preserves the upside without rubber-stamping snapshot conclusions.

## when to use

Use this skill when:

- the operator wants to upload a repository/subsystem to a non-agentic or out-of-band model
- a deep external model should audit one hard architecture/correctness/security problem
- an external report comes back and needs validation before code changes
- the live coding agent wants an independent deep reasoning pass that should not receive broad machine access

Do not use it when the selected reviewer already has trustworthy live-repo access and can be handled as a normal reviewer through `agent-orchestration`.

## handoff lifecycle

```text
scope
  -> snapshot
  -> secret gate
  -> prompt
  -> external review
  -> receive report
  -> live verification
  -> triage
  -> implementation
```

Never skip the live verification gate.

## 1. scope one hard problem

A useful handoff has one coherent decision to make, not `review everything`.

Pin:

- **decision** — critique, chosen design, prioritized fix list, bug hunt, performance diagnosis, etc.
- **frame** — current architecture, constraints and decisions that are already settled
- **focus** — relevant files/subsystems and empirical signals
- **anti-focus** — tempting but out-of-scope rewrites or unrelated cleanup
- **invariants** — boundaries that recommendations must not weaken
- **intent** — `audit`, `exploration` or `blended`

For `audit`, use a tight brief. For `exploration`, supplied ideas are seeds, not a fence. For `blended`, label the two halves explicitly.

## 2. build a review snapshot

Prefer tracked source from a pinned Git commit over copying the whole working directory.

The snapshot should include:

- the relevant source by architectural subsystem
- tests because they encode contracts
- authoritative docs/ADRs/`AGENTS.md`
- schema and migrations when architecture depends on them
- a small manifest containing repository name, commit SHA, export timestamp and whether the live tree was dirty

Exclude by default:

- `.git`
- `.env*` and secret files
- build/cache output
- `node_modules` or equivalent dependency trees
- logs and temporary files
- binary/media assets unless required by the question
- database rows, dumps, seed data with real user/content data
- machine-local state
- credential stores, private keys and auth/session material

Split by **subsystem, not equal byte size**. The external model should be able to answer `which bundle contains this responsibility?` from the architecture.

For a small repository, one source bundle plus one docs bundle may be enough. Do not fragment a coherent subsystem merely to create more files.

Store handoff artifacts outside Git by default, for example:

```text
~/.local/share/vesper/review-handoffs/<repo>/<handoff-id>/
```

Do not commit prompts, reports or bundles unless the operator explicitly wants them versioned.

## 3. secret gate

Before any bundle leaves the machine, scan the staged snapshot rather than trusting filename exclusions alone.

Treat likely provider-issued credentials and private-key material as **hard hits**. Examples include:

- private-key headers
- GitHub tokens
- cloud access keys
- Slack tokens
- Google API keys
- long provider `sk-*` tokens
- JWT-like bearer tokens when they look live

A hard hit aborts the handoff until it is removed or deliberately reviewed by the operator.

Generic `password =`, `secret =` and test fixture strings are **soft findings**: inspect them, but do not train the workflow to ignore its scanner by making every fixture fatal.

Never add a permanent `allow secrets` default. An explicit one-off operator decision must name the reviewed hit and applies only to that handoff.

## 4. author the external prompt

The prompt is the model's interface to the snapshot. Make it self-contained.

Use this structure conceptually:

```text
ROLE
  concrete senior lens for this problem

TASK
  the decision/output required

CONTEXT
  architecture, file anchors, empirical signals, hypotheses and snapshot metadata

INVARIANTS
  boundaries that recommendations must preserve

UNCERTAINTY
  distinguish verified-in-snapshot evidence from hypotheses; never invent file/line/API facts

OUTPUT
  prioritized findings, impact/effort/risk and explicit DO-NOT recommendations where relevant
```

Rules:

- ground claims in the uploaded code/docs
- ask the reviewer to **confirm, refute or extend** seeded hypotheses rather than agree with them
- for exploration, explicitly invite it to challenge the framing and surface directions not present in the seed list
- state invariants as hard constraints even when exploration is open-ended
- ask it to flag code/doc drift
- ask it to distinguish evidence found in the snapshot from assumptions
- do not ask for hidden chain-of-thought; ask for concise rationale, evidence anchors and uncertainty
- do not put runtime-specific reasoning-tier instructions in the prompt when the operator controls that separately

For audit-style reports, prefer a prioritized table or equivalent containing:

```text
finding | evidence | status | impact | effort | risk | recommendation
```

Include `DO-NOT` recommendations for tempting changes that cross an invariant.

## 5. snapshot freshness

Every handoff must know which code it reviewed.

Record:

- base commit SHA
- export timestamp
- dirty/clean state
- important uncommitted files intentionally included, if any

If material code changed before the review is sent, rebuild the affected snapshot. If changes land while the model is reviewing, do not throw the report away; treat drift as something the live verification gate must resolve.

A precise stale report is worse than an explicitly dated one.

## 6. receive the report

Save the returned report beside the handoff metadata outside the repository by default.

Do not immediately turn findings into patches.

For each material finding, create a verification item containing:

```text
claim
snapshot evidence cited
live file/symbol to inspect
reproduction/test needed
invariant affected
verification owner
final disposition
```

Large reports may be split into independent verification lanes using `agent-orchestration`. The supervisor still owns the conclusion.

## 7. live verification gate

For every finding:

1. **re-anchor** — locate the current live file/symbol; cited lines may have moved or disappeared
2. **re-check code truth** — confirm the mechanism still exists in the live tree
3. **reproduce when applicable** — trace the path, run the relevant test or write a failing test when the claim is executable
4. **check invariants** — reject fixes that weaken a protected boundary
5. **check drift** — determine whether the snapshot was stale, the issue was already fixed or the architecture changed
6. **classify** — `implement`, `needs-decision`, `discard`, or `unresolved`
7. **record why** — one concise reason so rejected claims are not re-litigated without new evidence

`VERIFIED` in an external report means at most `found in the snapshot` unless runtime behavior was actually reproduced later. Never promote snapshot confidence into runtime proof automatically.

## 8. implementation

Only implement findings that survive live verification.

Then return to normal repository discipline:

- follow `AGENTS.md` and the authoritative subsystem document
- use `agent-orchestration` when several verified fixes are independent
- run targeted tests with each change
- run the required repository-level gate before claiming done
- keep rejected/unresolved external findings out of the implementation merely because the reviewer sounded confident

## anti-patterns

- vague `review the whole repo` handoffs
- uploading the current directory without a secret gate
- splitting bundles by arbitrary size instead of architecture
- omitting tests while keeping generated output
- treating many repeated secondary claims as independent evidence
- asking an exploratory model only to validate the operator's seed list
- implementing directly from a static report
- treating confident file/line references as current without re-anchoring
- letting an external model relitigate hard invariants that were explicitly out of scope

## Vesper boundary

This skill adds a review workflow, not another AI control plane.

It does not require a particular model, vendor or chat product. It complements the canonical shared skill tree and Vesper's backend-neutral Agent Teams boundary.

Credentials remain governed by `AGENTS.md`, `docs/SECRETS.md` and the active runtime/provider. Review bundles must never become a side channel for secrets.
