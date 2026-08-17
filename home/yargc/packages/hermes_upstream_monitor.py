from __future__ import annotations

import json
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, load_json, now

REPOSITORIES = [
    "NousResearch/hermes-agent",
    "numtide/llm-agents.nix",
    "NixOS/nixpkgs",
    "nix-community/home-manager",
    "hyprwm/Hyprland",
    "caelestia-dots/shell",
    "0xc000022070/zen-browser-flake",
    "schembriaiden/helium-browser-nix-flake",
    "alioguzhan/codexbar-flake",
    "monero-project/monero",
    "Cuprate/cuprate",
]


def _gh_json(path: str) -> dict[str, Any] | None:
    gh = shutil.which("gh")
    if not gh:
        return None
    try:
        proc = subprocess.run(
            [gh, "api", path],
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=10,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if proc.returncode != 0:
        return None
    try:
        value = json.loads(proc.stdout)
    except Exception:
        return None
    return value if isinstance(value, dict) else None


def _repo_head(repository: str) -> tuple[str, dict[str, Any] | None]:
    meta = _gh_json(f"repos/{repository}")
    if not meta:
        return repository, None
    branch = str(meta.get("default_branch") or "main")
    head = _gh_json(f"repos/{repository}/commits/{branch}")
    if not head:
        return repository, None
    commit = head.get("commit") or {}
    author = commit.get("author") or {}
    return repository, {
        "branch": branch,
        "sha": head.get("sha"),
        "date": author.get("date"),
        "message": str(commit.get("message") or "").splitlines()[0][:180],
    }


def upstream_change_gate() -> dict[str, Any]:
    state_path = STATE_ROOT / "watches" / "upstream-edge-radar.json"
    previous = load_json(state_path, {})
    previous_snapshot = previous.get("snapshot", {}) if isinstance(previous, dict) else {}
    if not isinstance(previous_snapshot, dict):
        previous_snapshot = {}

    observed: dict[str, dict[str, Any]] = {}
    with ThreadPoolExecutor(max_workers=6) as pool:
        futures = [pool.submit(_repo_head, repository) for repository in REPOSITORIES]
        for future in as_completed(futures):
            repository, value = future.result()
            if isinstance(value, dict) and value.get("sha"):
                observed[repository] = value

    minimum_healthy = max(3, len(REPOSITORIES) // 2)
    if len(observed) < minimum_healthy:
        return {
            "shouldResearch": False,
            "reason": "insufficient-upstream-snapshot",
            "observed": len(observed),
            "required": minimum_healthy,
        }

    changed = sorted(
        repository
        for repository, value in observed.items()
        if previous_snapshot.get(repository) != value
    )
    merged = dict(previous_snapshot)
    merged.update(observed)
    checked_at = now().isoformat(timespec="seconds")
    atomic_json(
        state_path,
        {
            "snapshot": merged,
            "changedRepositories": changed,
            "checkedAt": checked_at,
        },
    )
    atomic_json(
        STATE_ROOT / "upstream-edge-radar" / "monitor-change.json",
        {
            "changedRepositories": changed,
            "snapshot": {repository: observed[repository] for repository in changed},
            "checkedAt": checked_at,
        },
    )
    return {
        "shouldResearch": not previous_snapshot or bool(changed),
        "reason": "initial-baseline" if not previous_snapshot else ("changed" if changed else "unchanged"),
        "changedRepositories": changed,
        "observed": len(observed),
    }
