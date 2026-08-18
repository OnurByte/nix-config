# Reliability and durable execution

Use this reference when an agent is scheduled, long-running, resumable, unhealthy, hanging or making claims about completion.

## separate four persistence layers

Do not call all persistence `memory`.

```text
runtime state     -> sessions, logs, job manifests, retries, temporary artifacts
semantic memory   -> associative recall of durable facts when a memory backend exists
durable context   -> human-readable decisions, projects, rationale and history
procedural skill  -> rules automatically loaded for a class of work
```

Runtime state keeps work alive. Semantic memory retrieves facts. Durable context preserves why a decision exists. Skills change future behavior.

Raw tool output belongs in runtime state. Decisions and reasons belong in durable context. Repeatable procedures belong in reviewed skills.

Continuity must be a lifecycle, not a hope:

```text
session start -> load compact last-state + active threads
session work  -> write durable progress/artifacts
session end   -> preserve unresolved work and important decisions
scheduled bridge -> recover meaningful sessions that ended without a clean handoff
```

Use overlap in scheduled continuity windows so scheduling jitter creates duplicates rather than gaps. Deduplicate by stable session identity. Preserve unresolved items that were not mentioned in the newest window instead of silently erasing them.

Do not keep one enormous conversation alive as a substitute for continuity. Repeated compression, degraded latency or context pressure is a reason to checkpoint and start a fresh session from durable context.

## durable-job contract

A controller timeout means only that the controller stopped waiting.

For work that may outlive the interactive turn or agent timebox, persist a manifest before execution:

```json
{
  "jobId": "stable-id",
  "createdAt": "ISO-8601",
  "total": 0,
  "units": {
    "unit-id": {
      "status": "pending|running|succeeded|failed|blocked",
      "attempts": 0,
      "output": "path-or-remote-id",
      "sha256": "",
      "durationSeconds": 0,
      "error": ""
    }
  }
}
```

Rules:

- write state atomically: temporary file then rename
- validate an output before calling it reusable; `exists()` alone is insufficient for partial files
- skip already-valid units during resume
- retry only missing/invalid units
- record attempts and errors durably
- use content hashes when duplicates would be harmful
- keep large replaceable artifacts outside Git; version compact manifests, reports and decision evidence

For side-effecting remote work, file existence is not an idempotency boundary:

```text
write intent + idempotency key
-> perform remote action
-> write result

resume sees intent without result
-> query remote state
-> only retry if remote state proves the action did not happen
```

Never blindly replay an externally visible action after an ambiguous timeout.

## evidence discipline

Completion is a postcondition.

Before reporting `done`, prove the relevant combination of:

- expected artifact exists
- expected count equals actual count
- format/size/schema is valid
- hashes or identities are unique when required
- repository/version state records the change
- remote state was re-read after an API mutation
- failure did not leave an unexpected partial mutation

Examples:

```text
command said update complete -> re-read installed version
API returned 200             -> fetch object and compare state
worker said generated 30     -> count 30 valid artifacts and 30 expected identities
memory is configured         -> write, close session, retrieve in a new session
```

A green component status is not end-to-end evidence. Prefer one narrow smoke probe that exercises the real path and has one unambiguous expected result.

Measured claims use numbers. Unknown metrics remain unknown.

## silent death and dead-man checks

Always-on agents fail quietly because the thing that should report the outage may be the thing that died.

Use two distinct layers when practical:

1. **internal health** — service/unit/config/disk checks for diagnosis
2. **external liveness** — a dead-man service or an independent probe that notices missing success from outside the failing agent path

A dead-man signal is useful because scheduler, network or whole-machine failure all stop the ping. The external alert path must not depend on the agent being healthy.

Vesper supports a scheduler dead-man ping through the optional file configured by `VESPER_DEADMAN_URL_FILE`. Keep the real URL/identifier out of Git and command-line arguments.

An active external probe is stronger when available:

```text
independent machine/service
-> send a real minimal request through the agent/model path
-> require exact expected response within a bounded time
-> alarm after consecutive failures
```

Do not page on one transient failure. Do not use the same broken channel as the only alarm route.

Also monitor absence:

- expected scheduled artifact did not arrive
- run record is stale
- context mirror stopped updating
- memory writes are unexpectedly zero
- provider route changed silently
- credential expiration is approaching

## physical-chain diagnosis

When behavior looks impossible, enumerate the physical path before theorizing about the model:

```text
trigger -> scheduler -> process -> credential -> DNS -> route -> socket -> provider -> artifact/state
```

Prove each link.

If a process hangs without returning an application error, inspect the syscall/network path before blaming model quota or reasoning.

A quick service restart can hide crashes. Check restart counters and logs before debugging downstream behavior.

A check that detects failure but lets the caller continue as if success occurred is not a gate.

## network diagnosis

Do not treat all timeouts as the same fault.

Useful distinctions:

- reset/refused -> a peer actively rejected or no service is listening
- timeout -> packets may be silently dropped or a route/address-family path may be black-holed
- IPv4 works / IPv6 fails -> dual-stack clients may behave differently from `curl`
- host works / container fails -> inspect forwarding, bridge firewall, DHCP/DNS and namespace-specific routes

Test both address families explicitly when both are configured. Test with the same HTTP/network stack used by the agent whenever possible; a fallback-friendly diagnostic client can hide the path the real runtime blocks on.

Any manual network repair must become declarative or boot-safe if it is meant to survive a restart.

## failure catalog habit

For every expensive incident, keep a compact entry:

```text
symptom
root cause
proof
fix
regression test / monitor
```

A second occurrence should normally create an automated detection or repair. Automatic repair must increment a visible counter or log a receipt; a system healing itself dozens of times per day is still unhealthy.
