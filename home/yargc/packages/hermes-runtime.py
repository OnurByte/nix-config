#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fcntl
import json
import os
import shutil
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path
from typing import Any

LANES = ("unknown-frontier-ai", "agenda", "free-ai-radar")

STATE_ROOT = Path(
    os.environ.get("VESPER_RESEARCH_STATE_DIR", "~/.local/state/vesper/research")
).expanduser()
BRIEFING_ROOT = Path(
    os.environ.get("VESPER_BRIEFING_DIR", "~/.local/share/vesper/briefings")
).expanduser()
MODEL = os.environ.get("HERMES_RESEARCH_MODEL", "grok-4.5")
PROVIDER = os.environ.get("HERMES_RESEARCH_PROVIDER", "xai-oauth")


def ensure_dirs() -> None:
    STATE_ROOT.mkdir(parents=True, exist_ok=True)
    BRIEFING_ROOT.mkdir(parents=True, exist_ok=True)


def atomic_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(
        f".{path.name}.{os.getpid()}.{threading.get_ident()}.tmp"
    )
    tmp.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")
    tmp.replace(path)


def load_json(path: Path, default: Any) -> Any:
    try:
        return json.loads(path.read_text(errors="replace"))
    except Exception:
        return default


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


def index_items() -> list[dict[str, Any]]:
    return rebuild_index()


def status_payload() -> dict[str, Any]:
    items = index_items()
    unread = [item for item in items if item.get("unread") is True]
    high = [
        item
        for item in unread
        if item.get("priority", "normal") in {"high", "critical"}
    ]
    latest = items[0] if items else {}
    if high:
        state = "attention"
    elif unread:
        state = "unread"
    else:
        state = "idle"
    latest_title = str(latest.get("title") or "No briefings yet")
    tooltip = (
        "Hermes · no briefings yet"
        if not items
        else f"Hermes · {len(unread)} unread · {latest_title}"
    )
    return {
        "count": len(items),
        "unread": len(unread),
        "high": len(high),
        "class": state,
        "latestTitle": latest_title,
        "latestLane": str(latest.get("lane") or ""),
        "tooltip": tooltip,
    }


def state_context(lane_dir: Path, max_chars: int = 42000) -> str:
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


def build_prompt(lane: str, context: str) -> str:
    return f"""Run the installed Vesper skill `hermes-research-radar` for exactly one lane: {lane}.

This is a persistent scheduled research run, not a generic news answer. Respect that lane's objective, breadth, verification, deduplication, exploration, legitimacy and anti-hype rules from the skill. User-supplied sources are seeds, never an allowlist. Prefer primary evidence for important claims. Do not pad weak results.

Existing durable lane state is included below. Treat delivered findings as known and avoid repeating them unless there is a meaningful change. Use candidate sources, heuristics and open questions as research hints, but continue exploring outside them.

----- EXISTING STATE -----
{context}
----- END STATE -----

Return exactly one JSON object and nothing else. Do not use Markdown fences.

Required shape:
{{
  "title": "short briefing title",
  "summary": "1-3 sentence summary",
  "body": "concise but useful report body",
  "priority": "low|normal|high|critical",
  "confidence": 0.0,
  "sources": [{{"title":"source title","url":"https://..."}}],
  "statePatch": {{
    "knownConcepts": [],
    "candidateSources": [],
    "heuristics": [],
    "openQuestions": []
  }}
}}

For unknown-frontier-ai also include when useful: visibility, whyHidden, whyUseful, whyNew, informationGain.
For free-ai-radar also include when useful: freeTier, limits, expiresAt, catch.
confidence is a number from 0 to 1.
Never invent URLs. If there is genuinely nothing worth surfacing, return a truthful low-priority record saying so; do not fabricate findings.
"""


def extract_object(text: str) -> dict[str, Any]:
    decoder = json.JSONDecoder()
    best: dict[str, Any] | None = None
    for index, char in enumerate(text):
        if char != "{":
            continue
        try:
            value, _ = decoder.raw_decode(text[index:])
        except Exception:
            continue
        if isinstance(value, dict) and value.get("title") and value.get("summary") is not None:
            best = value
    if best is None:
        raise RuntimeError("Hermes did not return a valid briefing JSON object")
    return best


def merge_unique(path: Path, incoming: list[Any], limit: int = 500) -> None:
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


def persist_state(report: dict[str, Any], lane_dir: Path) -> None:
    known_path = lane_dir / "known.json"
    known = load_json(known_path, [])
    if not isinstance(known, list):
        known = []
    known.insert(
        0,
        {
            "id": report.get("id"),
            "title": report.get("title"),
            "createdAt": report.get("createdAt"),
            "sources": report.get("sources", []),
        },
    )
    atomic_json(known_path, known[:500])

    patch = report.get("statePatch") or {}
    if not isinstance(patch, dict):
        return
    for key in ("knownConcepts", "candidateSources", "heuristics", "openQuestions"):
        incoming = patch.get(key) or []
        if isinstance(incoming, list) and incoming:
            merge_unique(lane_dir / f"{key}.json", incoming)


def markdown_for(report: dict[str, Any]) -> str:
    lines = [
        f"# {report.get('title', 'Hermes briefing')}",
        "",
        f"- lane: `{report.get('lane', 'unknown')}`",
        f"- priority: `{report.get('priority', 'normal')}`",
        f"- confidence: `{report.get('confidence', 0.5)}`",
        f"- created: `{report.get('createdAt', '')}`",
        "",
        str(report.get("summary") or ""),
        "",
        str(report.get("body") or ""),
    ]
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


def notify(report: dict[str, Any]) -> None:
    if report.get("priority") not in {"high", "critical"}:
        return
    binary = shutil.which("notify-send")
    if not binary:
        return
    subprocess.run(
        [
            binary,
            "-a",
            "Hermes",
            f"Hermes · {report.get('lane', 'research')}",
            f"{report.get('title', 'Briefing')} — {report.get('summary', '')}",
        ],
        check=False,
    )


def run_lane(lane: str) -> dict[str, Any]:
    if lane not in LANES:
        raise ValueError(f"unsupported lane: {lane}")
    ensure_dirs()
    lane_dir = STATE_ROOT / lane
    lane_dir.mkdir(parents=True, exist_ok=True)

    lock_handle = (lane_dir / ".lock").open("w")
    try:
        fcntl.flock(lock_handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError as exc:
        lock_handle.close()
        raise RuntimeError(f"Hermes lane already running: {lane}") from exc

    hermes = shutil.which("hermes")
    if not hermes:
        lock_handle.close()
        raise RuntimeError("hermes executable not found in PATH")

    prompt = build_prompt(lane, state_context(lane_dir))
    command = [
        hermes,
        "-z",
        prompt,
        "--provider",
        PROVIDER,
        "-m",
        MODEL,
        "-t",
        "web",
        "--yolo",
    ]
    try:
        completed = subprocess.run(
            command,
            text=True,
            capture_output=True,
            timeout=900,
            check=False,
        )
        if completed.returncode != 0:
            detail = (completed.stderr or completed.stdout)[-6000:]
            raise RuntimeError(
                f"Hermes lane failed: {lane} (rc={completed.returncode})\n{detail}"
            )
        raw = (completed.stdout or "") + "\n" + (completed.stderr or "")
        report = extract_object(raw)

        created = datetime.now().astimezone()
        report_id = f"{lane}-{created.strftime('%Y%m%dT%H%M%S')}"
        report.update(
            {
                "id": report_id,
                "lane": lane,
                "createdAt": created.isoformat(timespec="seconds"),
                "unread": True,
                "priority": report.get("priority") or "normal",
                "confidence": report.get("confidence", 0.5),
                "sources": report.get("sources") or [],
            }
        )

        day_dir = BRIEFING_ROOT / created.strftime("%Y/%m/%d")
        json_path = day_dir / f"{report_id}.json"
        md_path = day_dir / f"{report_id}.md"
        atomic_json(json_path, report)
        md_path.parent.mkdir(parents=True, exist_ok=True)
        md_path.write_text(markdown_for(report))
        persist_state(report, lane_dir)
        rebuild_index()
        notify(report)
        return report
    finally:
        lock_handle.close()


def mark_read(report_id: str) -> dict[str, Any]:
    for item in index_items():
        if item.get("id") != report_id:
            continue
        path = Path(str(item["_jsonPath"]))
        report = load_json(path, {})
        if not isinstance(report, dict):
            break
        report["unread"] = False
        atomic_json(path, report)
        rebuild_index()
        return report
    raise KeyError(f"unknown briefing id: {report_id}")


def mark_all_read() -> None:
    for item in index_items():
        if item.get("unread") is not True:
            continue
        path = Path(str(item["_jsonPath"]))
        report = load_json(path, {})
        if isinstance(report, dict):
            report["unread"] = False
            atomic_json(path, report)
    rebuild_index()


def report_text(report: dict[str, Any]) -> str:
    sources = report.get("sources") or []
    source_lines: list[str] = []
    for source in sources if isinstance(sources, list) else []:
        if isinstance(source, str):
            source_lines.append(f"- {source}")
        elif isinstance(source, dict):
            title = source.get("title") or source.get("url") or "source"
            url = source.get("url") or ""
            source_lines.append(f"- {title}" + (f" — {url}" if url else ""))
    suffix = "\n\nSources:\n" + "\n".join(source_lines) if source_lines else ""
    return (
        f"# {report.get('title', 'Untitled')}\n\n"
        f"lane: {report.get('lane', 'unknown')} · priority: {report.get('priority', 'normal')} · "
        f"confidence: {report.get('confidence', 'unknown')}\n\n"
        f"{report.get('summary', '')}\n\n{report.get('body', '')}{suffix}\n"
    )


def tui() -> None:
    items = index_items()
    print("VESPER · HERMES BRIEFINGS\n")
    if not items:
        print("no briefings yet")
    for item in items[:30]:
        marker = "●" if item.get("unread") else "○"
        priority = str(item.get("priority") or "normal").upper()
        print(f"{marker} {priority} · {item.get('lane', 'unknown')}")
        print(f"  {item.get('title', 'Untitled')}")
        print(f"  {item.get('summary', '')}")
        print(f"  id: {item.get('id', '')}\n")
    print("Commands: vesper-hermes read <id> · mark-read <id> · mark-all-read")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="vesper-hermes")
    parser.set_defaults(json=False)
    sub = parser.add_subparsers(dest="command")

    run = sub.add_parser("run")
    run.add_argument("lane", choices=LANES)

    sub.add_parser("daily")

    status = sub.add_parser("status")
    status.add_argument("--json", action="store_true")

    listing = sub.add_parser("list")
    listing.add_argument("--json", action="store_true")

    read = sub.add_parser("read")
    read.add_argument("id")

    mark = sub.add_parser("mark-read")
    mark.add_argument("id")

    sub.add_parser("mark-all-read")
    sub.add_parser("reindex")
    sub.add_parser("tui")
    sub.add_parser("inbox")
    return parser.parse_args()


def main() -> int:
    ensure_dirs()
    args = parse_args()
    command = args.command or "status"

    try:
        if command == "run":
            print(json.dumps(run_lane(args.lane), ensure_ascii=False, indent=2))
            return 0
        if command == "daily":
            failures: list[str] = []
            with ThreadPoolExecutor(max_workers=len(LANES)) as pool:
                futures = {pool.submit(run_lane, lane): lane for lane in LANES}
                for future in as_completed(futures):
                    lane = futures[future]
                    try:
                        future.result()
                    except Exception as exc:
                        failures.append(f"{lane}: {exc}")
            print(json.dumps(status_payload(), ensure_ascii=False))
            if failures:
                print("\n".join(failures), file=sys.stderr)
                return 1
            return 0
        if command == "status":
            payload = status_payload()
            if args.json:
                print(json.dumps(payload, ensure_ascii=False))
            else:
                print(
                    f"Hermes · {payload['unread']} unread · {payload['high']} high priority · "
                    f"{payload['count']} total\nlatest: {payload['latestTitle']}"
                )
            return 0
        if command == "list":
            items = index_items()
            if args.json:
                print(json.dumps(items, ensure_ascii=False, indent=2))
            else:
                for item in items:
                    marker = "●" if item.get("unread") else "○"
                    print(
                        f"{marker} {item.get('id', '')} · {item.get('priority', 'normal')} · "
                        f"{item.get('title', 'Untitled')}"
                    )
            return 0
        if command == "read":
            report = mark_read(args.id)
            print(report_text(report))
            return 0
        if command == "mark-read":
            mark_read(args.id)
            return 0
        if command == "mark-all-read":
            mark_all_read()
            return 0
        if command == "reindex":
            rebuild_index()
            return 0
        if command == "tui":
            tui()
            return 0
        if command == "inbox":
            runtime = shutil.which("vesper-hermes") or sys.argv[0]
            os.execvp(
                "ghostty",
                ["ghostty", "--class=vesper-hermes-inbox", "-e", runtime, "tui"],
            )
        raise RuntimeError(f"unsupported command: {command}")
    except (KeyError, ValueError, RuntimeError, subprocess.TimeoutExpired) as exc:
        print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
