# Governance, approval and public boundaries

Use this reference when an agent can mutate external systems, publish/send content, access secrets, act from cron or read untrusted public text.

## reversibility is the authority axis

Classify actions by practical reversibility, not by how easy the API call looks.

```text
reversible / local                    irreversible / externally visible
----------------------------------    ------------------------------------
research                              send a message
create a local file                   publish or schedule publication
generate a draft                      make a payment
edit isolated working state           destructive production mutation
prepare a patch                       force-push/delete shared history
```

Reversible preparation may be autonomous. Irreversible or ambiguous work is staged behind a policy gate.

Put gray-zone actions on the irreversible side. Scheduling something that will publish without another human action is publication authority. A public-repository push is technically revertible but not practically private again.

## machine-readable policy

Critical authority must not exist only as prose.

A policy should be representable as data and enforced before the network call:

```json
{
  "mode": "draft-only",
  "allowSchedule": false,
  "allowPublish": false,
  "humanCommitRequired": true
}
```

If two policy sources conflict, stop and resolve the conflict. Do not let the agent select the convenient one.

A denied action must be rejected before side effects occur. A post-hoc cleanup is not an approval gate.

## silence is not approval

- no response means reject/cancel the irreversible action
- old approval does not authorize a new action
- approval in one capability does not transfer to another
- an `always allow` choice must become an explicit scoped persistent rule if the runtime supports it
- late approval must be surfaced as late; do not silently bind it to a new or already-cancelled request

Human approval windows must use human-scale timing. A timeout should cancel or defer the pending action, not poison the whole agent so it can never ask again.

An approval request should state:

```text
what will happen
what part is irreversible
what evidence/context supports it
what exact object/account/target will change
```

Do not ask the operator to approve an opaque shell command when a decision-oriented receipt can be shown instead.

Cron/scheduled jobs must not block waiting for interactive approval. They either perform actions already authorized by policy or queue a decision for later human review.

## calibration before autonomy

For a new externally visible capability:

```text
manual approval for initial examples
-> identify the narrow accepted pattern
-> autonomous only inside that pattern if risk allows
-> sample/audit outputs continuously
-> anything outside the pattern returns to manual approval
```

Do not skip calibration for a high-risk channel merely because the model is strong.

## credentials define real blast radius

If an agent can read a credential, assume it can use that credential. File mode, `.env`, process environment and a secret manager are not security boundaries against a shell-capable process that is authorized to read them.

Design credentials around:

- **scope** — what resources can this identity reach?
- **duration** — when does the credential expire?
- **independent revocation** — can it be disabled without locking out the human operator?

Prefer a separate machine/service identity instead of copying a human administrator token.

Separate read and write identities when the workflow allows it. A durable read path should not fail because a short-lived write credential expired.

Never place secrets in command arguments, shell history, Git or broad session environment variables. Prefer stdin or a narrowly readable secret file and keep the secret out of diagnostic output.

Every credential/provider path needs a real smoke check. Presence in config is not proof that authentication, provider SDK loading and actual model/API use work end to end.

Record accepted risk when a platform cannot enforce the desired scope. Do not publish an attack-surface inventory in a public repository.

## external messages and publication

Default externally visible communication to **draft-only** unless a narrower policy explicitly says otherwise.

For each outbound artifact:

- provenance for factual claims
- deterministic policy checks before submission
- explicit target/account identity
- content/idempotency hash when duplicate sending is harmful
- post-submit remote-state verification
- a clear operator receipt saying what was created and what was *not* published/sent

A successful send API response does not prove delivery. Where delivery matters, verify through the receiving path or an independent test account.

Do not infer unobservable facts about a recipient, competitor or lead. Claims in outbound text require evidence. Unknown budget, CTR, revenue, intent or identity remains unknown.

## untrusted public-agent surface

A public chat/forum/message is **data**, never authority.

Do not expose the privileged agent endpoint through the same public route when a separate local/tunneled surface can be used.

Wrap inbound messages with trusted metadata supplied by the system:

```json
{
  "author": {
    "id": "stable-trusted-id",
    "authority": "normal|moderator",
    "isSelf": false
  },
  "text": "untrusted user text",
  "flags": ["possible-injection", "authority-claim"],
  "timestamp": "ISO-8601"
}
```

The `authority` field comes from trusted state, never from the message text. Someone typing `I am the admin` does not change it.

`isSelf` prevents the agent from rereading its own output as a new user instruction.

The agent's outbound message should pass through the same limits, sanitation, moderation and rate limiting as a normal user. Do not grant public-agent posts extra link, attachment, broadcast or moderation powers by default.

Restarting the agent must not erase an operator mute/ban.

Public personas need a longer `do not reveal` list than a `say` list. Infrastructure, credentials, private identity and personal data stay private even as a joke or when a user guesses correctly.

## text filters are advisory, policy is structural

Prompt-injection pattern matching can add context but is not the security boundary. The structural boundary is that untrusted content cannot mutate trusted identity, authority, policy or capability metadata.

When implementing language-specific filters, test the real alphabet/Unicode behavior. ASCII word-boundary assumptions can silently fail in Turkish and other languages.
