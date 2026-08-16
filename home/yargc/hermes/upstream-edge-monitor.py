#!/usr/bin/env python3
"""Emit a stable upstream snapshot for Hermes `monitor_script` mode.

Hermes hashes this script's exact stdout. Byte-identical output suppresses the
agent run; changed output is diffed and injected automatically by the scheduler.
No timestamps or local state are included here on purpose.
"""

from __future__ import annotations

import json
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed

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


def gh_api(path: str) -> dict:
    proc = subprocess.run(
        ["gh", "api", path],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=10,
        check=False,
    )
    if proc.returncode != 0:
        return {"error": proc.stderr.strip()[-240:] or f"gh exited {proc.returncode}"}
    try:
        return json.loads(proc.stdout)
    except Exception as exc:
        return {"error": f"invalid JSON: {type(exc).__name__}: {exc}"}


def repository_snapshot(repository: str) -> tuple[str, dict]:
    meta = gh_api(f"repos/{repository}")
    if meta.get("error"):
        return repository, meta

    branch = meta.get("default_branch") or "main"
    ref = gh_api(f"repos/{repository}/commits/{branch}")
    if ref.get("error"):
        return repository, {"defaultBranch": branch, **ref}

    commit = ref.get("commit") or {}
    author = commit.get("author") or {}
    return repository, {
        "defaultBranch": branch,
        "sha": ref.get("sha"),
        "date": author.get("date"),
        "message": str(commit.get("message") or "").splitlines()[0][:180],
    }


def main() -> int:
    if not shutil.which("gh"):
        # Stable error output means Hermes wakes once when this condition begins,
        # then remains quiet until the monitor state changes again.
        print('{"error":"gh executable not found"}')
        return 0

    snapshot: dict[str, dict] = {}
    with ThreadPoolExecutor(max_workers=6) as pool:
        futures = [pool.submit(repository_snapshot, repository) for repository in REPOSITORIES]
        for future in as_completed(futures):
            try:
                repository, value = future.result()
            except Exception as exc:
                repository = "collector-error"
                value = {"error": f"{type(exc).__name__}: {exc}"}
            snapshot[repository] = value

    print(json.dumps(snapshot, ensure_ascii=False, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
