#!/usr/bin/env python3
"""Emit a stable upstream snapshot for Hermes monitor_script mode.

Hermes hashes this exact stdout. If none of the tracked upstream heads changes,
the Upstream Edge Radar agent is not started at all.
"""

from __future__ import annotations

import json
import shutil
import subprocess

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


def main() -> int:
    if not shutil.which("gh"):
        print(json.dumps({"error": "gh executable not found"}, sort_keys=True))
        return 0

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

    # No timestamp: unchanged upstream state must produce byte-identical stdout.
    print(json.dumps(snapshot, ensure_ascii=False, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
