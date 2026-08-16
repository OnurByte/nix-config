#!/usr/bin/env python3
"""Low-volume deterministic helpers for the Vesper Hermes automation fleet.

High-volume internet collection lives in research-collectors.py. This file is
installed under the health/integrity/project/usage entrypoint names and selects
behaviour from argv[0].
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HOME = Path.home()
HERMES_HOME = Path(os.environ.get("HERMES_HOME", HOME / ".hermes")).expanduser()
STATE_ROOT = Path(
    os.environ.get("VESPER_RESEARCH_STATE_DIR", HOME / ".local/state/vesper/research")
).expanduser()
STATE_ROOT.mkdir(parents=True, exist_ok=True)


def compact(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def run(cmd: list[str], timeout: int = 20) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )


def emit_change_alert(
    key: str,
    payload: Any,
    message: str,
    recovery: str | None = None,
) -> None:
    """Emit only when a watchdog state changes; remain silent on repeats."""
    state_file = STATE_ROOT / f"watch-{key}.json"
    encoded = compact(payload)
    digest = hashlib.sha256(encoded.encode()).hexdigest()
    previous: dict[str, Any] = {}
    try:
        previous = json.loads(state_file.read_text())
    except Exception:
        pass

    current_problem = bool(payload)
    previous_problem = bool(previous.get("problem"))
    previous_digest = previous.get("digest")

    state_file.write_text(
        json.dumps(
            {
                "digest": digest,
                "problem": current_problem,
                "updatedAt": datetime.now(timezone.utc).isoformat(),
            },
            indent=2,
        )
    )

    if current_problem and digest != previous_digest:
        print(message)
        print(json.dumps(payload, ensure_ascii=False, indent=2))
    elif not current_problem and previous_problem and recovery:
        print(recovery)


def health_watch() -> None:
    problems: dict[str, Any] = {}

    if shutil.which("vesper-doctor"):
        try:
            proc = run(["vesper-doctor", "--json"], timeout=45)
            if proc.returncode == 0:
                try:
                    report = json.loads(proc.stdout)
                    warnings = [
                        item
                        for item in report.get("checks", [])
                        if str(item.get("level", "")).lower() == "warn"
                    ]
                    if warnings:
                        problems["doctor"] = warnings
                except Exception as exc:
                    problems["doctorParse"] = str(exc)
            else:
                problems["doctorError"] = (proc.stderr or proc.stdout).strip()[-1000:]
        except Exception as exc:
            problems["doctorException"] = f"{type(exc).__name__}: {exc}"

    for scope, command in (
        ("system", ["systemctl", "--failed", "--no-legend", "--plain"]),
        ("user", ["systemctl", "--user", "--failed", "--no-legend", "--plain"]),
    ):
        try:
            proc = run(command, timeout=10)
            lines = [line for line in proc.stdout.splitlines() if line.strip()]
            if lines:
                problems[f"{scope}Units"] = lines[:20]
        except Exception as exc:
            problems[f"{scope}UnitsCheck"] = f"{type(exc).__name__}: {exc}"

    emit_change_alert(
        "health",
        problems,
        "Vesper Health Watch detected a new or changed problem:",
        "Vesper Health Watch: previously reported problems are clear.",
    )


def installed_skill_names() -> set[str]:
    """Discover skills through nested and symlinked skill trees safely."""
    names: set[str] = set()
    seen_dirs: set[tuple[int, int]] = set()

    for root in (HERMES_HOME / "skills", HOME / ".agents" / "skills"):
        if not root.exists():
            continue
        root = root.resolve()
        root_depth = len(root.parts)
        for current, dirs, files in os.walk(root, followlinks=True):
            path = Path(current)
            try:
                stat = path.stat()
                identity = (stat.st_dev, stat.st_ino)
            except OSError:
                dirs[:] = []
                continue

            if identity in seen_dirs:
                dirs[:] = []
                continue
            seen_dirs.add(identity)

            if len(path.parts) - root_depth > 4:
                dirs[:] = []
                continue

            if "SKILL.md" not in files:
                continue

            skill_md = path / "SKILL.md"
            names.add(path.name)
            try:
                for line in skill_md.read_text(errors="replace").splitlines()[:40]:
                    if line.startswith("name:"):
                        names.add(line.split(":", 1)[1].strip().strip('"\''))
                        break
            except Exception:
                pass

    return names


def skill_integrity_watch() -> None:
    jobs_file = HERMES_HOME / "cron" / "jobs.json"
    if not jobs_file.exists():
        return

    try:
        jobs = json.loads(jobs_file.read_text()).get("jobs", [])
    except Exception as exc:
        emit_change_alert(
            "skills",
            {"jobsJson": f"{type(exc).__name__}: {exc}"},
            "Hermes Skill Integrity Watch could not read cron state:",
        )
        return

    installed = installed_skill_names()
    missing: dict[str, list[str]] = {}

    for job in jobs:
        if not job.get("enabled", True):
            continue
        wanted = job.get("skills") or ([job.get("skill")] if job.get("skill") else [])
        absent = [skill for skill in wanted if skill and skill not in installed]
        if absent:
            missing[job.get("name") or job.get("id") or "unknown"] = absent

    emit_change_alert(
        "skills",
        missing,
        "Hermes Skill Integrity Watch found missing skills:",
        "Hermes Skill Integrity Watch: previously missing skills are available again.",
    )


def discover_git_repositories(limit: int = 80) -> list[Path]:
    roots = [HOME / name for name in ("Documents", "Projects", "Code", "src", "Source")]
    excluded = {
        "node_modules",
        "vendor",
        "target",
        ".venv",
        "venv",
        "dist",
        "build",
        ".next",
        ".cache",
        ".direnv",
    }
    repos: list[Path] = []
    seen: set[Path] = set()

    for root in roots:
        if not root.exists():
            continue
        root_depth = len(root.parts)
        for current, dirs, files in os.walk(root):
            path = Path(current)
            depth = len(path.parts) - root_depth
            if depth > 4:
                dirs[:] = []
                continue

            # Detect repository ownership before pruning .git from traversal.
            is_repo = ".git" in dirs or ".git" in files
            if is_repo:
                try:
                    resolved = path.resolve()
                except OSError:
                    resolved = path
                if resolved not in seen:
                    repos.append(resolved)
                    seen.add(resolved)
                if len(repos) >= limit:
                    return repos

            dirs[:] = [
                directory
                for directory in dirs
                if directory != ".git"
                and directory not in excluded
                and not directory.startswith(".direnv")
            ]

    return repos


def project_inventory() -> None:
    result: list[dict[str, Any]] = []

    for repo in discover_git_repositories():
        def git(*args: str) -> str:
            try:
                proc = run(["git", "-C", str(repo), *args], timeout=8)
            except Exception:
                return ""
            return proc.stdout.strip() if proc.returncode == 0 else ""

        status = git("status", "--porcelain=v1")
        todo_files = [
            candidate
            for candidate in ("TODO.md", "TODOS.md", "PROGRESS.md", "PLAN.md", "ROADMAP.md")
            if (repo / candidate).exists()
        ]
        result.append(
            {
                "path": str(repo),
                "branch": git("branch", "--show-current"),
                "dirtyFiles": len([line for line in status.splitlines() if line.strip()]),
                "recent": git("log", "-1", "--format=%h %cs %s"),
                "remote": git("remote", "get-url", "origin"),
                "todoFiles": todo_files,
            }
        )

    print(compact({"generatedAt": datetime.now(timezone.utc).isoformat(), "repos": result}))


def usage_snapshot() -> None:
    commands = [
        ["turnlens", "report", "weekly"],
        ["ccusage", "weekly"],
        ["codexbar", "cards"],
    ]
    data: list[dict[str, Any]] = []

    for command in commands:
        if not shutil.which(command[0]):
            continue
        try:
            proc = run(command, timeout=30)
            data.append(
                {
                    "command": " ".join(command),
                    "returncode": proc.returncode,
                    "stdout": proc.stdout[-12000:],
                    "stderr": proc.stderr[-2000:],
                }
            )
        except Exception as exc:
            data.append({"command": " ".join(command), "error": str(exc)})

    print(compact({"generatedAt": datetime.now(timezone.utc).isoformat(), "usage": data}))


def main() -> None:
    mode = Path(sys.argv[0]).stem
    handlers = {
        "vesper-health-watch": health_watch,
        "vesper-skill-integrity-watch": skill_integrity_watch,
        "project-inventory": project_inventory,
        "ai-usage-snapshot": usage_snapshot,
    }
    handler = handlers.get(mode)
    if handler is None:
        raise SystemExit(f"Unsupported automation-support mode: {mode}")
    handler()


if __name__ == "__main__":
    main()
