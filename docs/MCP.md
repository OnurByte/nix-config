# MCP

Vesper keeps one Home Manager MCP registry and feeds it to Codex, Claude Code and OpenCode.

Configured servers:

- `nixos` — NixOS, nixpkgs and Home Manager package/option lookup through `mcp-nixos`
- `helium-devtools` — Chrome DevTools MCP pointed at the Nix-managed Helium binary
- `zen-devtools` — Mozilla Firefox DevTools MCP pointed at the Nix-managed Zen beta binary

## use it

After a switch:

```bash
nh os switch
```

Codex, Claude Code and OpenCode pick the servers up from Home Manager automatically. There is no separate per-agent setup.

Useful requests are ordinary agent requests, for example:

```text
use the nixos MCP to find the correct Home Manager option for this
use helium-devtools to inspect this page's network requests and console
use zen-devtools to open this site and reproduce the Firefox-side bug
```

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

The MCP npm packages are version-pinned but fetched by `npx` on first use because they are not currently part of the Vesper Nix package set. Their npm cache is isolated under:

```text
~/.cache/vesper-mcp/npm
```
