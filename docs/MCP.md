# MCP

Status: **current**

Vesper keeps one Home Manager MCP registry and feeds it to Codex, Claude Code and OpenCode.

Configured servers:

- `nixos` — NixOS, nixpkgs and Home Manager package/option lookup through `mcp-nixos`
- `context7` — current library and API documentation through Context7
- `github` — GitHub repositories, issues, pull requests and Actions through GitHub's official MCP server
- `hypruse` — Hyprland-native desktop inspection and confined GUI control
- `helium-devtools` — Chrome DevTools MCP pointed at the Nix-managed Helium binary
- `zen-devtools` — Mozilla Firefox DevTools MCP pointed at the Nix-managed Zen beta binary

## use it

After a switch:

```bash
nh os switch
```

Codex, Claude Code and OpenCode pick the servers up from Home Manager automatically. There is no separate per-agent MCP setup.

Useful requests are ordinary agent requests:

```text
use the nixos MCP to find the correct Home Manager option for this
use context7 for the current Next.js API before changing this code
use the github MCP to inspect the failing Actions run and related pull request
use hypruse to launch Vesper Settings and inspect the real Hyprland UI
use helium-devtools to inspect this page's network requests and console
use zen-devtools to open this site and reproduce the Firefox-side bug
```

## GitHub

Vesper pins GitHub MCP Server `1.9.0` and enables the `context`, `repos`, `issues`, `pull_requests` and `actions` toolsets.

The wrapper first reuses the current GitHub CLI login:

```bash
gh auth status
```

If `gh auth token` is available, that token is passed to the MCP process without writing it into Nix or Git. If there is no GitHub CLI login, the official release can fall back to its browser OAuth flow.

For manual diagnostics:

```bash
vesper-github-mcp
```

The command is normally started by the agent and speaks MCP over stdio, so running it manually is only useful for startup/auth errors.

## Hypruse

`hypruse` runs upstream `hypruse==0.9.4` through an isolated `uvx` wrapper instead of adding Python or uv to the normal user toolchain.

It talks directly to the active Hyprland session through `hyprctl`, native Wayland input, `wtype`, `grim` and AT-SPI. It does not require `ydotool`, a root daemon or a RemoteDesktop portal.

Vesper does not expose it unconstrained. The wrapper sets:

```text
HYPRUSE_CONFINE=launched
HYPRUSE_AUTH_GUARD=strict
HYPRUSE_STRICT=1
HYPRUSE_MARK=1
```

This means input is limited to windows launched by the MCP, authentication dialogs remain guarded, an unexpected human/focus change forces the agent to re-observe before acting and agent-owned windows are marked when supported. Clipboard access is not enabled.

Confinement is an input boundary, not a privacy boundary. Desktop state and screenshots can still reveal visible information, and an MCP client still decides whether to approve tool calls. Do not treat Hypruse inventory as a stronger sandbox than the enforcement it actually provides.

The first run may populate the isolated uv cache at:

```text
~/.cache/vesper-mcp/uv
```

For manual startup/diagnostics:

```bash
vesper-hypruse-mcp
```

Use it for real desktop QA where browser-only MCPs are insufficient. Prefer semantic desktop/accessibility state before screenshots when the target exposes it.

## Context7

Context7 is pinned to `@upstash/context7-mcp@4.0.2`.

Basic use does not require an API key. Ask the agent to `use context7` when a task depends on current framework or library documentation.

An API key is optional for higher limits and private repositories. If one is added later, keep it in sops-nix when it is consumed declaratively by the MCP server.

## Helium

`helium-devtools` runs pinned `chrome-devtools-mcp@1.7.0` with the Helium executable from the flake input.

Its persistent automation profile is:

```text
~/.local/share/vesper/helium-mcp
```

The first browser tool call starts Helium. Sign in to sites in that profile once when an authenticated session is useful. Cookies and browser state remain there for later agent sessions.

This is not a permission sandbox. An agent using the MCP can operate the pages and authenticated sessions available in that profile.

## Zen

`zen-devtools` runs pinned `@mozilla/firefox-devtools-mcp@0.9.15` with `zen-beta` and the developer tool preset.

Its persistent automation profile is:

```text
~/.local/share/vesper/zen-mcp
```

The first browser tool call starts Zen. Sign in once in that profile when needed and the profile persists between sessions.

The Mozilla server talks to the Firefox engine through WebDriver BiDi, which fits Zen's Firefox base instead of pretending it is a Chromium browser.

## reset browser state

Close the related agent/browser and remove only the automation profile you want to reset:

```bash
rm -rf ~/.local/share/vesper/helium-mcp
rm -rf ~/.local/share/vesper/zen-mcp
```

The browser and Context7 npm packages are version-pinned but launched through Bun's `bunx` because they are not currently part of the Vesper Nix package set.
Their isolated Bun cache lives under:

```text
~/.cache/vesper-mcp/bun
```
