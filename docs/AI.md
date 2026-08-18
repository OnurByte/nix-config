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

## adaptive icons

Adaptive icons use the automatic Rust-owned pipeline defined in `ADAPTIVE-ICONS.md`; the old Apps → Experimental manual request/review queue is obsolete.

The engine discovers effective `.desktop` applications, resolves trustworthy packaged icon sources, fingerprints and deduplicates canonical work, persists conversion jobs, reuses accepted `.vicon` packages, and updates the generated Vesper freedesktop icon theme. Provider outages or missing credentials leave existing/original fallback icons usable instead of breaking the desktop.

Generation/provider controls live under AI. Appearance and material controls stay under Appearance/Theme, while application-specific retry/exclusion/original/diagnostic actions stay under Apps.

A configured shared provider key is reused automatically. Palette, wallpaper, appearance and renderer changes are local recompiles and must not consume another AI request for an already valid canonical package.

Per-app diagnostic export is local-only. Vesper intentionally has no bulk icon export UI or bulk export backend command.
