#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import sys

from hermes_automation_common import STATE_ROOT, atomic_json, load_json, load_registry, now
from hermes_automation_contract import validate_registry
from hermes_automation_scheduler import WATCHDOG_TASKS, dispatch_job, job_lock, record_run, run_watchdog, sync_cron
from hermes_automation_tasks import TASKS, run_task
from hermes_research_link_registry import prune_web_links, web_link_records
from hermes_research_web import fetch_document


def parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="vesper-hermes-automations")
    sub = p.add_subparsers(dest="command", required=True)

    trigger = sub.add_parser("trigger")
    trigger.add_argument("job")

    execute = sub.add_parser("execute")
    execute.add_argument("job")

    dispatch = sub.add_parser("dispatch")
    dispatch.add_argument("job")

    watch = sub.add_parser("watch")
    watch.add_argument("job")

    sync = sub.add_parser("sync-cron")
    sync.add_argument("--prune", action="store_true")

    tor_fetch = sub.add_parser("tor-fetch")
    tor_fetch.add_argument("url")
    tor_fetch.add_argument("--max-chars", type=int, default=50000)

    links = sub.add_parser("links")
    links.add_argument("--all", action="store_true", help="include retired link records")
    links.add_argument("--prune", action="store_true", help="run autonomous learned-link GC before listing")

    sub.add_parser("validate-registry")
    sub.add_parser("jobs")
    return p


def edge_watch(name: str, output: str) -> str:
    path = STATE_ROOT / "watches" / f"{name}.json"
    previous = load_json(path, {})
    previous_text = str(previous.get("output") or "") if isinstance(previous, dict) else ""
    current = (output or "").strip()
    fingerprint = hashlib.sha256(current.encode()).hexdigest() if current else ""
    previous_fingerprint = str(previous.get("fingerprint") or "") if isinstance(previous, dict) else ""
    atomic_json(path, {"fingerprint": fingerprint, "output": current, "checkedAt": now().isoformat(timespec="seconds")})
    if fingerprint == previous_fingerprint:
        return ""
    if not current and previous_text:
        return f"[Hermes watch] {name} recovered"
    return current


def failure_notice(name: str, message: str) -> None:
    text = f"Hermes automation failed: {name}\n{message[-1800:]}"
    try:
        subprocess.run(["notify-send", "-a", "Hermes", "Hermes automation failed", text], check=False)
    except Exception:
        pass
    try:
        subprocess.run(
            ["hermes", "send", "--to", "telegram", "--quiet"],
            input=text,
            text=True,
            capture_output=True,
            timeout=45,
            check=False,
        )
    except Exception:
        pass


def _validate_or_print(registry: dict[str, dict]) -> int:
    errors = validate_registry(registry, task_names=TASKS, watchdog_names=WATCHDOG_TASKS)
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 2
    print(f"ok: {len(registry)} declarative Hermes jobs")
    return 0


def main() -> int:
    args = parser().parse_args()

    if args.command == "tor-fetch":
        try:
            max_chars = max(1, min(250000, int(args.max_chars)))
            print(json.dumps(fetch_document(args.url, max_chars=max_chars), ensure_ascii=False, indent=2))
            return 0
        except Exception as exc:
            print(str(exc), file=sys.stderr)
            return 1

    if args.command == "links":
        result: dict[str, object] = {"schemaVersion": 1}
        if args.prune:
            result["gc"] = prune_web_links()
        records = web_link_records(include_retired=args.all)
        result["count"] = len(records)
        result["links"] = records
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0

    registry = load_registry()

    if args.command == "jobs":
        print(json.dumps(registry, ensure_ascii=False, indent=2))
        return 0

    if args.command == "validate-registry":
        return _validate_or_print(registry)

    if args.command == "sync-cron":
        validation = validate_registry(registry, task_names=TASKS, watchdog_names=WATCHDOG_TASKS)
        if validation:
            print(json.dumps({"ok": False, "errors": validation}, ensure_ascii=False, indent=2))
            return 2
        result = sync_cron(prune=args.prune)
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 0 if result.get("ok") else 2

    if args.command == "watch":
        output = edge_watch(args.job, run_watchdog(args.job))
        if output:
            print(output)
        return 0

    if args.command == "dispatch":
        dispatch_job(args.job)
        return 0

    if args.command == "trigger":
        spec = registry.get(args.job)
        if spec is None:
            print(f"unknown Hermes job: {args.job}", file=sys.stderr)
            return 2
        mode = str(spec.get("mode") or "dispatch")
        task = str(spec.get("task") or args.job)
        if mode == "watchdog":
            output = edge_watch(task, run_watchdog(task))
            if output:
                print(output)
            return 0
        if mode != "dispatch":
            print(f"unsupported Hermes job mode: {mode}", file=sys.stderr)
            return 2
        dispatch_job(task)
        return 0

    if args.command == "execute":
        if args.job not in TASKS:
            print(f"unknown Hermes automation task: {args.job}", file=sys.stderr)
            return 2
        started = now()
        try:
            lock = job_lock(args.job)
        except Exception as exc:
            if "already running" in str(exc):
                return 0
            print(str(exc), file=sys.stderr)
            return 2
        try:
            report = run_task(args.job)
            record_run(args.job, "ok", started)
            print(json.dumps(report, ensure_ascii=False))
            return 0
        except Exception as exc:
            message = str(exc)
            record_run(args.job, "error", started, error=message)
            failure_notice(args.job, message)
            print(message, file=sys.stderr)
            return 1
        finally:
            lock.close()

    return 2


if __name__ == "__main__":
    raise SystemExit(main())
