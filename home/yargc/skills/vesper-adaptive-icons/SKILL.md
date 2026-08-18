---
name: vesper-adaptive-icons
description: Inspect and operate Vesper's automatic adaptive icon pipeline, canonical packages, conversion queue and generated icon theme.
---

# Vesper adaptive icons

Use this skill when working on Vesper adaptive application icons.

`docs/ADAPTIVE-ICONS.md` is the single source of truth for architecture and behavior. Do not recreate the old manual request/review workflow.

## runtime contract

- inventory and queue state live under `~/.local/state/vesper/adaptive-icons/`
- canonical assets live under `~/.local/share/vesper/adaptive-icons/canonical/`
- generated theme generations live under `~/.local/share/vesper/adaptive-icons/themes/`
- the active freedesktop theme is `~/.local/share/icons/Vesper-Adaptive`
- never overwrite packaged application icons or anything in `/nix/store`
- never use Vesper-generated outputs as a future upstream source
- keep canonical conversion independent from palette and appearance rendering
- source-hash identical work must be deduplicated before remote conversion
- provider outages must leave original or previously accepted icons usable

## queue behavior

The Rust conversion queue is persistent. Jobs may be `pending`, `ready`, `running`, `retry-wait`, `blocked-no-provider`, `blocked-no-consent`, `succeeded`, `failed`, `superseded` or `cancelled`.

A missing provider key is not a permanent failure. Keep the job blocked and allow the daemon to move it to `ready` automatically when the selected provider becomes available.

Do not create one remote request per desktop entry when several entries share the same trustworthy source fingerprint.

## canonicalization rules

AI is used for semantic decomposition or reconstruction only when local canonicalization is insufficient. It must not generate the final glossy PNG, own palette colors or bake Vesper's material renderer into source artwork.

Preserve recognizable identity and reliable original vector geometry whenever possible. Reject external URLs, scripts, embedded raster payloads disguised as SVG, unsafe resources and recursive Vesper-generated provenance.

Canonical output follows the `.vicon` package contract in `docs/ADAPTIVE-ICONS.md`. Flattened SVG or PNG files are compiled outputs, not canonical source.

## operations

Use `vesper-control icon status` for engine state and `vesper-control icon queue-status` for persistent conversion queue state.

Use `vesper-control icon reconcile` after debugging discovery or source resolution. Use `vesper-control icon app-retry <desktop-id>` only when an explicitly failed or invalidated application should be retried.

Do not require per-app manual review or approval for normal automatic operation.