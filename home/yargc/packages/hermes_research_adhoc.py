from __future__ import annotations

import json
import math
import os
import re
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, invoke_json, now
from hermes_automation_reports import research_skill_context, state_context, write_report
from hermes_research_intake import compact_intake, load_source_registry, reddit_rss_intake, reinforce_source_registry, x_mirror_intake
from hermes_research_link_registry import prune_web_links, web_link_records
from hermes_research_web import compact_web_intake, reinforce_web_registry, web_core_intake

ADHOC_SURFACES = ("github", "reddit", "x", "web")
ADHOC_MAX_PAGES = max(3000, int(os.environ.get("VESPER_ADHOC_MAX_PAGES", "10000")))
ADHOC_WAVE_PAGES = max(100, min(500, int(os.environ.get("VESPER_ADHOC_WAVE_PAGES", "250"))))
ADHOC_MAX_WORKERS = max(1, min(4, int(os.environ.get("VESPER_ADHOC_MAX_WORKERS", "2"))))
ADHOC_SURFACE_WEIGHTS = {"github": 0.30, "reddit": 0.25, "x": 0.25, "web": 0.20}


def _slug(value: str, limit: int = 48) -> str:
    text = re.sub(r"[^a-zA-Z0-9]+", "-", value.strip().lower()).strip("-")
    return (text or "research")[:limit]


def _allocate(total: int) -> dict[str, int]:
    total = max(len(ADHOC_SURFACES), int(total))
    values = {name: max(1, round(total * ADHOC_SURFACE_WEIGHTS[name])) for name in ADHOC_SURFACES}
    delta = total - sum(values.values())
    order = list(ADHOC_SURFACES)
    index = 0
    while delta != 0:
        name = order[index % len(order)]
        if delta > 0:
            values[name] += 1
            delta -= 1
        elif values[name] > 1:
            values[name] -= 1
            delta += 1
        index += 1
    return values


def _default_deep_reads(pages: int) -> int:
    return max(48, min(300, round(pages * 0.06)))


def research_plan(query: str, pages: int, deep_reads: int | None = None) -> dict[str, Any]:
    clean_query = " ".join(str(query).split())
    if not clean_query:
        raise ValueError("research query must not be empty")
    pages = max(50, min(ADHOC_MAX_PAGES, int(pages)))
    deep = _default_deep_reads(pages) if deep_reads is None else max(8, min(pages, int(deep_reads)))
    page_budget = _allocate(pages)
    deep_budget = _allocate(deep)
    waves: dict[str, list[dict[str, int]]] = {}
    for surface in ADHOC_SURFACES:
        remaining = page_budget[surface]
        deep_remaining = deep_budget[surface]
        count = max(1, math.ceil(remaining / ADHOC_WAVE_PAGES))
        specs: list[dict[str, int]] = []
        for wave in range(count):
            target = min(ADHOC_WAVE_PAGES, remaining)
            if wave == count - 1:
                deep_target = deep_remaining
            else:
                deep_target = min(deep_remaining, max(1, round(deep_budget[surface] * target / max(1, page_budget[surface]))))
            specs.append({"index": wave + 1, "candidateTarget": target, "deepReadTarget": max(1, deep_target)})
            remaining -= target
            deep_remaining = max(0, deep_remaining - deep_target)
        waves[surface] = specs
    return {
        "query": clean_query,
        "candidateTarget": pages,
        "deepReadTarget": deep,
        "surfaceBudget": page_budget,
        "surfaceDeepReadBudget": deep_budget,
        "waveSize": ADHOC_WAVE_PAGES,
        "waves": waves,
    }


def _initial_intake(surface: str, target: int, deep_target: int) -> dict[str, Any] | None:
    if surface == "reddit":
        return reddit_rss_intake(target)
    if surface == "x":
        return x_mirror_intake(target)
    if surface == "web":
        prune_web_links()
        return web_core_intake(target, deep_fetch_limit=min(60, max(1, deep_target)))
    return None


def _slice_intake(value: dict[str, Any] | None, offset: int, limit: int) -> dict[str, Any] | None:
    if not value:
        return None
    out = dict(value)
    candidates = value.get("candidates") or []
    if isinstance(candidates, list):
        out["candidates"] = candidates[offset : offset + limit]
        out["canonicalCandidates"] = len(out["candidates"])
        out["slice"] = {"offset": offset, "limit": limit, "available": len(candidates)}
    return out


def _urls_from_report(report: dict[str, Any]) -> set[str]:
    urls: set[str] = set()
    for candidate in report.get("candidates") or []:
        if not isinstance(candidate, dict):
            continue
        for value in candidate.get("urls") or []:
            if isinstance(value, str) and value.startswith(("http://", "https://")):
                urls.add(value)
    for source in report.get("sources") or []:
        value = source.get("url") if isinstance(source, dict) else source
        if isinstance(value, str) and value.startswith(("http://", "https://")):
            urls.add(value)
    return urls


def _wave_prompt(
    query: str,
    surface: str,
    wave: dict[str, int],
    wave_count: int,
    intake: dict[str, Any] | None,
    prior_urls: set[str],
) -> str:
    rules = {
        "github": "Search GitHub repositories, code-adjacent docs, issues, PRs, commits, discussions, releases, forks, authors and dependency neighborhoods. Prefer primary technical evidence and small overlooked projects over popularity.",
        "reddit": "Reddit is mandatory for this lane. Use RSS intake as the cheap sensor layer, then inspect relevant posts, comments and linked primary evidence. Search beyond the seed communities when the query points elsewhere.",
        "x": "X/Twitter is mandatory for this lane. Use direct X when available and XCancel/Nitter-compatible fallbacks when needed. Inspect posts, replies, quote chains, accounts and linked code/docs; mirror copies are one canonical item.",
        "web": "Normal sites, forums and Tor onion services are mandatory for this lane. Use the locally supplied web/onion intake; onion content in that intake was fetched through local Tor. Expand through outgoing links and normal web search where appropriate.",
    }
    refs = ["research-pipeline.md", "source-governance.md", "central-sources.md", "adhoc-research.md"]
    if surface == "reddit":
        refs.append("reddit-rss.md")
    if surface == "x":
        refs.append("x-research.md")
    if surface == "web":
        refs.append("web-tor.md")
    skill = research_skill_context(refs, max_chars=36000)
    if surface == "web" and intake:
        intake_text = compact_web_intake(intake, max_chars=85000)
    elif intake:
        intake_text = compact_intake(intake, max_chars=75000)
    else:
        intake_text = "No deterministic intake for this lane; build coverage using search/browsing tools."
    prior = "\n".join(sorted(prior_urls)[-300:]) if prior_urls else "none"
    return f"""You are Vesper's ad-hoc `{surface}` research scout, wave {wave['index']} of {wave_count}.

Research question:
{query}

{rules[surface]}

This is a measured high-volume research run, not a generic answer. `--pages` means distinct canonical candidate items/pages inspected. It does not mean every candidate must be dumped into model context. Use cheap intake/search metadata for breadth, then fully open only the strongest subset.

Wave contract:
- inspect about {wave['candidateTarget']} new canonical candidate items for this surface;
- deep-read about {wave['deepReadTarget']} strongest candidates when enough signal exists;
- do not count the same canonical URL twice;
- do not count a mirror copy as independent evidence;
- expand to new source neighborhoods when seeds are weak;
- report real shortfall instead of padding or inventing counts;
- preserve useful contradictory evidence;
- follow important community/social claims to primary sources when possible.

Earlier waves already surfaced these URLs. Avoid re-counting them except when verification genuinely requires revisiting one:
{prior}

----- RESEARCH PROCEDURE -----
{skill}
----- END PROCEDURE -----
----- READ-ONLY LEARNED FRONTIER STATE -----
{state_context('unknown-frontier-ai', 14000)}
----- END STATE -----
----- WAVE INTAKE -----
{intake_text}
----- END INTAKE -----

Return exactly one JSON object and nothing else:
{{"title":"{surface} wave {wave['index']}","summary":"short wave summary","body":"dense technical notes","priority":"low|normal|high|critical","confidence":0.0,"sources":[{{"title":"source","url":"https://..."}}],"candidates":[{{"title":"candidate","topic":"...","whyUseful":"...","evidence":"...","urls":["https://..."]}}],"coverage":{{"candidateTarget":{wave['candidateTarget']},"candidatesInspected":0,"canonicalCandidates":0,"deepReads":0,"primaryVerifications":0,"surfaces":["{surface}"],"limitations":[]}},"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
Never invent URLs, page contents or coverage numbers.
"""


def _coverage_int(report: dict[str, Any], key: str) -> int:
    coverage = report.get("coverage") or {}
    if not isinstance(coverage, dict):
        return 0
    try:
        return max(0, int(coverage.get(key) or 0))
    except (TypeError, ValueError):
        return 0


def _run_surface(run_dir: Path, query: str, surface: str, specs: list[dict[str, int]], target: int, deep_target: int) -> dict[str, Any]:
    intake: dict[str, Any] | None = None
    intake_error = ""
    try:
        intake = _initial_intake(surface, target, deep_target)
    except Exception as exc:
        intake_error = str(exc)[-2000:]
    prior_urls: set[str] = set()
    reports: list[dict[str, Any]] = []
    failures: list[str] = []
    offset = 0
    for spec in specs:
        sliced = _slice_intake(intake, offset, spec["candidateTarget"])
        offset += spec["candidateTarget"]
        try:
            report = invoke_json(_wave_prompt(query, surface, spec, len(specs), sliced, prior_urls), web_only=True)
            coverage = report.get("coverage")
            if not isinstance(coverage, dict):
                coverage = {}
                report["coverage"] = coverage
            coverage["candidateTarget"] = spec["candidateTarget"]
            if sliced:
                coverage["deterministicIntakeCandidates"] = int(sliced.get("canonicalCandidates") or 0)
            if intake_error:
                coverage.setdefault("limitations", []).append(f"deterministic intake failed: {intake_error}")
            prior_urls.update(_urls_from_report(report))
            if surface in {"reddit", "x"}:
                reinforce_source_registry(surface, report)
            elif surface == "web":
                reinforce_web_registry(report)
            reports.append(report)
            atomic_json(run_dir / "waves" / surface / f"{spec['index']:02d}.json", report)
        except Exception as exc:
            failures.append(f"wave {spec['index']}: {str(exc)[-1800:]}")
    return {"surface": surface, "reports": reports, "failures": failures, "intakeError": intake_error}


def _distill(report: dict[str, Any]) -> dict[str, Any]:
    candidates = report.get("candidates") or []
    return {
        "title": report.get("title"),
        "summary": report.get("summary"),
        "coverage": report.get("coverage") or {},
        "candidates": candidates[:20] if isinstance(candidates, list) else [],
        "sources": (report.get("sources") or [])[:25],
    }


def _standard_sources() -> list[dict[str, Any]]:
    registry = load_source_registry()
    records: list[dict[str, Any]] = []
    for key, entry in registry.get("sources", {}).items():
        if not isinstance(entry, dict):
            continue
        kind = str(entry.get("kind") or "")
        if kind == "reddit":
            name = str(entry.get("name") or "")
            url = f"https://www.reddit.com/r/{name}/" if name else ""
            label = f"r/{name}" if name else key
        elif kind == "x":
            name = str(entry.get("name") or "")
            url = f"https://x.com/{name}" if name else ""
            label = f"@{name}" if name else key
        elif kind == "web":
            url = str(entry.get("url") or entry.get("name") or "")
            label = str(entry.get("label") or url or key)
        else:
            continue
        records.append({
            "id": str(entry.get("id") or key),
            "kind": kind,
            "url": url,
            "label": label,
            "topic": str(entry.get("topic") or ""),
            "seed": bool(entry.get("seed") or entry.get("protected") or str(entry.get("origin") or "").startswith("central")),
            "tier": str(entry.get("tier") or "probation"),
            "score": float(entry.get("score") or 0.0),
            "hits": int(entry.get("hits") or 0),
            "observations": int(entry.get("observations") or 0),
            "failures": int(entry.get("failures") or 0),
            "origin": str(entry.get("origin") or ""),
            "firstSeen": str(entry.get("firstSeen") or ""),
            "lastSeen": str(entry.get("lastSeen") or ""),
            "lastUseful": str(entry.get("lastUseful") or ""),
        })
    records.sort(key=lambda item: (item["kind"], item["label"].lower()))
    return records


def source_records() -> list[dict[str, Any]]:
    # web_link_records normalizes legacy web entries before we expose the shared view.
    web_link_records(include_retired=True)
    return _standard_sources()


def run_adhoc_research(query: str, *, pages: int = 600, deep_reads: int | None = None, max_workers: int | None = None) -> dict[str, Any]:
    plan = research_plan(query, pages, deep_reads)
    created = now()
    run_id = f"{created.strftime('%Y%m%dT%H%M%S')}-{_slug(plan['query'])}"
    run_dir = STATE_ROOT / "adhoc-research" / run_id
    atomic_json(run_dir / "plan.json", plan | {"runId": run_id, "createdAt": created.isoformat(timespec="seconds")})

    workers = ADHOC_MAX_WORKERS if max_workers is None else max(1, min(4, int(max_workers)))
    outputs: dict[str, dict[str, Any]] = {}
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = {
            pool.submit(
                _run_surface,
                run_dir,
                plan["query"],
                surface,
                plan["waves"][surface],
                plan["surfaceBudget"][surface],
                plan["surfaceDeepReadBudget"][surface],
            ): surface
            for surface in ADHOC_SURFACES
        }
        for future in as_completed(futures):
            surface = futures[future]
            try:
                outputs[surface] = future.result()
            except Exception as exc:
                outputs[surface] = {"surface": surface, "reports": [], "failures": [str(exc)[-2000:]], "intakeError": ""}

    all_reports = [report for surface in ADHOC_SURFACES for report in outputs.get(surface, {}).get("reports", [])]
    if not all_reports:
        raise RuntimeError("all ad-hoc research surfaces failed")

    candidate_total = sum(_coverage_int(report, "candidatesInspected") for report in all_reports)
    canonical_total = sum(_coverage_int(report, "canonicalCandidates") for report in all_reports)
    deep_total = sum(_coverage_int(report, "deepReads") for report in all_reports)
    primary_total = sum(_coverage_int(report, "primaryVerifications") for report in all_reports)
    failures = {
        surface: outputs.get(surface, {}).get("failures", [])
        for surface in ADHOC_SURFACES
        if outputs.get(surface, {}).get("failures")
    }
    coverage = {
        "candidateTarget": plan["candidateTarget"],
        "candidatesInspected": candidate_total,
        "canonicalCandidates": canonical_total,
        "deepReadTarget": plan["deepReadTarget"],
        "deepReads": deep_total,
        "primaryVerifications": primary_total,
        "shortfall": max(0, plan["candidateTarget"] - candidate_total),
        "surfaceBudget": plan["surfaceBudget"],
        "surfacesAttempted": list(ADHOC_SURFACES),
        "surfacesCompleted": [surface for surface in ADHOC_SURFACES if outputs.get(surface, {}).get("reports")],
        "failures": failures,
    }
    distilled = {
        surface: [_distill(report) for report in outputs.get(surface, {}).get("reports", [])]
        for surface in ADHOC_SURFACES
    }
    synthesis_context = json.dumps({"plan": plan, "coverage": coverage, "waves": distilled}, ensure_ascii=False, indent=2)[:150000]
    skill = research_skill_context(("research-pipeline.md", "source-governance.md", "central-sources.md", "adhoc-research.md", "reddit-rss.md", "x-research.md", "web-tor.md"), max_chars=36000)
    synthesis_prompt = f"""Synthesize Vesper's ad-hoc multi-surface research run.

Question:
{plan['query']}

The run attempted GitHub, Reddit, X/Twitter, normal web/forums and Tor/onion coverage. Preserve the strongest evidence, dedupe cross-platform copies, explain contradictions, separate verified facts from weak community claims, and prioritize actionable high-information-gain findings. Do not pretend the requested page budget was reached if coverage shows a shortfall.

----- RESEARCH CONTRACT -----
{skill}
----- END CONTRACT -----
----- WAVE RESULTS -----
{synthesis_context}
----- END RESULTS -----

Return exactly one JSON object and nothing else:
{{"title":"short research title","summary":"1-3 sentence executive summary","body":"dense final research report","priority":"low|normal|high|critical","confidence":0.0,"sources":[{{"title":"source","url":"https://..."}}],"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
Never invent URLs or numeric coverage.
"""
    final = invoke_json(synthesis_prompt, web_only=True)
    final["query"] = plan["query"]
    final["runId"] = run_id
    final["coverage"] = coverage
    final["surfaceReports"] = {surface: len(outputs.get(surface, {}).get("reports", [])) for surface in ADHOC_SURFACES}
    final = write_report(final, "adhoc-research")
    atomic_json(run_dir / "final.json", final)
    return final
