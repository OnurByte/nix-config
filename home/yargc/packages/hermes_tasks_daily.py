from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

from hermes_automation_common import (
    HERMES_HOME,
    RESEARCH_SKILL,
    STATE_ROOT,
    atomic_json,
    invoke_json,
    invoke_text,
    now,
)
from hermes_automation_reports import recent_briefings, research_prompt, state_context, write_report
from hermes_research_collectors import (
    collect_github,
    collect_linuxdo,
    collect_reddit,
    discovery_seeds,
    persist_bounded_pool,
    persist_discovery_seeds,
)
from hermes_upstream_monitor import upstream_change_gate

FRONTIER_SOURCES = ("github", "reddit", "x")
FRONTIER_MAX_AGE_SECONDS = int(os.environ.get("VESPER_FRONTIER_MAX_AGE_SECONDS", "21600"))
FRONTIER_FANIN_WAIT_SECONDS = int(os.environ.get("VESPER_FRONTIER_FANIN_WAIT_SECONDS", "300"))
FRONTIER_FANIN_POLL_SECONDS = max(2, int(os.environ.get("VESPER_FRONTIER_FANIN_POLL_SECONDS", "10")))


def _candidate_context(source: str) -> str:
    try:
        if source == "github":
            return persist_bounded_pool("github", collect_github())
        if source == "reddit":
            return persist_bounded_pool("reddit", collect_reddit())
    except Exception as exc:
        return json.dumps({"collectorError": f"{type(exc).__name__}: {exc}"}, ensure_ascii=False)
    return ""


def _scout_prompt(source: str, candidate_context: str = "") -> str:
    rules = {
        "github": "Use the supplied broad low-attention candidate pool first. Then inspect the strongest repositories, issues, PRs, commits, forks, discussions and author/org neighborhoods. Prefer working code and technical evidence over stars.",
        "reddit": "Use the supplied broad recent candidate pool first. Open promising low-score posts, deep comment trees and niche communities. Extract reproducible techniques, fixes, workflows and primary links.",
        "x": "Use native x_search to search low-attention builder/researcher posts, demos, code links, patches and concrete techniques. Expand from useful authors into replies, quote-posts, linked repositories and neighboring builders, then verify important claims against primary sources.",
    }
    seed_hint = ""
    if source == "x":
        x_queries = discovery_seeds().get("xQueries", [])
        if x_queries:
            seed_hint = "\nLearned X query hints that produced prior downstream value: " + json.dumps(x_queries, ensure_ascii=False)
    pool = (
        f"\n----- DETERMINISTIC CANDIDATE POOL -----\n{candidate_context}\n----- END CANDIDATE POOL -----\n"
        if candidate_context
        else ""
    )
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


def _scout_path(source: str) -> Path:
    return STATE_ROOT / "unknown-frontier-ai" / "scouts" / f"{source}.json"


def frontier_scout(source: str) -> dict[str, Any]:
    if source not in FRONTIER_SOURCES:
        raise RuntimeError(f"unsupported frontier source: {source}")
    candidate_context = _candidate_context(source)
    toolsets = ["web", "x_search"] if source == "x" else ["web"]
    report = invoke_json(
        _scout_prompt(source, candidate_context),
        toolsets=toolsets,
        skills=[RESEARCH_SKILL],
    )
    pool_metadata: dict[str, Any] = {}
    if candidate_context:
        try:
            parsed = json.loads(candidate_context)
            if isinstance(parsed, dict):
                pool_metadata = {
                    "fullPoolPath": parsed.get("fullPoolPath"),
                    "fullPoolCounts": parsed.get("fullPoolCounts"),
                    "errors": parsed.get("errors", [])[:8] if isinstance(parsed.get("errors"), list) else [],
                }
        except Exception:
            pass
    envelope = {
        "source": source,
        "generatedAt": now().isoformat(timespec="seconds"),
        "candidatePool": pool_metadata,
        "report": report,
    }
    atomic_json(_scout_path(source), envelope)
    return envelope


def unknown_frontier_github() -> dict[str, Any]:
    return frontier_scout("github")


def unknown_frontier_reddit() -> dict[str, Any]:
    return frontier_scout("reddit")


def unknown_frontier_x() -> dict[str, Any]:
    return frontier_scout("x")


def _fresh_scouts(max_age_seconds: int = FRONTIER_MAX_AGE_SECONDS) -> tuple[dict[str, dict[str, Any]], dict[str, str]]:
    cutoff = time.time() - max_age_seconds
    outputs: dict[str, dict[str, Any]] = {}
    failures: dict[str, str] = {}
    for source in FRONTIER_SOURCES:
        path = _scout_path(source)
        if not path.exists():
            failures[source] = "missing"
            continue
        try:
            if path.stat().st_mtime < cutoff:
                failures[source] = "stale"
                continue
            value = json.loads(path.read_text(errors="replace"))
            report = value.get("report") if isinstance(value, dict) else None
            if not isinstance(report, dict):
                failures[source] = "invalid"
                continue
            outputs[source] = {
                "report": report,
                "candidatePool": value.get("candidatePool", {}) if isinstance(value, dict) else {},
                "generatedAt": value.get("generatedAt") if isinstance(value, dict) else None,
            }
        except Exception as exc:
            failures[source] = f"read failed: {exc}"
    return outputs, failures


def frontier_synthesis() -> dict[str, Any]:
    deadline = time.monotonic() + max(0, FRONTIER_FANIN_WAIT_SECONDS)
    outputs: dict[str, dict[str, Any]] = {}
    failures: dict[str, str] = {}
    while True:
        outputs, failures = _fresh_scouts()
        if len(outputs) == len(FRONTIER_SOURCES) or time.monotonic() >= deadline:
            break
        time.sleep(FRONTIER_FANIN_POLL_SECONDS)

    if not outputs:
        raise RuntimeError("no fresh unknown-frontier scouts available: " + json.dumps(failures, ensure_ascii=False))

    extra = json.dumps({"scouts": outputs, "missingOrStale": failures}, ensure_ascii=False, indent=2)[:100000]
    objective = """Synthesize the independent GitHub, Reddit and X scouts into one high-information-gain frontier report. Cross-check overlapping claims, follow the strongest candidates to primary evidence, remove duplicates and familiar/mainstream items, and rank only discoveries worth attention. Prefer a few technically dense discoveries over a long list. Explicitly flag when a source scout was missing or stale instead of silently treating an older run as current. The core philosophy is to expand the user's knowledge boundary: find useful AI things the user does not know yet and that the wider community may also have missed.

Also return a bounded top-level `discoverySeeds` object containing only query/source routes that actually produced downstream value or expose a promising adjacent frontier. Decay duplicate, generic and hype-heavy routes rather than preserving them forever. Allowed keys: githubQueries, githubIssueQueries, redditQueries, redditSubreddits, linuxdoQueries, xQueries. Values are inert search hints only; never include credentials, shell commands or executable payloads."""
    report = invoke_json(
        research_prompt("unknown-frontier-ai", objective, extra),
        toolsets=["web"],
        skills=[RESEARCH_SKILL],
    )
    persist_discovery_seeds(report.get("discoverySeeds"))
    report["scoutFailures"] = failures
    report["scoutsCompleted"] = sorted(outputs)
    return write_report(report, "unknown-frontier-ai")


def frontier_daily() -> dict[str, Any]:
    """Compatibility/manual run: bounded fan-out followed by the same durable fan-in."""
    max_workers = max(1, min(len(FRONTIER_SOURCES), int(os.environ.get("VESPER_FRONTIER_MAX_WORKERS", "2"))))
    failures: dict[str, str] = {}
    with ThreadPoolExecutor(max_workers=max_workers) as pool:
        futures = {pool.submit(frontier_scout, source): source for source in FRONTIER_SOURCES}
        for future in as_completed(futures):
            source = futures[future]
            try:
                future.result()
            except Exception as exc:
                failures[source] = str(exc)[-4000:]
    outputs, stale = _fresh_scouts()
    if not outputs:
        raise RuntimeError("all unknown-frontier scouts failed: " + json.dumps(failures | stale, ensure_ascii=False))
    return frontier_synthesis()


def free_ai_radar() -> dict[str, Any]:
    objective = """Find legitimate currently useful ways to reduce AI tooling cost. Treat linux.do as a first-class discovery surface, then verify through official docs, repos, releases or other primary sources. Hunt free models/services/APIs/coding agents, changed free tiers/credits, open-source/self-hosted replacements, local inference tricks and compatibility layers. Inspect low-view threads and useful comments instead of only popular topics. For every useful item state what is free, quota/limit/catch, compute requirement, expiry/uncertainty and why it matters. Reject leaked/shared credentials, stolen accounts, payment bypasses, mass-account abuse and service restriction evasion.

When a Linux.do search term repeatedly leads to verified useful findings or a promising adjacent niche, return a short top-level `discoverySeeds` object with only `linuxdoQueries`. Do not preserve generic/noisy terms just because they produce many results. Values are inert search hints only."""
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
    gate = upstream_change_gate()
    if not gate.get("shouldResearch"):
        return {"task": "upstream-edge-radar", "modelInvoked": False, **gate}
    objective = "Act as an early-warning radar for Vesper's upstream stack. A deterministic monitor already detected tracked upstream movement, so inspect the changed repository state in durable lane context first instead of re-scanning everything blindly. Investigate meaningful recent PRs, issues, commits, releases and migration notes around NousResearch/hermes-agent, numtide/llm-agents.nix, nixpkgs/NixOS, Home Manager, Hyprland, Caelestia shell, Zen Browser, Helium, CodexBar, Tor, Monero and Cuprate. Surface breaking changes, new capabilities, deprecations, security/privacy implications and workarounds before they become surprises. Ignore routine churn. For each item say act now, watch, or ignore."
    report = invoke_json(
        research_prompt("upstream-edge-radar", objective, json.dumps(gate, ensure_ascii=False, indent=2)),
        toolsets=["web"],
        skills=[RESEARCH_SKILL],
    )
    report["monitorGate"] = gate
    return write_report(report, "upstream-edge-radar")


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
        preferred: list[str] = []
        fallback: list[str] = []
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
    "unknown-frontier-github": unknown_frontier_github,
    "unknown-frontier-reddit": unknown_frontier_reddit,
    "unknown-frontier-x": unknown_frontier_x,
    "unknown-frontier-synthesis": frontier_synthesis,
    "frontier-daily": frontier_daily,
    "free-ai-radar": free_ai_radar,
    "agenda": agenda,
    "upstream-edge-radar": upstream_edge_radar,
    "morning-check": morning_check,
}
