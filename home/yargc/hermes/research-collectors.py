#!/usr/bin/env python3
"""Parallel, bounded collectors for Vesper's high-volume Hermes research lanes.

This module is copied inside ~/.hermes/scripts and invoked by bounded-collector.py.
It deliberately keeps network fan-out below Hermes' default pre-run script timeout
so the agent window is reserved for verification rather than mechanical polling.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

HTTP_TIMEOUT = 8
GH_TIMEOUT = 12
MAX_WORKERS = 8


def compact(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def read_json_url(url: str) -> Any:
    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": "VesperHermesResearch/1.0 (+personal research automation)",
            "Accept": "application/json",
        },
    )
    with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT) as response:
        return json.load(response)


def gh_search(endpoint: str, query: str) -> tuple[str, Any]:
    proc = subprocess.run(
        [
            "gh",
            "api",
            "-X",
            "GET",
            endpoint,
            "-f",
            f"q={query}",
            "-f",
            "sort=updated",
            "-f",
            "order=desc",
            "-f",
            "per_page=50",
        ],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=GH_TIMEOUT,
        check=False,
    )
    if proc.returncode != 0:
        return query, {"error": proc.stderr.strip()[-400:] or f"gh exited {proc.returncode}"}
    try:
        return query, json.loads(proc.stdout)
    except Exception as exc:
        return query, {"error": f"invalid JSON: {type(exc).__name__}: {exc}"}


def github_collect() -> None:
    if not shutil.which("gh"):
        print(compact({"source": "github", "error": "gh is not available", "repoCandidates": [], "issueCandidates": []}))
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
    tasks = [("search/repositories", q, "repo") for q in repo_queries] + [
        ("search/issues", q, "issue") for q in issue_queries
    ]

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        future_map = {
            pool.submit(gh_search, endpoint, query): kind
            for endpoint, query, kind in tasks
        }
        for future in as_completed(future_map):
            kind = future_map[future]
            try:
                query, data = future.result()
            except Exception as exc:
                errors.append(f"{kind}: {type(exc).__name__}: {exc}")
                continue
            if data.get("error"):
                errors.append(f"{query}: {data['error']}")
                continue
            for item in data.get("items", []):
                url = item.get("html_url")
                if not url:
                    continue
                if kind == "repo":
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
                else:
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
                "errors": errors[:8],
            }
        )
    )


def reddit_query(query: str) -> tuple[str, Any]:
    params = urllib.parse.urlencode(
        {"q": query, "sort": "new", "t": "week", "limit": 100, "raw_json": 1}
    )
    return query, read_json_url(f"https://www.reddit.com/search.json?{params}")


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
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        futures = [pool.submit(reddit_query, query) for query in queries]
        for future in as_completed(futures):
            try:
                query, data = future.result()
            except Exception as exc:
                errors.append(f"{type(exc).__name__}: {exc}")
                continue
            for child in ((data.get("data") or {}).get("children") or []):
                post = child.get("data") or {}
                permalink = post.get("permalink")
                if not permalink:
                    continue
                url = "https://www.reddit.com" + permalink
                items[url] = {
                    "query": query,
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
                "errors": errors[:8],
            }
        )
    )


def linuxdo_request(kind: str, value: str) -> tuple[str, str, Any]:
    if kind == "latest":
        url = f"https://linux.do/latest.json?page={value}"
    else:
        url = "https://linux.do/search.json?" + urllib.parse.urlencode({"q": value})
    return kind, value, read_json_url(url)


def linuxdo_collect() -> None:
    topics: dict[int, dict[str, Any]] = {}
    posts: dict[int, dict[str, Any]] = {}
    errors: list[str] = []
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
    requests = [("latest", str(page)) for page in range(3)] + [
        ("search", term) for term in search_terms
    ]

    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        futures = [pool.submit(linuxdo_request, kind, value) for kind, value in requests]
        for future in as_completed(futures):
            try:
                kind, value, data = future.result()
            except Exception as exc:
                errors.append(f"{type(exc).__name__}: {exc}")
                continue

            if kind == "latest":
                source_topics = ((data.get("topic_list") or {}).get("topics") or [])
            else:
                source_topics = data.get("topics", []) or []

            for topic in source_topics:
                topic_id = topic.get("id")
                if not topic_id:
                    continue
                slug = topic.get("slug")
                topics[topic_id] = {
                    "id": topic_id,
                    "title": topic.get("title"),
                    "slug": slug,
                    "url": f"https://linux.do/t/{slug}/{topic_id}" if slug else f"https://linux.do/t/{topic_id}",
                    "postsCount": topic.get("posts_count"),
                    "replyCount": topic.get("reply_count"),
                    "views": topic.get("views"),
                    "likeCount": topic.get("like_count"),
                    "created": topic.get("created_at"),
                    "lastPosted": topic.get("last_posted_at"),
                    "tags": topic.get("tags", []),
                }

            if kind == "search":
                for post in data.get("posts", []) or []:
                    post_id = post.get("id")
                    topic_id = post.get("topic_id")
                    if not post_id:
                        continue
                    posts[post_id] = {
                        "id": post_id,
                        "topicId": topic_id,
                        "query": value,
                        "username": post.get("username"),
                        "created": post.get("created_at"),
                        "likeCount": post.get("like_count"),
                        "blurb": (post.get("blurb") or "")[:800],
                        "url": f"https://linux.do/t/{topic_id}/{post.get('post_number', 1)}" if topic_id else None,
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
                "errors": errors[:10],
            }
        )
    )


def main() -> None:
    mode = Path(sys.argv[0]).stem
    handlers = {
        "frontier-github-collect": github_collect,
        "frontier-reddit-collect": reddit_collect,
        "free-ai-linuxdo-collect": linuxdo_collect,
    }
    handler = handlers.get(mode)
    if handler is None:
        raise SystemExit(f"Unsupported research collector mode: {mode}")
    handler()


if __name__ == "__main__":
    main()
