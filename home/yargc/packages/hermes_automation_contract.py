from __future__ import annotations

from typing import Any, Iterable


def _cron_fields(schedule: str) -> list[str]:
    return [field for field in schedule.strip().split() if field]


def validate_registry(
    registry: dict[str, dict[str, Any]],
    *,
    task_names: Iterable[str],
    watchdog_names: Iterable[str],
) -> list[str]:
    errors: list[str] = []
    tasks = set(task_names)
    watchdogs = set(watchdog_names)
    cron_names: set[str] = set()
    scripts: set[str] = set()

    if not registry:
        return ["registry is empty"]

    for name, spec in registry.items():
        if not name or not isinstance(spec, dict):
            errors.append(f"{name or '<empty>'}: invalid job spec")
            continue

        schedule = str(spec.get("schedule") or "").strip()
        if len(_cron_fields(schedule)) != 5:
            errors.append(f"{name}: schedule must be a 5-field cron expression")

        mode = str(spec.get("mode") or "dispatch")
        task = str(spec.get("task") or name)
        if mode == "dispatch":
            if task not in tasks:
                errors.append(f"{name}: unknown dispatch task {task}")
            if str(spec.get("deliver") or "local") != "local":
                errors.append(f"{name}: dispatch cron delivery must be local; worker owns final delivery")
        elif mode == "watchdog":
            if task not in watchdogs:
                errors.append(f"{name}: unknown watchdog task {task}")
            if str(spec.get("deliver") or "") == "local":
                errors.append(f"{name}: watchdog needs a non-local alert target")
        else:
            errors.append(f"{name}: unsupported mode {mode}")

        cron_name = str(spec.get("cronName") or f"vesper:{name}")
        if cron_name in cron_names:
            errors.append(f"{name}: duplicate cron name {cron_name}")
        cron_names.add(cron_name)

        script = str(spec.get("script") or f"vesper-{name}.sh")
        if script in scripts:
            errors.append(f"{name}: duplicate script {script}")
        scripts.add(script)

    return errors
