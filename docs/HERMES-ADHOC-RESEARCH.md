# Hermes ad-hoc research

Status: **current**

The scheduled frontier stays bounded. One-off research uses `vesper-research` and the same Rust state/report layer without creating another scheduler.

```bash
vesper-research "monero opsec" --pages 600
```

Equivalent explicit form:

```bash
vesper-research run "monero opsec" --pages 600
```

`--pages` is the candidate-inspection target, not a promise that every candidate becomes a full model page read. The current Rust CLI clamps explicit targets to `50..2000` and defaults to `600`.

Deep-read budget scales separately from the candidate target and can be overridden:

```bash
vesper-research run "coding agent context engineering" --pages 1200 --deep-reads 80
```

The default deep-read target is roughly `pages / 12`, clamped to `12..80`. An explicit `--deep-reads` value is clamped to `1..120`.

The research agent can use GitHub, Reddit, X, normal web/forums and local Tor/onion access where relevant. Existing sources are seeds, never an allowlist. Important community claims should be followed to primary technical evidence.

The final result is persisted as an `adhoc-research` briefing under the normal Vesper briefing/state roots. The current Rust implementation does not claim the removed Python-era wave-local artifact tree or a fake multi-wave guarantee.

## source view

Inspect the shared evidence-backed source registry:

```bash
vesper-research sources
vesper-hermes-automations links
```

Current records are intentionally compact. They can include:

```text
id
url
tier
score
hits
failures
origin
firstSeen
lastSeen
lastUseful
```

A URL is reinforced only when it survives into final report evidence. Repeated useful hits move it from `probation` to `trusted` and then `promoted`.

## coverage policy

A request is a target, not invented coverage. The report should state actual candidate/deep-read counts and blocked surfaces when available.

For large research:

- use broad cheap discovery before deep reading
- deduplicate mirror/canonical identities conceptually
- keep GitHub, Reddit, X and relevant web/onion surfaces in the plan
- spend deep-read budget on the strongest candidates
- verify important claims against primary sources
- report shortfall instead of padding weak results

Tor is transport, not corroboration. Onion access uses the local helper:

```bash
vesper-hermes-automations tor-fetch 'http://example.onion/path/'
```
