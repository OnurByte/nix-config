#!/usr/bin/env python3
"""Prune old Hermes cron outputs, candidate pools and ended cron sessions.

Silent on success so the no-agent cron job only notifies on an error.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import time
from pathlib import Path

HOME = Path.home()
HERMES_HOME = Path(os.environ.get("HERMES_HOME", HOME / ".hermes")).expanduser()
RESEARCH_ROOT = Path(
    os.environ.get("VESPER_RESEARCH_STATE_DIR", HOME / ".local/state/vesper/research")
).expanduser()
CUTOFF = time.time() - 30 * 86400


def prune_files(root: Path, pattern: str, errors: list[str]) -> int:
    removed = 0
    if not root.exists():
        return removed
    for path in root.rglob(pattern):
        if not path.is_file():
            continue
        try:
            if path.stat().st_mtime < CUTOFF:
                path.unlink()
                removed += 1
        except Exception as exc:
            errors.append(f"{path}: {exc}")
    for directory in sorted(
        (path for path in root.rglob("*") if path.is_dir()),
        key=lambda path: len(path.parts),
        reverse=True,
    ):
        try:
            directory.rmdir()
        except OSError:
            pass
    return removed


def main() -> int:
    errors: list[str] = []
    removed_outputs = prune_files(HERMES_HOME / "cron" / "output", "*.md", errors)
    removed_pools = prune_files(RESEARCH_ROOT / "candidate-pools", "*.json", errors)

    hermes = shutil.which("hermes")
    if not hermes:
        errors.append("hermes executable not found")
    else:
        try:
            proc = subprocess.run(
                [
                    hermes,
                    "sessions",
                    "prune",
                    "--older-than",
                    "30",
                    "--source",
                    "cron",
                    "--yes",
                ],
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=90,
                check=False,
            )
            if proc.returncode != 0:
                errors.append((proc.stderr or proc.stdout).strip()[-1200:])
        except Exception as exc:
            errors.append(f"sessions prune: {type(exc).__name__}: {exc}")

    if errors:
        print("Hermes Cron Retention encountered errors:")
        print(f"removed outputs: {removed_outputs}; removed candidate pools: {removed_pools}")
        for error in errors[:8]:
            print(f"- {error}")
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
