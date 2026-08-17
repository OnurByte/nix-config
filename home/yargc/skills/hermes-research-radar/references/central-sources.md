# Central Research Sources

Central sources are durable anchors, not an allowlist.

The researcher must inspect them every normal frontier run while still reserving substantial budget for autonomous discovery. An anchor cannot be demoted merely because one or several runs are quiet. Newly discovered sources can earn more budget, but they do not silently replace the anchors.

## Reddit anchors

Primary Reddit anchors:

- `r/MoneroMeansMoney` — Monero/privacy culture, ecosystem edges, niche discussion
- `r/Monero` — protocol/ecosystem/community developments
- `r/LocalLLaMA` — local models, inference, open model tooling and agent techniques
- `r/privacy` — privacy software, policy and practical privacy changes
- `r/NixOS` — Nix/NixOS ecosystem and workstation-relevant changes
- `r/selfhosted` — self-hosted replacements and practical infrastructure
- `r/Tor` — Tor/onion/privacy operational developments
- `r/netsec` — technical security research and tooling

These anchors receive guaranteed intake before general and discovered communities.

## X / Twitter anchors

The initial X anchor graph is selected from the user's followed accounts for technical signal rather than raw engagement.

### AI / developer tooling

- `@Teknium` — Hermes Agent / NousResearch
- `@thdxr` — OpenCode / developer tooling
- `@XOpenSource` — X open-source engineering
- `@ZixuanLi_` — Z.ai / model and AI engineering

### Monero / privacy / decentralized payments

- `@eigenwallet` — Monero-Bitcoin atomic-swap DEX
- `@kyc_rip` — no-KYC swap ecosystem signals
- `@XBToshi` — Bitcoin/Monero privacy tooling ecosystem
- `@schmidt1024` — privacy/open-source/Monero projects
- `@XMRHub_org` — Monero ecosystem aggregation/discovery
- `@CR1337` — crypto-agorism, Monero and privacy projects
- `@linuxuser1996` — Linux/GrapheneOS/Monero/privacy
- `@Examare1` — Monero ecosystem
- `@ZcashLabs` — Zcash engineering/ecosystem counterpoint

### security / threat intelligence

- `@akaclandestine` — OSINT, threat research, OPSEC, threat intelligence
- `@DailyDarkWeb` — dark-web threat intelligence

### private communications

- `@SimpleXChat` — private messaging/network architecture

The following followed accounts are intentionally not central by default when their expected signal is dominated by memes, general trading, price/charts, or broad consumer activity. They remain discoverable through the autonomous graph and may earn promotion if repeated technical value is demonstrated.

## Autonomous source evolution

Every non-anchor source starts as `probation`.

The source lifecycle is:

`probation -> trusted -> promoted`

A source earns promotion only when its discoveries repeatedly survive triage/deep-read and produce useful evidence-bearing findings. One lucky hit is not enough.

The runtime keeps a source registry under:

`~/.local/state/vesper/research/unknown-frontier-ai/source-registry.json`

For discovered sources track at least:

- type and canonical identity
- tier
- score
- useful-hit count
- first/last seen time
- last useful time
- failure count when relevant
- discovery provenance when available

Anchors are marked `protected=true`. They may accumulate health/failure history but cannot be automatically demoted or removed.

## Budget policy

A normal source budget should roughly reserve:

- 35-50% for central anchors
- 25-40% for trusted/promoted discovered sources
- 20-30% for exploration/query tail/new sources

These are adaptive bands, not rigid quotas. If one transport is blocked, move unused budget to another route while recording the access failure.

Do not let autonomous optimization collapse the source graph into a small echo chamber. Preserve explicit exploration and cross-source diversity even when a few sources have high historical hit rates.
