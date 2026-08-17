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
from hermes_research_intake import compact_intake, reddit_rss_intake, reinforce_source_registry, x_mirror_intake
from hermes_research_link_registry import prune_web_links
from hermes_research_web import compact_web_intake, reinforce_web_registry, web_core_intake

FRONTIER_SOURCES = ("github", "reddit", "x", "web")
FRONTIER_MAX_AGE_SECONDS = int(os.environ.get("VESPER_FRONTIER_MAX_AGE_SECONDS", "21600"))
FRONTIER_FANIN_WAIT_SECONDS = int(os.environ.get("VESPER_FRONTIER_FANIN_WAIT_SECONDS", "300"))
FRONTIER_FANIN_POLL_SECONDS = max(2, int(os.environ.get("VESPER_FRONTIER_FANIN_POLL_SECONDS", "10")))
FRONTIER_TOTAL_CANDIDATE_TARGET = max(200, min(1000, int(os.environ.get("VESPER_FRONTIER_CANDIDATE_TARGET", "600"))))
FRONTIER_TOTAL_DEEP_READ_TARGET = max(24, min(60, int(os.environ.get("VESPER_FRONTIER_DEEP_READ_TARGET", "48"))))


def _budget_map(total: int) -> dict[str, int]:
    github = max(1, round(total * 0.30))
    reddit = max(1, round(total * 0.25))
    x = max(1, round(total * 0.25))
    web = max(1, total - github - reddit - x)
    while github + reddit + x + web > total:
        for name in ("github", "reddit", "x", "web"):
            value = {"github": github, "reddit": reddit, "x": x, "web": web}[name]
            if value <= 1:
                continue
            if name == "github":
                github -= 1
            elif name == "reddit":
                reddit -= 1
            elif name == "x":
                x -= 1
            else:
                web -= 1
            if github + reddit + x + web <= total:
                break
    return {"github": github, "reddit": reddit, "x": x, "web": web}


FRONTIER_CANDIDATE_BUDGET = _budget_map(FRONTIER_TOTAL_CANDIDATE_TARGET)
FRONTIER_DEEP_READ_BUDGET = _budget_map(FRONTIER_TOTAL_DEEP_READ_TARGET)


def _scout_prompt(source: str, intake: dict[str, Any] | None = None) -> str:
    rules = {
        "github": "Search recent/small repositories, issues, PRs, commits, forks, discussions, package/release surfaces and author/org neighborhoods. Prioritize coding-agent/vibe-coding workflows and Monero/privacy engineering; prefer working code and technical evidence over stars.",
        "reddit": "Use the supplied RSS/Atom intake as the cheap first pass, then search/fetch only where it adds coverage or verification. Inspect recent/low-score posts, comment branches and niche communities. Extract reproducible coding-agent workflows, Monero/privacy techniques, fixes and primary links.",
        "x": "X/Twitter is mandatory. Use direct X when accessible; otherwise use XCancel and configured Nitter-compatible mirrors, with HTML/search fallback when RSS is blocked. Search low-attention builders/researchers for coding-agent workflows, Monero/privacy engineering, replies/quotes, demos, code, patches and concrete techniques. Verify important claims against primary sources.",
        "web": "Treat protected web/onion anchors as first-class research surfaces. The supplied intake was fetched locally: clearnet through normal HTTP(S), .onion through the machine's Tor SOCKS proxy. For .onion candidates, use the supplied Tor-fetched content as the page fetch result; do not pretend the normal web tool can resolve an onion URL. Follow clearnet links with web tools when useful, and verify important claims against the strongest available primary evidence.",
    }
    references = ["research-pipeline.md", "source-governance.md", "central-sources.md"]
    if source == "reddit":
        references.append("reddit-rss.md")
    if source == "x":
        references.append("x-research.md")
    if source == "web":
        references.append("web-tor.md")
    skill = research_skill_context(references, max_chars=44000)
    if intake and source == "web":
        intake_text = compact_web_intake(intake, max_chars=105000)
    elif intake:
        intake_text = compact_intake(intake, max_chars=90000)
    else:
        intake_text = "No deterministic intake was supplied; build broad intake with the web tools and track it honestly."
    candidate_target = FRONTIER_CANDIDATE_BUDGET[source]
    deep_target = FRONTIER_DEEP_READ_BUDGET[source]
    return f"""You are one independent `{source}` scout inside Vesper's `unknown-frontier-ai` run.

{rules[source]}

Research profile, in priority order:
1. vibe coding / agentic software engineering: Codex, Claude Code, OpenCode, Hermes, harnesses, skills, MCP, context engineering, evals, agent orchestration, practical workflows and overlooked developer tools;
2. Monero/privacy: Monero protocol/ecosystem, Cuprate, wallets, atomic swaps, private payments, Tor, onion services, OPSEC, SimpleX, GrapheneOS/privacy engineering and useful adjacent infrastructure;
3. Nix/Linux, security and open-source developer infrastructure when it improves the workstation or the two priorities above.

Generic local-LLM/model-quantization/inference hobby content is not a target. Only surface model/inference material when it materially improves coding-agent workflows, privacy, cost, or deployment. Do not spend frontier budget on price charts, trading chatter, generic AI news or familiar mainstream releases without a genuinely useful technical angle.

Goal: find high-information-gain capabilities, techniques, tools and changes outside the user's current map. Low engagement is a discovery hint, never a quality score. Use broad discovery first, verify the strongest candidates, and avoid repeated known items, hype and filler.

Coverage contract for this scout:
- candidate target: about {candidate_target} distinct canonical items/URLs
- deep-read target: about {deep_target} strongest items
- the full daily bundle target is {FRONTIER_TOTAL_CANDIDATE_TARGET} candidates and {FRONTIER_TOTAL_DEEP_READ_TARGET} deep reads across GitHub + Reddit + X + web/onion
- deterministic RSS/mirror/Tor intake counts as cheap candidate inspection; only pages whose substantive content is actually opened count as deep reads
- central anchors are mandatory inspection seeds but are not an allowlist
- learned sources may receive future budget only after repeated useful evidence-bearing results
- if access failures prevent the target, report the actual count and limitation; never invent coverage
- a Tor transport or mirror is access infrastructure, not independent corroboration

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
{{"title":"{source} scout","summary":"short scout summary","body":"technical scout notes","priority":"low|normal|high|critical","confidence":0.0,"sources":[{{"title":"source","url":"https://..."}}],"candidates":[{{"title":"candidate","topic":"vibe-coding|monero-privacy|privacy-opsec|nix-linux|security|other","whyNew":"...","whyUseful":"...","evidence":"...","urls":["https://..."]}}],"coverage":{{"candidateTarget":{candidate_target},"candidatesInspected":0,"canonicalCandidates":0,"deepReads":0,"primaryVerifications":0,"surfaces":[],"limitations":[]}},"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
Never invent URLs, page contents or coverage numbers.
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
        if source == "web":
            prune_web_links()
            return web_core_intake(target, deep_fetch_limit=FRONTIER_DEEP_READ_BUDGET[source])
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
        if isinstance(intake.get("budget"), dict):
            coverage["intakeBudget"] = intake["budget"]
        if source == "web":
            coverage["onionPrefetched"] = int(intake.get("onionPrefetched") or 0)
    if source == "web":
        reinforce_web_registry(report)
    else:
        reinforce_source_registry(source, report)
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


def unknown_frontier_web() -> dict[str, Any]:
    return frontier_scout("web")


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
    extra = json.dumps({"scouts": outputs, "missingOrStale": failures, "coverage": coverage_summary}, ensure_ascii=False, indent=2)[:125000]
    report = invoke_json(
        research_prompt(
            "unknown-frontier-ai",
            "Synthesize the independent GitHub, Reddit, X and web/onion scouts into one high-information-gain frontier report. Rank vibe-coding/agentic software-engineering findings and Monero/privacy/OPSEC findings first, with Nix/Linux/security as secondary. Cross-check overlapping claims, follow the strongest candidates to primary evidence, remove duplicates and familiar/mainstream items, and run a counter-review before final selection. Generic local-model/inference content is out of scope unless directly useful to those priorities. X and protected web/onion anchors are mandatory discovery surfaces when their scouts are available. Onion content supplied by the local Tor intake is evidence from that page, but Tor itself is transport rather than corroboration. Explicitly flag missing/stale scouts and any coverage shortfall.",
            extra,
            skill_references=("research-pipeline.md", "source-governance.md", "central-sources.md", "x-research.md", "reddit-rss.md", "web-tor.md"),
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
    objective = "Find legitimate currently useful ways to reduce the cost of coding-agent and developer workflows. Treat linux.do as a first-class discovery surface, then verify through official docs, repos, releases or other primary sources. Hunt free/cheap coding agents, APIs, model access, credits, open-source replacements, compatibility layers and workflow tricks. Local inference matters only when it materially improves coding-agent cost/privacy/workflow; do not turn this into a local-model hobby feed. For every useful item state what is free, quota/limit/catch, compute requirement, expiry/uncertainty and why it matters. Reject leaked/shared credentials, stolen accounts, payment bypasses, mass-account abuse and service restriction evasion."
    return write_report(invoke_json(research_prompt("free-ai-radar", objective), web_only=True), "free-ai-radar")


def agenda() -> dict[str, Any]:
    objective = "Produce the compact current agenda the user should know today. Highest bias: coding agents/vibe coding/dev tooling and Monero/privacy. Secondary: Nix/Linux, Tor/onion, OPSEC, security, web technology, private communications, open source and meaningful startup/business changes; include major broader technology events only when consequential. Do not fill space with generic model benchmark chatter, local-LLM hobby news, token-price analysis or filler. Prefer official/primary reporting and corroboration."
    return write_report(invoke_json(research_prompt("agenda", objective), web_only=True), "agenda")


def upstream_edge_radar() -> dict[str, Any]:
    objective = "Act as an early-warning radar for Vesper's upstream stack. Inspect meaningful recent PRs, issues, commits, releases and migration notes around NousResearch/hermes-agent, numtide/llm-agents.nix, OpenAI Codex, Anthropic Claude Code, anomalyco/opencode, nixpkgs/NixOS, Hyprland, Caelestia shell, Zen Browser, Helium, Tor, Monero and Cuprate. Surface breaking changes, new capabilities, deprecations, security/privacy implications and workarounds before they become surprises. Ignore routine churn. For each item say act now, watch, or ignore."
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
3) **News** — 10-15 genuinely important items when available. Prioritize coding agents/vibe coding/dev tooling and Monero/privacy; then Tor/onion/OPSEC, Nix/Linux, private communications, security/development/web privacy/startups/major tech. No coin-price talk and no generic local-model/inference filler. Each item: bold numbered title, one concise sentence, then URL. Never invent URLs.
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
    "unknown-frontier-web": unknown_frontier_web,
    "unknown-frontier-synthesis": frontier_synthesis,
    "frontier-daily": frontier_daily,
    "free-ai-radar": free_ai_radar,
    "agenda": agenda,
    "upstream-edge-radar": upstream_edge_radar,
    "morning-check": morning_check,
}
