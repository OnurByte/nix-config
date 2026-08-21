# operations and evidence

Commands:

```text
vesper-xpatla sources
vesper-xpatla scan
vesper-xpatla status --json
vesper-xpatla manual https://x.com/<handle>/status/<id>
vesper-xpatla media-plan <post-id>
vesper-xpatla prepare-media https://video.twimg.com/...mp4
```

Hermes runs `vesper-xpatla scan` every three minutes through the existing
Hermes scheduler. It does not create another timer or cron layer.

Before claiming a scan succeeded, check:

- dynamic enabled source count
- per-source FxTwitter request result
- persisted `runs` row
- new `observed_posts` count
- media asset/provenance rows
- error/partial state

Before claiming a publication succeeded, check the remote FxTwitter status,
exact text, author, media type and publication reconciliation state. Never
retry an ambiguous write without reading the remote account first.

Do not place cookies, X sessions, API keys or x-use credentials in this skill,
the source JSON, logs, Nix expressions or process arguments.
