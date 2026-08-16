#!/usr/bin/env python3
"""Cheap pre-run gate for the Upstream Edge Radar.

Track upstream repository heads in Vesper research state. Unchanged state emits
wakeAgent=false, so Hermes spends zero model tokens for that tick. Changed
state is passed as structured context to the agent for deeper PR/issue/release
research.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path

HOME = Path.home()
STATE_ROOT = Path(
    os.environ.get("VESPER_RESEARCH_STATE_DIR", HOME / ".local/state/vesper/research")
).expanduser()
STATE_ROOT.mkdir(parents=True, exist_ok=True)
STATE_FILE = STATE_ROOT / "upstream-edge-snapshot.json"

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
        timeout=20,
        check=False,
    )
    if proc.returncode != 0:
        return {"error": proc.stderr.strip()[-240:] or f"gh exited {proc.returncode}"}
    try:
        return json.loads(proc.stdout)
    except Exception as exc:
        return {"error": f"invalid JSON: {type(exc).__name__}: {exc}"}


def build_snapshot() -> dict[str, dict]:
    snapshot: dict[str, dict] = {}
    for repository in REPOSITORIES:
        meta = gh_api(f"repos/{repository}")
        if meta.get("error"):
            snapshot[repository] = meta
            continue

        branch = meta.get("default_branch") or "main"
        ref = gh_api(f"repos/{repository}/commits/{branch}")
        if ref.get("error"):
            snapshot[repository] = {"defaultBranch": branch, **ref}
            continue

        commit = ref.get("commit") or {}
        author = commit.get("author") or {}
        snapshot[repository] = {
            "defaultBranch": branch,
            "sha": ref.get("sha"),
            "date": author.get("date"),
            "message": str(commit.get("message") or "").splitlines()[0][:180],
        }
    return snapshot


def main() -> int:
    if not shutil.which("gh"):
        # A broken gate must wake the agent rather than silently hide changes.
        print(
            json.dumps(
                {
                    "wakeAgent": True,
                    "context": {"monitorError": "gh executable not found"},
                },
                sort_keys=True,
            )
        )
        return 0

    current = build_snapshot()
    previous: dict[str, dict] = {}
    try:
        previous = json.loads(STATE_FILE.read_text())
    except Exception:
        pass

    if current == previous:
        print('{"wakeAgent":false}')
        return 0

    changed: dict[str, dict] = {}
    for repository in sorted(set(previous) | set(current)):
        before = previous.get(repository)
        after = current.get(repository)
        if before != after:
            changed[repository] = {"before": before, "after": after}

    STATE_FILE.write_text(json.dumps(current, ensure_ascii=False, indent=2, sort_keys=True))
    print(
        json.dumps(
            {
                "wakeAgent": True,
                "context": {
                    "changed": changed,
                    "current": current,
                },
            },
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
