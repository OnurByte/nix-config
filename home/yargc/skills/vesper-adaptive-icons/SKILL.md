---
name: vesper-adaptive-icons
description: Process Vesper's experimental adaptive app-icon queue into reviewed, palette-aware Linux icons without silently replacing originals.
---

# Vesper adaptive icons

Use this skill when the user asks to process or review queued adaptive app icons from Vesper Settings.

## Contract

- Queue: `~/.local/state/vesper/adaptive-icons/queue/*.json`
- Generated assets: `~/.local/share/vesper/adaptive-icons/generated/`
- Never overwrite the application's packaged icon.
- Never install or activate a generated icon without explicit user approval.
- Preserve recognisable brand geometry and symbols. Adapt framing, background, padding, corner treatment and palette rather than redrawing an unrelated logo.
- Prefer SVG when the source can be represented cleanly; otherwise produce a high-resolution PNG.
- Follow the current Caelestia/Vesper palette instead of hard-coding a permanent colour scheme.
- Do not upload source icons or app metadata to a remote service unless the user explicitly asked for remote generation and the selected provider is configured for that scope.

## Workflow

1. Read every queued JSON item and validate `schemaVersion`, `appId`, `sourceIcon` and `state`.
2. Resolve the installed desktop entry and its real icon source. Do not guess from the application name.
3. Inspect the source icon and current Vesper palette.
4. Produce one conservative adaptive candidate with consistent safe-area/padding and transparent outer canvas.
5. Save it under `~/.local/share/vesper/adaptive-icons/generated/<appId>.svg` or `.png`.
6. Update the queue item to `state: "review"` and add `generatedPath` plus a short `notes` field describing what changed.
7. Stop there. Activation is a separate explicit approval step.

## Review rules

Reject a candidate if it loses the app's recognisable mark, has illegible small details, bakes text into the icon unnecessarily, violates transparency, or clashes with the current palette. If the source is already visually compatible, mark the queue item `state: "no-change"` instead of changing it for the sake of activity.
