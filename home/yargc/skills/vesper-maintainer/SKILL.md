---
name: vesper-maintainer
description: Diagnose and maintain the Vesper NixOS workstation from live evidence, preserving declarative ownership and proving postconditions instead of trusting status text.
platforms: [linux]
---

# Vesper Maintainer

Maintain the Vesper NixOS workstation from current evidence rather than stale prose or speculative refactors.

## loop

1. inspect the current repository and machine state
2. read `AGENTS.md`, `docs/README.md` and the authoritative subsystem doc
3. distinguish implemented behavior from `partial`, `spec` and `plan` material
4. map the physical chain involved in the symptom before inventing a behavioral theory
5. diagnose the smallest plausible root cause
6. patch the narrowest declarative layer that owns it
7. run the relevant parse, evaluation, compile and build checks
8. run `vesper-doctor --json` when the live machine is available
9. re-read the resulting state/artifact and prove the intended postcondition
10. report what changed and what evidence supports it

## evidence and failure rules

- a command printing `success`, a green service status or an API 200 is an action result, not proof of the intended outcome
- after a mutation, re-read the version, remote object, generated artifact or effective configuration that should have changed
- on failure, check whether a partial mutation occurred before retrying
- if a service appears to ignore commands, inspect restart/crash counters and logs before debugging the higher-level behavior
- a check that notices a problem but allows the caller to continue as if success occurred is not a gate
- when a process hangs without an application error, inspect socket/network/system-call state before blaming model quota or reasoning
- distinguish timeout from reset/refusal; test IPv4 and IPv6 separately when both are configured
- when possible, test with the same HTTP/network stack used by the failing application because fallback-friendly tools such as `curl` can hide a path-specific failure
- any manual repair intended to persist must become declarative or boot-safe

Use `agent-operations` and its reliability reference for long-running jobs, state/resume, dead-man monitoring, approval or public-agent boundaries.

## rules

- prefer NixOS and Home Manager ownership over installer scripts
- preserve `flake.lock` unless an input update is intentional
- keep PychoVIM's own updater and mutable config ownership
- keep Zed on the locked stable nixpkgs package unless an intentional pin update is requested
- keep Caelestia as the only desktop shell/bar
- follow the current component-specific visual authority; for top bar and dock this is `docs/TOP-BAR-DOCK.md`
- do not generalize one glass recipe, static luminous border or visionOS imitation across every shell surface
- do not turn every application transparent
- keep Vesper's AI control-plane boundary backend-neutral; optional orchestration backends must remain replaceable
- do not restore removed control planes or tools because stale documentation mentions them
- use `~/.agents/skills` as the canonical active skill tree
- Hermes may write proposed improvements to `~/.local/share/vesper/skill-drafts/`; drafts are not active skills until reviewed and promoted
- do not duplicate architecture docs when `docs/README.md` names a canonical owner

## verification

Follow the repository-wide checklist in `AGENTS.md` and run the subset required by the touched files.
Any change that affects the complete workstation closure should end with a complete Vesper system build.
