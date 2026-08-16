---
name: hermes-automation-fleet
description: Operate and evolve Vesper's Hermes cron fleet: multi-stage research, cheap collectors, watchdogs, synthesis, second-brain consolidation and safe reconciliation.
platforms: [linux]
metadata:
  hermes:
    tags: [automation, cron, research, obsidian, operations]
---

# Hermes Automation Fleet

Vesper treats Hermes cron as one durable automation system, not as a pile of unrelated timers.

## Core design

Use the right execution mode for the job:

- deterministic polling/health checks -> `no_agent` script jobs
- broad discovery -> cheap script collection first, then a bounded agent triage
- sources that need native Hermes search tools -> dedicated agent scouts
- multi-source reports -> `context_from` fan-in after upstream jobs finish
- long-term knowledge -> Obsidian / Vesper research state, not cron conversation memory
- reusable procedure learning -> skill drafts, then review/promotion

Hermes cron sessions are fresh and do not receive normal built-in memory. Never design a scheduled workflow that depends on the previous cron conversation. Persist state explicitly.

## Daily intelligence pipeline

The daily pipeline intentionally separates objectives:

1. `Unknown Frontier AI — GitHub Scout`
2. `Unknown Frontier AI — Reddit Scout`
3. `Unknown Frontier AI — X Scout`
4. `Free AI Radar`
5. `Unknown Frontier AI — Synthesis`
6. `Daily Agenda`
7. `Morning Check`

Unknown Frontier answers: **what useful AI thing exists outside the current knowledge map?**

Daily Agenda answers: **what important thing happened or changed today?**

Free AI Radar answers: **what legitimate free AI capability is actually usable now, and what is the catch?**

Do not merge those ranking rules. Morning Check may present them together, but each upstream lane must remain independently researched and scored.

## Broad discovery without wasting the agent window

Do not ask one agent turn to browse hundreds of pages sequentially.

Vesper pre-run collectors create large candidate funnels for GitHub, Reddit and Linux.do. The agent spends its limited cron turn on:

- filtering
- opening the strongest candidates
- following important edges
- verification
- ranking
- synthesis

X/Twitter remains a native `x_search` scout because a reliable unauthenticated bulk collector is not assumed.

Collector output is input, never truth. Verify important claims against primary sources.

## Notification policy

Avoid notification spam.

- raw scouts: local output only
- intermediate synthesis: local output only
- Morning Check: notification target
- Weekly Intelligence Review: notification target
- health / skill-integrity watchdogs: notification only on state change
- Upstream Edge Radar: `[SILENT]` when nothing material changed
- second-brain reflection: local and `[SILENT]` when no synthesis exists

The notification target is resolved locally from `VESPER_HERMES_DELIVER` or an existing Morning Check origin. Never commit personal chat IDs into the public Nix repository.

## Weekly intelligence

Weekly jobs look for compounding value rather than more news:

- `User Pain Miner` — repeated under-served problems across relevant ecosystems
- `Project Archaeologist` — forgotten/risky/high-value local project state
- `AI Usage Economist` — premium-agent usage waste and cheaper substitutions
- `Skill Evolution Review` — research-derived procedures that deserve draft/merge/retire decisions
- `Weekly Intelligence Review` — decision-oriented fan-in

Curator still owns generic Hermes skill maintenance. Skill Evolution Review only evaluates research-derived procedural learning.

## Second brain

Use `vesper-obsidian-second-brain` together with Hermes' bundled `obsidian` skill.

Promote only durable knowledge:

- strong discoveries
- useful source relationships
- corrected assumptions
- unresolved research questions
- recurring procedures

Do not dump scrape corpora into Obsidian and do not copy full reports into Hermes memory.

## Operations

The Nix repository is the desired-state source for the Vesper fleet. Hermes' mutable `jobs.json` remains runtime state.

After applying Nix changes:

```bash
nh os switch
vesper-hermes-cron-sync
vesper-hermes-cron-sync --apply
hermes cron list
```

The first sync command is a dry-run. `--apply` reconciles by job name, preserves existing job IDs when possible, migrates the legacy `Sabah check` name to `Morning Check`, and wires `context_from` using canonical upstream job IDs.

Do not patch `~/.hermes/cron/jobs.json` directly.

## Safety and failure behavior

- A failed scout must not prevent unrelated lanes from running.
- Watchdogs must deduplicate unchanged alarms.
- Free AI research must exclude stolen/shared credentials, leaked keys, account theft, abusive mass-account creation, payment bypass, and service-restriction evasion.
- Retention may delete old cron outputs and old cron-source sessions, but never active jobs or current research state.
- Do not create a second scheduler with systemd timers or GitHub Actions for these jobs; Hermes cron is the scheduling owner.
- Keep schedules staggered. `context_from` consumes the latest completed upstream output and does not wait for jobs running in the same scheduler tick.

## Verification

A healthy fleet has:

- exactly one managed job for each desired English job name
- valid attached skills
- no duplicate `Sabah check` / `Morning Check`
- scout jobs delivering locally
- synthesis jobs referencing canonical upstream IDs
- health/integrity jobs running in `no_agent` mode
- Morning Check and weekly review using the resolved notification target
- durable research and second-brain state outside cron conversation memory
