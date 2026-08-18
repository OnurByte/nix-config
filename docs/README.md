# Vesper docs

- `AI.md` — native AI settings, API-key manager, skills, MCP inventory and optional agent orchestration backends such as CCCC
- `AI-ANALYTICS.md` — canonical AI telemetry and analytics semantics: CodexBar + ccusage + TurnLens sources, quota/reset history, Vibe Coding Activity heatmap, active-time/agent-hours definitions, model/agent/project statistics and local history rules
- `APPS-SETTINGS.md` — installed-app controls, wellbeing and Vesper Store integration
- `ADAPTIVE-ICONS.md` — single source of truth for adaptive icon discovery, GPT semantic decomposition, multi-layer `.vicon`, Apple-style geometry/material rendering, runtime identity, validation and fallback
- `MARKETPLACE.md` — Vesper Store: separate native Qt 6/QML app, Rust backend, Nixpkgs-first local catalogue, transactions, rollback and optional Flathub source
- `NETWORK-SETTINGS.md` — airplane mode, Wi-Fi QR, proxy and DPI status

Adaptive icon architecture lives only in `ADAPTIVE-ICONS.md`. Do not split AI, Apple compatibility, auto-fit, layered rendering, fidelity or export rules into additional icon-specific Markdown files.

Vesper Store architecture lives only in `MARKETPLACE.md`. Do not split catalogue, Nix transaction, Flathub, Qt/QML UI or application-source rules into additional Store-specific Markdown files.

AI control-plane architecture and agent orchestration live in `AI.md`. Do not make CCCC a core dependency; CCCC is an optional replaceable backend and a development-time orchestration tool.

AI usage/telemetry measurement semantics live in `AI-ANALYTICS.md`. Do not create parallel quota, token, cost, active-time or vibe-coding definitions elsewhere. `AI.md` owns the product/control-plane boundary; `AI-ANALYTICS.md` owns analytics source normalization and measurement semantics.

The existing install, backup, Hermes, MCP, secrets and skills documents remain authoritative for their underlying subsystems.
