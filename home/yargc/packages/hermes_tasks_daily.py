from __future__ import annotations

import json
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, invoke_json, invoke_text
from hermes_automation_reports import recent_briefings, research_prompt, state_context, write_report


def _scout_prompt(source: str) -> str:
    rules = {
        "github": "Search recent/small repositories, issues, PRs, commits, forks, discussions and author/org neighborhoods. Prefer working code and technical evidence over stars.",
        "reddit": "Search recent/low-score posts, deep comment trees and niche communities. Extract reproducible techniques, fixes, workflows and primary links.",
        "x": "Search low-attention builder/researcher posts, demos, code links, patches and concrete techniques. Verify important claims against primary sources.",
    }
    return f"""You are one independent `{source}` scout inside Vesper's `unknown-frontier-ai` run.
{rules[source]}
Goal: discover useful AI/coding-agent/model/dev-tooling capabilities outside the user's current map and not yet obvious mainstream items. Low engagement is a discovery hint, never a quality score. Use broad discovery first, verify the strongest candidates, and avoid generic news, repeated known items, hype, price chatter and filler.

Durable frontier state:
{state_context('unknown-frontier-ai', 28000)}

Return exactly one JSON object and nothing else:
{{"title":"{source} scout","summary":"short scout summary","body":"technical scout notes","priority":"low|normal|high|critical","confidence":0.0,"sources":[{{"title":"source","url":"https://..."}}],"candidates":[{{"title":"candidate","whyNew":"...","whyUseful":"...","evidence":"...","urls":["https://..."]}}],"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
Never invent URLs.
"""


def frontier_daily() -> dict[str, Any]:
    scout_dir = STATE_ROOT / "unknown-frontier-ai" / "scouts"
    scout_dir.mkdir(parents=True, exist_ok=True)
    outputs: dict[str, dict[str, Any]] = {}
    failures: dict[str, str] = {}
    with ThreadPoolExecutor(max_workers=3) as pool:
        futures = {pool.submit(invoke_json, _scout_prompt(source), web_only=True): source for source in ("github", "reddit", "x")}
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
    extra = json.dumps({"scouts": outputs, "failures": failures}, ensure_ascii=False, indent=2)[:90000]
    report = invoke_json(research_prompt(
        "unknown-frontier-ai",
        "Synthesize the independent GitHub, Reddit and X scouts into one high-information-gain frontier report. Cross-check overlapping claims, follow the strongest candidates to primary evidence, remove duplicates and familiar/mainstream items, and rank only discoveries worth attention. Prefer a few technically dense discoveries over a long list.",
        extra,
    ), web_only=True)
    report["scoutFailures"] = failures
    report["scoutsCompleted"] = sorted(outputs)
    return write_report(report, "unknown-frontier-ai")


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
    "frontier-daily": frontier_daily,
    "free-ai-radar": free_ai_radar,
    "agenda": agenda,
    "upstream-edge-radar": upstream_edge_radar,
    "morning-check": morning_check,
}
