#!/usr/bin/env python3
"""Prune old Hermes cron outputs and ended cron-source sessions.

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
CUTOFF = time.time() - 30 * 86400


def main() -> int:
    errors: list[str] = []
    removed = 0
    output_root = HERMES_HOME / "cron" / "output"

    if output_root.exists():
        for path in output_root.rglob("*.md"):
            try:
                if path.stat().st_mtime < CUTOFF:
                    path.unlink()
                    removed += 1
            except Exception as exc:
                errors.append(f"{path}: {exc}")

        for directory in sorted(
            (path for path in output_root.rglob("*") if path.is_dir()),
            key=lambda path: len(path.parts),
            reverse=True,
        ):
            try:
                directory.rmdir()
            except OSError:
                pass

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
        print(f"removed old output files before failure: {removed}")
        for error in errors[:8]:
            print(f"- {error}")
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
