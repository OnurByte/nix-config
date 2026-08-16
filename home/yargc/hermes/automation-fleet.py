#!/usr/bin/env python3
"""Reconcile Vesper's desired Hermes cron fleet without editing jobs.json by hand.

Run through the Nix-provided `vesper-hermes-cron-sync` wrapper. Dry-run is the
default. `--apply` uses Hermes' own cron.jobs mutation API so schedule parsing,
locking, snapshots, ownership, and derived metadata stay canonical.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path
from typing import Any

try:
    from cron.jobs import create_job, list_jobs, update_job
except Exception as exc:  # pragma: no cover - runtime diagnostics
    raise SystemExit(
        "Could not import Hermes cron modules. Run this through "
        "vesper-hermes-cron-sync so it can select Hermes' Python runtime. "
        f"Import error: {exc}"
    )

HOME = Path.home()
HERMES_HOME = Path(os.environ.get("HERMES_HOME", HOME / ".hermes")).expanduser()

DEFAULT_PROVIDER = os.environ.get("VESPER_HERMES_CRON_PROVIDER") or None
DEFAULT_MODEL = os.environ.get("VESPER_HERMES_CRON_MODEL") or None

RESEARCH_SKILL = "hermes-research-radar"
SECOND_BRAIN_SKILL = "vesper-obsidian-second-brain"


def agent_defaults() -> dict[str, Any]:
    return {"provider": DEFAULT_PROVIDER, "model": DEFAULT_MODEL, "no_agent": False}


def jobs() -> list[dict[str, Any]]:
    a = agent_defaults()
    return [
        {
            "name": "Unknown Frontier AI — GitHub Scout",
            "schedule": "20 8 * * *",
            "deliver": "local",
            "script": "frontier-github-collect.py",
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["web", "terminal", "file"],
            **a,
            "prompt": """You are the GitHub scout for the Unknown Frontier AI lane. The pre-run script gives you a large low-attention candidate pool of young repositories and recently active issues. Do not summarize the pool. Triage it aggressively for useful AI discoveries that have not broken through yet: agent harnesses, coding agents, MCP tooling, inference tricks, wrappers, compatibility layers, research utilities, unusual implementations, overlooked issues/PRs/commits, and small projects that solve real problems. Prefer low visibility plus working evidence, not obscurity alone. Deeply verify the strongest candidates with repository pages, code, README, issues, PRs, commits, release history, or primary author material. Return 5-12 findings maximum. For each include URL, visibility context, what is actually new/useful, evidence, confidence, and why it may have been missed. Explicitly mark weak leads as unresolved instead of promoting them.""",
        },
        {
            "name": "Unknown Frontier AI — Reddit Scout",
            "schedule": "30 8 * * *",
            "deliver": "local",
            "script": "frontier-reddit-collect.py",
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["web", "file"],
            **a,
            "prompt": """You are the Reddit scout for the Unknown Frontier AI lane. The script output is a broad recent candidate pool. Search below the obvious top posts: low-score posts, technical comments, niche subreddits, reproducible workflows, benchmarks, fixes, prompts, agent techniques, local inference tricks, and links to obscure projects. Open promising threads and comment trees; follow external links to primary sources when needed. Reject unsupported hype. Return 5-12 high-information-gain findings maximum with Reddit URL, primary source when available, engagement context, what is new/useful, evidence, confidence, and why it may have been missed.""",
        },
        {
            "name": "Unknown Frontier AI — X Scout",
            "schedule": "40 8 * * *",
            "deliver": "local",
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["x_search", "web"],
            **a,
            "prompt": """You are the X/Twitter scout for the Unknown Frontier AI lane. Search recent low-engagement technical posts from builders, researchers, maintainers, small labs, and tool authors. Hunt for code, demos, patches, benchmarks, unusual techniques, agent workflows, wrappers, model/inference findings, and early capabilities that have not reached mainstream AI discussion. Do not optimize for viral posts. Expand from good authors into replies, quoted posts, linked repositories, and neighboring accounts. Verify strong claims through primary code/docs when possible. Return 5-12 findings maximum with post URL, engagement context, primary source, what is new/useful, evidence, confidence, and why it may have been missed.""",
        },
        {
            "name": "Free AI Radar",
            "schedule": "55 8 * * *",
            "deliver": "local",
            "script": "free-ai-linuxdo-collect.py",
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["web", "browser", "terminal"],
            **a,
            "prompt": """Run the Free AI Radar with Linux.do as a first-class source. The script provides a wide latest/search candidate pool. Find legitimate, genuinely useful free AI access: free tiers, official credits/promotions, free model endpoints, coding agents, APIs, open-source/self-hosted replacements, local workflows, bridges, wrappers, compatibility layers, and low-hit GitHub projects. Inspect low-view threads and useful comments, then verify outward against the original repository, release, docs, author, or provider. Exclude leaked/shared credentials, stolen keys, account theft, payment bypass, abusive mass-account creation, or service-restriction evasion. Return only worthwhile findings. For each state exactly what is free, quota/limit/catch, self-hosting cost if any, expiration/uncertainty, why it is useful, confidence, Linux.do source, and primary source.""",
        },
        {
            "name": "Unknown Frontier AI — Synthesis",
            "schedule": "5 9 * * *",
            "deliver": "local",
            "context_from": [
                "Unknown Frontier AI — GitHub Scout",
                "Unknown Frontier AI — Reddit Scout",
                "Unknown Frontier AI — X Scout",
            ],
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["web", "file"],
            **a,
            "prompt": """Synthesize today's three independent Unknown Frontier AI scout outputs. Cross-deduplicate the same project/claim across sources and reward independent corroboration. Select the 5-10 discoveries with the highest information gain, utility, novelty, evidence, technical density, and early-signal value. Do not add mainstream AI news merely to fill space. For every selected discovery provide: title, concise explanation, source links, whyNew, whyUseful, whyHidden, visibility context, confidence, and one concrete follow-up if it deserves deeper research. End with 1-3 newly discovered source paths/authors/communities worth exploring in future runs.""",
        },
        {
            "name": "Daily Agenda",
            "schedule": "20 9 * * *",
            "deliver": "local",
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["web", "x_search"],
            **a,
            "prompt": """Produce the Daily Agenda. This is not a hidden-gem task. Find the important current developments the user should know today, ranked by importance, recency, consequence, and relevance. Bias toward AI/coding, software, privacy, security, Nix/Linux, Tor/onion, Monero/Zcash technology, web/privacy-tech, startups/business, and major technology developments. Exclude investment price/TA coverage, coupons, and filler. Verify consequential claims with primary or high-quality independent sources. Return 10-15 concise items with title, one-sentence event summary, date/context when useful, and URL.""",
        },
        {
            "name": "Morning Check",
            "aliases": ["Sabah check"],
            "schedule": "0 10 * * *",
            "deliver": "notify",
            "script_candidates": ["morning-check-collect.sh", "sabah-check-collect.sh"],
            "context_from": ["Daily Agenda", "Unknown Frontier AI — Synthesis", "Free AI Radar"],
            "enabled_toolsets": ["web", "file"],
            **a,
            "prompt": """Morning Check — final daily briefing. Script output, when present, contains local Git/project/todo data. The preceding cron outputs contain independent Daily Agenda, Unknown Frontier AI synthesis, and Free AI Radar reports. Combine them without collapsing their purposes. Write in English. Keep it compact enough for Telegram while preserving the strongest links. Sections: **Git / Projects**, **Todos** (3-5 important maximum), **Agenda** (important mainstream/current developments), **Unknown Frontier AI** (useful overlooked AI discoveries), **Free AI Radar** (only worthwhile legitimate opportunities with catches), and optional **Actions** (1-3 concrete actions). Do not re-run broad research unless a supplied claim needs quick verification. Do not invent information or URLs.""",
        },
        {
            "name": "Upstream Edge Radar",
            "schedule": "15 19 * * *",
            "deliver": "notify",
            "script": "upstream-edge-monitor.py",
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["web", "terminal"],
            **a,
            "prompt": """The pre-run gate has detected changed upstream repository heads and supplied the before/after snapshot. Investigate only the changed areas plus any directly related Tor/privacy or Codex/Claude/OpenCode developments needed to understand Vesper impact. Focus on recently merged or active PRs, issues, commits, deprecations, breaking changes, new capabilities, fixes, and workarounds in Hermes Agent, llm-agents.nix, nixpkgs/NixOS, Home Manager, Hyprland, Caelestia, Zen/Helium integrations, Cuprate/Monero and adjacent agent tooling. Report only material changes with evidence-backed upstream links and suggested Vesper impact. If the changed head is routine/noise and nothing is meaningfully actionable or informative, respond with exactly [SILENT].""",
        },
        {
            "name": "Second Brain Reflection",
            "schedule": "30 23 * * *",
            "deliver": "local",
            "skills": ["obsidian", SECOND_BRAIN_SKILL],
            "enabled_toolsets": ["file", "terminal"],
            **a,
            "prompt": """Run the nightly second-brain reflection. Read today's durable Vesper briefing/research outputs and existing relevant Obsidian notes. Consolidate only durable knowledge: new facts, useful project/tool relationships, corrected assumptions, source-quality lessons, unresolved questions, and repeated research procedures. Update existing notes instead of duplicating them. Use Obsidian for the long knowledge graph and keep Hermes memory small; cron sessions themselves do not receive built-in memory. Stage reusable procedures as skill drafts rather than silently promoting active skills. Create a Dream/reflection note only when meaningful synthesis exists. If there is no meaningful synthesis, return exactly [SILENT].""",
        },
        {
            "name": "Vesper Health Watch",
            "schedule": "every 3h",
            "deliver": "notify",
            "script": "vesper-health-watch.py",
            "skills": [],
            "enabled_toolsets": None,
            "provider": None,
            "model": None,
            "no_agent": True,
            "prompt": "",
        },
        {
            "name": "Hermes Skill Integrity Watch",
            "schedule": "50 7 * * *",
            "deliver": "notify",
            "script": "vesper-skill-integrity-watch.py",
            "skills": [],
            "enabled_toolsets": None,
            "provider": None,
            "model": None,
            "no_agent": True,
            "prompt": "",
        },
        {
            "name": "Hermes Cron Retention",
            "schedule": "15 3 * * 1",
            "deliver": "notify",
            "script": "vesper-cron-retention.py",
            "skills": [],
            "enabled_toolsets": None,
            "provider": None,
            "model": None,
            "no_agent": True,
            "prompt": "",
        },
        {
            "name": "User Pain Miner",
            "schedule": "30 10 * * 0",
            "deliver": "local",
            "skills": [RESEARCH_SKILL],
            "enabled_toolsets": ["web", "terminal"],
            **a,
            "prompt": """Mine recurring user pain across the AI/developer ecosystems relevant to Vesper: Hermes, Codex, Claude Code, OpenCode, NixOS/Home Manager, Hyprland/Caelestia, privacy tooling, and adjacent agent infrastructure. Look for the same friction appearing independently in issues, discussions, Reddit, forums, and technical posts. Cluster by root problem rather than keyword. For each strong cluster provide: problem, evidence from multiple sources, recurrence level, existing workarounds, why current solutions are insufficient, and one better solution/product/skill/automation opportunity. Prefer problems that are real, repeated, technically tractable, and under-served.""",
        },
        {
            "name": "Project Archaeologist",
            "schedule": "40 10 * * 0",
            "deliver": "local",
            "script": "project-inventory.py",
            "skills": [],
            "enabled_toolsets": ["terminal", "file"],
            **a,
            "prompt": """Analyze the supplied local project inventory as a project archaeologist. Identify forgotten but valuable work: dirty repositories, stale branches, unfinished TODO/PROGRESS/PLAN files, projects with recent momentum that stopped abruptly, risky uncommitted work, and experiments worth reviving or archiving. Do not produce a generic repo list. Return the 3-8 highest-value findings with evidence, recommended next action, and whether to continue, archive, clean up, or investigate.""",
        },
        {
            "name": "AI Usage Economist",
            "schedule": "50 10 * * 0",
            "deliver": "local",
            "script": "ai-usage-snapshot.py",
            "skills": [],
            "enabled_toolsets": ["file"],
            **a,
            "prompt": """Analyze the supplied weekly Codex/Claude/agent usage snapshot. Find waste, concentration, quota/reset risk, expensive repetitive work, and places where a cheaper/faster model or deterministic script could replace premium-agent turns without degrading outcomes. Separate missing/uncertain accounting from real evidence. Return concise findings, the largest optimization opportunities, and 1-3 concrete workflow changes.""",
        },
        {
            "name": "Skill Evolution Review",
            "schedule": "0 11 * * 0",
            "deliver": "local",
            "skills": [SECOND_BRAIN_SKILL],
            "enabled_toolsets": ["file", "terminal"],
            **a,
            "prompt": """Review Vesper research state, skill drafts, recent briefings, and existing active skills. Identify repeated procedures/heuristics that have enough evidence to become a skill candidate, active heuristics that are decaying or too broad, duplicate skills, and missing reusable procedures. Do not automatically promote or mutate active skills. Return a review queue with evidence, trial count when available, scope, confidence, suggested skill name, and recommendation: retain, narrow, merge, draft, or retire. Curator owns generic skill lifecycle maintenance; this job only evaluates research-derived procedural learning.""",
        },
        {
            "name": "Weekly Intelligence Review",
            "schedule": "0 12 * * 0",
            "deliver": "notify",
            "context_from": [
                "User Pain Miner",
                "Project Archaeologist",
                "AI Usage Economist",
                "Skill Evolution Review",
                "Upstream Edge Radar",
            ],
            "skills": [],
            "enabled_toolsets": ["file"],
            **a,
            "prompt": """Create the Weekly Intelligence Review from the supplied weekly outputs. Do not repeat every item. Select the highest-leverage problems, projects, upstream changes, cost optimizations, and skill-learning opportunities. Organize as: **What changed**, **What is worth building/fixing**, **Projects to act on**, **Agent/AI efficiency**, **Skills to evolve**, and **Top 3 actions for next week**. Preserve source links from upstream reports. English, concise, decision-oriented.""",
        },
    ]


def delivery_target(existing: list[dict[str, Any]]) -> str:
    explicit = os.environ.get("VESPER_HERMES_DELIVER")
    if explicit:
        return explicit

    preferred_names = {"morning check", "sabah check"}
    ordered = sorted(
        existing,
        key=lambda job: 0 if str(job.get("name", "")).lower() in preferred_names else 1,
    )
    for job in ordered:
        origin = job.get("origin") or {}
        platform = origin.get("platform")
        chat_id = origin.get("chat_id")
        if platform and chat_id:
            thread_id = origin.get("thread_id")
            target = f"{platform}:{chat_id}"
            if thread_id is not None:
                target += f":{thread_id}"
            return target
        raw = job.get("deliver")
        values = [raw] if isinstance(raw, str) else list(raw or [])
        for value in values:
            if value and value not in {"origin", "local"}:
                return value
    return "local"


def existing_by_name(existing: list[dict[str, Any]]) -> dict[str, list[dict[str, Any]]]:
    result: dict[str, list[dict[str, Any]]] = {}
    for job in existing:
        result.setdefault(str(job.get("name", "")).lower(), []).append(job)
    return result


def choose_existing(spec: dict[str, Any], by_name: dict[str, list[dict[str, Any]]]) -> dict[str, Any] | None:
    candidates = [spec["name"], *spec.get("aliases", [])]
    for candidate in candidates:
        matches = by_name.get(candidate.lower(), [])
        if len(matches) > 1:
            ids = ", ".join(str(job.get("id")) for job in matches)
            raise RuntimeError(f"Ambiguous cron job name {candidate!r}: {ids}")
        if matches:
            return matches[0]
    return None


def resolve_script(spec: dict[str, Any]) -> str | None:
    candidates = []
    if spec.get("script"):
        candidates.append(spec["script"])
    candidates.extend(spec.get("script_candidates", []))
    if not candidates:
        return None
    for name in candidates:
        if (HERMES_HOME / "scripts" / name).exists():
            return name
    # Nix-managed scripts may not exist yet during a dry-run before nh os switch.
    return candidates[0]


def desired_fields(spec: dict[str, Any], notify_target: str) -> dict[str, Any]:
    deliver = notify_target if spec.get("deliver") == "notify" else spec.get("deliver", "local")
    return {
        "name": spec["name"],
        "prompt": spec.get("prompt", ""),
        "skills": list(spec.get("skills") or []),
        "model": spec.get("model"),
        "provider": spec.get("provider"),
        "script": resolve_script(spec),
        "no_agent": bool(spec.get("no_agent", False)),
        "enabled_toolsets": spec.get("enabled_toolsets"),
        "deliver": deliver,
    }


def diff_job(job: dict[str, Any], spec: dict[str, Any], notify_target: str) -> dict[str, Any]:
    desired = desired_fields(spec, notify_target)
    updates: dict[str, Any] = {}
    current_schedule = str(job.get("schedule_display") or "")
    if current_schedule != spec["schedule"]:
        updates["schedule"] = spec["schedule"]
    for key, value in desired.items():
        current = job.get(key)
        if key == "skills":
            current = list(current or [])
        if current != value:
            updates[key] = value
    return updates


def reconcile(apply: bool) -> int:
    existing = list_jobs(include_disabled=True)
    notify_target = delivery_target(existing)
    by_name = existing_by_name(existing)
    specs = jobs()
    planned = 0
    resolved: dict[str, dict[str, Any]] = {}

    if notify_target == "local":
        print(
            "WARNING: no existing messaging origin/target was found. "
            "Notification jobs will use local delivery until VESPER_HERMES_DELIVER is set."
        )
    else:
        print(f"Notification target: {notify_target}")

    # Pass 1: create/update every job without context_from. This gives all
    # upstream jobs stable IDs before fan-in edges are written.
    for spec in specs:
        current = choose_existing(spec, by_name)
        if current is None:
            planned += 1
            print(f"CREATE  {spec['name']}  [{spec['schedule']}]")
            if apply:
                fields = desired_fields(spec, notify_target)
                created = create_job(
                    prompt=fields["prompt"],
                    schedule=spec["schedule"],
                    name=fields["name"],
                    deliver=fields["deliver"],
                    skills=fields["skills"],
                    model=fields["model"],
                    provider=fields["provider"],
                    script=fields["script"],
                    enabled_toolsets=fields["enabled_toolsets"],
                    no_agent=fields["no_agent"],
                )
                current = created
                by_name.setdefault(spec["name"].lower(), []).append(created)
            else:
                # Synthetic placeholder; context edges are displayed in dry-run.
                current = {"id": f"dry-{len(resolved):08d}", "name": spec["name"]}
        else:
            updates = diff_job(current, spec, notify_target)
            if updates:
                planned += 1
                print(f"UPDATE  {current.get('name')} -> {spec['name']}: {', '.join(updates)}")
                if apply:
                    current = update_job(current["id"], updates) or current
            else:
                print(f"OK      {spec['name']}")
        resolved[spec["name"]] = current

    # Pass 2: wire fan-in edges using canonical job IDs. Hermes accepts names in
    # its user-facing API, but canonical IDs make the stored graph unambiguous.
    for spec in specs:
        source_names = spec.get("context_from") or []
        if not source_names:
            continue
        current = resolved[spec["name"]]
        source_ids = [resolved[name]["id"] for name in source_names if name in resolved]
        if not apply:
            print(f"CHAIN   {spec['name']} <- {', '.join(source_names)}")
            continue
        refreshed = next(
            (job for job in list_jobs(include_disabled=True) if job.get("id") == current.get("id")),
            current,
        )
        if list(refreshed.get("context_from") or []) != source_ids:
            planned += 1
            print(f"CHAIN   {spec['name']} <- {', '.join(source_names)}")
            update_job(refreshed["id"], {"context_from": source_ids})

    action = "Applied" if apply else "Planned"
    print(f"{action} reconciliation: {planned} change(s).")
    if not apply:
        print("Run again with --apply after `nh os switch` to mutate Hermes cron state.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Reconcile Vesper Hermes cron jobs")
    parser.add_argument("--apply", action="store_true", help="apply changes; default is dry-run")
    args = parser.parse_args()
    return reconcile(args.apply)


if __name__ == "__main__":
    sys.exit(main())
