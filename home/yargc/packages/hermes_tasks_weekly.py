from __future__ import annotations

import json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Any

from hermes_automation_common import SKILL_DRAFT_ROOT, STATE_ROOT, invoke_json
from hermes_automation_reports import recent_briefings, research_prompt, write_report


def user_pain_miner() -> dict[str, Any]:
    objective = "Mine recurring evidence-backed user pain across Hermes, Codex, Claude Code, OpenCode, NixOS, Hyprland and adjacent agent/Linux tooling. Cluster the same complaint across multiple issues, comments, threads or communities. For strong clusters report problem, independent examples, recurrence evidence, existing solutions, why those are insufficient, and the smallest useful project/skill/tool opportunity. Do not turn isolated complaints into fake trends."
    return write_report(invoke_json(research_prompt("user-pain-miner", objective), web_only=True), "user-pain-miner")


def _git(args: list[str], cwd: Path, timeout: int = 8) -> str:
    completed = subprocess.run(["git", *args], cwd=cwd, text=True, capture_output=True, timeout=timeout, check=False)
    return (completed.stdout or "").strip()


def _discover_repos(limit: int = 60) -> list[Path]:
    roots = [Path.home() / name for name in ("Documents", "Projects", "Code", "src")]
    ignored = {"node_modules", ".venv", "venv", ".cache", ".direnv", "target", "vendor", ".npm", ".pnpm-store"}
    found: list[Path] = []
    seen: set[Path] = set()
    for root in roots:
        if not root.is_dir():
            continue
        root_depth = len(root.parts)
        for current, dirs, _files in os.walk(root):
            path = Path(current)
            dirs[:] = [name for name in dirs if name not in ignored]
            if ".git" in dirs:
                resolved = path.resolve()
                if resolved not in seen:
                    found.append(resolved)
                    seen.add(resolved)
                dirs.remove(".git")
                if len(found) >= limit:
                    return found
            if len(path.parts) - root_depth >= 5:
                dirs[:] = []
    return found


def _project_snapshot() -> str:
    records: list[dict[str, Any]] = []
    for repo in _discover_repos():
        try:
            status = _git(["status", "--porcelain=v1", "--untracked-files=normal"], repo)
            records.append({
                "path": str(repo),
                "branch": _git(["branch", "--show-current"], repo) or _git(["rev-parse", "--short", "HEAD"], repo),
                "lastCommit": _git(["log", "-1", "--date=iso", "--pretty=%h|%ad|%s"], repo),
                "dirtyCount": len(status.splitlines()) if status else 0,
                "dirtyPreview": status.splitlines()[:25],
                "branches": _git(["for-each-ref", "--sort=-committerdate", "--count=8", "--format=%(refname:short)|%(committerdate:iso)|%(subject)", "refs/heads"], repo).splitlines(),
            })
        except Exception as exc:
            records.append({"path": str(repo), "error": str(exc)})
    return json.dumps(records, ensure_ascii=False, indent=2)[:100000]


def project_archaeologist() -> dict[str, Any]:
    objective = "Analyze the bounded local Git snapshot as a weekly project archaeologist. Find forgotten but valuable unfinished work: stale dirty repos, branches with meaningful work, abandoned experiments and projects whose state suggests a blocker. Prioritize 3-8 things actually worth revisiting and explain why. Do not recommend cleanup merely for aesthetics and do not infer file contents that were not supplied."
    return write_report(invoke_json(research_prompt("project-archaeologist", objective, _project_snapshot()), web_only=False), "project-archaeologist")


def _append_context(chunks: list[str], path: Path, label: str, *, max_chars: int = 18000) -> int:
    if not path.is_file():
        return 0
    try:
        text = path.read_text(errors="replace")[:max_chars]
    except OSError:
        return 0
    chunk = f"\n--- {label}: {path} ---\n{text}\n"
    chunks.append(chunk)
    return len(chunk)


def _skill_review_context() -> str:
    chunks: list[str] = []
    used = 0

    active = Path.home() / ".agents" / "skills" / "hermes-research-radar"
    for relative, label, limit in (
        ("SKILL.md", "active research skill", 24000),
        ("references/research-evolution.md", "research evolution policy", 20000),
        ("references/central-sources.md", "central source policy", 18000),
        ("evals/evals.json", "research skill eval set", 30000),
    ):
        used += _append_context(chunks, active / relative, label, max_chars=limit)

    source_registry = STATE_ROOT / "unknown-frontier-ai" / "source-registry.json"
    used += _append_context(chunks, source_registry, "adaptive source registry", max_chars=28000)

    runs_root = STATE_ROOT / "runs"
    if runs_root.is_dir():
        for task in ("unknown-frontier-github", "unknown-frontier-reddit", "unknown-frontier-x", "unknown-frontier-synthesis"):
            used += _append_context(chunks, runs_root / task / "latest.json", f"latest run {task}", max_chars=6000)

    if SKILL_DRAFT_ROOT.exists():
        for path in sorted(SKILL_DRAFT_ROOT.rglob("*")):
            if path.is_file() and path.stat().st_size <= 100000:
                used += _append_context(chunks, path, "skill draft", max_chars=15000)
                if used > 125000:
                    break

    if used <= 125000:
        for path in sorted(STATE_ROOT.glob("*/heuristics.json")):
            used += _append_context(chunks, path, f"heuristics {path.parent.name}", max_chars=12000)
            if used > 145000:
                break
    return "".join(chunks)[:150000]


def skill_evolution_review() -> dict[str, Any]:
    objective = "Review Vesper's active research skill, its representative eval set, source-registry behavior, accumulated heuristics and skill drafts using an eval-first skill-creator mindset. Distinguish reversible runtime policy changes from Nix-owned active skill changes. For each candidate procedure decide promote, keep-testing, merge, narrow-scope, retire or rollback. Require repeated evidence and representative with-skill-vs-current/baseline evaluation before recommending an active skill promotion. Evaluate relevance, primary-source verification, duplicate/familiar rate, source diversity, anchor/dynamic/explore balance, access-failure reporting and token/time cost when measured. CandidateSources mentions alone are not proof of source usefulness. Never edit the active skill automatically; output a concrete evidence-backed review queue and any missing eval cases."
    return write_report(invoke_json(research_prompt("skill-evolution-review", objective, _skill_review_context()), web_only=False), "skill-evolution-review")


def _capture(command: list[str], timeout: int = 25) -> dict[str, Any]:
    binary = shutil.which(command[0])
    if not binary:
        return {"available": False, "command": command[0]}
    try:
        completed = subprocess.run([binary, *command[1:]], text=True, capture_output=True, timeout=timeout, check=False)
        return {"available": True, "returncode": completed.returncode, "stdout": (completed.stdout or "")[-30000:], "stderr": (completed.stderr or "")[-5000:]}
    except Exception as exc:
        return {"available": True, "error": str(exc)}


def ai_usage_economist() -> dict[str, Any]:
    measurements = {"ccusage": _capture(["ccusage", "--json"]), "codexbar": _capture(["codexbar-status"]), "turnlens": _capture(["turnlens", "--help"])}
    objective = "Analyze the local accounting surfaces as a weekly workflow economist. Clearly separate measured facts from recommendations. Identify which agents/providers appear to consume the most usage, where expensive models may be used for low-value work, what could move to cheaper/free routes without harming quality, and which measurements are missing or unreliable. Do not invent costs or token counts absent from the data."
    return write_report(invoke_json(research_prompt("ai-usage-economist", objective, json.dumps(measurements, ensure_ascii=False, indent=2)[:90000]), web_only=False), "ai-usage-economist")


def _resolve_vault() -> Path | None:
    configured = os.environ.get("OBSIDIAN_VAULT_PATH", "").strip()
    if configured:
        path = Path(configured).expanduser()
        return path if (path / ".obsidian").is_dir() else None

    likely = [
        Path.home() / "Documents" / "Obsidian",
        Path.home() / "Documents" / "Notes",
        Path.home() / "Notes",
    ]
    for path in likely:
        if (path / ".obsidian").is_dir():
            return path

    ignored = {"node_modules", ".git", ".cache", ".direnv", "target", "vendor", ".venv", "venv"}
    for root in (Path.home() / "Documents", Path.home() / "Notes"):
        if not root.is_dir():
            continue
        root_depth = len(root.parts)
        for current, dirs, _files in os.walk(root):
            path = Path(current)
            dirs[:] = [name for name in dirs if name not in ignored]
            if ".obsidian" in dirs:
                return path
            if len(path.parts) - root_depth >= 3:
                dirs[:] = []
    return None


def second_brain_dream() -> dict[str, Any]:
    vault = _resolve_vault()
    prompt = f"""Use the installed `vesper-obsidian-second-brain` skill for Vesper's nightly dream/consolidation cycle.
Resolved vault: {str(vault) if vault else 'UNRESOLVED'}
Recent durable research:
{recent_briefings(days=2, max_chars=90000)}
Do real consolidation, not a transcript summary. Deduplicate against existing notes when a vault is available. Promote only durable facts, useful relationships, corrected beliefs, open questions and proven source paths. Stage repeated procedures under {SKILL_DRAFT_ROOT}; never auto-promote them into active skills. If the vault is unresolved, do not invent or create one; report ingestion as pending.
Return exactly one JSON object and nothing else: {{"title":"...","summary":"...","body":"...","priority":"low|normal|high|critical","confidence":0.0,"sources":[],"statePatch":{{"knownConcepts":[],"candidateSources":[],"heuristics":[],"openQuestions":[]}}}}
"""
    return write_report(invoke_json(prompt, web_only=False), "second-brain-dream", notify_user=False)


WEEKLY_TASKS = {
    "user-pain-miner": user_pain_miner,
    "project-archaeologist": project_archaeologist,
    "skill-evolution-review": skill_evolution_review,
    "ai-usage-economist": ai_usage_economist,
    "second-brain-dream": second_brain_dream,
}
