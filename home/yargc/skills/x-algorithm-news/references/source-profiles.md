# source profiles

The active configuration is `~/.config/vesper/xpatla/sources.json`. It has a
top-level `sources` array. The array length is the account count; no code or
skill may assume a fixed number.

Minimal record:

```json
{
  "handle": "bpthaber",
  "enabled": true,
  "maxPosts": 20,
  "rightsStatus": "unknown",
  "profile": {
    "ideology": "mainstream-aggregator",
    "tone": "short-classic-news",
    "topics": ["politics", "current-events"],
    "certainty": "attributed"
  }
}
```

Use the profile to interpret source framing, certainty and media behavior. Do
not copy its ideology, prose or factual claims into the target account voice.
Keep per-source health separate from editorial profile data. An inaccessible
source is `unavailable`, not a zero-engagement source.

The repository ships a seed example with the currently verified Turkish
accounts. Users may add or disable accounts without code changes. FxTwitter
profile timeline requests are bounded by `maxPosts` (1-20 in the current
single-page collector) and use the last persisted created timestamp as `since`
when available. Deeper history is intentionally out of scope for the three-
minute lane.
