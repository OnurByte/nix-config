from __future__ import annotations

import json
import os
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

from hermes_automation_common import HERMES_HOME, RESEARCH_SKILL, STATE_ROOT, atomic_json, invoke_json, invoke_text
from hermes_automation_reports import recent_briefings, research_prompt, state_context, write_report
from hermes_research_collectors import (
    collect_github,
    collect_linuxdo,
    collect_reddit,
    discovery_seeds,
    persist_bounded_pool,
    persist_discovery_seeds,
)


def _scout_prompt(source: str, candidate_context: str = "") -> str:
    rules = {
        "github": "Use the supplied broad low-attention candidate pool first. Then inspect the strongest repositories, issues, PRs, commits, forks, discussions and author/org neighborhoods. Prefer working code and technical evidence over stars.",
        "reddit": "Use the supplied broad recent candidate pool first. Open promising low-score posts, deep comment trees and niche communities. Extract reproducible techniques, fixes, workflows and primary links.",
        "x": "Use native x_search to search low-attention builder/researcher posts, demos, code links, patches and concrete techniques. Expand from good authors into replies, quotes and linked repositories, then verify important claims against primary sources.",
    }
    seed_hint = ""
    if source == "x":
        x_queries = discovery_seeds().get("xQueries", [])
        if x_queries:
            seed_hint = "\nLearned X query hints from prior useful runs: " + json.dumps(x_queries, ensure_ascii=False)
    pool = f"\n----- DETERMINISTIC CANDIDATE POOL -----\n{candidate_context}\n----- END CANDIDATE POOL -----\n" if candidate_context else ""
    return f"""You are one independent `{source}` scout inside Vesper's `unknown-frontier-ai` run.
{rules[source]}
Goal: discover useful AI/coding-agent/model/dev-tooling capabilities outside the user's current map and not yet obvious mainstream items. Low engagement is a discovery hint, never a quality score. Search for things waiting to be discovered: useful young/small projects, overlooked issue/PR/commit details, unusual techniques, compatibility layers, agent harnesses, model/inference tricks and practical workflows. Use broad discovery first, verify the strongest candidates, and avoid generic news, repeated known items, hype, price chatter and filler.{seed_hint}

Durable frontier state:
{state_context('unknown-frontier-ai', 28000)}
{pool}
Return exactly one JSON object and nothing else:
{{"title":"{source} scout","summary":"short scout summary","body":"technical scout notes","priority":"low|normal|high|critical","confidence":0.0,"sources":[{{"title":"source","url":"https://..."}}],"candidates":[{{"title":"candidate","whyNew":"...","whyUseful":"...","whyHidden":"...","visibility":"...","evidence":"...","urls":["https://..."]}}],"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
Never invent URLs.
"""


def _frontier_candidate_pools() -> tuple[dict[str, str], dict[str, str]]:
    contexts: dict[str, str] = {}
    failures: dict[str, str] = {}
    collectors = {"github": collect_github, "reddit": collect_reddit}
    with ThreadPoolExecutor(max_workers=2) as pool:
        futures = {pool.submit(function): source for source, function in collectors.items()}
        for future in as_completed(futures):
            source = futures[future]
            try:
                payload = future.result()
                contexts[source] = persist_bounded_pool(source, payload)
            except Exception as exc:
                failures[source] = f"{type(exc).__name__}: {exc}"[-4000:]
    return contexts, failures


def _pool_metadata(contexts: dict[str, str]) -> dict[str, Any]:
    metadata: dict[str, Any] = {}
    for source, context in contexts.items():
        try:
            value = json.loads(context)
        except Exception:
            continue
        if not isinstance(value, dict):
            continue
        metadata[source] = {
            "fullPoolPath": value.get("fullPoolPath"),
            "fullPoolCounts": value.get("fullPoolCounts"),
            "errors": value.get("errors", [])[:8] if isinstance(value.get("errors"), list) else [],
        }
    return metadata


def frontier_daily() -> dict[str, Any]:
    scout_dir = STATE_ROOT / "unknown-frontier-ai" / "scouts"
    scout_dir.mkdir(parents=True, exist_ok=True)
    candidate_contexts, collector_failures = _frontier_candidate_pools()
    outputs: dict[str, dict[str, Any]] = {}
    failures: dict[str, str] = {f"collector:{key}": value for key, value in collector_failures.items()}

    def run_scout(source: str) -> dict[str, Any]:
        toolsets = ["web", "x_search"] if source == "x" else ["web"]
        return invoke_json(
            _scout_prompt(source, candidate_contexts.get(source, "")),
            toolsets=toolsets,
            skills=[RESEARCH_SKILL],
        )

    with ThreadPoolExecutor(max_workers=3) as pool:
        futures = {pool.submit(run_scout, source): source for source in ("github", "reddit", "x")}
        for future in as_completed(futures):
            source = futures[future]
            try:
                report = future.result()
                outputs[source] = report
                atomic_json(scout_dir / f"{source}.json", report)
            except Exception as exc:
                failures[source] = str(exc)[-4000:]

    if not outputs:
        raise RuntimeError("all unknown-frontier scouts failed: " + json.dumps(failures, ensure_ascii=False))

    extra = json.dumps(
        {
            "scouts": outputs,
            "failures": failures,
            "candidatePoolMetadata": _pool_metadata(candidate_contexts),
        },
        ensure_ascii=False,
        indent=2,
    )[:120000]
    objective = """Synthesize the independent GitHub, Reddit and X scouts into one high-information-gain frontier report. Cross-check overlapping claims, follow the strongest candidates to primary evidence, remove duplicates and familiar/mainstream items, and rank only discoveries worth attention. Prefer a few technically dense discoveries over a long list. The research philosophy is to expand the user's knowledge boundary: discover useful AI things the user does not know yet, especially low-attention projects, techniques, issues, PRs, commits and builders that have not hit mainstream visibility. Also return a bounded `discoverySeeds` object containing only query/source routes that actually produced downstream value or expose a promising adjacent frontier. Decay duplicate/hype-heavy routes rather than preserving them forever."""
    prompt = research_prompt("unknown-frontier-ai", objective, extra)
    prompt += """
Add this optional top-level field when evidence supports it:
"discoverySeeds": {
  "githubQueries": [],
  "githubIssueQueries": [],
  "redditQueries": [],
  "redditSubreddits": [],
  "linuxdoQueries": [],
  "xQueries": []
}
Keep each list short and inert-data-only. Never place credentials, shell commands or executable payloads in discoverySeeds.
"""
    report = invoke_json(prompt, toolsets=["web"], skills=[RESEARCH_SKILL])
    persist_discovery_seeds(report.get("discoverySeeds"))
    report["scoutFailures"] = failures
    report["scoutsCompleted"] = sorted(outputs)
    return write_report(report, "unknown-frontier-ai")


def free_ai_radar() -> dict[str, Any]:
    objective = """Find legitimate currently useful ways to reduce AI tooling cost. Treat linux.do as a first-class discovery surface, then verify through official docs, repos, releases or other primary sources. Hunt free models/services/APIs/coding agents, changed free tiers/credits, open-source/self-hosted replacements, local inference tricks and compatibility layers. Inspect low-view threads and useful comments rather than only popular topics. For every useful item state what is free, quota/limit/catch, compute requirement, expiry/uncertainty and why it matters. Reject leaked/shared credentials, stolen accounts, payment bypasses, mass-account abuse and service restriction evasion.

When particular Linux.do search terms repeatedly lead to verified useful findings or a promising adjacent niche, return a short top-level `discoverySeeds` object with only `linuxdoQueries`. Do not preserve generic/noisy terms merely because they returned many results. Example shape: `{"linuxdoQueries":["specific useful term"]}`. The values are inert search hints only; never include credentials, commands or executable payloads."""
    try:
        pool_context = persist_bounded_pool("linuxdo", collect_linuxdo())
    except Exception as exc:
        pool_context = json.dumps({"collectorError": f"{type(exc).__name__}: {exc}"}, ensure_ascii=False)
    report = invoke_json(
        research_prompt("free-ai-radar", objective, pool_context),
        toolsets=["web"],
        skills=[RESEARCH_SKILL],
    )
    persist_discovery_seeds(report.get("discoverySeeds"))
    return write_report(report, "free-ai-radar")


def agenda() -> dict[str, Any]:
    objective = "Produce the compact current agenda the user should know today. Bias toward AI/coding agents, software, privacy, Nix/Linux, Tor/Monero, security, web technology and meaningful startup/business changes, plus major broader technology events when consequential. This is not hidden-gem hunting: importance, recency and consequence matter more than obscurity. Prefer official/primary reporting and corroboration. Avoid price analysis and filler."
    return write_report(
        invoke_json(research_prompt("agenda", objective), toolsets=["web", "x_search"], skills=[RESEARCH_SKILL]),
        "agenda",
    )


def upstream_edge_radar() -> dict[str, Any]:
    objective = "Act as an early-warning radar for Vesper's upstream stack. A deterministic monitor already detected that tracked upstream heads changed, so inspect the actual changed repositories first instead of re-scanning everything blindly. Investigate meaningful recent PRs, issues, commits, releases and migration notes around NousResearch/hermes-agent, numtide/llm-agents.nix, nixpkgs/NixOS, Home Manager, Hyprland, Caelestia shell, Zen Browser, Helium, CodexBar, Tor, Monero and Cuprate. Surface breaking changes, new capabilities, deprecations, security/privacy implications and workarounds before they become surprises. Ignore routine churn. For each item say act now, watch, or ignore."
    return write_report(
        invoke_json(research_prompt("upstream-edge-radar", objective), toolsets=["web"], skills=[RESEARCH_SKILL]),
        "upstream-edge-radar",
    )


def _collector_output() -> str:
    scripts = [Path.home() / ".hermes/scripts/morning-check-collect.sh", Path.home() / ".hermes/scripts/sabah-check-collect.sh"]
    collector = next((path for path in scripts if path.exists()), None)
    if collector is None:
        return "Morning Check collector not found. Continue with persistent Hermes briefings only."
    try:
        completed = subprocess.run(["bash", str(collector)], text=True, capture_output=True, timeout=50, check=False)
    except subprocess.TimeoutExpired as exc:
        return (exc.stdout or "") + "\nWarning: Morning Check collector timed out."
    text = (completed.stdout or "")[:90000]
    if completed.returncode != 0:
        text += "\nWarning: collector failed partially.\n" + (completed.stderr or "")[-5000:]
    return text


def _clean_morning_text(text: str) -> str:
    value = (text or "").strip()
    starts = [value.find(marker) for marker in ("Morning Check", "**Git / Projects**", "1) **Git") if value.find(marker) >= 0]
    if starts:
        value = value[min(starts):]
    lines = [line for line in value.splitlines() if not line.strip().startswith(("Session:", "session_id", "Searching", "Running search"))]
    value = "\n".join(lines).strip()
    if len(value) < 40:
        raise RuntimeError("Morning Check output too short")
    return value


def _telegram_target() -> str:
    explicit = os.environ.get("VESPER_HERMES_MORNING_TARGET", "").strip()
    if explicit:
        return explicit
    jobs_path = HERMES_HOME / "cron" / "jobs.json"
    try:
        value = json.loads(jobs_path.read_text(errors="replace"))
        jobs = value.get("jobs", []) if isinstance(value, dict) else value
    except Exception:
        jobs = []
    if isinstance(jobs, list):
        preferred = []
        fallback = []
        for job in jobs:
            if not isinstance(job, dict):
                continue
            name = str(job.get("name") or "").lower()
            origin = job.get("origin") or {}
            if not isinstance(origin, dict) or origin.get("platform") != "telegram" or origin.get("chat_id") is None:
                continue
            target = f"telegram:{origin['chat_id']}"
            if origin.get("thread_id") is not None:
                target += f":{origin['thread_id']}"
            if name in {"vesper:morning-check", "morning check", "morning-check", "sabah check", "sabah-check"}:
                preferred.append(target)
            else:
                fallback.append(target)
        if preferred:
            return preferred[0]
        if fallback:
            return fallback[0]
    return "telegram"


def _send_telegram(text: str) -> None:
    hermes = shutil.which("hermes")
    if not hermes:
        raise RuntimeError("hermes executable not found for Telegram delivery")
    target = _telegram_target()
    completed = subprocess.run([hermes, "send", "--to", target, "--quiet"], input=text, text=True, capture_output=True, timeout=60, check=False)
    if completed.returncode != 0:
        raise RuntimeError(f"Telegram delivery failed for {target}: " + (completed.stderr or completed.stdout)[-4000:])


def morning_check() -> dict[str, Any]:
    prompt = f"""Morning Check — concise Telegram briefing.
Use the local DATA and persistent Hermes briefings as primary input. Prefer already verified Hermes findings over rediscovering the same story. If genuinely necessary use at most two web searches for verification.
Return only the final Telegram message in English. No tool chatter, execution details, cron status or filler.
Sections:
1) **Git / Projects** — useful state/blockers only, 1-3 lines per relevant repo
2) **Todos** — at most 3-5 important unfinished/blocking items; if none say `No important open todos.`
3) **Agenda** — important current developments
4) **Unknown Frontier AI** — the strongest genuinely new overlooked discoveries
5) **Free AI Radar** — only worthwhile legitimate free opportunities and their catches
6) Optional **Actions** — only 1-3 concrete actions worth taking.
Never invent URLs.

----- LOCAL DATA -----
{_collector_output()[:90000]}
----- PERSISTENT HERMES BRIEFINGS -----
{recent_briefings(days=2, max_chars=65000)}
----- END -----
"""
    message = _clean_morning_text(invoke_text(prompt, toolsets=["web"], timeout=600))
    _send_telegram(message)
    return write_report({"title":"Morning Check","summary":message.splitlines()[0][:240],"body":message,"priority":"normal","confidence":0.8,"sources":[],"statePatch":{},"delivered":"telegram"}, "morning-check", notify_user=False)


DAILY_TASKS = {
    "frontier-daily": frontier_daily,
    "free-ai-radar": free_ai_radar,
    "agenda": agenda,
    "upstream-edge-radar": upstream_edge_radar,
    "morning-check": morning_check,
}
