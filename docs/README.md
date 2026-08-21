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
| `SETTINGS.md` | partial | Settings information architecture, cross-page UX and runtime-to-declarative control model |
| `NETWORK-SETTINGS.md` | current | Vesper network settings behavior |
| `ADAPTIVE-ICONS.md` | partial | single source of truth for adaptive icon architecture and implementation state |
| `AI.md` | partial | AI control-plane product boundary, credentials, agents and orchestration |
| `AI-ANALYTICS.md` | spec | analytics normalization and measurement semantics |
| `APPS-SETTINGS.md` | partial | installed-app settings contract and Store handoff |
| `MARKETPLACE.md` | spec | Vesper Store architecture and target transaction model |
| `DESKTOP-ERGONOMICS.md` | plan | high-frequency desktop interaction and ergonomics implementation plan |
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

### desktop ergonomics

`DESKTOP-ERGONOMICS.md` owns the planned high-frequency interaction layer built on the existing Hyprland + Caelestia desktop.

Its status is `plan`. It may reference existing implementation primitives but must not be treated as proof that the planned interaction is active.

### visual shell

`TOP-BAR-DOCK.md` is the visual authority for the planned top-bar and dock redesign.

Its status is `plan`. It corrects older generic glass assumptions, but it does not authorize implementation by itself.

For components outside that plan, follow current Caelestia/Vesper code and the repo-wide UX guardrails in `AGENTS.md`.

## active remediation ledger

This is the repository-wide ledger for confirmed cross-subsystem reliability and architecture issues. It is not a second architecture document. Each item points to the canonical contract that owns the final behavior.

Priority vocabulary:

- `P0` — root architecture/state/lifecycle error that can make current behavior false or block reliable implementation
- `P1` — correctness, security, identity, reliability or maintainability defect that should be fixed before expanding the affected subsystem
- `P2` — bounded cleanup or dependency-hygiene issue with lower immediate operational risk

Class vocabulary:

- `state ownership` — two layers can claim or mutate the same state without one authority
- `lifecycle` — process/session/service ownership is split or not bound to the correct target
- `semantic drift` — implemented behavior does not match the declared protocol/model
- `source drift` — reviewed source differs from the source actually built or shipped
- `identity` — application/package/runtime identity can be attributed incorrectly
- `security boundary` — capability or trust scope is wider than intended
- `observability` — UI/doctor status can report inferred or stale state as authoritative
- `documentation drift` — high-authority documentation can steer implementation away from current code
- `dependency hygiene` — dependency/update/reproducibility boundaries are unnecessarily opaque or broad

An item stays open until the implementation is changed, the verification condition passes and the owning document describes the resulting current behavior.

| priority | class | root cause / issue | required fix | verification | canonical owner |
|---|---|---|---|---|---|
| P0 | state ownership | Caelestia runtime settings write `~/.config/caelestia/shell.json`; Vesper now seeds that path instead of keeping a live Home Manager file owner, but activation and restart persistence still need a live proof | keep the runtime-owned Caelestia config writable and use the declarative seed only for first-run defaults | change a native Caelestia setting, restart the shell/session and confirm the value persists without Home Manager/store write failures; a later HM activation must not silently erase runtime-owned state | `SETTINGS.md` + current Caelestia integration |
| P0 | lifecycle | the repository now defines an explicit `hyprland-session.target` bridge and binds Caelestia/Vesper daemons to `graphical-session.target`, but the activated generation has not yet proven the target and service graph | activate the Home Manager generation, start the target from Hyprland and remove all direct long-lived process spawns from compositor startup | login must activate the intended graphical session target; Vicinae, Caelestia, portals and migrated desktop daemons must start, stop and restart with the session | `SETTINGS.md`, `DESKTOP-ERGONOMICS.md`, current Hyprland and Home Manager config |
| P0 | source drift | first-party Vesper Rust behavior is assembled from a long ordered patch stack while the package itself disables Cargo checks, so repository source is not the source actually shipped | fold accepted first-party patches into canonical Rust sources, delete obsolete first-party patch assembly and run package checks against the exact sources Nix ships; keep patches for genuine upstream boundaries only | `nix build` and `cargo test --locked` use the same tracked final sources with no first-party patch reconstruction | `ADAPTIVE-ICONS.md`, `NETWORK-SETTINGS.md` |
| P0 | semantic drift | adaptive-icon `.vicon` packages are generated but the active theme compiler still renders from `canonical.svg`; semantic group/depth/material data is therefore not the rendering authority | make `.vicon` manifest/groups the renderer input and keep `canonical.svg` only as compatibility/fallback input | a multi-group fixture must produce a rendered result whose output changes when group/depth/material metadata changes; renderer tests must fail if `.vicon` data is ignored | `ADAPTIVE-ICONS.md` |
| P1 | semantic drift | remote icon analysis asks for semantic grouping/retain-raster decisions but the current production path reduces output to a single primary group and does not carry the full semantic response into rendering | define one provider-neutral typed decomposition contract and preserve all decisions through canonicalization and rendering | provider fixtures with different valid group counts/retain-raster decisions must result in different validated canonical `.vicon` structures | `ADAPTIVE-ICONS.md` |
| P1 | semantic drift | provider adapters do not enforce one equally strong structured-output contract end to end, so provider choice can change how strictly the semantic decomposition schema is enforced | define one canonical response schema and capability-aware adapter contract; every supported provider must either enforce that contract at the API boundary or pass through the same strict local validator before acceptance | valid and malformed response fixtures for every provider must produce provider-independent acceptance/rejection and the same typed decomposition object | `ADAPTIVE-ICONS.md`, `AI.md` |
| P1 | identity | adaptive-icon desktop discovery does not implement full freedesktop precedence: `Hidden=true` entries do not tombstone lower-precedence copies, and `OnlyShowIn`/`NotShowIn`/`TryExec` semantics are incomplete | implement the Desktop Entry specification in one canonical resolver shared by Apps and adaptive icons | fixtures must cover hidden tombstones, desktop-environment visibility and `TryExec`; lower-precedence entries must stay hidden when shadowed | `ADAPTIVE-ICONS.md`, `APPS-SETTINGS.md` |
| P1 | identity | icon lookup uses a global candidate scoring heuristic instead of the freedesktop icon-theme `index.theme`/inheritance/size-scale lookup algorithm | implement theme-aware lookup with inheritance and `hicolor` fallback before AppStream recovery | fixtures must prove theme inheritance, size/scale choice and `hicolor` fallback match freedesktop semantics | `ADAPTIVE-ICONS.md` |
| P1 | identity | runtime identity now parses `Exec=` quotes, backslash escapes and field-code tokens before deriving executable/class/app-id aliases; full desktop-entry integration proof is still pending | keep parsing aligned with the Desktop Entry specification and never turn field codes into identity aliases | quoted-path, escaped-argument and field-code fixtures must resolve without false aliases | `ADAPTIVE-ICONS.md`, `APPS-SETTINGS.md` |
| P1 | identity | wellbeing now records canonical desktop ids from the adaptive-icon identity inventory and matches summaries by exact canonical/runtime keys; activated identity-file proof is still pending | keep runtime-window attribution on the shared identity graph and avoid a second fuzzy resolver | fixtures with similar names/classes such as editor variants, PWAs and Electron apps must charge time only to the canonical matched app and remain order-independent | `APPS-SETTINGS.md`, `ADAPTIVE-ICONS.md` |
| P1 | identity | Apps can infer Flatpak ownership from a matching desktop/application ID rather than proving which winning desktop entry/package source owns the visible application | separate app identity from install/source ownership and derive ownership from the effective desktop entry plus package/source evidence | native and Flatpak copies with overlapping IDs must be attributed to the winning source; permissions/remove/size controls must follow that owner only | `APPS-SETTINGS.md`, `MARKETPLACE.md` |
| P1 | semantic drift | adaptive-icon jobs now store a semantic contract revision beside the source fingerprint and requeue old revisions, including a running job that completes after the revision changes; activated-generation proof is still pending | bump the contract revision with prompt/schema/renderer changes and keep the source fingerprint as the deduplication key | changing the declared semantic contract revision must deterministically requeue affected work without changing the source file | `ADAPTIVE-ICONS.md` |
| P1 | lifecycle | adaptive-icon worker now refreshes its five-minute lease every 60 seconds and forwards bounded numeric provider `Retry-After` delays; long-running and activated-runtime proof is still pending | keep lease refresh tied to the running worker and preserve bounded fallback backoff | long-running fixture must not be reclaimed while heartbeating; mocked 429/Retry-After must schedule the requested retry window | `ADAPTIVE-ICONS.md` |
| P1 | observability | `vesper-doctor` now exposes bounded desktop/session keys for the graphical target, portals, Vicinae, Caelestia config writability and Vesper clipboard/icon services; live activated-session proof is still pending | keep checks ownership-specific and skip them explicitly outside a graphical session | `vesper-doctor --json` must expose stable keys and return warnings when each invariant is intentionally broken in a fixture/manual test | `SETTINGS.md` + `vesper-doctor` implementation |
| P1 | security boundary | Agent Cockpit now persists only the process identity from `ps comm` and a schema-versioned bounded snapshot; activated-generation proof is still pending | keep process snapshots bounded and diagnostic; never persist raw argv or prompt/token-like arguments | fixture with token/prompt-like argv must not place the sensitive value in durable agent snapshots | `AI.md`, `../AGENTS.md` |
| P1 | observability | Privacy HUD now reads the two `vesper-cliphist-*` user services instead of inferring clipboard-history health from Caelestia/Quickshell, but activated-session proof is still pending | keep clipboard history tied to the ownership-specific watcher/service state | killing both watchers while leaving Caelestia alive must make clipboard-history state unhealthy/offline | `../AGENTS.md` privacy contract |
| P1 | observability | Privacy HUD and wellbeing still launch subprocess-heavy probes on short UI polling intervals; Agent Cockpit now reuses a bounded 10-second status snapshot | move remaining expensive probes to event-driven/cached backends where practical and decouple UI refresh cadence from probe cadence | repeated UI refreshes must not spawn full Git/process/device probe sets at the same rate when source state has not changed | `AI.md`, `APPS-SETTINGS.md`, `../AGENTS.md` |
| P1 | security boundary | Nix daemon trusted users are now limited to `root`; the configured substituter/key does not require wheel-wide trust, but the activated-user workflow still needs a Nix-capable host proof | keep trusted users at the minimum required principals and retain the existing cache configuration | normal `nh`/flake/cache workflows must still function after removing unnecessary wheel-wide trust | current Nix module + `../AGENTS.md` Nix contract |
| P1 | security boundary | Zapret2 now limits the NFQUEUE path to the verified physical `wlan0` and `enp2s0` interfaces; activated nftables proof is still pending | keep VPN, tunnel, loopback, container and Wi-Fi P2P interfaces outside the desync path | nftables/Zapret rules must show only the declared physical interfaces entering the desync path; VPN/tunnel control traffic must remain outside unless explicitly desired | `NETWORK-SETTINGS.md` + current privacy module |
| P1 | dependency hygiene | browser/context MCPs launched with `bunx` are exact-versioned but still fetched from the package registry at runtime rather than from the Nix build graph | package high-value MCP runtimes through Nix where practical or explicitly document/cache-pin the mutable runtime boundary | offline/session-start test must distinguish packaged MCPs from intentionally network-dependent runtimes; authenticated-browser MCP availability must not depend on an undocumented first-run download | `MCP.md`, `AI.md` |
| P1 | security boundary | one shared MCP registry is automatically exposed to multiple agent runtimes while per-agent capability enforcement is not implemented | keep inventory separate from authorization and introduce enforceable per-agent MCP capability policy before exposing allow/deny controls | denied agent fixture must be technically unable to invoke the restricted MCP, not merely show a disabled label | `AI.md`, `MCP.md` |
| P1 | semantic drift | Store catalogue readiness validates only a subset of the schema tables, so a partial catalogue can be reported as available | validate the complete schema contract plus metadata/revision coherence needed by the UI | malformed fixture missing categories/keywords/screenshots/aliases or revision metadata must report unavailable | `MARKETPLACE.md` |
| P1 | semantic drift | Store spec suggests `~/.local/state/vesper/store/profile` while also requiring retained generations to stay GC-rooted and visible to the desktop; profile GC ownership/session exposure are not yet defined coherently | use a Nix profile location/registration model with explicit GC roots and define how its `bin`/`share/applications` outputs enter the effective user session | install/rollback/`nh clean` integration test must keep retained Store apps alive and make their desktop entries visible without manual environment edits | `MARKETPLACE.md` |
| P1 | state ownership | wellbeing now gates each sample on explicit logind `IdleHint=no` and `LockedHint=no`; live lock/idle proof is still pending | keep accounting on the session owner and skip samples when the state is idle, locked or unknown; keep foreground sampling explicitly approximate | lock/idle interval must not increase app usage totals | `APPS-SETTINGS.md` |
| P1 | observability | Privacy HUD now uses a running PipeWire `Stream/Input/Audio` node for microphone attention and `tor.service` for system Tor, but live capture/browser-isolation proof is still pending | keep microphone attention tied to real capture and system-Tor state tied to the system-owned unit | an unmuted unused microphone must remain inactive; active capture must be detected; Tor Browser alone must not make the system-Tor indicator active | `../AGENTS.md` privacy contract |
| P1 | state ownership | proxy configuration now derives status from the effective private `environment.d` file and writes it atomically; injected write-failure proof is still pending | keep effective-file status authoritative and remove legacy bookkeeping coherently | injected write failure must leave status unconfigured and must not expose secret-bearing files broadly | `NETWORK-SETTINGS.md` |
| P1 | semantic drift | airplane mode now snapshots Wi-Fi/WWAN and optional Bluetooth state in the user runtime directory and restores it on exit; live hardware round-trip proof is still pending | keep the state runtime-only, report radio failures and restore the exact captured state | round-trip test with mixed Wi-Fi/WWAN/Bluetooth initial states must restore the exact prior state | `NETWORK-SETTINGS.md` |
| P1 | documentation drift | repository name is `vesper` while `nh`, aliases and command-memory still assume the local checkout path `~/nix-config` | keep one explicit local checkout contract; until code migrates, installation docs must clone `OnurByte/vesper` into `~/nix-config` | fresh-install instructions and all hard-coded local paths must agree | `INSTALL.md` |
| P1 | observability | `vesper-doctor` now tests the configured Restic file's existence without reading its root-only contents; activated-generation proof is still pending | keep configuration health on existence/metadata or another privileged service-visible condition, and keep repository checks privileged | correct `0600 root:root` configuration must report configured without exposing contents | `BACKUP.md` |
| P2 | dependency hygiene | adaptive-icon Cargo `src = lib.cleanSource ./.` uses the entire `home/yargc/packages/` directory as the crate source, coupling unrelated package edits to icon rebuilds | move the crate into its own source directory or filter the source to the files it actually owns | unrelated QML/package edit must not change the icon crate source hash | `ADAPTIVE-ICONS.md` |
| P2 | dependency hygiene | `llm-agents` packages are consumed from their own nixpkgs graph even though upstream exposes a shared-nixpkgs overlay | evaluate moving to the upstream shared-nixpkgs overlay so agent packages build against Vesper's pinned package universe, unless upstream compatibility requires isolation | closure/eval comparison must show no regression and one intentional nixpkgs authority for integrated packages | `AI.md` + current flake |
| P2 | dependency hygiene | the normal update alias advances many fast-moving inputs together, including heavily patched Caelestia/Quickshell and AI/browser packages | split update workflows by input family or require grouped compatibility verification for broad updates | update procedure must support advancing shell, AI and application inputs independently and still run the relevant build checks | current flake + maintenance docs |
| P2 | dependency hygiene | external skills are pinned by `builtins.fetchGit` outside the flake input/lock graph | keep immutable revisions and document them as intentional lockfile-external source dependencies, or promote them to explicit flake inputs when the dependency boundary justifies it | `home/yargc/skills.nix` must expose every external skills source and exact revision in one obvious maintenance location | `SKILLS.md` + current skills wiring |
| P2 | semantic drift | AppStream icon recovery uses a bounded hand-written XML substring parser rather than a real AppStream/XML parser | keep it fallback-only or replace with a maintained parser before relying on richer AppStream semantics | malformed/entity/attribute-order fixtures must fail safely without overriding the primary freedesktop resolver | `ADAPTIVE-ICONS.md` |
| P2 | state ownership | GNOME Keyring + greetd PAM enablement is declared in more than one module | assign the configuration to one module boundary and remove duplicate declarations | Nix evaluation stays unchanged after deduplication and only one module owns the policy | current security/desktop modules |

### remediation order

Use this order unless a smaller prerequisite is required:

1. Caelestia writable-state ownership
2. Hyprland/systemd graphical-session lifecycle
3. first-party patch-stack consolidation and production/CI source convergence
4. `.vicon` renderer authority and provider-neutral semantic pipeline integrity
5. canonical freedesktop application/icon identity shared by icons, Apps and wellbeing
6. Apps package/source ownership attribution
7. adaptive-icon queue versioning, heartbeat and provider retry semantics
8. doctor and HUD truthfulness
9. security-boundary narrowing for Nix, Zapret2 and shared MCP capability exposure
10. Store profile/GC/session-exposure contract before install implementation
11. dependency/build-boundary cleanup
12. lower-risk module and fallback-parser cleanup

Do not mark an item closed because prose was updated. The implementation change and verification must land first.

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
