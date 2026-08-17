# Vesper Maintainer

Maintain the Vesper NixOS workstation with evidence instead of speculative refactors.

## loop

1. observe the current machine, repository and failure
2. diagnose the smallest plausible root cause
3. patch the narrowest declarative layer that owns it
4. run the relevant Nix parse/evaluation/build checks
5. run `vesper-doctor --json` when the live machine is available
6. explain what changed and what evidence proved it
7. apply only after the change is testable and reversible

## local context

- `vesper-control wellbeing-summary` is the canonical read-only machine-readable Wellbeing context for agents
- the JSON reports whether collection is enabled, the local date, total foreground time and per-app foreground time
- Wellbeing is enabled by default but the user can disable collection in Settings → Apps; when disabled, do not attempt to re-enable it automatically
- treat Wellbeing as local context only: do not upload, sync or include raw usage history in external prompts unless the user explicitly asks
- agents may use the summary to reason about workflows and app usage, but user-facing Settings owns the collection toggle

## rules

- prefer NixOS and Home Manager ownership over installer scripts
- preserve `flake.lock` unless an input update is intentional
- keep PychoVIM's own updater and mutable config ownership
- keep Zed on the locked stable nixpkgs package unless an intentional pin update is requested
- keep Caelestia as the only desktop shell/bar
- preserve the Apple/visionOS-inspired glass language for shell surfaces: layered translucency, readable blur, restrained rounding, soft shadow and thin luminous borders
- do not turn every application transparent
- keep `bb` as the multi-agent control plane instead of adding another orchestrator
- use `~/.agents/skills` as the canonical active skill tree
- Hermes may write proposed improvements to `~/.local/share/vesper/skill-drafts/`; drafts are not active skills until reviewed and promoted

## verification

For repository changes follow `AGENTS.md` and run the subset required by the touched files. Any change that affects the complete workstation closure should end with a complete Vesper system build.
