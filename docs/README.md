# Vesper docs

This file is the documentation index and authority map.

Agents must read this before making architecture changes.

## status vocabulary

Use these labels consistently near the top of architecture documents:

- `current` — documents implemented behavior and current operational truth
- `partial` — some described behavior exists, some remains target work
- `spec` — target architecture or feature contract, not proof of implementation
- `plan` — implementation/design plan that must not be treated as active behavior unless explicitly activated

When a `partial`, `spec` or `plan` document conflicts with current code, current code wins unless the task explicitly changes the implementation.

## authority rules

Repository-wide guardrails live in `../AGENTS.md`.

Subsystem architecture must have one canonical document. Do not create parallel Markdown files for the same contract when one of the documents below already owns it.

| document | status | authority |
|---|---|---|
| `INSTALL.md` | current | verified storage/install topology |
| `BACKUP.md` | current | Restic backup and restore contract |
| `SECRETS.md` | current | secret ownership and sops-nix usage |
| `MCP.md` | current | shared MCP registry and runtime behavior |
| `SKILLS.md` | current | canonical active skill tree and skill lifecycle |
| `HERMES.md` | current | Hermes scheduling and recurring research contract |
| `HERMES-ADHOC-RESEARCH.md` | current | ad-hoc Hermes research workflow |
| `HERMES-RESEARCH-LINKS.md` | current | Hermes source/link discovery contract |
| `SETTINGS.md` | spec | Settings information architecture, cross-page UX and runtime-to-declarative control model |
| `NETWORK-SETTINGS.md` | current | Vesper network settings behavior |
| `ADAPTIVE-ICONS.md` | partial | single source of truth for adaptive icon architecture and implementation state |
| `AI.md` | partial | AI control-plane product boundary, credentials, agents and orchestration |
| `AI-ANALYTICS.md` | spec | analytics normalization and measurement semantics |
| `APPS-SETTINGS.md` | partial | installed-app settings contract and Store handoff |
| `MARKETPLACE.md` | spec | Vesper Store architecture and target transaction model |
| `TOP-BAR-DOCK.md` | plan | Apple-aligned top-bar and Liquid Glass dock design plan |

## canonical boundaries

### Settings

`SETTINGS.md` is the only repository-wide Settings architecture document.

It owns:

- Settings navigation/information architecture
- cross-page UX rules
- System/NixOS, Display, Power, System Health, Input, Shortcuts, Services and global Settings search target surfaces
- the runtime -> declarative persistence workflow
- placement of Recovery, Network/Privacy, AI, Automation, Apps, Wellbeing and Appearance capabilities inside Settings

It does **not** replace the underlying subsystem contracts.

For example:

- backup/restore behavior stays in `BACKUP.md`
- Hermes scheduling stays in `HERMES.md`
- AI capability semantics stay in `AI.md`
- analytics semantics stay in `AI-ANALYTICS.md`
- installed-app behavior stays in `APPS-SETTINGS.md`
- network runtime behavior stays in `NETWORK-SETTINGS.md`
- adaptive icon rendering stays in `ADAPTIVE-ICONS.md`

Do not create separate `DISPLAY.md`, `POWER.md`, `RECOVERY-CENTER.md`, `SHORTCUTS.md`, `PRIVACY-SETTINGS.md`, `AUTOMATION-SETTINGS.md` or similar Settings feature docs unless their subsystem becomes independently complex enough to need a non-UI operational contract.

### adaptive icons

`ADAPTIVE-ICONS.md` is the only adaptive-icon architecture document.

Do not split AI conversion, Apple compatibility, auto-fit, layered rendering, fidelity, appearance or export rules into additional icon-specific Markdown files.

### Vesper Store

`MARKETPLACE.md` is the only Vesper Store architecture document.

Do not split catalogue, Nix transaction, Flathub, Qt/QML UI or application-source rules into additional Store-specific Markdown files.

The document is a target specification. Current code must be inspected before claiming a Store capability is implemented.

### AI

`AI.md` owns the AI control-plane product boundary.

Do not make any orchestration backend a core product dependency. CCCC is optional and replaceable behind Vesper's backend-neutral Agent Teams/orchestration boundary.

`AI-ANALYTICS.md` owns analytics source normalization and measurement semantics. Do not create parallel quota, token, cost, active-time or vibe-coding definitions elsewhere.

`SETTINGS.md` may define where AI controls appear and how they interact with the wider Settings control plane, but it must not redefine provider, permission or analytics truth.

### apps

`APPS-SETTINGS.md` owns installed-application settings behavior.
`MARKETPLACE.md` owns discovery and installation.

Do not build a second installed-app management surface inside Vesper Store.

### visual shell

`TOP-BAR-DOCK.md` is the visual authority for the planned top-bar and dock redesign.

Its status is `plan`. It corrects older generic glass assumptions, but it does not authorize implementation by itself.

For components outside that plan, follow current Caelestia/Vesper code and the repo-wide UX guardrails in `AGENTS.md`.

## agent documentation rules

Before changing a subsystem:

1. inspect current code
2. read `../AGENTS.md`
3. read this index
4. read the authoritative subsystem document
5. distinguish current behavior from target behavior
6. make the smallest coherent change
7. update the document status or current-state section when implementation materially changes

Do not infer implementation from examples, diagrams, desired UI copy or future-tense requirements.
Do not resurrect removed features from stale prose.
Do not duplicate facts across many docs when a link to the canonical contract is enough.
