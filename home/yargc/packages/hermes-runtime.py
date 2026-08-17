#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import threading
from pathlib import Path
from typing import Any

BRIEFING_ROOT = Path(
    os.environ.get("VESPER_BRIEFING_DIR", "~/.local/share/vesper/briefings")
).expanduser()

COMPAT_RUN_MAP = {
    "unknown-frontier-ai": "frontier-daily",
    "agenda": "agenda",
    "free-ai-radar": "free-ai-radar",
}


def ensure_dirs() -> None:
    BRIEFING_ROOT.mkdir(parents=True, exist_ok=True)


def atomic_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.{os.getpid()}.{threading.get_ident()}.tmp")
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
    high = [item for item in unread if item.get("priority", "normal") in {"high", "critical"}]
    latest = items[0] if items else {}
    if high:
        state = "attention"
    elif unread:
        state = "unread"
    else:
        state = "idle"
    latest_title = str(latest.get("title") or "No briefings yet")
    tooltip = "Hermes · no briefings yet" if not items else f"Hermes · {len(unread)} unread · {latest_title}"
    return {
        "count": len(items),
        "unread": len(unread),
        "high": len(high),
        "class": state,
        "latestTitle": latest_title,
        "latestLane": str(latest.get("lane") or ""),
        "tooltip": tooltip,
    }


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


def automation_binary() -> str:
    binary = shutil.which("vesper-hermes-automations")
    if not binary:
        raise RuntimeError("vesper-hermes-automations is not available")
    return binary


def run_compat_lane(lane: str) -> int:
    return subprocess.call([automation_binary(), "execute", COMPAT_RUN_MAP[lane]])


def run_compat_daily() -> int:
    for task in ("frontier-daily", "free-ai-radar", "agenda"):
        status = subprocess.call([automation_binary(), "execute", task])
        if status != 0:
            return status
    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(prog="vesper-hermes")
    parser.set_defaults(json=False)
    sub = parser.add_subparsers(dest="command")

    # Compatibility shims only. Research execution is owned by
    # vesper-hermes-automations; this client never implements a second engine.
    run = sub.add_parser("run")
    run.add_argument("lane", choices=sorted(COMPAT_RUN_MAP))
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
            return run_compat_lane(args.lane)
        if command == "daily":
            return run_compat_daily()
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
                    print(f"{marker} {item.get('id', '')} · {item.get('priority', 'normal')} · {item.get('title', 'Untitled')}")
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
            os.execvp("ghostty", ["ghostty", "--class=vesper-hermes-inbox", "-e", runtime, "tui"])
        raise RuntimeError(f"unsupported command: {command}")
    except (KeyError, ValueError, RuntimeError, subprocess.TimeoutExpired) as exc:
        print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
