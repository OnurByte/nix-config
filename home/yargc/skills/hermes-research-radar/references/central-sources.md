# Central Research Sources

Central sources are durable anchors, not an allowlist.

They receive guaranteed inspection budget because they map directly to the user's highest-value domains. Autonomous discovery remains mandatory and can promote better sources, but learned sources do not silently replace protected anchors.

## Interest map

The research center of gravity is:

1. vibe coding / coding agents / agent harnesses / skills / MCP / context engineering;
2. Monero / privacy / private payments / Tor / private communications;
3. Nix/Linux/security/open-source infrastructure when it improves those workflows or the workstation.

Generic local-model/inference discussion is not a standing interest. `r/LocalLLaMA` is explicitly excluded from the default research graph.

## Reddit anchors

### Vibe coding / agentic development

- `r/vibecoding` — practical AI-built software, workflows, failures and emerging patterns
- `r/ClaudeCode` — Claude Code workflows, subagents, skills, context and real-repo usage
- `r/codex` — Codex CLI/IDE/cloud workflows, failures, usage and workarounds
- `r/opencodeCLI` — OpenCode workflows, providers, integrations and community tooling
- `r/cursor` — agentic editor workflows, context/repo practices and product-edge techniques

### Monero / privacy

- `r/MoneroMeansMoney` — Monero/privacy culture, ecosystem edges and niche discussion
- `r/Monero` — protocol/ecosystem/community developments
- `r/privacy` — privacy software, policy and practical privacy changes
- `r/Tor` — Tor/onion/privacy operational developments
- `r/netsec` — technical security research and tooling

### Workstation / infrastructure

- `r/NixOS` — Nix/NixOS ecosystem and workstation-relevant changes

Anchor feeds should not be concatenated and blindly truncated. Select candidates with a bounded anchor quota and source-diverse round-robin so prolific communities cannot consume the entire Reddit budget.

## X / Twitter anchors

The initial X anchor graph is selected from the user's followed accounts for technical signal rather than raw engagement.

### Coding agents / developer tooling

- `@Teknium` — Hermes Agent / NousResearch
- `@thdxr` — OpenCode / developer tooling
- `@XOpenSource` — X open-source engineering
- `@ZixuanLi_` — Z.ai / AI engineering

### Monero / privacy / decentralized payments

- `@eigenwallet` — Monero-Bitcoin atomic-swap DEX
- `@kyc_rip` — no-KYC swap ecosystem signals
- `@XBToshi` — Bitcoin/Monero privacy tooling ecosystem
- `@schmidt1024` — privacy/open-source/Monero projects
- `@XMRHub_org` — Monero ecosystem discovery
- `@CR1337` — Monero/privacy projects and crypto-agorism
- `@linuxuser1996` — Linux/GrapheneOS/Monero/privacy
- `@Examare1` — Monero ecosystem
- `@ZcashLabs` — Zcash engineering/ecosystem counterpoint

### Security / threat intelligence

- `@akaclandestine` — OSINT, threat research, OPSEC and threat intelligence
- `@DailyDarkWeb` — dark-web threat intelligence

### Private communications

- `@SimpleXChat` — private messaging/network architecture

Accounts dominated by memes, general trading, price/charts or broad consumer activity are not protected anchors. They remain discoverable and can earn adaptive promotion only through repeated technical value.

## GitHub anchor neighborhoods

GitHub frontier research should always know where to begin, then expand through issues, PRs, commits, forks, authors and dependency neighborhoods.

### Coding-agent anchors

- `NousResearch/hermes-agent`
- `openai/codex`
- `anthropics/claude-code`
- `anomalyco/opencode`

### Monero/privacy anchors

- `monero-project/monero`
- `Cuprate/cuprate`

These are starting neighborhoods, not the final report. The highest-value finding may live in a small adjacent repo, fork, issue, author account or dependency discovered from them.

## Autonomous source evolution

Every non-anchor source starts as `probation`.

Lifecycle:

`probation -> trusted -> promoted -> decay/review -> probation/retired`

Discovery alone does not count as a useful hit. A source earns credit only when a candidate survives deep reading and contributes evidence-bearing value.

The runtime registry lives at:

`~/.local/state/vesper/research/unknown-frontier-ai/source-registry.json`

Track at least:

- source type and canonical identity
- tier and protected status
- score
- useful-hit count
- observation/fetch count
- failure count
- first/last seen
- last useful time
- discovery provenance
- retirement reason when applicable

Protected anchors may accumulate failure/health history but cannot be automatically removed. Explicitly user-excluded sources are retired from the active graph and should not be rediscovered into probation.

## Candidate budget policy

A normal social-source candidate set should roughly reserve:

- **45%** protected anchors
- **30%** trusted/promoted/probation learned sources
- **25%** query-tail/new-source exploration

These are candidate-selection quotas, not hard request-count quotas. If one pool is empty or blocked, redistribute unused budget to the others while recording the failure.

Within each pool, use source-diverse selection. One high-volume subreddit or X account must not dominate merely because it emitted more entries.

## Anti-echo-chamber rules

- never optimize exploration to zero;
- never treat a mirror copy as independent corroboration;
- periodically re-test trusted/promoted sources for freshness and downstream utility;
- retire dead/noisy learned sources conservatively;
- keep cross-platform paths: Reddit -> X -> GitHub -> docs/paper and the reverse;
- preserve contradictory sources when they materially challenge a finding.
