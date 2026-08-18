# Vesper docs

- `AI.md` — native AI settings, API-key manager, skills, MCP inventory and optional agent orchestration backends such as CCCC
- `APPS-SETTINGS.md` — per-app controls and wellbeing
- `ADAPTIVE-ICONS.md` — single source of truth for adaptive icon discovery, GPT semantic decomposition, multi-layer `.vicon`, Apple-style geometry/material rendering, runtime identity, validation and fallback
- `MARKETPLACE.md` — Nixpkgs-first native application marketplace, local catalogue, transactions, rollback and optional Flathub source
- `NETWORK-SETTINGS.md` — airplane mode, Wi-Fi QR, proxy and DPI status

Adaptive icon architecture lives only in `ADAPTIVE-ICONS.md`. Do not split AI, Apple compatibility, auto-fit, layered rendering, fidelity or export rules into additional icon-specific Markdown files.

Marketplace architecture lives only in `MARKETPLACE.md`. Do not split catalogue, Nix transaction, Flathub or application-source rules into additional Marketplace-specific Markdown files.

Agent orchestration architecture lives in `AI.md`. Do not make CCCC a core dependency or create a second source of truth for the Vesper AI control plane; CCCC is an optional replaceable backend and a development-time orchestration tool.

The existing install, backup, Hermes, MCP, secrets and skills documents remain authoritative for their underlying subsystems.
