---
name: agent-orchestration
description: Coordinate complex work across Codex, Claude Code, OpenCode or another already-selected runtime with a model-agnostic supervisor/worker/reviewer hierarchy, explicit model policy, difficulty-first routing, bounded parallelism, write isolation, evidence gates and final fan-in. Use when a task has multiple separable workstreams or benefits from a strong lead delegating bounded work to cheaper or specialized models.
---

# Agent Orchestration

Act as the supervisor unless the parent explicitly assigns a worker or reviewer role.

The supervisor remains accountable for scope, architecture, routing, integration and final verification. Delegates buy throughput and specialized attention; they do not replace the final decision maker.

## core laws

1. **Route on difficulty and consequence, not vendor name.** Model families are interchangeable inputs to the policy, not the architecture.
2. **Decide the route before spawn.** Do not let each lane improvise its own provider/model at launch time.
3. **One accountable supervisor.** Workers may execute and reviewers may challenge, but synthesis and merge decisions return to one owner.
4. **Completion is a claim, not evidence.** A worker saying `done` is never the acceptance gate.
5. **Parallel writes require ownership or isolation.** If neither is available, parallelize reads and serialize writes.
6. **No silent provider or cost fallback.** A runtime must not quietly move a lane to another provider, paid route or weaker model outside the selected policy.
7. **A model is not a runtime.** Do not install Qwen Code, a DeepSeek CLI or another model-vendor runtime just because that model is used through Codex, Claude Code or OpenCode.

## when to orchestrate

Use this skill when at least one is true:

- the task has two or more independent workstreams
- repository-wide inspection can be split by subsystem
- implementation, tests and review can proceed independently
- several hypotheses should be investigated in parallel
- a strong lead can delegate mechanical or specialized work to cheaper/faster models
- a large external report contains several independently verifiable findings

Do not orchestrate a tiny edit, a single obvious command or a tightly sequential change where coordination costs more than it saves.

## roles

Use three roles:

- **supervisor** — owns the whole task, architecture, task graph, route policy, integration and final gate
- **worker** — owns one bounded deliverable and returns evidence, a patch or a concrete result
- **reviewer** — independently challenges integrated work; read-only by default and never self-approves its own implementation

Default to depth two: one supervisor plus workers/reviewers. Avoid recursive swarms. A worker may delegate a subtree only when the runtime explicitly supports it, the supervisor asked for it and the extra level has a clear payoff.

## model policy

Model selection is policy-driven rather than hard-coded.

Use one of three modes:

### `auto`

The supervisor selects among models already exposed by the active runtime/provider.

- choose the strongest reliable model within budget for supervision, ambiguous architecture, risky integration and final synthesis
- choose cheaper/faster models for bounded recon, test generation, documentation and mechanical execution when their output is independently verifiable
- choose a domain-specialized model when it materially improves the lane
- use an independent capable model for review when practical

Do not ask the operator about every routine lane. Ask only when a choice would materially change cost, privilege, data exposure or a hard user constraint and the runtime cannot resolve it safely.

### `preferred`

Use operator-defined ordered preferences per role. Example shape:

```text
mode: preferred
supervisor: [best-general]
worker: [cheap-coder, cheap-general, best-general]
reviewer: [independent-reviewer, best-general]
fallback: auto
```

Use the first available route that satisfies the task. `fallback: auto` may leave the preference list only when needed. `fallback: deny` must stop substitution and surface the unavailable route.

### `fixed`

Use exact operator-selected assignments.

```text
mode: fixed
supervisor: provider/model-a
worker: provider/model-b
reviewer: provider/model-c
```

Never silently substitute a fixed model. If it is unavailable, report the blocker or let the supervisor perform the task directly when that still respects the operator's instruction.

An explicit model named by the operator for a scoped task overrides the current policy for that task only, then the previous policy resumes.

## route integrity

When the runtime supports provider-qualified model identifiers, prefer them over ambiguous names.

Before dispatch, record the intended route for every lane:

```text
task -> role -> runtime -> provider/model -> effort/tier -> permissions
```

If the runtime exposes actual provider/model metadata after completion, compare it with the intended route. Treat an unexpected provider, metered fallback or model substitution as a routing failure, not a successful lane.

Keep the number of active model tiers small. More tiers are useful only when they correspond to real capability or cost boundaries; otherwise they make routing hard to audit.

## difficulty-first routing

Use task shape and failure cost together:

- **cheap/mechanical** — inventory, file/symbol discovery, repetitive edits against an established pattern, basic test scaffolding, formatting, compact summaries
- **substantial/precise** — implementation against a complete spec, migrations, multi-file refactors, correctness-sensitive tests, large but bounded changes
- **judgment/risky** — architecture, ambiguous debugging, security/privacy boundaries, secrets, destructive operations, data migrations, contract changes, cross-lane integration and final review

A task can be execution-heavy and still deserve the strongest worker when precision or consequence is high. Do not reduce routing to `research = cheap` and `coding = expensive`.

The quality of a worker lane is bounded by the quality of its brief. Spend supervisor attention on the contract before spending premium-model tokens doing the worker's job.

## task graph before dispatch

Before spawning workers:

1. read repository instructions and the authoritative subsystem documentation
2. state acceptance criteria and important non-goals
3. split the work into tasks with explicit dependencies
4. define file/subsystem ownership for every writing task
5. assign each task a role and model route under the active policy
6. assign tools, permission scope and a timebox when the runtime supports one
7. choose the smallest useful concurrency bound from actual machine/provider headroom
8. identify the final integration gate and independent review step

Prefer independent leaf tasks. Keep cross-cutting architecture decisions with the supervisor unless a worker is explicitly producing analysis only.

Do not defer model selection until spawn time unless a runtime makes the choice only then and the policy still constrains the allowed set.

## worker contract

Every delegated task should contain:

```text
task: short stable identifier
goal: one concrete outcome
context: only the facts needed for this task
dependencies: prerequisites that must already hold
owns: exact files/subsystem/evidence sources this lane may change
must-not-touch: conflicting/shared surfaces and which sibling or supervisor owns them
route: role plus selected provider/model policy result when available
permissions: read/write/network/secret/root scope actually required
timebox: bounded runtime when supported
acceptance: commands or evidence that must pass
return: findings, changed files/diff summary, acceptance evidence and open risks
```

A delegate cannot be assumed to see the parent conversation. Make the brief self-contained.

Workers must not broaden scope because they noticed unrelated cleanup. Report adjacent problems separately.

Do not ask workers for hidden reasoning transcripts. Ask for decisions, evidence, diffs, test output and unresolved risks.

## baseline before parallel writes

Before a write fleet starts, establish the base state when practical:

- pin the base commit or working-tree state
- run the narrow acceptance gate that the lanes will depend on
- record pre-existing failures instead of making a worker own them accidentally
- prove worktree or sandbox mechanics on a disposable/baseline lane before relying on them for risky writes

A red baseline does not forbid work, but it must be distinguished from regressions introduced by the fleet.

## isolation and ownership

Prefer a separate Git worktree or equivalent isolated workspace for concurrent writing agents when the runtime supports it.

Use worktree-per-lane when:

- lanes may touch the same file or nearby integration seams
- a lane can make a coherent commit/patch that the supervisor can inspect before integration
- live collisions would be expensive to untangle

A shared checkout is acceptable only for clearly disjoint files with explicit `owns` and `must-not-touch` lists.

Shared files such as export barrels, registries, central schemas, common config and root entry points should normally be owned by the supervisor/integration stage. If a worker must touch one, allocate it to exactly one writing lane.

Workers in a shared checkout should run targeted checks for their owned region. The supervisor runs the full repository gate after fan-in so sibling mid-flight changes do not create misleading lane failures.

Never let two parallel workers edit the same file without explicit serialization or isolated integration.

## resource-aware concurrency

Do not copy a fixed fleet size from another machine or harness.

Choose concurrency from:

- available RAM and CPU
- provider/rate-limit headroom
- per-lane startup cost
- MCP/tool servers duplicated per process
- repository build/test pressure
- write-conflict probability

Start around 2–4 active workers for mixed coding work unless measurement justifies more. Increase mainly for independent read-only work.

When a runtime can fan out subagents inside one process, prefer that over many processes when it provides equivalent isolation and avoids duplicating large MCP/tool stacks. Conversely, use separate processes/worktrees when failure isolation or conflicting writes matter more than startup overhead.

For external process fleets:

- stagger initial launches slightly when simultaneous cold starts can create a thundering herd
- use a timebox when supported
- treat a lane whose observable output/liveness has stalled as suspect rather than trusting the process table alone
- never kill unrelated processes merely because they look like agents

## fan-out and fan-in

Use this loop:

1. dispatch independent leaf tasks in parallel
2. keep dependent tasks blocked until prerequisites return
3. collect worker results and reject outputs missing required evidence
4. inspect diffs/findings before integration; never accept a worker result blindly
5. independently run the lane acceptance check or reproduce its critical evidence
6. integrate compatible changes one at a time
7. resolve cross-worker assumptions centrally
8. run an independent reviewer over the integrated state when the risk justifies it
9. fix material review findings
10. run the final repository-level verification from the integrated state
11. report the final result, including discarded worker results and why they were discarded when material

The supervisor owns the final answer. Do not concatenate worker messages and call that synthesis.

## failure and retry policy

- do not respawn the same failed task indefinitely
- retry once with a narrower brief, repaired environment or different allowed worker/model when the failure looks recoverable
- a route change on retry must still obey `auto`, `preferred` or `fixed` policy
- after a second failure, the supervisor should take over, choose a different decomposition or report the blocker
- cancel/ignore stale workers when another result makes their task obsolete
- do not spawn a worker solely to confirm an answer already verified by deterministic tests

## runtime adapters

Use native delegation, model-routing and isolation primitives from the already selected runtime.

### Codex

- keep the supervisor thread responsible for decomposition and integration
- use native parallel-agent/worktree primitives when exposed
- place independent writing agents in isolated worktrees
- use separate review work rather than letting the implementing agent approve itself
- when per-agent model selection is unavailable, do not invent it
- do not launch arbitrary nested CLI processes just to simulate subagents if that bypasses Codex permissions, worktree isolation or credential boundaries

### Claude Code / OpenCode / other selected runtime

Map equivalent primitives onto the same contract:

```text
spawn/delegate -> bounded worker
worktree/sandbox -> write isolation
model selector -> policy-constrained route
join/result collection -> fan-in
review agent -> independent verification
provider metadata -> route-integrity check
```

If a runtime lacks write isolation, parallel workers should remain read-only. If it lacks model routing, keep role/context separation and use only models it actually exposes.

Do not add another runtime solely to reach a model that the current runtime/provider can already expose.

## Vesper boundary

Follow `AGENTS.md` and the authoritative subsystem document for every repository change.

This skill is procedural memory. It does not make CCCC, Codex, Claude Code, OpenCode or any other orchestration backend a mandatory Vesper dependency, and it does not change the backend-neutral Agent Teams product boundary in `docs/AI.md`.

For Vesper changes, the supervisor must keep one source of truth, avoid duplicated control planes, respect capability/secret boundaries and run the repository verification required by the files actually changed.
