# Hermes ad-hoc research

The daily frontier remains a bounded recurring radar. Large one-off research uses a separate command and run state.

```bash
vesper-research "monero opsec" --pages 3000
```

Equivalent explicit form:

```bash
vesper-research run "monero opsec" --pages 3000
```

The default run attempts GitHub, Reddit, X/Twitter, normal web/forums and Tor onion sources. A 3000-page request is split into bounded waves instead of one huge model context. `--pages` means canonical candidate inspection; the default deep-read budget scales separately. Override depth when necessary:

```bash
vesper-research run "coding agent context engineering" --pages 3000 --deep-reads 220
```

Run artifacts live under:

```text
~/.local/state/vesper/research/adhoc-research/<run-id>/
```

The final report is persisted as an `adhoc-research` briefing. Wave state stays run-local; only useful evidence-bearing source discoveries reinforce the shared source registry.

## Source view

Show Reddit, X, web and onion sources through one normalized view:

```bash
vesper-research sources
vesper-research sources --json
```

The normalized public fields are:

```text
id kind url label topic seed tier score hits observations failures origin firstSeen lastSeen lastUseful
```

## Coverage policy

Default surface allocation is approximately:

- GitHub 30%
- Reddit 25%
- X 25%
- web/onion 20%

Every surface gets a non-zero budget. Access failure becomes an explicit limitation and contributes to shortfall.

## Current structural notes

- Daily research and ad-hoc research are deliberately separate: recurring radar is clamped to 200-1000 candidates, while ad-hoc research supports larger explicit targets.
- Large ad-hoc runs use wave-local artifacts, avoiding a single giant prompt and reducing duplicate counting.
- Reddit RSS and X mirrors are cheap intake layers; community claims still need primary-source verification.
- Onion pages are fetched locally through Tor and passed into the research lane; Tor is transport, not corroboration.
- `r/opsec` is configured as a Reddit discovery/comment seed.
