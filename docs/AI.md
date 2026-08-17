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

The current implementation is still the temporary Apps → Experimental review queue. Enabling it does not modify icons by itself.

A queued app writes metadata under:

```text
~/.local/state/vesper/adaptive-icons/queue/
```

The `vesper-adaptive-icons` skill can produce a candidate under `~/.local/share/vesper/adaptive-icons/generated/`, but activation remains a separate explicit review step.

The replacement architecture is defined in `ADAPTIVE-ICONS.md`. The target moves automatic generation/provider controls into the AI page, replaces per-app manual queueing with automatic `.desktop` discovery and keeps palette/appearance recompilation local so theme changes do not consume AI requests.
