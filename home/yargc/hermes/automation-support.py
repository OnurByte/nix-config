#!/usr/bin/env python3
"""Deterministic collectors/watchdogs for the Vesper Hermes automation fleet.

The file is installed under several names. Behaviour is selected by argv[0] so
Hermes can attach each name as a normal pre-run or no-agent cron script.
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.parse
import urllib.request
from datetime import datetime, timedelta, timezone
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


def read_json_url(url: str, timeout: int = 15) -> Any:
    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": "VesperHermesResearch/1.0 (+personal research automation)",
            "Accept": "application/json",
        },
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        return json.load(response)


def emit_change_alert(key: str, payload: Any, message: str, recovery: str | None = None) -> None:
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


def github_collect() -> None:
    if not shutil.which("gh"):
        print(compact({"error": "gh is not available", "candidates": []}))
        return

    since = (datetime.now(timezone.utc) - timedelta(days=10)).date().isoformat()
    repo_queries = [
        f'agent llm created:>{since} stars:<150',
        f'coding agent created:>{since} stars:<150',
        f'mcp ai created:>{since} stars:<150',
        f'inference ai created:>{since} stars:<150',
        f'openai compatible created:>{since} stars:<150',
        f'ai wrapper created:>{since} stars:<150',
        f'local ai created:>{since} stars:<150',
        f'llm cli created:>{since} stars:<150',
    ]
    issue_queries = [
        f'agent llm updated:>{since} is:issue',
        f'mcp ai updated:>{since} is:issue',
        f'coding agent updated:>{since} is:issue',
        f'inference llm updated:>{since} is:issue',
    ]

    repos: dict[str, dict[str, Any]] = {}
    issues: dict[str, dict[str, Any]] = {}
    errors: list[str] = []

    for query in repo_queries:
        proc = run(
            [
                "gh",
                "api",
                "-X",
                "GET",
                "search/repositories",
                "-f",
                f"q={query}",
                "-f",
                "sort=updated",
                "-f",
                "order=desc",
                "-f",
                "per_page=50",
            ],
            timeout=25,
        )
        if proc.returncode != 0:
            errors.append(proc.stderr.strip()[-300:])
            continue
        try:
            data = json.loads(proc.stdout)
        except Exception:
            continue
        for item in data.get("items", []):
            url = item.get("html_url")
            if not url:
                continue
            repos[url] = {
                "name": item.get("full_name"),
                "url": url,
                "description": item.get("description"),
                "stars": item.get("stargazers_count"),
                "forks": item.get("forks_count"),
                "created": item.get("created_at"),
                "updated": item.get("updated_at"),
                "pushed": item.get("pushed_at"),
                "language": item.get("language"),
                "topics": item.get("topics", []),
            }

    for query in issue_queries:
        proc = run(
            [
                "gh",
                "api",
                "-X",
                "GET",
                "search/issues",
                "-f",
                f"q={query}",
                "-f",
                "sort=updated",
                "-f",
                "order=desc",
                "-f",
                "per_page=50",
            ],
            timeout=25,
        )
        if proc.returncode != 0:
            errors.append(proc.stderr.strip()[-300:])
            continue
        try:
            data = json.loads(proc.stdout)
        except Exception:
            continue
        for item in data.get("items", []):
            url = item.get("html_url")
            if not url:
                continue
            issues[url] = {
                "title": item.get("title"),
                "url": url,
                "comments": item.get("comments"),
                "created": item.get("created_at"),
                "updated": item.get("updated_at"),
                "state": item.get("state"),
                "body": (item.get("body") or "")[:700],
            }

    ordered_repos = sorted(
        repos.values(),
        key=lambda item: (item.get("stars") or 0, item.get("updated") or ""),
    )[:350]
    ordered_issues = sorted(
        issues.values(), key=lambda item: item.get("updated") or "", reverse=True
    )[:200]
    print(
        compact(
            {
                "source": "github",
                "generatedAt": datetime.now(timezone.utc).isoformat(),
                "repoCandidates": ordered_repos,
                "issueCandidates": ordered_issues,
                "errors": [e for e in errors if e][:5],
            }
        )
    )


def reddit_collect() -> None:
    queries = [
        "AI agent",
        "coding agent",
        "MCP AI",
        "LLM tooling",
        "local AI",
        "open source AI",
        "inference server",
        "LLM CLI",
        "agent harness",
    ]
    items: dict[str, dict[str, Any]] = {}
    errors: list[str] = []
    for query in queries:
        params = urllib.parse.urlencode(
            {"q": query, "sort": "new", "t": "week", "limit": 100, "raw_json": 1}
        )
        try:
            data = read_json_url(f"https://www.reddit.com/search.json?{params}")
        except Exception as exc:
            errors.append(f"{query}: {type(exc).__name__}: {exc}")
            continue
        for child in ((data.get("data") or {}).get("children") or []):
            post = child.get("data") or {}
            permalink = post.get("permalink")
            if not permalink:
                continue
            url = "https://www.reddit.com" + permalink
            items[url] = {
                "title": post.get("title"),
                "url": url,
                "externalUrl": post.get("url_overridden_by_dest") or post.get("url"),
                "subreddit": post.get("subreddit"),
                "score": post.get("score"),
                "comments": post.get("num_comments"),
                "createdUtc": post.get("created_utc"),
                "author": post.get("author"),
                "selftext": (post.get("selftext") or "")[:700],
            }
    ordered = sorted(
        items.values(), key=lambda item: item.get("createdUtc") or 0, reverse=True
    )[:400]
    print(
        compact(
            {
                "source": "reddit",
                "generatedAt": datetime.now(timezone.utc).isoformat(),
                "candidates": ordered,
                "errors": errors[:5],
            }
        )
    )


def linuxdo_collect() -> None:
    topics: dict[int, dict[str, Any]] = {}
    posts: dict[int, dict[str, Any]] = {}
    errors: list[str] = []

    for page in range(3):
        try:
            data = read_json_url(f"https://linux.do/latest.json?page={page}")
        except Exception as exc:
            errors.append(f"latest page {page}: {type(exc).__name__}: {exc}")
            continue
        for topic in ((data.get("topic_list") or {}).get("topics") or []):
            topic_id = topic.get("id")
            if not topic_id:
                continue
            topics[topic_id] = {
                "id": topic_id,
                "title": topic.get("title"),
                "slug": topic.get("slug"),
                "url": f"https://linux.do/t/{topic.get('slug')}/{topic_id}",
                "postsCount": topic.get("posts_count"),
                "replyCount": topic.get("reply_count"),
                "views": topic.get("views"),
                "likeCount": topic.get("like_count"),
                "created": topic.get("created_at"),
                "lastPosted": topic.get("last_posted_at"),
                "tags": topic.get("tags", []),
            }

    search_terms = [
        "免费 AI",
        "免费 API",
        "公益 AI",
        "Claude",
        "Codex",
        "Grok",
        "MCP",
        "开源 AI",
        "free tier",
        "API 中转",
    ]
    for query in search_terms:
        try:
            data = read_json_url(
                "https://linux.do/search.json?" + urllib.parse.urlencode({"q": query})
            )
        except Exception as exc:
            errors.append(f"search {query}: {type(exc).__name__}: {exc}")
            continue
        for topic in data.get("topics", []) or []:
            topic_id = topic.get("id")
            if not topic_id:
                continue
            topics[topic_id] = {
                "id": topic_id,
                "title": topic.get("title"),
                "slug": topic.get("slug"),
                "url": f"https://linux.do/t/{topic.get('slug')}/{topic_id}",
                "postsCount": topic.get("posts_count"),
                "views": topic.get("views"),
                "likeCount": topic.get("like_count"),
                "created": topic.get("created_at"),
                "lastPosted": topic.get("last_posted_at"),
                "tags": topic.get("tags", []),
            }
        for post in data.get("posts", []) or []:
            post_id = post.get("id")
            topic_id = post.get("topic_id")
            if not post_id:
                continue
            posts[post_id] = {
                "id": post_id,
                "topicId": topic_id,
                "username": post.get("username"),
                "created": post.get("created_at"),
                "likeCount": post.get("like_count"),
                "blurb": (post.get("blurb") or "")[:800],
                "url": f"https://linux.do/t/{topic_id}/{post.get('post_number', 1)}"
                if topic_id
                else None,
            }

    ordered_topics = sorted(
        topics.values(),
        key=lambda item: (item.get("views") or 0, item.get("created") or ""),
    )[:350]
    ordered_posts = sorted(
        posts.values(), key=lambda item: item.get("created") or "", reverse=True
    )[:250]
    print(
        compact(
            {
                "source": "linux.do",
                "generatedAt": datetime.now(timezone.utc).isoformat(),
                "topics": ordered_topics,
                "posts": ordered_posts,
                "errors": errors[:8],
            }
        )
    )


def health_watch() -> None:
    problems: dict[str, Any] = {}
    if shutil.which("vesper-doctor"):
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
            problems["doctorError"] = proc.stderr.strip()[-1000:]

    for scope, command in (
        ("system", ["systemctl", "--failed", "--no-legend", "--plain"]),
        ("user", ["systemctl", "--user", "--failed", "--no-legend", "--plain"]),
    ):
        proc = run(command, timeout=10)
        lines = [line for line in proc.stdout.splitlines() if line.strip()]
        if lines:
            problems[f"{scope}Units"] = lines[:20]

    emit_change_alert(
        "health",
        problems,
        "Vesper Health Watch detected a new or changed problem:",
        "Vesper Health Watch: previously reported problems are clear.",
    )


def skill_integrity_watch() -> None:
    jobs_file = HERMES_HOME / "cron" / "jobs.json"
    if not jobs_file.exists():
        return
    try:
        jobs = json.loads(jobs_file.read_text()).get("jobs", [])
    except Exception as exc:
        emit_change_alert("skills", {"jobsJson": str(exc)}, "Hermes skill integrity problem:")
        return

    installed: set[str] = set()
    for root in (HERMES_HOME / "skills", HOME / ".agents" / "skills"):
        if not root.exists():
            continue
        for skill_md in root.rglob("SKILL.md"):
            try:
                for line in skill_md.read_text(errors="replace").splitlines()[:30]:
                    if line.startswith("name:"):
                        installed.add(line.split(":", 1)[1].strip().strip('"\''))
                        break
            except Exception:
                pass
            installed.add(skill_md.parent.name)

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


def retention() -> None:
    errors: list[str] = []
    cutoff = time.time() - 30 * 86400
    output_root = HERMES_HOME / "cron" / "output"
    removed = 0
    if output_root.exists():
        for path in output_root.rglob("*.md"):
            try:
                if path.stat().st_mtime < cutoff:
                    path.unlink()
                    removed += 1
            except Exception as exc:
                errors.append(f"{path}: {exc}")
        for directory in sorted(
            [p for p in output_root.rglob("*") if p.is_dir()], reverse=True
        ):
            try:
                directory.rmdir()
            except OSError:
                pass

    if shutil.which("hermes"):
        proc = run(
            [
                "hermes",
                "sessions",
                "prune",
                "--older-than",
                "30d",
                "--source",
                "cron",
                "--yes",
            ],
            timeout=90,
        )
        if proc.returncode != 0:
            errors.append((proc.stderr or proc.stdout).strip()[-1200:])

    if errors:
        print("Hermes Cron Retention encountered errors:")
        print(json.dumps({"removedOutputs": removed, "errors": errors[:8]}, indent=2))


def project_inventory() -> None:
    roots = [HOME / name for name in ("Documents", "Projects", "Code", "src", "Source")]
    excluded = {
        ".cache",
        ".git",
        "node_modules",
        "vendor",
        "target",
        ".venv",
        "venv",
        "dist",
        "build",
        ".next",
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
            dirs[:] = [d for d in dirs if d not in excluded and not d.startswith(".direnv")]
            if depth > 4:
                dirs[:] = []
                continue
            if ".git" in dirs or ".git" in files:
                resolved = path.resolve()
                if resolved not in seen:
                    repos.append(resolved)
                    seen.add(resolved)
                dirs[:] = [d for d in dirs if d != ".git"]
            if len(repos) >= 80:
                break

    result = []
    for repo in repos[:80]:
        def git(*args: str) -> str:
            proc = run(["git", "-C", str(repo), *args], timeout=8)
            return proc.stdout.strip() if proc.returncode == 0 else ""

        status = git("status", "--porcelain=v1")
        todo_files: list[str] = []
        for candidate in ("TODO.md", "TODOS.md", "PROGRESS.md", "PLAN.md", "ROADMAP.md"):
            if (repo / candidate).exists():
                todo_files.append(candidate)
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
        "frontier-github-collect": github_collect,
        "frontier-reddit-collect": reddit_collect,
        "free-ai-linuxdo-collect": linuxdo_collect,
        "vesper-health-watch": health_watch,
        "vesper-skill-integrity-watch": skill_integrity_watch,
        "vesper-cron-retention": retention,
        "project-inventory": project_inventory,
        "ai-usage-snapshot": usage_snapshot,
    }
    handler = handlers.get(mode)
    if handler is None:
        raise SystemExit(f"Unsupported automation-support mode: {mode}")
    handler()


if __name__ == "__main__":
    main()
