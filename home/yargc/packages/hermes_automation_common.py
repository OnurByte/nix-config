from __future__ import annotations

import json
import os
import shutil
import subprocess
import threading
from datetime import datetime
from pathlib import Path
from typing import Any

STATE_ROOT = Path(os.environ.get("VESPER_RESEARCH_STATE_DIR", "~/.local/state/vesper/research")).expanduser()
BRIEFING_ROOT = Path(os.environ.get("VESPER_BRIEFING_DIR", "~/.local/share/vesper/briefings")).expanduser()
SKILL_DRAFT_ROOT = Path(os.environ.get("VESPER_SKILL_DRAFT_DIR", "~/.local/share/vesper/skill-drafts")).expanduser()
REGISTRY_PATH = Path(os.environ.get("VESPER_HERMES_JOB_REGISTRY", "~/.config/vesper/hermes-jobs.json")).expanduser()
HERMES_HOME = Path(os.environ.get("HERMES_HOME", "~/.hermes")).expanduser()
MODEL = os.environ.get("HERMES_RESEARCH_MODEL", "grok-4.5")
PROVIDER = os.environ.get("HERMES_RESEARCH_PROVIDER", "xai-oauth")
RUN_TIMEOUT = int(os.environ.get("VESPER_HERMES_RUN_TIMEOUT", "1800"))
RESEARCH_SKILL = "hermes-research-radar"
SECOND_BRAIN_SKILLS = ["obsidian", "vesper-obsidian-second-brain"]


def now() -> datetime:
    return datetime.now().astimezone()


def ensure_dirs() -> None:
    for path in (STATE_ROOT, BRIEFING_ROOT, SKILL_DRAFT_ROOT):
        path.mkdir(parents=True, exist_ok=True)


def atomic_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.{os.getpid()}.{threading.get_ident()}.tmp")
    tmp.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n")
    tmp.replace(path)


def load_json(path: Path, default: Any) -> Any:
    try:
        return json.loads(path.read_text(errors="replace"))
    except Exception:
        return default


def load_registry() -> dict[str, dict[str, Any]]:
    value = load_json(REGISTRY_PATH, {})
    if not isinstance(value, dict):
        raise RuntimeError(f"invalid Hermes job registry: {REGISTRY_PATH}")
    return {str(key): item for key, item in value.items() if isinstance(item, dict)}


def hermes_bin() -> str:
    binary = os.environ.get("HERMES_BIN") or shutil.which("hermes")
    if not binary:
        raise RuntimeError("hermes executable not found")
    return binary


def extract_object(text: str) -> dict[str, Any]:
    decoder = json.JSONDecoder()
    best: dict[str, Any] | None = None
    for index, char in enumerate(text):
        if char != "{":
            continue
        try:
            value, _ = decoder.raw_decode(text[index:])
        except Exception:
            continue
        if isinstance(value, dict) and value.get("title") and value.get("summary") is not None:
            best = value
    if best is None:
        raise RuntimeError("Hermes did not return a valid JSON object")
    return best


def extract_json_relaxed(text: str) -> Any:
    value = (text or "").strip()
    try:
        return json.loads(value)
    except Exception:
        pass
    decoder = json.JSONDecoder()
    for index, char in enumerate(value):
        if char not in "[{":
            continue
        try:
            parsed, _ = decoder.raw_decode(value[index:])
            return parsed
        except Exception:
            continue
    return None


def _invoke(
    prompt: str,
    *,
    web_only: bool = False,
    toolsets: list[str] | None = None,
    skills: list[str] | None = None,
    timeout: int,
) -> subprocess.CompletedProcess[str]:
    command = [hermes_bin(), "-z", prompt, "--provider", PROVIDER, "-m", MODEL, "--yolo"]
    selected_toolsets = toolsets if toolsets is not None else (["web"] if web_only else None)
    if selected_toolsets:
        command.extend(["--toolsets", ",".join(dict.fromkeys(selected_toolsets))])
    if skills:
        command.extend(["--skills", ",".join(dict.fromkeys(skills))])
    completed = subprocess.run(command, text=True, capture_output=True, timeout=timeout, check=False)
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout)[-8000:]
        raise RuntimeError(f"Hermes run failed (rc={completed.returncode})\n{detail}")
    return completed


def invoke_json(
    prompt: str,
    *,
    web_only: bool = False,
    toolsets: list[str] | None = None,
    skills: list[str] | None = None,
    timeout: int = RUN_TIMEOUT,
) -> dict[str, Any]:
    completed = _invoke(prompt, web_only=web_only, toolsets=toolsets, skills=skills, timeout=timeout)
    return extract_object((completed.stdout or "") + "\n" + (completed.stderr or ""))


def invoke_text(
    prompt: str,
    *,
    web_only: bool = False,
    toolsets: list[str] | None = None,
    skills: list[str] | None = None,
    timeout: int = RUN_TIMEOUT,
) -> str:
    completed = _invoke(prompt, web_only=web_only, toolsets=toolsets, skills=skills, timeout=timeout)
    text = (completed.stdout or "").strip()
    if not text:
        raise RuntimeError("Hermes returned an empty response")
    return text
