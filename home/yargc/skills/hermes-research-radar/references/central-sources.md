# Central Research Sources

Central sources are **initial seeds**, not an allowlist and not permanent providers.

They exist to give a fresh researcher a useful starting graph. Once enough runtime evidence exists, learned sources can outrank or replace them. A seed that lowers research quality can be removed.

## Interest map

The research center of gravity is:

1. vibe coding / coding agents / agent harnesses / skills / MCP / context engineering;
2. Monero / privacy / private payments / Tor / onion services / OPSEC / private communications;
3. Nix/Linux/security/open-source infrastructure when it improves those workflows or the workstation.

Generic local-model/inference discussion is not a standing interest. `r/LocalLLaMA` is explicitly excluded from the default research graph.

## Reddit seed communities

### Vibe coding / agentic development

- `r/vibecoding`
- `r/ClaudeCode`
- `r/codex`
- `r/opencodeCLI`
- `r/cursor`

### Monero / privacy

- `r/MoneroMeansMoney`
- `r/Monero`
- `r/privacy`
- `r/Tor`
- `r/netsec`

### Workstation / infrastructure

- `r/NixOS`

These are starting communities, not eternal priorities. Runtime evidence should decide which sources keep receiving budget.

## X / Twitter seed accounts

The initial X graph is selected from the user's followed accounts for technical signal rather than raw engagement.

### Coding agents / developer tooling

- `@Teknium`
- `@thdxr`
- `@XOpenSource`
- `@ZixuanLi_`

### Monero / privacy / decentralized payments

- `@eigenwallet`
- `@kyc_rip`
- `@XBToshi`
- `@schmidt1024`
- `@XMRHub_org`
- `@CR1337`
- `@linuxuser1996`
- `@Examare1`
- `@ZcashLabs`

### Security / threat intelligence

- `@akaclandestine`
- `@DailyDarkWeb`

### Private communications

- `@SimpleXChat`

Accounts dominated by memes, general trading, price/charts or broad consumer activity are not seed priorities.

## GitHub seed neighborhoods

### Coding-agent starting points

- `NousResearch/hermes-agent`
- `openai/codex`
- `anthropics/claude-code`
- `anomalyco/opencode`

### Monero/privacy starting points

- `monero-project/monero`
- `Cuprate/cuprate`

Expand outward through issues, PRs, commits, forks, authors, dependencies and small adjacent repositories.

## Web / onion seed links

The initial standardized link registry starts with:

- `https://opbible7nans45sg33cbyeiwqmlp5fu7lklu6jd6f3mivrjeqadco5yd.onion/opsec/`
  - label: `OP Bible OPSEC`
  - topic: `privacy-opsec`
  - seed: `true`
- `https://monero.forum/`
  - label: `Monero Forum`
  - topic: `monero-privacy`
  - seed: `true`

The onion URL is fetched only through the local Tor SOCKS path. Monero Forum should prefer technical, security, services, wallets, private-payment, Tor/privacy and useful outgoing-link threads; routine price/trading/chart discussion is down-ranked.

`seed=true` means only “bootstrap source”. It does **not** prevent demotion or deletion.

## Web/onion quality replacement

The researcher may add new web/onion links from useful outgoing edges. New links begin as learned probation sources.

A web/onion link is eligible for deletion when it has enough observations and stays poor for **84 hours / 3.5 days** without a useful evidence-bearing result. Fetch failure, repeated low-value output and failure to survive deep reading all count against it.

This applies to seed links too. Nothing in the web/onion active registry is immortal.

Deleted seed links are recorded in the bounded GC audit so the bootstrap code does not silently recreate them every run. If the researcher later rediscovers the same URL through a genuinely useful path, it may return as a learned probation source.

Explicit user exclusions remain tombstones and are not autonomously resurrected.

## Standard web/onion record

All active web/onion links use the same public shape:

```text
id
kind
url
label
topic
seed
tier
score
hits
observations
failures
origin
firstSeen
lastSeen
lastUseful
```

Inspect them with:

```bash
vesper-hermes-automations links
```

## Candidate budget policy

A normal source candidate set should roughly reserve:

- **45%** initial seed/anchor candidates
- **30%** learned dynamic sources
- **25%** query-tail/new-source exploration

Unused quota is redistributed if a pool is blocked or empty. Within each pool, use source-diverse selection so a single prolific source cannot monopolize the budget.

## Anti-echo-chamber rules

- never optimize exploration to zero;
- never treat a mirror or Tor transport as independent corroboration;
- remove sources that consume budget without producing useful findings;
- allow better learned sources to replace original seeds;
- keep cross-platform paths: Reddit -> X -> GitHub -> web/onion -> docs/paper and the reverse;
- preserve contradictory sources when they materially challenge a finding.
