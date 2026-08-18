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

## active remediation ledger

This is the repository-wide ledger for confirmed cross-subsystem reliability issues found during the 2026-08-18 audit. It is not a second architecture document. Each item names the canonical contract that owns the final behavior.

An item stays open until the implementation is changed, the relevant tests/builds pass and the owning document describes the resulting current behavior.

| priority | issue | required fix | canonical owner |
|---|---|---|---|
| high | `vesper-doctor` treats `/etc/vesper/restic.env` as missing when the normal user cannot read the intentionally `0600 root:root` file | configuration health must test existence/ownership or a privileged service-visible condition without requiring `yargc` to read the secret; Hermes must not inherit this false unhealthy state | `BACKUP.md` |
| high | wellbeing keeps charging the last Hyprland foreground app while the session is idle or locked | gate accounting on authoritative idle/lock state and do not add samples while the user is inactive; keep foreground sampling explicitly approximate | `APPS-SETTINGS.md` |
| high | Privacy HUD treats an unmuted default input as active microphone use | report microphone attention only from real capture/recording activity; muted/unmuted device state is not usage state | `../AGENTS.md` desktop/privacy contract |
| medium-high | Privacy HUD can treat Tor Browser's bundled `tor` process as system Tor | system-Tor status must come from the Vesper system Tor service or another ownership-specific signal, never generic `pgrep tor` | `../AGENTS.md` privacy contract |
| medium-high | proxy configuration writes a Vesper marker before the effective `environment.d` file, allowing status to say configured after a partial failure | write/validate the effective environment atomically, commit the status marker last or derive status from the effective file, and keep secret-bearing files user-private | `NETWORK-SETTINGS.md` |
| medium | proxy copy implies newly started desktop processes immediately inherit the new environment | document the real session boundary: `environment.d` is guaranteed for the next user-session environment; a session restart is the clean global handoff | `NETWORK-SETTINGS.md` |
| medium-high | adaptive-icon production behavior is assembled from a long ordered patch stack while the package disables its own checks | fold accepted patches into tracked Rust sources, delete obsolete patch assembly, run Cargo tests from the package build and keep CI testing the same sources shipped by Nix | `ADAPTIVE-ICONS.md` |
| medium | the Wi-Fi QR stdin/security correction exists as a packaging patch instead of the tracked `vesper-control` source | fold the correction into the canonical Rust source and remove the source/runtime drift | `NETWORK-SETTINGS.md` |
| medium | `ADAPTIVE-ICONS.md` still describes the icon engine as a direct-`rustc` prototype even though it already has `Cargo.toml`, `Cargo.lock` and `rustPlatform.buildRustPackage` packaging | replace stale implementation prose with the actual Cargo package boundary while keeping future architecture clearly marked as target work | `ADAPTIVE-ICONS.md` |
| medium | Agent Cockpit, Privacy HUD and wellbeing independently launch subprocess-heavy probes on short polling intervals | use event-driven state where practical and otherwise cache/throttle expensive backend probes; UI refresh cadence must not imply rerunning every Git/process/system probe | `AI.md`, `APPS-SETTINGS.md`, `../AGENTS.md` |
| medium | airplane mode models only Wi-Fi/Bluetooth, ignores WWAN in status and does not preserve pre-airplane radio state | snapshot Wi-Fi/WWAN/Bluetooth state before enabling airplane mode, disable all intended radios, then restore only the previous state; surface command failures | `NETWORK-SETTINGS.md` |
| medium | remote repository name is `vesper` while `nh`, aliases and command-memory still assume `~/nix-config` | keep one explicit local checkout contract; until code is migrated, installation must clone `OnurByte/vesper` into `~/nix-config` so existing commands remain valid | `INSTALL.md` |

### remediation order

Use this order unless a dependency forces a smaller prerequisite change:

1. Restic doctor false-negative
2. wellbeing idle/lock accounting
3. Privacy HUD microphone and Tor truth
4. proxy atomic state and session semantics
5. adaptive-icon and Wi-Fi QR source/patch consolidation
6. polling/caching cleanup
7. airplane state preservation
8. checkout-path/documentation cleanup

Do not mark an item closed because prose was updated. The code change and verification must land first.

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
