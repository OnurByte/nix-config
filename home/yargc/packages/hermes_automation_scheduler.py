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

WATCHDOG_TASKS = {"vesper-health-watch", "cron-skill-integrity-watch"}


def slug(value: str) -> str:
    return "".join(ch if ch.isalnum() or ch == "-" else "-" for ch in value.lower()).strip("-")[:50]


def runtime_binary() -> str:
    return os.environ.get("VESPER_HERMES_AUTOMATION_BIN") or shutil.which("vesper-hermes-automations") or str(Path(__file__).with_name("hermes-automations.py"))


def dispatch_job(name: str) -> None:
    registry = load_registry()
    if name not in registry and name not in {"unknown-frontier-github", "unknown-frontier-reddit", "unknown-frontier-x", "unknown-frontier-synthesis", "frontier-daily"}:
        raise RuntimeError(f"unknown Hermes job: {name}")
    binary = shutil.which("systemd-run")
    if not binary:
        raise RuntimeError("systemd-run is not available")
    unit = f"vesper-hermes-{slug(name)}-{int(time.time())}-{os.getpid()}"
    try:
        completed = subprocess.run([
            binary, "--user", "--no-block", "--collect", "--quiet", f"--unit={unit}",
            "--property=Nice=10", "--property=IOSchedulingClass=best-effort", "--property=KillMode=mixed",
            runtime_binary(), "execute", name,
        ], text=True, capture_output=True, timeout=30, check=False)
    except subprocess.TimeoutExpired as exc:
        raise RuntimeError(f"timed out dispatching {name}") from exc
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


def _command_output(argv: list[str], timeout: int = 20) -> tuple[int, str]:
    try:
        completed = subprocess.run(argv, text=True, capture_output=True, timeout=timeout, check=False)
    except (OSError, subprocess.TimeoutExpired):
        return 124, ""
    return completed.returncode, ((completed.stdout or "") + "\n" + (completed.stderr or "")).strip()


def _failed_units(scope: str) -> list[str]:
    argv = ["systemctl"]
    if scope == "user":
        argv.append("--user")
    argv.extend(["--failed", "--no-legend", "--plain"])
    rc, text = _command_output(argv)
    if rc != 0 or not text:
        return []
    return [line.strip() for line in text.splitlines() if line.strip() and "0 loaded units listed" not in line][:8]


def _restic_timer_state() -> list[str]:
    found: list[str] = []
    for scope in ("user", "system"):
        argv = ["systemctl"]
        if scope == "user":
            argv.append("--user")
        argv.extend(["list-timers", "--all", "--no-legend", "--plain"])
        rc, text = _command_output(argv)
        if rc != 0:
            continue
        for line in text.splitlines():
            if "restic" in line.lower():
                found.append(f"{scope}: {line.strip()}")
    return found[:8]


def _health_watch() -> str:
    problems: list[str] = []
    doctor = shutil.which("vesper-doctor")
    if not doctor:
        problems.append("vesper-doctor is not available")
    else:
        try:
            completed = subprocess.run([doctor, "--json"], text=True, capture_output=True, timeout=90, check=False)
        except subprocess.TimeoutExpired:
            completed = subprocess.CompletedProcess([doctor, "--json"], 124, "", "timed out")
        if completed.returncode != 0:
            problems.append(f"vesper-doctor failed rc={completed.returncode}: {(completed.stderr or completed.stdout)[-1200:]}")
        else:
            payload = extract_json_relaxed(completed.stdout)
            if not isinstance(payload, dict):
                problems.append("could not parse vesper-doctor JSON")
            elif payload.get("healthy") is not True:
                checks = payload.get("checks", [])
                messages = [
                    str(check.get("message") or check.get("key") or "health check warning")
                    for check in checks
                    if isinstance(check, dict) and str(check.get("level") or "").lower() in {"warn", "warning", "error", "critical"}
                ]
                problems.extend(messages[:10] or ["vesper-doctor reports unhealthy state"])

    for scope in ("user", "system"):
        units = _failed_units(scope)
        if units:
            problems.append(f"{scope} failed units: " + " | ".join(units))

    threshold = max(1, min(99, int(os.environ.get("VESPER_DISK_ALERT_PERCENT", "90"))))
    checked: set[str] = set()
    for path in (Path("/"), Path.home()):
        try:
            resolved = str(path.resolve())
            if resolved in checked:
                continue
            checked.add(resolved)
            usage = shutil.disk_usage(path)
            pct = int(round((usage.used / usage.total) * 100)) if usage.total else 0
            if pct >= threshold:
                problems.append(f"disk {path}: {pct}% used (threshold {threshold}%)")
        except OSError:
            continue

    restic_timers = _restic_timer_state()
    if restic_timers and problems:
        problems.append("restic timers: " + " | ".join(restic_timers))

    return "" if not problems else "[Hermes health]\n" + "\n".join(f"- {item}" for item in problems[:20])


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
    by_name: dict[str, dict[str, Any]] = {}
    duplicate_names: set[str] = set()
    for job in jobs:
        name = str(job.get("name") or "")
        if not name:
            continue
        if name in by_name:
            duplicate_names.add(name)
        by_name[name] = job
    for name in sorted(duplicate_names):
        problems.append(f"duplicate job name {name}")

    for short_name, spec in registry.items():
        desired_name = str(spec.get("cronName") or f"vesper:{short_name}")
        job = by_name.get(desired_name)
        if not job:
            problems.append(f"missing job {desired_name}")
            continue
        expected_enabled = bool(spec.get("enabled", True))
        actual_enabled = bool(job.get("enabled", True))
        if actual_enabled != expected_enabled:
            state = "enabled" if actual_enabled else "disabled"
            expected = "enabled" if expected_enabled else "disabled"
            problems.append(f"state drift {desired_name}: {state}, expected {expected}")
        expected_schedule = str(spec.get("schedule") or "")
        schedule = job.get("schedule") or {}
        actual_schedule = str(job.get("schedule_display") or (schedule.get("display") if isinstance(schedule, dict) else "") or (schedule.get("value") if isinstance(schedule, dict) else schedule) or "")
        if expected_schedule and actual_schedule and expected_schedule != actual_schedule:
            problems.append(f"schedule drift {desired_name}: {actual_schedule!r}")
        expected_script = str(spec.get("script") or f"vesper-{short_name}.sh")
        actual_script = str(job.get("script") or "")
        if expected_script and expected_script not in actual_script:
            problems.append(f"script drift {desired_name}: {actual_script!r}")
        if job.get("no_agent") is not True:
            problems.append(f"mode drift {desired_name}: expected no_agent=true")

    roots = [HERMES_HOME / "skills", HERMES_HOME / "skills" / "vesper", Path.home() / ".agents" / "skills"]
    for job in jobs:
        skills = job.get("skills") or ([job["skill"]] if job.get("skill") else [])
        if not isinstance(skills, list):
            continue
        for skill in skills:
            name = str(skill)
            if not any((root / name).exists() for root in roots):
                problems.append(f"missing skill {name} referenced by {job.get('name') or job.get('id')}")

    status = _run([hermes_bin(), "cron", "status"], timeout=30)
    text = (status.stdout or "") + (status.stderr or "")
    if status.returncode != 0 or "will NOT fire" in text or "STALLED" in text:
        problems.append("Hermes cron scheduler/gateway is unhealthy")
    return "" if not problems else "[Hermes cron integrity]\n" + "\n".join(f"- {item}" for item in problems[:20])


def run_watchdog(name: str) -> str:
    if name == "vesper-health-watch":
        return _health_watch()
    if name in {"cron-skill-integrity-watch", "cron-integrity-watch"}:
        return _cron_integrity_watch()
    raise RuntimeError(f"unknown watchdog: {name}")


def _run(argv: list[str], timeout: int = 45) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(argv, text=True, capture_output=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout if isinstance(exc.stdout, str) else ""
        stderr = exc.stderr if isinstance(exc.stderr, str) else ""
        return subprocess.CompletedProcess(argv, 124, stdout, (stderr + "\ncommand timed out").strip())


def cron_edit_argv(hermes: str, ref: str, cron_name: str, schedule: str, prompt: str, deliver: str, script_path: Path) -> list[str]:
    return [
        hermes, "cron", "edit", ref,
        "--name", cron_name,
        "--schedule", schedule,
        "--prompt", prompt,
        "--deliver", deliver,
        "--script", str(script_path),
        "--no-agent",
    ]


def cron_create_argv(hermes: str, cron_name: str, schedule: str, prompt: str, deliver: str, script_path: Path) -> list[str]:
    return [
        hermes, "cron", "create", schedule, prompt,
        "--name", cron_name,
        "--deliver", deliver,
        "--script", str(script_path),
        "--no-agent",
    ]


def _reconcile_enabled(hermes: str, ref: str, desired_enabled: bool, currently_enabled: bool) -> tuple[bool, str]:
    if desired_enabled == currently_enabled:
        return True, ""
    action = "resume" if desired_enabled else "pause"
    result = _run([hermes, "cron", action, ref])
    if result.returncode == 0:
        return True, action
    detail = (result.stderr or result.stdout).strip()[-3000:]
    return False, f"{action} failed: {detail}"


def sync_cron(prune: bool = False) -> dict[str, Any]:
    registry = load_registry()
    hermes = hermes_bin()
    _, jobs = jobs_store()
    by_name = {str(job.get("name")): job for job in jobs if str(job.get("name") or "")}
    created: list[str] = []
    updated: list[str] = []
    resumed: list[str] = []
    paused: list[str] = []
    removed: list[str] = []
    errors: list[str] = []

    for short_name, spec in registry.items():
        cron_name = str(spec.get("cronName") or f"vesper:{short_name}")
        schedule = str(spec.get("schedule") or "")
        prompt = str(spec.get("prompt") or "Run the declarative Vesper Hermes trigger.")
        deliver = str(spec.get("deliver") or "local")
        desired_enabled = bool(spec.get("enabled", True))
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
            result = _run(cron_edit_argv(hermes, ref, cron_name, schedule, prompt, deliver, script_path))
            if result.returncode != 0:
                errors.append(f"{short_name}: edit failed: {(result.stderr or result.stdout).strip()[-3000:]}")
                continue
            updated.append(short_name)
            ok, action = _reconcile_enabled(hermes, ref, desired_enabled, bool(existing.get("enabled", True)))
            if not ok:
                errors.append(f"{short_name}: {action}")
            elif action == "resume":
                resumed.append(short_name)
            elif action == "pause":
                paused.append(short_name)
        else:
            result = _run(cron_create_argv(hermes, cron_name, schedule, prompt, deliver, script_path))
            if result.returncode != 0:
                errors.append(f"{short_name}: create failed: {(result.stderr or result.stdout).strip()[-3000:]}")
                continue
            created.append(short_name)
            if not desired_enabled:
                ok, action = _reconcile_enabled(hermes, cron_name, False, True)
                if not ok:
                    errors.append(f"{short_name}: {action}")
                elif action == "pause":
                    paused.append(short_name)

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

    return {
        "created": created,
        "updated": updated,
        "resumed": resumed,
        "paused": paused,
        "removed": removed,
        "errors": errors,
        "ok": not errors,
    }
