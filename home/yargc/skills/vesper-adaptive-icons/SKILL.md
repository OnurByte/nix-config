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
- remote semantic conversion requires explicit `remoteConsent`; do not infer consent from feature enablement or from an existing API key
- bulk icon export is intentionally unsupported; keep only per-app local diagnostic export

## queue behavior

The Rust conversion queue is persistent. Jobs may be `pending`, `ready`, `running`, `retry-wait`, `blocked-no-provider`, `blocked-no-consent`, `succeeded`, `failed`, `superseded` or `cancelled`.

A missing provider key is not a permanent failure. Keep the job blocked and allow the daemon to move it to `ready` automatically when the selected provider becomes available.

Missing remote consent is also a capability block, not a failure. Keep affected work `blocked-no-consent`; enabling consent should automatically make eligible work `ready` when the selected provider is configured. The worker must not claim new remote conversion work while consent is disabled.

Do not create one remote request per desktop entry when several entries share the same trustworthy source fingerprint.

For long conversion waves, queue state is the source of truth. A worker timeout does not mean the conversion failed; reconcile durable queue/artifact state before retrying.

## canonicalization rules

AI is used for semantic decomposition or reconstruction only when local canonicalization is insufficient. It must not generate the final glossy PNG, own palette colors or bake Vesper's material renderer into source artwork.

Preserve recognizable identity and reliable original vector geometry whenever possible. Reject external URLs, scripts, embedded raster payloads disguised as SVG, unsafe resources and recursive Vesper-generated provenance.

Canonical output follows the `.vicon` package contract in `docs/ADAPTIVE-ICONS.md`. Flattened SVG or PNG files are compiled outputs, not canonical source.

## identity lock and QA

When AI reconstruction is required, consistency is not maintained by descriptive adjectives alone.

Use the best trustworthy upstream icon as the canonical visual identity reference and preserve explicit identity constraints:

```text
must     -> structural traits required to remain recognizable
never    -> concrete drift patterns that invalidate the candidate
```

Add a `never` rule only when there is a real recurring failure mode or a documented canonical constraint. Do not grow a generic aesthetic blacklist unrelated to the app.

After generation, deterministic checks come before semantic/model QA:

- package/schema validity
- safe resource/provenance checks
- expected render dimensions/formats
- artifact readability/integrity
- source/output fingerprint rules
- duplicate detection when a batch should contain distinct outputs

For a batch diagnostic, a contact-sheet/grid can expose systematic identity/style drift quickly. It is a diagnostic tool, not a return to mandatory per-app human approval.

Generated text should not become part of icon identity unless the canonical upstream asset actually contains it and the pipeline can preserve it reliably.

## evidence rule

A provider/model saying a conversion succeeded is not enough. The persistent queue state and validated canonical/output artifact are the acceptance evidence. On a failed remote call, verify whether a usable artifact or partial state was produced before retrying.

Use `agent-operations` for the general durable-job, postcondition and governance rules behind this pipeline.

## operations

Use `vesper-control icon status` for engine state and `vesper-control icon queue-status` for persistent conversion queue state.

Use `vesper-control icon remote-consent on|off` to change the explicit remote artwork-analysis permission. This setting is independent from provider credentials and from enabling the local adaptive-icon feature.

Use `vesper-control icon reconcile` after debugging discovery or source resolution. Use `vesper-control icon app-retry <desktop-id>` only when an explicitly failed or invalidated application should be retried.

Do not require per-app manual review or approval for normal automatic operation.
