# media contract

## candidate selection

For each event cluster collect every `media.photos[]` and `media.videos[]`
candidate from the hydrated FxTwitter posts. Keep both the post author and
`media.publisher`; an aggregator post is not proof that the aggregator owns the
media.

Score a candidate out of 100:

- event/media semantic fit: 35
- primary provenance quality: 25
- technical quality: 15
- freshness: 10
- unused exact/perceptual identity: 10
- information added beyond text: 5

Hard gates are separate from the score:

- rights `unknown` requires manual review; `prohibited` is rejected
- sensitive, graphic, manipulated or context-uncertain media is not autopilot
- previously published exact or perceptual media is rejected
- a close score (within five points) requires manual choice
- if no safe, relevant candidate passes, use text-only rather than a wrong image

Prefer a video only when it explains the event better than the best photo. A
video's duration is never fabricated, looped or trimmed only for ranking.

## format and download gates

For video choose a direct `mp4`/H.264 entry from FxTwitter `formats[]`, then the
highest compatible resolution and bitrate within the local limit. Reject
m3u8-only or unsupported codecs in v1. Do not transcode.

Conservative v1 video limits:

- 0.5–140 seconds
- <= 512 MiB
- <= 60 FPS
- YUV 4:2:0
- AAC-LC audio when audio exists

For photos accept static JPG, PNG or WebP, up to four files and <= 5 MiB each.
Do not mix photos and video. Validate MIME/magic bytes, not only extensions.

Download only from `https://video.twimg.com` or `https://pbs.twimg.com`.
Stream to `.part`, enforce the byte cap, calculate SHA-256, atomically rename to
`~/.cache/vesper-xpatla/media/<sha256>.<ext>`, and retain the provenance in
SQLite. Cache cleanup happens only after remote publication confirmation.

Video dedup uses FxTwitter media ID, canonical URL path, SHA-256 and sampled
frame fingerprints. Photo dedup uses media ID, SHA-256 and a perceptual image
fingerprint. Do not silently reuse a cache path as proof of identity.

## publishing boundary

The x-use composer receives local file paths only after all gates pass. It must
fail closed on missing media, mixed types, upload timeout or unknown extension.
The browser's click/toast is an action receipt, not remote proof. Reconcile the
new post through FxTwitter before marking it confirmed.
