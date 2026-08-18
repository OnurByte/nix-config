# AI

Vesper exposes AI as a native Caelestia Nexus settings page.

It combines provider status, live agents, the canonical skill tree, MCP inventory and Hermes state without adding another desktop shell.

## API keys

The credential manager is API-key only. It does not implement OAuth.

Keys are stored through freedesktop Secret Service with `secret-tool`. They are not written into Nix source, Home Manager session variables, shell history or process arguments.

Check configured providers:

```bash
vesper-control ai-status
```

Run one command with a single provider key scoped to that child process:

```bash
vesper-control credential exec openai your-command --args
```

Supported shared key slots are OpenAI, Anthropic, xAI, OpenRouter and Google AI.

## skills and MCP

The page reads skills from the canonical `~/.agents/skills` tree. Agent-specific skill directories stay links into that tree.

The MCP list is generated from `programs.mcp.servers`, so the settings page reports the same registry that Home Manager exposes to Codex, Claude Code and OpenCode.

## agent orchestration and CCCC

CCCC (`ChesterRa/cccc`) is useful to Vesper as an agent-orchestration layer, but it must not become a core Vesper dependency or the implementation foundation of the AI control plane.

The architectural boundary is deliberate:

```text
Vesper
└── AI Control Plane
    ├── providers and shared credentials
    ├── skills
    ├── MCP registry and permissions
    ├── usage / status
    ├── Hermes integration
    └── orchestration backends
        ├── native Vesper
        └── CCCC (optional)
            ├── ChatGPT Web
            ├── Codex
            ├── Hermes
            └── other supported agent runtimes
```

Vesper remains the owner of provider configuration, secrets, permissions, skills, desktop integration and user-facing AI settings. CCCC may be used behind that boundary for agent lifecycle, persistent coordination, foreman/worker teams, task/message state, nudging, automation and cross-runtime orchestration.

Do not expose CCCC as the product model in the primary UI. If an optional CCCC backend is added later, the user-facing concept should be a generic Vesper capability such as **Orchestration** or **Agent Teams**. CCCC is an implementation choice behind that interface.

### use CCCC to develop Vesper

CCCC may be used immediately as a development tool without any product integration. A useful development topology is:

```text
ChatGPT Web / strong reasoning model   -> foreman / reviewer
Codex                                  -> implementation worker
Hermes                                 -> research / long-running worker
CCCC                                   -> coordination, persistent task state and hand-offs
Vesper repository                      -> shared workspace
```

A typical loop is:

```text
inspect repository
-> plan a bounded change
-> delegate research or implementation
-> edit
-> test / typecheck / build
-> inspect diff
-> fix failures
-> checkpoint
-> continue with the next useful task
```

This is the preferred near-term role for CCCC: **use CCCC to build Vesper, do not build Vesper on CCCC**.

### future optional backend

An optional CCCC backend is acceptable only if it remains replaceable. Vesper must define its own internal orchestration interface instead of leaking CCCC-specific actor, ledger or runtime concepts through the rest of the codebase.

The integration boundary should be narrow enough that Vesper can later:

- replace CCCC with a native Rust implementation;
- support another orchestration engine alongside it;
- run simple single-agent flows without CCCC;
- keep credentials and permissions controlled by Vesper rather than delegated to the backend;
- preserve Vesper state if the external orchestration backend is removed.

Do not duplicate CCCC wholesale inside Vesper. Reuse it first to validate which orchestration features are actually valuable. Only migrate proven, Vesper-specific primitives into the native Rust control plane when there is a concrete reason to own them.

## adaptive icons

Adaptive icons use the automatic Rust-owned pipeline defined in `ADAPTIVE-ICONS.md`; the old Apps → Experimental manual request/review queue is obsolete.

The engine discovers effective `.desktop` applications, resolves trustworthy packaged icon sources, fingerprints and deduplicates canonical work, persists conversion jobs, reuses accepted `.vicon` packages, and updates the generated Vesper freedesktop icon theme. Provider outages or missing credentials leave existing/original fallback icons usable instead of breaking the desktop.

Generation/provider controls live under AI. Appearance and material controls stay under Appearance/Theme, while application-specific retry/exclusion/original/diagnostic actions stay under Apps.

Remote semantic conversion requires explicit **Allow remote icon analysis** consent. Consent is off by default. With consent off, local vector handling, accepted canonical packages and fallback rendering continue to work, while jobs that require a provider remain `blocked-no-consent` and the worker does not claim new remote work. Enabling consent allows eligible jobs to resume automatically; a provider key is still required separately.

A configured shared provider key is reused automatically. Palette, wallpaper, appearance and renderer changes are local recompiles and must not consume another AI request for an already valid canonical package.

Per-app diagnostic export is local-only. Vesper intentionally has no bulk icon export UI or bulk export backend command.
