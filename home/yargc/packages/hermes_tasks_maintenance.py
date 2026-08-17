from __future__ import annotations

import shutil
import subprocess
import time
from pathlib import Path
from typing import Any

from hermes_automation_common import HERMES_HOME, STATE_ROOT, hermes_bin

DAY = 86400


def _prune_tree(root: Path, *, older_than_days: int) -> int:
    if not root.exists():
        return 0
    cutoff = time.time() - older_than_days * DAY
    removed = 0
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        try:
            if path.stat().st_mtime < cutoff:
                path.unlink()
                removed += 1
        except OSError:
            pass
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


def cron_retention() -> dict[str, Any]:
    """Bound ephemeral cron/research storage without deleting durable briefings."""
    removed_output = _prune_tree(HERMES_HOME / "cron" / "output", older_than_days=30)
    removed_pools = _prune_tree(STATE_ROOT / "candidate-pools", older_than_days=30)
    # Detailed worker run records are operational telemetry, not long-term
    # knowledge. Keep a longer horizon than candidate pools for debugging.
    removed_runs = _prune_tree(STATE_ROOT / "runs", older_than_days=90)

    command = [
        hermes_bin(),
        "sessions",
        "prune",
        "--older-than",
        "30",
        "--source",
        "cron",
        "--yes",
    ]
    completed = subprocess.run(
        command,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=120,
        check=False,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout).strip()[-4000:]
        raise RuntimeError(f"Hermes cron session prune failed (rc={completed.returncode}): {detail}")

    return {
        "maintenance": "cron-retention",
        "removedCronOutputFiles": removed_output,
        "removedCandidatePoolFiles": removed_pools,
        "removedOldRunRecords": removed_runs,
        "sessionPrune": "ok",
    }


MAINTENANCE_TASKS = {
    "cron-retention": cron_retention,
}
