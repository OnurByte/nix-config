from __future__ import annotations

import fcntl
import os
import shutil
import subprocess
import time
from pathlib import Path
from typing import Any

from hermes_automation_common import (
    HERMES_HOME,
    STATE_ROOT,
    atomic_json,
    extract_json_relaxed,
    hermes_bin,
    load_json,
    load_registry,
    now,
)


def slug(value: str) -> str:
    return "".join(ch if ch.isalnum() or ch == "-" else "-" for ch in value.lower()).strip("-")[:50]


def runtime_binary() -> str:
    return os.environ.get("VESPER_HERMES_AUTOMATION_BIN") or shutil.which("vesper-hermes-automations") or str(Path(__file__).with_name("hermes-automations.py"))


def dispatch_job(name: str) -> None:
    registry = load_registry()
    if name not in registry:
        raise RuntimeError(f"unknown Hermes job: {name}")
    binary = shutil.which("systemd-run")
    if not binary:
        raise RuntimeError("systemd-run is not available")
    unit = f"vesper-hermes-{slug(name)}-{int(time.time())}-{os.getpid()}"
    completed = subprocess.run([
        binary, "--user", "--no-block", "--collect", "--quiet", f"--unit={unit}",
        "--property=Nice=10", "--property=IOSchedulingClass=best-effort", "--property=KillMode=mixed",
        runtime_binary(), "execute", name,
    ], text=True, capture_output=True, check=False)
    if completed.returncode != 0:
        raise RuntimeError(f"failed to dispatch {name}: {(completed.stderr or completed.stdout).strip()}")


def job_lock(name: str):
    lock_dir = STATE_ROOT / "locks"
    lock_dir.mkdir(parents=True, exist_ok=True)
    handle = (lock_dir / f"{name}.lock").open("w")
    try:
        fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError as exc:
        handle.close()
        raise RuntimeError(f"Hermes job already running: {name}") from exc
    return handle


def record_run(name: str, status: str, started, *, error: str = "") -> None:
    finished = now()
    payload = {
        "job": name,
        "status": status,
        "startedAt": started.isoformat(timespec="seconds"),
        "finishedAt": finished.isoformat(timespec="seconds"),
        "durationSeconds": round((finished - started).total_seconds(), 3),
        "error": error[-12000:] if error else "",
    }
    path = STATE_ROOT / "runs" / name / f"{finished.strftime('%Y%m%dT%H%M%S')}.json"
    atomic_json(path, payload)
    atomic_json(STATE_ROOT / "runs" / name / "latest.json", payload)


def _health_watch() -> str:
    doctor = shutil.which("vesper-doctor")
    if not doctor:
        return "[Hermes health] vesper-doctor is not available"
    completed = subprocess.run([doctor, "--json"], text=True, capture_output=True, timeout=90, check=False)
    if completed.returncode != 0:
        return f"[Hermes health] vesper-doctor failed rc={completed.returncode}\n{(completed.stderr or completed.stdout)[-3000:]}"
    payload = extract_json_relaxed(completed.stdout)
    if not isinstance(payload, dict):
        return "[Hermes health] could not parse vesper-doctor JSON"
    if payload.get("healthy") is True:
        return ""
    warnings = [str(check.get("message") or check.get("key") or "warning") for check in payload.get("checks", []) if isinstance(check, dict) and check.get("level") == "warn"]
    return "" if not warnings else "[Hermes health]\n" + "\n".join(f"- {item}" for item in warnings[:12])


def jobs_store() -> tuple[Path, list[dict[str, Any]]]:
    path = HERMES_HOME / "cron" / "jobs.json"
    value = load_json(path, [])
    jobs = value.get("jobs", []) if isinstance(value, dict) else value
    if not isinstance(jobs, list):
        jobs = []
    return path, [job for job in jobs if isinstance(job, dict)]


def _cron_integrity_watch() -> str:
    registry = load_registry()
    jobs_path, jobs = jobs_store()
    if not jobs_path.exists():
        return f"[Hermes cron integrity] jobs store missing: {jobs_path}"
    problems: list[str] = []
    by_name = {str(job.get("name")): job for job in jobs}
    for short_name, spec in registry.items():
        desired_name = str(spec.get("cronName") or f"vesper:{short_name}")
        job = by_name.get(desired_name)
        if not job:
            problems.append(f"missing job {desired_name}")
            continue
        if not job.get("enabled", True):
            problems.append(f"job disabled {desired_name}")
        expected_schedule = str(spec.get("schedule") or "")
        schedule = job.get("schedule") or {}
        actual_schedule = str(job.get("schedule_display") or (schedule.get("display") if isinstance(schedule, dict) else "") or (schedule.get("value") if isinstance(schedule, dict) else schedule) or "")
        if expected_schedule and actual_schedule and expected_schedule != actual_schedule:
            problems.append(f"schedule drift {desired_name}: {actual_schedule!r}")
        expected_script = str(spec.get("script") or f"vesper-{short_name}.sh")
        actual_script = str(job.get("script") or "")
        if expected_script and expected_script not in actual_script:
            problems.append(f"script drift {desired_name}: {actual_script!r}")

    roots = [HERMES_HOME / "skills", HERMES_HOME / "skills" / "vesper", Path.home() / ".agents" / "skills"]
    for job in jobs:
        skills = job.get("skills") or ([job["skill"]] if job.get("skill") else [])
        if not isinstance(skills, list):
            continue
        for skill in skills:
            name = str(skill)
            if not any((root / name).exists() for root in roots):
                problems.append(f"missing skill {name} referenced by {job.get('name') or job.get('id')}")

    status = subprocess.run([hermes_bin(), "cron", "status"], text=True, capture_output=True, timeout=30, check=False)
    text = (status.stdout or "") + (status.stderr or "")
    if status.returncode != 0 or "will NOT fire" in text or "STALLED" in text:
        problems.append("Hermes cron scheduler/gateway is unhealthy")
    return "" if not problems else "[Hermes cron integrity]\n" + "\n".join(f"- {item}" for item in problems[:20])


def run_watchdog(name: str) -> str:
    if name == "vesper-health-watch":
        return _health_watch()
    if name == "cron-integrity-watch":
        return _cron_integrity_watch()
    raise RuntimeError(f"unknown watchdog: {name}")


def _run(argv: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, text=True, capture_output=True, check=False)


def sync_cron(prune: bool = False) -> dict[str, Any]:
    registry = load_registry()
    hermes = hermes_bin()
    _, jobs = jobs_store()
    by_name = {str(job.get("name")): job for job in jobs if str(job.get("name") or "")}
    created: list[str] = []
    updated: list[str] = []
    removed: list[str] = []
    errors: list[str] = []

    for short_name, spec in registry.items():
        cron_name = str(spec.get("cronName") or f"vesper:{short_name}")
        schedule = str(spec.get("schedule") or "")
        prompt = str(spec.get("prompt") or "Run the declarative Vesper Hermes trigger.")
        deliver = str(spec.get("deliver") or "local")
        script = str(spec.get("script") or f"vesper-{short_name}.sh")
        script_path = HERMES_HOME / "scripts" / script
        existing = by_name.get(cron_name)

        if existing is None and short_name == "morning-check":
            for candidate in jobs:
                candidate_script = Path(str(candidate.get("script") or "")).name
                candidate_name = str(candidate.get("name") or "").lower()
                if candidate_script in {"sabah-check-deliver.sh", "morning-check-deliver.sh"} or candidate_name in {"sabah check", "sabah-check", "morning check", "morning-check"}:
                    existing = candidate
                    break

        if not schedule:
            errors.append(f"{short_name}: missing schedule")
            continue
        if not script_path.exists():
            errors.append(f"{short_name}: script missing at {script_path}")
            continue

        if existing:
            ref = str(existing.get("id") or cron_name)
            argv = [hermes, "cron", "edit", ref, "--name", cron_name, "--schedule", schedule, "--prompt", prompt, "--deliver", deliver, "--script", str(script_path), "--no-agent"]
            result = _run(argv)
            if result.returncode == 0:
                updated.append(short_name)
            else:
                errors.append(f"{short_name}: edit failed: {(result.stderr or result.stdout).strip()[-3000:]}")
        else:
            argv = [hermes, "cron", "create", schedule, prompt, "--name", cron_name, "--deliver", deliver, "--script", str(script_path), "--no-agent"]
            result = _run(argv)
            if result.returncode == 0:
                created.append(short_name)
            else:
                errors.append(f"{short_name}: create failed: {(result.stderr or result.stdout).strip()[-3000:]}")

    if prune:
        desired = {str(spec.get("cronName") or f"vesper:{name}") for name, spec in registry.items()}
        for job in jobs:
            name = str(job.get("name") or "")
            if not name.startswith("vesper:") or name in desired:
                continue
            result = _run([hermes, "cron", "remove", str(job.get("id") or name)])
            if result.returncode == 0:
                removed.append(name)
            else:
                errors.append(f"{name}: remove failed: {(result.stderr or result.stdout).strip()[-3000:]}")

    return {"created": created, "updated": updated, "removed": removed, "errors": errors, "ok": not errors}
