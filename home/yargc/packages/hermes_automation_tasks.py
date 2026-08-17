from __future__ import annotations

from typing import Any, Callable

from hermes_tasks_daily import DAILY_TASKS
from hermes_tasks_maintenance import MAINTENANCE_TASKS
from hermes_tasks_weekly import WEEKLY_TASKS

TASKS: dict[str, Callable[[], dict[str, Any]]] = (
    DAILY_TASKS | WEEKLY_TASKS | MAINTENANCE_TASKS
)


def run_task(name: str) -> dict[str, Any]:
    task = TASKS.get(name)
    if task is None:
        raise RuntimeError(f"unknown Hermes automation task: {name}")
    return task()
