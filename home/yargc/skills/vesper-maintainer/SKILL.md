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
