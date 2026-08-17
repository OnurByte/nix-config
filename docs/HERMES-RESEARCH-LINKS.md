# Hermes research link registry

Vesper's web/onion research lane keeps active links in the same runtime registry used by the adaptive research system.

## Initial seed links

The bootstrap set currently contains:

1. `https://opbible7nans45sg33cbyeiwqmlp5fu7lklu6jd6f3mivrjeqadco5yd.onion/opsec/`
   - label: `OP Bible OPSEC`
   - topic: `privacy-opsec`
   - seed: `true`
2. `https://monero.forum/`
   - label: `Monero Forum`
   - topic: `monero-privacy`
   - seed: `true`

These are seeds, not permanent providers. The researcher can add new links and can remove either seed if runtime quality is poor.

## Standard active record

Every active web/onion source is exposed with exactly these public fields:

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

Inspect the live active set:

```bash
vesper-hermes-automations links
```

Include retired records when present:

```bash
vesper-hermes-automations links --all
```

Run link GC explicitly before listing:

```bash
vesper-hermes-automations links --prune
```

The normal web scout runs the same GC before intake.

## Learning

Outgoing links discovered from useful seed/learned pages may enter the registry as `probation` sources. Discovery alone gives no useful-hit credit.

A source moves toward `trusted` / `promoted` only when later research from that source survives deep reading and contributes evidence-bearing value.

A deleted URL may be learned again later if it is rediscovered through a useful route. It returns as a learned probation source, not as a privileged seed.

## Deletion policy

Web/onion source deletion is deliberately aggressive enough to keep the graph clean.

Default threshold:

- poor for `84` hours / `3.5` days;
- at least `3` observations;
- no recent useful evidence-bearing output, or persistent fetch failure.

This applies to both seed and learned links.

Deletion events are retained in a bounded audit file:

```text
~/.local/state/vesper/research/unknown-frontier-ai/web/link-gc.json
```

The audit also prevents a deleted built-in seed from being recreated automatically on the next process start. A genuinely useful rediscovery can still restore it as a learned source.

Explicit user-excluded sources are the exception: they remain tombstones so autonomous discovery does not override the user's direct preference.

## Tor fetching

`.onion` URLs use the machine's local Tor SOCKS endpoint and remote hostname resolution:

```text
127.0.0.1:9050
curl --socks5-hostname ...
```

Clearnet sources use normal HTTP(S). Tor is transport, not independent corroboration.

Manual fetch/debug:

```bash
vesper-hermes-automations tor-fetch 'https://exampleonion.onion/path/'
```
