# Deterministic pipelines, evidence funnels and catalogs

Use this reference when an agent repeatedly collects data, ranks candidates, produces analytics, transforms source material, manages a corpus or runs a high-volume production funnel.

## deterministic collection before model judgment

The cheapest and most reliable model call is the one not made.

Use this shape whenever the first stage can be expressed deterministically:

```text
script / API client
  -> normalize + validate + immutable snapshot
  -> cheap mechanical compression when semantics are actually needed
  -> strong judgment / synthesis
```

Do not spend model calls on HTTP requests, JSON parsing, arithmetic, unit/date conversion, sorting, filtering, file enumeration or stable identity mapping.

Collection must distinguish:

```text
missing != empty != zero != stale != invalid
```

Never turn a missing record into zero just to keep the pipeline moving. Record failures and gaps explicitly. An incomplete report may still be useful, but the incompleteness belongs at the top of the report rather than in a footnote.

Timestamp immutable raw snapshots when historical comparison matters. Keep enough raw evidence to reproduce a later judgment, with a retention policy so operational state does not grow forever.

## mechanic -> judgment handoff

A mechanical/grunt stage prepares evidence; it does not make the final editorial or architectural decision.

Prefer a structured handoff:

```json
{
  "source": "what was inspected",
  "scan": {
    "inspected": 0,
    "retained": 0,
    "excludedByReason": {}
  },
  "findings": [
    {
      "id": "stable-id",
      "summary": "one sentence",
      "evidence": [],
      "source": "file-or-url",
      "confidence": "high|medium|low"
    }
  ],
  "uncertainties": []
}
```

Every exclusion needs a reason. Otherwise a bad filter can erase the most valuable candidates without leaving evidence that it did so.

Put an upper bound on the handoff. If a mechanical stage passes hundreds of undifferentiated items to the judgment model, the separation has failed.

An empty handoff stops the judgment stage. Do not ask a downstream model to create a story when upstream evidence is empty.

Keep the judgment call in a fresh enough context that it is not merely continuing the mechanical stage's assumptions.

## analytics: state -> change -> anomaly -> judgment

A useful metric report answers four different questions:

```text
snapshot     what is true now?
history      what changed?
anomaly      is the change trustworthy/normal?
judgment     what should we do, and which evidence supports that advice?
```

Do not mix materially different products/populations into one total just because the provider exposes one aggregate field. Segment by real product behavior rather than trusting a convenient platform flag when the flag is known to be imperfect.

Do not delete dirty historical observations. Mark them as known anomalies and exclude them from decision calculations when justified. Upstream services can correct live data later; an archived snapshot can freeze a transient upstream error forever.

Every recommendation should point to measured evidence. Unobservable competitor/private metrics remain `null`/unknown, not estimates disguised as facts.

## catalogs and durable corpus identity

Use immutable primary identifiers for durable records. Titles, labels and filenames that humans edit are metadata, not identity.

A robust corpus keeps:

```text
machine-readable index keyed by stable ID
raw/original source
processed/clean representation
human-readable evidence/decision note
history of mutable titles/labels when needed
```

Keep raw and processed forms separate so processing logic can be changed without destroying provenance.

Audit catalogs periodically for:

- orphan: artifact exists without canonical record
- missing: canonical record exists without expected artifact
- content mismatch: artifact exists under the right ID but represents the wrong source

Content mismatch is the most dangerous because everything looks present. Use hashes, metadata signatures and semantic spot-checks to find suspicious cases.

Large binary/rebuildable artifacts stay outside Git when practical. Version compact metadata, manifests, previews, provenance and decisions.

## research and competitor funnels

Separate discovery from verification and judgment.

For competitive/domain research a useful generic shape is:

```text
discovery
-> public/observable metric verification
-> qualitative/creative/technical matrix
-> synthesis: clusters, gaps, own weaknesses, action
```

Record the collection date for volatile metrics. Refresh different layers at frequencies appropriate to how quickly they change and compare against the previous snapshot; the diff is often more useful than the new absolute state.

A good research report is allowed to criticize the operator's own system. If it only explains why competitors/sources are weak, it is advocacy rather than analysis.

## evidence-bound outbound and lead funnels

When a pipeline infers a need from public signals, every transition in the funnel remains auditable:

```text
candidate
-> exclude/retain with reason
-> evidence-backed qualification
-> counterargument / biggest risk
-> fit to a real capability or past work
-> draft strategy/message
-> governance gate before send
```

A factual claim in an outbound message needs a source that actually supports that claim. Verify quote/source matching when practical. Low-confidence facts belong in notes, not in external copy.

A decision not to contact/send also gets a reason; silent exclusion prevents later calibration.

## source-bound transformation

When transforming a trusted source into another format, the model is a translator, not an author of new facts.

```text
source corpus
-> candidate claims/angles, each anchored to source evidence
-> choose
-> transform format
-> independent QA against source
-> governed draft/output
```

If the source does not support a factual claim, do not add it. Keep source ID/hash and output hash so duplicates and provenance can be audited.

The correctness of the source corpus is the ceiling of the entire pipeline. A beautiful output from a mismatched source is still wrong.

## visual/creative identity pipelines

Consistency needs more than prose prompting.

Use:

- canonical subject/identity reference when the model supports it
- canonical style/composition reference when relevant
- explicit `must` traits
- explicit `never`/negative traits learned from real drift
- deterministic output checks for dimensions, file integrity and duplicates
- semantic visual QA only for properties code cannot test

A `never` list is often more auditable than an adjective-heavy style description because each violation can become a concrete QA finding.

For batches, a contact sheet/grid is a high-value diagnostic for spotting drift that is hard to see one item at a time. It is a diagnostic aid, not a requirement for manual approval of every Vesper adaptive-icon conversion.

Text inside generated images should be avoided or deterministically overlaid when reliable typography matters. If generated text is unavoidable, test the actual language and constrain known failure modes rather than assuming English behavior generalizes.

Different output channels may need different visual contracts. Do not blindly inherit a thumbnail rule into a social card, application icon or another product surface.

## grouping/shelf systems

A durable category is a place future items can enter, not a catchy summary of the current items.

Each grouping should define:

```text
name
intent
inclusion criteria
explicit exclusions
ordered membership when order carries meaning
ordering logic
```

If a new item fits two groups equally well, the criteria are not distinct enough.

When navigation matters, maintain one short hand-curated `start here` group rather than expecting a new user/agent to infer an entry path from the whole corpus.

## no-news behavior

A scheduled funnel that has no meaningful finding should usually stay quiet, while still recording enough local state to distinguish `nothing happened` from `the pipeline silently broke`.

Silence is an output policy. Liveness/freshness is a separate reliability signal.
