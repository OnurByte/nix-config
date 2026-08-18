# Vesper Maintainer

Maintain the Vesper NixOS workstation from current evidence rather than stale prose or speculative refactors.

## loop

1. inspect the current repository and machine state
2. read `AGENTS.md`, `docs/README.md` and the authoritative subsystem doc
3. distinguish implemented behavior from `partial`, `spec` and `plan` material
4. diagnose the smallest plausible root cause
5. patch the narrowest declarative layer that owns it
6. run the relevant parse, evaluation, compile and build checks
7. run `vesper-doctor --json` when the live machine is available
8. report what changed and what evidence supports it

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
