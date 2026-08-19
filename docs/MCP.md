# MCP

Status: **current**

Vesper keeps one Home Manager MCP registry and feeds it to Codex, Claude Code and OpenCode.

Configured servers:

- `nixos` — NixOS, nixpkgs and Home Manager package/option lookup through `mcp-nixos`
- `context7` — current library and API documentation through Context7
- `github` — GitHub repositories, issues, pull requests and Actions through GitHub's official MCP server
- `hypruse` — Hyprland-native desktop inspection and confined GUI control
- `semgrep` — local deterministic static/security analysis through Semgrep's built-in MCP server
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
use semgrep with an explicit ruleset to security-scan these changed files
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

Vesper packages upstream `hypruse 0.9.4` from its published `py3-none-any` wheel with the wheel SHA-256 pinned in Nix. The package uses the pinned nixpkgs `mcp` Python SDK and runs entirely from the Nix store; starting the MCP does not download Python packages or depend on a mutable uv environment.

Upstream Python stays inside that external package. Vesper's first-party runtime/control-plane code remains Rust.

Hypruse talks directly to the active Hyprland session through `hyprctl`, native Wayland input, `wtype`, `grim` and AT-SPI. It does not require `ydotool`, a root daemon or a RemoteDesktop portal.

Vesper does not expose it unconstrained. The wrapper sets:

```text
HYPRUSE_CONFINE=launched
HYPRUSE_AUTH_GUARD=strict
HYPRUSE_STRICT=1
HYPRUSE_MARK=1
```

This means input is limited to windows launched by the MCP, authentication dialogs remain guarded, an unexpected human/focus change forces the agent to re-observe before acting and agent-owned windows are marked when supported. Clipboard access is not enabled.

Hypruse tags windows it launches as `hypruse-owned`. Vesper keeps a matching Hyprland window rule that gives those windows a 2 px red safety outline, so the automation boundary remains visible even when Hypruse's runtime marking rule does not render on a particular compositor build.

Confinement is an input boundary, not a privacy boundary. Desktop state and screenshots can still reveal visible information, and an MCP client still decides whether to approve tool calls. Do not treat Hypruse inventory as a stronger sandbox than the enforcement it actually provides.

Hypruse tool calls and refusals are recorded to:

```text
~/.local/state/vesper/mcp/hypruse/journal.ndjson
```

The journal is bounded by Hypruse's upstream rotation policy. Typed/copied text stays redacted because `HYPRUSE_JOURNAL_TEXT` is deliberately not enabled, and observation results such as screenshots are not copied into the journal.

For manual startup/diagnostics:

```bash
vesper-hypruse-mcp
```

`Super + Shift + Backspace` is the emergency stop. It runs `vesper-hypruse-mcp stop`, which targets the PID advertised by Hypruse's runtime beacon and follows the upstream graceful shutdown path so held input state is cleaned up before the server exits.

Use it for real desktop QA where browser-only MCPs are insufficient. Prefer semantic desktop/accessibility state before screenshots when the target exposes it.

## Semgrep

`semgrep` uses the Nix-managed Semgrep CLI from the locked nixpkgs revision and starts its built-in stdio MCP server with `semgrep mcp`.

It is a deterministic analysis surface, not another coding agent. Use it to scan code or evaluate a custom Semgrep rule, then let the coding-agent workflow decide what to change from the findings.

Vesper sets:

```text
SEMGREP_SEND_METRICS=off
```

No Semgrep AppSec token is configured by default. Keep scans local and do not enable telemetry to make an MCP call succeed.

With metrics disabled, Semgrep intentionally refuses its config-less `auto` scan path. Pass an explicit configuration such as `p/default`, another reviewed registry ruleset or a custom rule when calling the scan tools. A request that fails because no configuration was supplied should be corrected by selecting a ruleset, not by enabling metrics.

Registry rules may require network access to retrieve the rules themselves. That is different from uploading the source being scanned; do not silently turn the optional cloud/AppSec integration into a Vesper dependency.

## Context7

Context7 is pinned to `@upstash/context7-mcp@4.0.2` and built into the Nix store.

The workstation-wide nixpkgs revision still carries `context7-mcp 4.0.0`, so Vesper overrides only this package with the immutable source and pnpm dependency hashes from the newer nixpkgs recipe instead of moving the whole flake pin or downloading the npm package at MCP startup.

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

The browser DevTools MCP packages are version-pinned but launched through Bun's `bunx` because they are not currently part of the Vesper Nix package set.
Their isolated Bun cache lives under:

```text
~/.cache/vesper-mcp/bun
```
