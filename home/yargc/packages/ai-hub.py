#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fcntl
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

CACHE_ROOT = Path(os.environ.get("XDG_CACHE_HOME", "~/.cache")).expanduser() / "vesper-ai-hub"
CACHE_PATH = CACHE_ROOT / "snapshot.json"
LOCK_PATH = CACHE_ROOT / "refresh.lock"
DEFAULT_MAX_AGE = max(15, int(os.environ.get("VESPER_AI_HUB_MAX_AGE", "60")))
CODEXBAR_TIMEOUT = max(5, int(os.environ.get("VESPER_AI_HUB_CODEXBAR_TIMEOUT", "30")))


def load_json(path: Path, default: Any = None) -> Any:
    try:
        return json.loads(path.read_text(errors="replace"))
    except Exception:
        return default


def run_json(command: list[str], timeout: int) -> Any:
    proc = subprocess.run(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        timeout=timeout,
        check=False,
    )
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip() or f"exit {proc.returncode}"
        raise RuntimeError(detail[:600])
    try:
        return json.loads(proc.stdout)
    except Exception as exc:
        raise RuntimeError(f"invalid JSON from {command[0]}: {exc}") from exc


def error_text(error: Any) -> str:
    if not error:
        return ""
    if isinstance(error, str):
        return error[:500]
    if isinstance(error, dict):
        for key in ("message", "error", "detail", "description"):
            if error.get(key):
                return str(error[key])[:500]
        return json.dumps(error, ensure_ascii=False)[:500]
    return str(error)[:500]


def normalize_provider(provider: Any) -> dict[str, Any] | None:
    if not isinstance(provider, dict):
        return None

    windows: list[dict[str, Any]] = []
    max_used: float | None = None
    for window in provider.get("windows") or []:
        if not isinstance(window, dict):
            continue
        try:
            used = float(window.get("usedPercent"))
            used = min(100.0, max(0.0, used))
        except (TypeError, ValueError):
            used = None
        try:
            remaining = float(window.get("remainingPercent"))
            remaining = min(100.0, max(0.0, remaining))
        except (TypeError, ValueError):
            remaining = None
        if used is not None and (max_used is None or used > max_used):
            max_used = used
        windows.append(
            {
                "kind": str(window.get("kind") or ""),
                "label": str(window.get("label") or window.get("kind") or "Usage"),
                "usedPercent": round(used, 1) if used is not None else None,
                "remainingPercent": round(remaining, 1) if remaining is not None else None,
                "resetAt": str(window.get("resetAt") or ""),
            }
        )

    identity = provider.get("identity") if isinstance(provider.get("identity"), dict) else {}
    status = provider.get("status") if isinstance(provider.get("status"), dict) else {}
    credits = provider.get("credits") if isinstance(provider.get("credits"), dict) else None
    cost = provider.get("cost") if isinstance(provider.get("cost"), dict) else None
    display = provider.get("display") if isinstance(provider.get("display"), dict) else {}
    err = error_text(provider.get("error"))
    remaining = None if max_used is None else 100.0 - max_used
    level = str(status.get("level") or "unknown")

    if err or level == "critical" or (remaining is not None and remaining <= 10):
        health = "critical"
    elif level == "warning" or (remaining is not None and remaining <= 25):
        health = "warning"
    else:
        health = "ok"

    try:
        sort_key = int(display.get("sortKey") or 0)
    except (TypeError, ValueError):
        sort_key = 0

    return {
        "id": str(provider.get("id") or "unknown"),
        "name": str(provider.get("name") or provider.get("id") or "Unknown"),
        "enabled": bool(provider.get("enabled", True)),
        "source": str(provider.get("source") or ""),
        "plan": str(identity.get("plan") or ""),
        "account": str(identity.get("accountEmail") or ""),
        "status": level,
        "statusLabel": str(status.get("label") or ""),
        "windows": windows,
        "maxUsedPercent": round(max_used, 1) if max_used is not None else None,
        "remainingPercent": round(remaining, 1) if remaining is not None else None,
        "credits": credits,
        "cost": cost,
        "sortKey": sort_key,
        "health": health,
        "error": err,
        "updatedAt": str(provider.get("updatedAt") or ""),
    }


def build_fresh() -> dict[str, Any]:
    raw = run_json(
        [
            "codexbar",
            "dashboard",
            "--identity",
            "redacted",
            "--timeout",
            str(CODEXBAR_TIMEOUT),
        ],
        CODEXBAR_TIMEOUT + 5,
    )

    providers: list[dict[str, Any]] = []
    for item in raw.get("providers", []) if isinstance(raw, dict) else []:
        provider = normalize_provider(item)
        if provider and provider["enabled"]:
            providers.append(provider)
    providers.sort(key=lambda item: (item["sortKey"], item["name"].lower()))

    try:
        agents = run_json(["vesper-agent-cockpit", "status"], 5)
    except Exception as exc:
        agents = {"count": 0, "class": "unknown", "tooltip": str(exc)[:300], "agents": []}

    try:
        hermes = run_json(["vesper-hermes", "status", "--json"], 8)
    except Exception as exc:
        hermes = {
            "count": 0,
            "unread": 0,
            "high": 0,
            "class": "unknown",
            "latestTitle": "",
            "tooltip": str(exc)[:300],
        }

    constrained = [p for p in providers if p.get("maxUsedPercent") is not None]
    worst = max(constrained, key=lambda p: float(p["maxUsedPercent"])) if constrained else None
    critical = sum(1 for p in providers if p["health"] == "critical")
    warning = sum(1 for p in providers if p["health"] == "warning")

    return {
        "schemaVersion": 1,
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "stale": False,
        "summary": {
            "providerCount": len(providers),
            "criticalCount": critical,
            "warningCount": warning,
            "maxUsedPercent": int(round(float(worst["maxUsedPercent"]))) if worst else -1,
            "maxProvider": worst["name"] if worst else "",
            "class": "critical" if critical else ("warning" if warning else "ok"),
        },
        "providers": providers,
        "agents": agents if isinstance(agents, dict) else {},
        "hermes": hermes if isinstance(hermes, dict) else {},
        "codexbar": {
            "version": str((raw.get("host") or {}).get("codexBarVersion") or "") if isinstance(raw, dict) else "",
            "generatedAt": str(raw.get("generatedAt") or "") if isinstance(raw, dict) else "",
        },
    }


def write_cache(value: dict[str, Any]) -> None:
    CACHE_ROOT.mkdir(parents=True, exist_ok=True)
    tmp = CACHE_PATH.with_name(f".{CACHE_PATH.name}.{os.getpid()}.tmp")
    tmp.write_text(json.dumps(value, ensure_ascii=False, separators=(",", ":")) + "\n")
    tmp.replace(CACHE_PATH)


def cache_age() -> float:
    try:
        return max(0.0, time.time() - CACHE_PATH.stat().st_mtime)
    except OSError:
        return float("inf")


def snapshot(force: bool = False, max_age: int = DEFAULT_MAX_AGE) -> dict[str, Any]:
    CACHE_ROOT.mkdir(parents=True, exist_ok=True)
    cached = load_json(CACHE_PATH)
    if not force and isinstance(cached, dict) and cache_age() <= max_age:
        return cached

    with LOCK_PATH.open("a+") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
        cached = load_json(CACHE_PATH)
        if not force and isinstance(cached, dict) and cache_age() <= max_age:
            return cached
        try:
            fresh = build_fresh()
            write_cache(fresh)
            return fresh
        except Exception as exc:
            if isinstance(cached, dict):
                fallback = dict(cached)
                fallback["stale"] = True
                fallback["backendError"] = str(exc)[:600]
                return fallback
            return {
                "schemaVersion": 1,
                "generatedAt": datetime.now(timezone.utc).isoformat(),
                "stale": True,
                "backendError": str(exc)[:600],
                "summary": {
                    "providerCount": 0,
                    "criticalCount": 0,
                    "warningCount": 0,
                    "maxUsedPercent": -1,
                    "maxProvider": "",
                    "class": "stale",
                },
                "providers": [],
                "agents": {"count": 0, "class": "unknown", "agents": []},
                "hermes": {"count": 0, "unread": 0, "high": 0, "class": "unknown"},
                "codexbar": {"version": "", "generatedAt": ""},
            }


def main() -> int:
    parser = argparse.ArgumentParser(prog="vesper-ai-hub")
    parser.add_argument("command", nargs="?", choices=("status", "refresh"), default="status")
    parser.add_argument("--max-age", type=int, default=DEFAULT_MAX_AGE)
    parser.add_argument("--pretty", action="store_true")
    args = parser.parse_args()

    payload = snapshot(force=args.command == "refresh", max_age=max(0, args.max_age))
    print(json.dumps(payload, ensure_ascii=False, indent=2 if args.pretty else None))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
