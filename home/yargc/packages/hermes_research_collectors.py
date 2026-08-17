from __future__ import annotations

import json
import re
import shutil
import subprocess
import urllib.parse
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, load_json

HTTP_TIMEOUT = 8
GH_TIMEOUT = 12
MAX_WORKERS = 8
MAX_PROMPT_CHARS = 80_000
MAX_LEARNED_PER_KIND = 20
SEED_PATH = STATE_ROOT / "unknown-frontier-ai" / "discovery-seeds.json"
POOL_ROOT = STATE_ROOT / "candidate-pools"


def _compact(value: Any) -> str:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def _learned_seeds() -> dict[str, list[str]]:
    raw = load_json(SEED_PATH, {})
    if not isinstance(raw, dict):
        return {}
    result: dict[str, list[str]] = {}
    for key in (
        "githubQueries",
        "githubIssueQueries",
        "redditQueries",
        "redditSubreddits",
        "linuxdoQueries",
        "xQueries",
    ):
        values = raw.get(key)
        if not isinstance(values, list):
            continue
        cleaned: list[str] = []
        for value in values:
            if not isinstance(value, str):
                continue
            value = value.strip()
            if not value or len(value) > 200 or value in cleaned:
                continue
            if key == "redditSubreddits" and not re.fullmatch(r"[A-Za-z0-9_]{2,50}", value):
                continue
            cleaned.append(value)
            if len(cleaned) >= MAX_LEARNED_PER_KIND:
                break
        if cleaned:
            result[key] = cleaned
    return result


def discovery_seeds() -> dict[str, list[str]]:
    return _learned_seeds()


def persist_discovery_seeds(incoming: Any) -> None:
    if not isinstance(incoming, dict):
        return
    current = _learned_seeds()
    merged: dict[str, list[str]] = {}
    for key in (
        "githubQueries",
        "githubIssueQueries",
        "redditQueries",
        "redditSubreddits",
        "linuxdoQueries",
        "xQueries",
    ):
        values = incoming.get(key)
        new_values = values if isinstance(values, list) else []
        combined = [*new_values, *current.get(key, [])]
        cleaned: list[str] = []
        for value in combined:
            if not isinstance(value, str):
                continue
            value = value.strip()
            if not value or len(value) > 200 or value in cleaned:
                continue
            if key == "redditSubreddits" and not re.fullmatch(r"[A-Za-z0-9_]{2,50}", value):
                continue
            cleaned.append(value)
            if len(cleaned) >= MAX_LEARNED_PER_KIND:
                break
        if cleaned:
            merged[key] = cleaned
    atomic_json(SEED_PATH, merged)


def _read_json_url(url: str) -> Any:
    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": "VesperHermesResearch/1.0 (+personal research automation)",
            "Accept": "application/json",
        },
    )
    with urllib.request.urlopen(request, timeout=HTTP_TIMEOUT) as response:
        return json.load(response)


def _gh_search(endpoint: str, query: str) -> tuple[str, Any]:
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
            "per_page=100",
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


def collect_github() -> dict[str, Any]:
    if not shutil.which("gh"):
        return {"source": "github", "error": "gh is not available", "repoCandidates": [], "issueCandidates": []}

    since = (datetime.now(timezone.utc) - timedelta(days=10)).date().isoformat()
    seeds = _learned_seeds()
    repo_queries = [
        f"agent llm created:>{since} stars:<150",
        f"coding agent created:>{since} stars:<150",
        f"mcp ai created:>{since} stars:<150",
        f"inference ai created:>{since} stars:<150",
        f"openai compatible created:>{since} stars:<150",
        f"ai wrapper created:>{since} stars:<150",
        f"local ai created:>{since} stars:<150",
        f"llm cli created:>{since} stars:<150",
    ]
    issue_queries = [
        f"agent llm updated:>{since} is:issue",
        f"mcp ai updated:>{since} is:issue",
        f"coding agent updated:>{since} is:issue",
        f"inference llm updated:>{since} is:issue",
    ]
    repo_queries.extend(f"{q} created:>{since} stars:<250" for q in seeds.get("githubQueries", [])[:8])
    issue_queries.extend(f"{q} updated:>{since} is:issue" for q in seeds.get("githubIssueQueries", [])[:4])
    repo_queries = list(dict.fromkeys(repo_queries))[:16]
    issue_queries = list(dict.fromkeys(issue_queries))[:8]

    repos: dict[str, dict[str, Any]] = {}
    issues: dict[str, dict[str, Any]] = {}
    errors: list[str] = []
    tasks = [("search/repositories", q, "repo") for q in repo_queries] + [("search/issues", q, "issue") for q in issue_queries]
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        future_map = {pool.submit(_gh_search, endpoint, query): kind for endpoint, query, kind in tasks}
        for future in as_completed(future_map):
            kind = future_map[future]
            try:
                query, data = future.result()
            except Exception as exc:
                errors.append(f"{kind}: {type(exc).__name__}: {exc}")
                continue
            if not isinstance(data, dict):
                continue
            if data.get("error"):
                errors.append(f"{query}: {data['error']}")
                continue
            for item in data.get("items", []):
                if not isinstance(item, dict):
                    continue
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

    return {
        "source": "github",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "learnedQueriesUsed": {
            "repositories": seeds.get("githubQueries", [])[:8],
            "issues": seeds.get("githubIssueQueries", [])[:4],
        },
        "repoCandidates": sorted(repos.values(), key=lambda item: (item.get("stars") or 0, item.get("updated") or ""))[:500],
        "issueCandidates": sorted(issues.values(), key=lambda item: item.get("updated") or "", reverse=True)[:300],
        "errors": errors[:12],
    }


def _reddit_query(query: str) -> tuple[str, str, Any]:
    params = urllib.parse.urlencode({"q": query, "sort": "new", "t": "week", "limit": 100, "raw_json": 1})
    return "query", query, _read_json_url(f"https://www.reddit.com/search.json?{params}")


def _reddit_subreddit(subreddit: str) -> tuple[str, str, Any]:
    params = urllib.parse.urlencode({"limit": 100, "raw_json": 1})
    return "subreddit", subreddit, _read_json_url(f"https://www.reddit.com/r/{urllib.parse.quote(subreddit)}/new.json?{params}")


def collect_reddit() -> dict[str, Any]:
    seeds = _learned_seeds()
    queries = list(dict.fromkeys([
        "AI agent",
        "coding agent",
        "MCP AI",
        "LLM tooling",
        "local AI",
        "open source AI",
        "inference server",
        "LLM CLI",
        "agent harness",
        *seeds.get("redditQueries", []),
    ]))[:30]
    subreddits = list(dict.fromkeys(seeds.get("redditSubreddits", [])))[:20]
    items: dict[str, dict[str, Any]] = {}
    errors: list[str] = []
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        futures = [pool.submit(_reddit_query, query) for query in queries]
        futures.extend(pool.submit(_reddit_subreddit, subreddit) for subreddit in subreddits)
        for future in as_completed(futures):
            try:
                source_kind, source_value, data = future.result()
            except Exception as exc:
                errors.append(f"{type(exc).__name__}: {exc}")
                continue
            if not isinstance(data, dict):
                continue
            for child in ((data.get("data") or {}).get("children") or []):
                post = child.get("data") or {}
                permalink = post.get("permalink")
                if not permalink:
                    continue
                url = "https://www.reddit.com" + permalink
                items[url] = {
                    "discoveredBy": f"{source_kind}:{source_value}",
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
    return {
        "source": "reddit",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "learnedQueriesUsed": seeds.get("redditQueries", []),
        "learnedSubredditsUsed": subreddits,
        "candidates": sorted(items.values(), key=lambda item: item.get("createdUtc") or 0, reverse=True)[:600],
        "errors": errors[:12],
    }


def _linuxdo_request(kind: str, value: str) -> tuple[str, str, Any]:
    if kind == "latest":
        url = f"https://linux.do/latest.json?page={value}"
    else:
        url = "https://linux.do/search.json?" + urllib.parse.urlencode({"q": value})
    return kind, value, _read_json_url(url)


def collect_linuxdo() -> dict[str, Any]:
    seeds = _learned_seeds()
    topics: dict[int, dict[str, Any]] = {}
    posts: dict[int, dict[str, Any]] = {}
    errors: list[str] = []
    search_terms = list(dict.fromkeys([
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
        *seeds.get("linuxdoQueries", []),
    ]))[:30]
    requests = [("latest", str(page)) for page in range(5)] + [("search", term) for term in search_terms]
    with ThreadPoolExecutor(max_workers=MAX_WORKERS) as pool:
        futures = [pool.submit(_linuxdo_request, kind, value) for kind, value in requests]
        for future in as_completed(futures):
            try:
                kind, value, data = future.result()
            except Exception as exc:
                errors.append(f"{type(exc).__name__}: {exc}")
                continue
            if not isinstance(data, dict):
                continue
            source_topics = ((data.get("topic_list") or {}).get("topics") or []) if kind == "latest" else (data.get("topics") or [])
            for topic in source_topics:
                if not isinstance(topic, dict) or not topic.get("id"):
                    continue
                topic_id = int(topic["id"])
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
                    if not isinstance(post, dict) or not post.get("id"):
                        continue
                    post_id = int(post["id"])
                    topic_id = post.get("topic_id")
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
    return {
        "source": "linux.do",
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "learnedQueriesUsed": seeds.get("linuxdoQueries", []),
        "topics": sorted(topics.values(), key=lambda item: (item.get("views") or 0, item.get("created") or ""))[:500],
        "posts": sorted(posts.values(), key=lambda item: item.get("created") or "", reverse=True)[:350],
        "errors": errors[:12],
    }


def _prune_old_pools(days: int = 30) -> None:
    cutoff = datetime.now(timezone.utc).timestamp() - days * 86400
    if not POOL_ROOT.exists():
        return
    for path in POOL_ROOT.glob("*.json"):
        try:
            if path.stat().st_mtime < cutoff:
                path.unlink()
        except OSError:
            pass


def persist_bounded_pool(source: str, payload: dict[str, Any]) -> str:
    POOL_ROOT.mkdir(parents=True, exist_ok=True)
    _prune_old_pools()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    path = POOL_ROOT / f"{source}-{stamp}.json"
    atomic_json(path, payload)

    result = dict(payload)
    result["fullPoolPath"] = str(path)
    result["fullPoolCounts"] = {key: len(value) for key, value in payload.items() if isinstance(value, list)}
    result["note"] = "Full deterministic candidate pool is stored at fullPoolPath; this is a bounded prompt sample."

    list_keys = [key for key, value in result.items() if isinstance(value, list)]
    encoded = _compact(result)
    while len(encoded) > MAX_PROMPT_CHARS:
        populated = [key for key in list_keys if isinstance(result.get(key), list) and len(result[key]) > 1]
        if not populated:
            break
        largest = max(populated, key=lambda key: len(result[key]))
        result[largest] = result[largest][: max(1, len(result[largest]) * 3 // 4)]
        encoded = _compact(result)
    if len(encoded) > MAX_PROMPT_CHARS:
        encoded = _compact({
            "source": payload.get("source"),
            "generatedAt": payload.get("generatedAt"),
            "fullPoolPath": str(path),
            "fullPoolCounts": result.get("fullPoolCounts"),
            "errors": payload.get("errors", []),
        })
    return encoded
