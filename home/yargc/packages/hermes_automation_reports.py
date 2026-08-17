from __future__ import annotations

import json
import shutil
import subprocess
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any, Iterable

from hermes_automation_common import BRIEFING_ROOT, HERMES_HOME, STATE_ROOT, atomic_json, ensure_dirs, load_json, now


def rebuild_index() -> list[dict[str, Any]]:
    ensure_dirs()
    items: list[dict[str, Any]] = []
    for path in BRIEFING_ROOT.rglob("*.json"):
        if path.name == "index.json":
            continue
        value = load_json(path, None)
        if not isinstance(value, dict) or not value.get("id"):
            continue
        item = dict(value)
        item["_jsonPath"] = str(path)
        markdown = path.with_suffix(".md")
        item["_markdownPath"] = str(markdown) if markdown.exists() else ""
        items.append(item)
    items.sort(key=lambda item: str(item.get("createdAt", "")), reverse=True)
    atomic_json(BRIEFING_ROOT / "index.json", items)
    return items


def state_context(lane: str, max_chars: int = 42000) -> str:
    lane_dir = STATE_ROOT / lane
    lane_dir.mkdir(parents=True, exist_ok=True)
    chunks: list[str] = []
    used = 0
    for path in sorted(lane_dir.glob("*.json")):
        text = path.read_text(errors="replace")[:12000]
        chunk = f"\n### {path.name}\n{text}\n"
        if used + len(chunk) > max_chars:
            chunk = chunk[: max_chars - used]
        chunks.append(chunk)
        used += len(chunk)
        if used >= max_chars:
            break
    return "".join(chunks)


def recent_briefings(days: int = 2, max_chars: int = 70000) -> str:
    cutoff = now() - timedelta(days=days)
    chunks: list[str] = []
    used = 0
    for item in rebuild_index():
        try:
            created = datetime.fromisoformat(str(item.get("createdAt", "")))
        except Exception:
            continue
        if created < cutoff:
            continue
        path_text = str(item.get("_markdownPath") or "")
        if not path_text or not Path(path_text).exists():
            continue
        text = Path(path_text).read_text(errors="replace")[:18000]
        chunk = f"\n--- {item.get('lane')} / {item.get('id')} ---\n{text}\n"
        if used + len(chunk) > max_chars:
            chunk = chunk[: max_chars - used]
        chunks.append(chunk)
        used += len(chunk)
        if used >= max_chars:
            break
    return "".join(chunks)


def research_skill_context(references: Iterable[str] = (), max_chars: int = 32000) -> str:
    roots = [
        Path.home() / ".agents" / "skills" / "hermes-research-radar",
        HERMES_HOME / "skills" / "vesper" / "hermes-research-radar",
    ]
    root = next((path for path in roots if (path / "SKILL.md").exists()), None)
    if root is None:
        return "The hermes-research-radar skill files were not readable at runtime; follow the research contract embedded in the task prompt."

    files = [root / "SKILL.md"]
    for name in references:
        clean = Path(str(name)).name
        files.append(root / "references" / clean)

    chunks: list[str] = []
    used = 0
    for path in files:
        if not path.exists():
            continue
        text = path.read_text(errors="replace")
        chunk = f"\n### {path.name}\n{text}\n"
        if used + len(chunk) > max_chars:
            chunk = chunk[: max_chars - used]
        chunks.append(chunk)
        used += len(chunk)
        if used >= max_chars:
            break
    return "".join(chunks)


def _merge_unique(path: Path, incoming: list[Any], limit: int = 500) -> None:
    current = load_json(path, [])
    if not isinstance(current, list):
        current = []
    merged: list[Any] = []
    seen: set[str] = set()
    for item in incoming + current:
        marker = json.dumps(item, sort_keys=True, ensure_ascii=False)
        if marker in seen:
            continue
        seen.add(marker)
        merged.append(item)
        if len(merged) >= limit:
            break
    atomic_json(path, merged)


def _persist_state(report: dict[str, Any], lane: str) -> None:
    lane_dir = STATE_ROOT / lane
    lane_dir.mkdir(parents=True, exist_ok=True)
    known_path = lane_dir / "known.json"
    known = load_json(known_path, [])
    if not isinstance(known, list):
        known = []
    known.insert(0, {"id": report.get("id"), "title": report.get("title"), "createdAt": report.get("createdAt"), "sources": report.get("sources", [])})
    atomic_json(known_path, known[:500])
    patch = report.get("statePatch") or {}
    if not isinstance(patch, dict):
        return
    for key in ("knownConcepts", "candidateSources", "heuristics", "openQuestions"):
        incoming = patch.get(key) or []
        if isinstance(incoming, list) and incoming:
            _merge_unique(lane_dir / f"{key}.json", incoming)


def _markdown(report: dict[str, Any]) -> str:
    lines = [
        f"# {report.get('title', 'Hermes briefing')}", "",
        f"- lane: `{report.get('lane', 'unknown')}`",
        f"- priority: `{report.get('priority', 'normal')}`",
        f"- confidence: `{report.get('confidence', 0.5)}`",
        f"- created: `{report.get('createdAt', '')}`", "",
        str(report.get("summary") or ""), "", str(report.get("body") or ""),
    ]
    coverage = report.get("coverage") or {}
    if isinstance(coverage, dict) and coverage:
        lines.extend(["", "## Coverage", "", "```json", json.dumps(coverage, ensure_ascii=False, indent=2), "```"])
    sources = report.get("sources") or []
    if isinstance(sources, list) and sources:
        lines.extend(["", "## Sources", ""])
        for source in sources:
            if isinstance(source, str):
                lines.append(f"- {source}")
            elif isinstance(source, dict):
                title = source.get("title") or source.get("url") or "source"
                url = source.get("url") or ""
                lines.append(f"- [{title}]({url})" if url else f"- {title}")
    return "\n".join(lines).rstrip() + "\n"


def _notify(report: dict[str, Any]) -> None:
    if str(report.get("priority")).lower() not in {"high", "critical"}:
        return
    binary = shutil.which("notify-send")
    if binary:
        subprocess.run([binary, "-a", "Hermes", f"Hermes · {report.get('lane', 'research')}", f"{report.get('title', 'Briefing')} — {report.get('summary', '')}"], check=False)


def write_report(report: dict[str, Any], lane: str, *, notify_user: bool = True) -> dict[str, Any]:
    created = now()
    report_id = f"{lane}-{created.strftime('%Y%m%dT%H%M%S')}"
    report.update({
        "id": report_id,
        "lane": lane,
        "createdAt": created.isoformat(timespec="seconds"),
        "unread": True,
        "priority": str(report.get("priority") or "normal").lower(),
        "confidence": report.get("confidence", 0.5),
        "sources": report.get("sources") or [],
    })
    day_dir = BRIEFING_ROOT / created.strftime("%Y/%m/%d")
    atomic_json(day_dir / f"{report_id}.json", report)
    md_path = day_dir / f"{report_id}.md"
    md_path.parent.mkdir(parents=True, exist_ok=True)
    md_path.write_text(_markdown(report))
    _persist_state(report, lane)
    rebuild_index()
    if notify_user:
        _notify(report)
    return report


def research_prompt(
    lane: str,
    objective: str,
    extra_context: str = "",
    *,
    skill_references: Iterable[str] = ("research-pipeline.md", "source-governance.md"),
) -> str:
    skill = research_skill_context(skill_references)
    return f"""Run Vesper's persistent Hermes research workflow for lane `{lane}`.

Objective:
{objective}

The installed `hermes-research-radar` skill is part of the execution contract. Follow the relevant procedure below rather than doing one superficial search.

----- RESEARCH SKILL -----
{skill}
----- END RESEARCH SKILL -----
----- DURABLE STATE -----
{state_context(lane)}
----- END DURABLE STATE -----
----- EXTRA CONTEXT -----
{extra_context}
----- END EXTRA CONTEXT -----

Return exactly one JSON object and nothing else:
{{"title":"short title","summary":"1-3 sentence summary","body":"concise but useful report","priority":"low|normal|high|critical","confidence":0.0,"sources":[{{"title":"source title","url":"https://..."}}],"coverage":{{"candidateTarget":0,"candidatesInspected":0,"canonicalCandidates":0,"deepReads":0,"primaryVerifications":0,"surfaces":[],"limitations":[]}},"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
Never invent URLs or numeric coverage. If there is nothing worth surfacing, say so without padding.
"""
