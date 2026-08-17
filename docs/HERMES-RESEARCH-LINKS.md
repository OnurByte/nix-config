# Hermes research source registry

Vesper keeps one compact evidence-backed source registry for research URLs:

```text
~/.local/state/vesper/research/unknown-frontier-ai/source-registry.json
```

It is shared by scheduled and ad-hoc research.

## inspect

```bash
vesper-hermes-automations links
vesper-research sources
```

Both commands expose the same current registry.

## record shape

The Rust control plane keeps records deliberately small. Current fields can include:

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

`id` is the registry key added to the public view.

## learning

The registry is evidence-driven rather than feed-volume-driven.

A URL is reinforced only when it survives into a final report's `sources` evidence. Merely appearing in a prompt, candidate list, mirror result or discovery hint gives no hit.

Current positive lifecycle:

```text
first useful evidence -> probation
second useful hit     -> trusted
fourth useful hit     -> promoted
```

The active research skill still requires exploration outside this registry. Promoted sources are useful hints, not an allowlist.

The previous Python implementation had a larger seed/failure/observation GC subsystem. That subsystem is not part of the current Rust API, so commands such as `links --prune` and `links --all` are intentionally not documented as available.

## Tor fetching

Onion pages use the machine's local Tor SOCKS endpoint with remote hostname resolution:

```text
127.0.0.1:9050
```

Manual fetch/debug:

```bash
vesper-hermes-automations tor-fetch 'http://example.onion/path/'
```

The helper validates that the URL authority itself ends in `.onion`; a clearnet URL containing `.onion` only in its path/query is rejected.

Clearnet sources use normal HTTP(S). Tor is transport, not independent corroboration.
