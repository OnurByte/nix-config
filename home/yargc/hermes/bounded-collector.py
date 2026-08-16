#!/usr/bin/env python3
"""Run a wide Vesper collector, persist the full pool, emit bounded valid JSON.

Installed under each high-volume collector filename. The wrapper preserves a
large discovery funnel without dumping the entire corpus into the agent prompt.
"""

from __future__ import annotations

import contextlib
import io
import json
import os
import runpy
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HOME = Path.home()
STATE_ROOT = Path(
    os.environ.get("VESPER_RESEARCH_STATE_DIR", HOME / ".local/state/vesper/research")
).expanduser()
POOL_ROOT = STATE_ROOT / "candidate-pools"
POOL_ROOT.mkdir(parents=True, exist_ok=True)
MAX_PROMPT_CHARS = 80_000
LIST_KEYS = ("repoCandidates", "issueCandidates", "candidates", "topics", "posts")


def item_count(payload: dict[str, Any]) -> dict[str, int]:
    return {
        key: len(payload.get(key) or [])
        for key in LIST_KEYS
        if isinstance(payload.get(key), list)
    }


def bounded_payload(payload: dict[str, Any], pool_path: Path) -> str:
    counts = item_count(payload)
    result = dict(payload)
    result["fullPoolPath"] = str(pool_path)
    result["fullPoolCounts"] = counts
    result["note"] = (
        "The complete deterministic candidate pool is stored at fullPoolPath. "
        "This prompt payload is a bounded sample; inspect the full file only when needed."
    )

    def encode() -> str:
        return json.dumps(result, ensure_ascii=False, separators=(",", ":"))

    encoded = encode()
    while len(encoded) > MAX_PROMPT_CHARS:
        populated = [
            key
            for key in LIST_KEYS
            if isinstance(result.get(key), list) and len(result[key]) > 1
        ]
        if not populated:
            break
        largest = max(populated, key=lambda key: len(result[key]))
        current = result[largest]
        result[largest] = current[: max(1, len(current) * 3 // 4)]
        encoded = encode()

    if len(encoded) > MAX_PROMPT_CHARS:
        # Last-resort valid JSON. The full pool remains available on disk.
        result = {
            "source": payload.get("source"),
            "generatedAt": payload.get("generatedAt"),
            "fullPoolPath": str(pool_path),
            "fullPoolCounts": counts,
            "errors": payload.get("errors", []),
            "note": "Candidate sample omitted because metadata alone exceeded the prompt budget.",
        }
        encoded = json.dumps(result, ensure_ascii=False, separators=(",", ":"))

    return encoded


def main() -> int:
    mode = Path(sys.argv[0]).stem
    support = Path(__file__).with_name("_vesper-automation-support.py")
    if not support.exists():
        print(json.dumps({"error": f"collector support missing: {support}"}))
        return 1

    buffer = io.StringIO()
    original_argv = list(sys.argv)
    exit_code = 0
    try:
        # automation-support chooses its handler from argv[0].
        sys.argv = [mode]
        with contextlib.redirect_stdout(buffer):
            try:
                runpy.run_path(str(support), run_name="__main__")
            except SystemExit as exc:
                if isinstance(exc.code, int):
                    exit_code = exc.code
                elif exc.code:
                    exit_code = 1
    finally:
        sys.argv = original_argv

    raw = buffer.getvalue().strip()
    if exit_code != 0:
        if raw:
            print(raw)
        return exit_code

    try:
        payload = json.loads(raw)
    except Exception as exc:
        print(
            json.dumps(
                {
                    "error": f"collector produced invalid JSON: {type(exc).__name__}: {exc}",
                    "preview": raw[:4000],
                },
                ensure_ascii=False,
            )
        )
        return 1

    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    pool_path = POOL_ROOT / f"{mode}-{stamp}.json"
    pool_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2))
    print(bounded_payload(payload, pool_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
