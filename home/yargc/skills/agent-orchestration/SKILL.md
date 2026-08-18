---
name: agent-orchestration
description: Coordinate complex work with a supervisor/worker hierarchy, bounded parallel subagents, model routing, worktree isolation, independent review and final fan-in. Use when a task has multiple separable workstreams or benefits from a strong lead delegating to cheaper or specialized workers.
---

# Agent Orchestration

Act as the supervisor unless the parent explicitly assigns a worker role.

The supervisor remains accountable for scope, architecture, integration and final verification. Subagents accelerate bounded work; they do not replace the final decision maker.

## when to orchestrate

Use this skill when at least one is true:

- the task has two or more independent workstreams
- repository-wide inspection can be split by subsystem
- implementation, tests and review can proceed independently
- several hypotheses should be investigated in parallel
- a strong lead can delegate mechanical or specialized work to cheaper/faster models

Do not orchestrate a tiny edit, a single obvious command or a tightly sequential change where delegation costs more than it saves.

## hierarchy

Use three roles:

- **supervisor** — understands the whole task, decomposes it, routes models, owns dependencies, integrates results and performs final verification
- **worker** — receives one bounded task with explicit scope and returns evidence, a patch or a concrete result
- **reviewer** — independently checks integrated work; read-only by default and never self-approves its own implementation

Default to a depth of two: one supervisor plus workers/reviewers. Do not create recursive swarms. A worker may delegate a subtree only when the runtime explicitly supports it, the supervisor asked for it and the extra level has a clear payoff.

## model routing

Route by task difficulty rather than vendor name.

- use the strongest reliable model available within budget for supervision, ambiguous architecture, risky integration and final synthesis
- use faster or cheaper models for bounded implementation, search, test generation, documentation and mechanical refactors when their output can be verified
- use a domain-specialized model when the task clearly benefits from it
- use an independent capable model for review when practical; for high-risk work prefer a different model/provider or at least a fresh isolated context
- do not downgrade work that controls secrets, destructive operations, security boundaries, data migrations or architecture with unresolved ambiguity
- do not spend the most expensive model on deterministic grunt work when a cheaper worker plus verification is enough

Codex, Qwen-family models, GLM, DeepSeek, Claude, Gemini and other compatible models may be supervisors or workers. Never hard-code one vendor as the hierarchy itself.

If the runtime exposes model grades or per-agent model selectors, use them. If it does not, preserve the hierarchy through role separation, scoped contexts and independent review instead of pretending model routing occurred.

## task graph before dispatch

Before spawning workers:

1. read repository instructions and the authoritative subsystem documentation
2. state the acceptance criteria and important non-goals
3. split the work into tasks with explicit dependencies
4. define file or subsystem ownership for each writing task
5. choose worker model tier, tools and permission scope
6. choose the smallest useful concurrency bound

Prefer independent leaf tasks. Keep architecture decisions and cross-cutting changes with the supervisor unless a worker is explicitly asked to produce analysis only.

## worker contract

Every delegated task should contain this information:

```text
task: short stable identifier
goal: one concrete outcome
context: only the facts needed for this task
scope: files, subsystem or evidence sources the worker may touch
constraints: project rules and explicit non-goals
verification: commands or evidence required before returning
return: findings, changed files/diff summary, verification evidence and open risks
```

Workers must not broaden scope because they noticed unrelated cleanup. They should report adjacent problems separately.

Do not ask workers for hidden reasoning transcripts. Ask for decisions, evidence, diffs, test output and unresolved risks.

## isolation and concurrency

- use a separate Git worktree or equivalent isolated workspace for parallel writing agents when the runtime supports it
- never let two parallel workers edit the same files unless the supervisor has explicitly serialized the conflicting stage
- when safe write isolation is unavailable, parallelize read-only investigation and let the supervisor perform the shared writes
- keep the main worktree owned by the supervisor/integrator
- default to at most four active workers; increase only for clearly independent read-only work with low coordination cost
- do not leak broad credentials or the parent environment to workers merely for convenience

One integrated correct result is better than many conflicting patches.

## fan-out and fan-in

Use this loop:

1. dispatch independent leaf tasks in parallel
2. keep dependent tasks blocked until prerequisites return
3. collect worker results and reject outputs that lack required evidence
4. inspect diffs before integration; never merge a worker result blindly
5. integrate compatible changes one at a time into the supervisor branch/worktree
6. resolve cross-worker assumptions centrally
7. run an independent review over the integrated state
8. fix material review findings
9. run the final repository-level verification from the integrated state
10. report the final result, including any worker result that was intentionally discarded

The supervisor owns the final answer. Do not concatenate worker messages and call that synthesis.

## failure and retry policy

- do not respawn the same failed task indefinitely
- retry once with a narrower prompt or a different worker/model when the failure looks recoverable
- after a second failure, the supervisor should take over, choose a different decomposition or report the blocker
- cancel or ignore stale workers when another result makes their task obsolete
- do not spawn a worker solely to confirm an answer already verified by deterministic tests

## runtime adapter: Qwen Code

Qwen Code has native subagents and should use them instead of emulating a swarm with shell scripts.

- use named subagents for focused specialists with fresh context
- use fork subagents for parallel work that genuinely needs parent conversation context
- use `model` selectors or configured model grades to put high-capability and low-cost models in different roles
- use `isolation: worktree` for independent writing agents
- do not use fork subagents for concurrent edits that require worktree isolation because forks share the parent working directory
- keep recursive delegation bounded; fork children cannot create further forks
- use cross-provider model selectors only when that provider is already intentionally configured for the runtime

The supervisor may itself be a Qwen-family, GLM, DeepSeek or another configured model. The hierarchy is based on capability and task shape, not nationality or vendor.

## runtime adapter: Codex

Use Codex's native multi-agent/parallel task and worktree primitives when the current Codex surface exposes them.

- keep the supervisor thread responsible for decomposition and integration
- place independent writing agents in isolated worktrees
- give each agent a narrow deliverable and verification contract
- use separate review work rather than letting the implementing agent approve itself
- when per-agent model selection is unavailable, do not invent it; use role/context separation and the models the Codex runtime actually exposes
- do not launch arbitrary nested CLI processes just to simulate subagents if that would bypass Codex permissions, worktree isolation or credential boundaries

## other runtimes

Map the same contract onto equivalent primitives:

```text
spawn/delegate -> bounded worker
worktree/sandbox -> write isolation
model selector/grade -> capability routing
join/result collection -> fan-in
review agent -> independent verification
```

If a runtime lacks one of these primitives, degrade safely. In particular, missing write isolation means parallel workers should remain read-only.

## Vesper boundary

Follow `AGENTS.md` and the authoritative subsystem document for every repository change.

This skill is procedural memory. It does not make CCCC, Qwen Code, Codex or any other orchestration backend a mandatory Vesper dependency, and it does not change the backend-neutral Agent Teams product boundary in `docs/AI.md`.

For Vesper changes, the supervisor must keep one source of truth, avoid duplicated control planes, respect capability/secret boundaries and run the repository verification required by the files actually changed.
