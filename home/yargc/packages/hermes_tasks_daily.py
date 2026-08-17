from __future__ import annotations

import json
import os
import shutil
import subprocess
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, invoke_json, invoke_text, now
from hermes_automation_reports import recent_briefings, research_prompt, research_skill_context, state_context, write_report
from hermes_research_intake import compact_intake, reddit_rss_intake, x_mirror_intake

FRONTIER_SOURCES = ("github", "reddit", "x")
FRONTIER_MAX_AGE_SECONDS = int(os.environ.get("VESPER_FRONTIER_MAX_AGE_SECONDS", "21600"))
FRONTIER_FANIN_WAIT_SECONDS = int(os.environ.get("VESPER_FRONTIER_FANIN_WAIT_SECONDS", "300"))
FRONTIER_FANIN_POLL_SECONDS = max(2, int(os.environ.get("VESPER_FRONTIER_FANIN_POLL_SECONDS", "10")))
FRONTIER_TOTAL_CANDIDATE_TARGET = max(200, min(1000, int(os.environ.get("VESPER_FRONTIER_CANDIDATE_TARGET", "600"))))
FRONTIER_TOTAL_DEEP_READ_TARGET = max(24, min(60, int(os.environ.get("VESPER_FRONTIER_DEEP_READ_TARGET", "48"))))


def _budget_map(total: int) -> dict[str, int]:
    github = max(1, round(total * 0.34))
    reddit = max(1, round(total * 0.36))
    x = max(1, total - github - reddit)
    return {"github": github, "reddit": reddit, "x": x}


FRONTIER_CANDIDATE_BUDGET = _budget_map(FRONTIER_TOTAL_CANDIDATE_TARGET)
FRONTIER_DEEP_READ_BUDGET = _budget_map(FRONTIER_TOTAL_DEEP_READ_TARGET)


def _scout_prompt(source: str, intake: dict[str, Any] | None = None) -> str:
    rules = {
        "github": "Search recent/small repositories, issues, PRs, commits, forks, discussions, package/release surfaces and author/org neighborhoods. Prefer working code and technical evidence over stars.",
        "reddit": "Use the supplied RSS/Atom intake as the cheap first pass, then search/fetch only where it adds coverage or verification. Inspect recent/low-score posts, comment branches and niche communities. Extract reproducible techniques, fixes, workflows and primary links.",
        "x": "X/Twitter is mandatory. Use direct X when accessible; otherwise use XCancel and configured Nitter-compatible mirrors, with HTML/search fallback when RSS is blocked. Search low-attention builder/researcher posts, replies/quotes, demos, code links, patches and concrete techniques. Verify important claims against primary sources.",
    }
    references = ["research-pipeline.md", "source-governance.md"]
    if source == "reddit":
        references.append("reddit-rss.md")
    if source == "x":
        references.append("x-research.md")
    skill = research_skill_context(references, max_chars=38000)
    intake_text = compact_intake(intake, max_chars=90000) if intake else "No deterministic intake was supplied; build broad intake with the web tools and track it honestly."
    candidate_target = FRONTIER_CANDIDATE_BUDGET[source]
    deep_target = FRONTIER_DEEP_READ_BUDGET[source]
    return f"""You are one independent `{source}` scout inside Vesper's `unknown-frontier-ai` run.

{rules[source]}

Goal: discover useful AI/coding-agent/model/dev-tooling capabilities outside the user's current map and not yet obvious mainstream items. Low engagement is a discovery hint, never a quality score. Use broad discovery first, verify the strongest candidates, and avoid generic news, repeated known items, hype, price chatter and filler.

Coverage contract for this scout:
- candidate target: about {candidate_target} distinct canonical items/URLs
- deep-read target: about {deep_target} strongest items
- the full daily bundle target is {FRONTIER_TOTAL_CANDIDATE_TARGET} candidates and {FRONTIER_TOTAL_DEEP_READ_TARGET} deep reads across GitHub + Reddit + X
- deterministic RSS/mirror intake counts as cheap candidate inspection, not as a full deep read
- if access failures prevent the target, report the actual count and limitation; never invent coverage

----- RESEARCH PROCEDURE -----
{skill}
----- END PROCEDURE -----
----- DURABLE FRONTIER STATE -----
{state_context('unknown-frontier-ai', 28000)}
----- END DURABLE STATE -----
----- CHEAP INTAKE -----
{intake_text}
----- END CHEAP INTAKE -----

Return exactly one JSON object and nothing else:
{{"title":"{source} scout","summary":"short scout summary","body":"technical scout notes","priority":"low|normal|high|critical","confidence":0.0,"sources":[{{"title":"source","url":"https://..."}}],"candidates":[{{"title":"candidate","whyNew":"...","whyUseful":"...","evidence":"...","urls":["https://..."]}}],"coverage":{{"candidateTarget":{candidate_target},"candidatesInspected":0,"canonicalCandidates":0,"deepReads":0,"primaryVerifications":0,"surfaces":[],"limitations":[]}},"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
Never invent URLs or coverage numbers.
"""


def _scout_path(source: str) -> Path:
    return STATE_ROOT / "unknown-frontier-ai" / "scouts" / f"{source}.json"


def _source_intake(source: str) -> dict[str, Any] | None:
    target = FRONTIER_CANDIDATE_BUDGET[source]
    try:
        if source == "reddit":
            return reddit_rss_intake(target)
        if source == "x":
            return x_mirror_intake(target)
    except Exception as exc:
        return {
            "source": f"{source}-intake",
            "generatedAt": now().isoformat(timespec="seconds"),
            "target": target,
            "canonicalCandidates": 0,
            "errors": [{"error": str(exc)[-2000:]}],
            "candidates": [],
        }
    return None


def frontier_scout(source: str) -> dict[str, Any]:
    if source not in FRONTIER_SOURCES:
        raise RuntimeError(f"unsupported frontier source: {source}")
    intake = _source_intake(source)
    report = invoke_json(_scout_prompt(source, intake), web_only=True)
    coverage = report.get("coverage")
    if not isinstance(coverage, dict):
        coverage = {}
        report["coverage"] = coverage
    coverage["candidateTarget"] = FRONTIER_CANDIDATE_BUDGET[source]
    if intake:
        coverage["intakeCandidates"] = int(intake.get("canonicalCandidates") or 0)
        coverage["intakeErrors"] = len(intake.get("errors") or [])
    envelope = {
        "source": source,
        "generatedAt": now().isoformat(timespec="seconds"),
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
            outputs[source] = report
        except Exception as exc:
            failures[source] = f"read failed: {exc}"
    return outputs, failures


def _coverage_number(report: dict[str, Any], key: str) -> int:
    coverage = report.get("coverage") or {}
    if not isinstance(coverage, dict):
        return 0
    try:
        return max(0, int(coverage.get(key) or 0))
    except (TypeError, ValueError):
        return 0


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

    candidate_total = sum(_coverage_number(report, "candidatesInspected") or _coverage_number(report, "intakeCandidates") for report in outputs.values())
    canonical_total = sum(_coverage_number(report, "canonicalCandidates") for report in outputs.values())
    deep_total = sum(_coverage_number(report, "deepReads") for report in outputs.values())
    primary_total = sum(_coverage_number(report, "primaryVerifications") for report in outputs.values())
    coverage_summary = {
        "target": FRONTIER_TOTAL_CANDIDATE_TARGET,
        "candidatesInspected": candidate_total,
        "canonicalCandidates": canonical_total,
        "deepReads": deep_total,
        "primaryVerifications": primary_total,
        "shortfall": max(0, FRONTIER_TOTAL_CANDIDATE_TARGET - candidate_total),
    }
    extra = json.dumps({"scouts": outputs, "missingOrStale": failures, "coverage": coverage_summary}, ensure_ascii=False, indent=2)[:110000]
    report = invoke_json(
        research_prompt(
            "unknown-frontier-ai",
            "Synthesize the independent GitHub, Reddit and X scouts into one high-information-gain frontier report. Cross-check overlapping claims, follow the strongest candidates to primary evidence, remove duplicates and familiar/mainstream items, and rank only discoveries worth attention. Prefer a few technically dense discoveries over a long list. Run a counter-review before final selection. X is a mandatory discovery surface when its scout is available. Explicitly flag missing/stale scouts and any coverage shortfall rather than silently treating old or inaccessible data as current.",
            extra,
            skill_references=("research-pipeline.md", "source-governance.md", "x-research.md", "reddit-rss.md"),
        ),
        web_only=True,
    )
    report["scoutFailures"] = failures
    report["scoutsCompleted"] = sorted(outputs)
    report["coverage"] = coverage_summary | {
        "sourcesCompleted": sorted(outputs),
        "limitations": [f"{source}: {reason}" for source, reason in failures.items()],
    }
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
    objective = "Find legitimate currently useful ways to reduce AI tooling cost. Treat linux.do as a first-class discovery surface, then verify through official docs, repos, releases or other primary sources. Hunt free models/services/APIs/coding agents, changed free tiers/credits, open-source/self-hosted replacements, local inference tricks and compatibility layers. For every useful item state what is free, quota/limit/catch, compute requirement, expiry/uncertainty and why it matters. Reject leaked/shared credentials, stolen accounts, payment bypasses, mass-account abuse and service restriction evasion."
    return write_report(invoke_json(research_prompt("free-ai-radar", objective), web_only=True), "free-ai-radar")


def agenda() -> dict[str, Any]:
    objective = "Produce the compact current agenda the user should know today. Bias toward AI/coding agents, software, privacy, Nix/Linux, Tor/Monero, security, web technology and meaningful startup/business changes, plus major broader technology events when consequential. This is not hidden-gem hunting: importance, recency and consequence matter more than obscurity. Prefer official/primary reporting and corroboration. Avoid price analysis and filler."
    return write_report(invoke_json(research_prompt("agenda", objective), web_only=True), "agenda")


def upstream_edge_radar() -> dict[str, Any]:
    objective = "Act as an early-warning radar for Vesper's upstream stack. Inspect meaningful recent PRs, issues, commits, releases and migration notes around NousResearch/hermes-agent, numtide/llm-agents.nix, nixpkgs/NixOS, Hyprland, Caelestia shell, Zen Browser, Helium, Tor, Monero and Cuprate. Surface breaking changes, new capabilities, deprecations, security/privacy implications and workarounds before they become surprises. Ignore routine churn. For each item say act now, watch, or ignore."
    return write_report(invoke_json(research_prompt("upstream-edge-radar", objective), web_only=True), "upstream-edge-radar")


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


def _send_telegram(text: str) -> None:
    hermes = shutil.which("hermes")
    if not hermes:
        raise RuntimeError("hermes executable not found for Telegram delivery")
    completed = subprocess.run([hermes, "send", "--to", "telegram", "--quiet"], input=text, text=True, capture_output=True, timeout=60, check=False)
    if completed.returncode != 0:
        raise RuntimeError("Telegram delivery failed: " + (completed.stderr or completed.stdout)[-4000:])


def morning_check() -> dict[str, Any]:
    prompt = f"""Morning Check — concise Telegram briefing.
Use the local DATA and persistent Hermes briefings as primary input. Prefer already verified Hermes findings over rediscovering the same story. If genuinely necessary use at most two web searches for verification.
Return only the final Telegram message in English. No tool chatter, execution details, cron status or filler.
Sections:
1) **Git / Projects** — useful state/blockers only, 1-3 lines per relevant repo
2) **Todos** — at most 3-5 important unfinished/blocking items; if none say `No important open todos.`
3) **News** — 10-15 genuinely important items when available; privacy, payments, Monero/Zcash (no price talk), Tor/onion, AI/coding agents/dev tooling, security/development/web privacy/startups/major tech. Each item: bold numbered title, one concise sentence, then URL. Never invent URLs.
4) Optional **Actions** — only 1-3 concrete actions worth taking.

----- LOCAL DATA -----
{_collector_output()[:90000]}
----- PERSISTENT HERMES BRIEFINGS -----
{recent_briefings(days=2, max_chars=50000)}
----- END -----
"""
    message = _clean_morning_text(invoke_text(prompt, web_only=True, timeout=600))
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
