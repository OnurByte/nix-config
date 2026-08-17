from __future__ import annotations

import html
import json
import os
import re
import time
import urllib.error
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from html.parser import HTMLParser
from pathlib import Path
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, now

USER_AGENT = os.environ.get(
    "VESPER_RESEARCH_USER_AGENT",
    "VesperHermesResearch/1.0 (+local personal research; contact: configured-by-user)",
)
FETCH_TIMEOUT = max(3, int(os.environ.get("VESPER_RESEARCH_FETCH_TIMEOUT", "10")))
MAX_RESPONSE_BYTES = max(256_000, int(os.environ.get("VESPER_RESEARCH_MAX_RESPONSE_BYTES", "2500000")))

DEFAULT_REDDIT_SEEDS = (
    "LocalLLaMA",
    "MachineLearning",
    "programming",
    "opensource",
    "selfhosted",
    "NixOS",
    "linux",
    "rust",
    "golang",
    "privacy",
    "netsec",
    "cybersecurity",
    "Tor",
    "Monero",
    "webdev",
)

DEFAULT_X_QUERIES = (
    '"AI agent" code',
    '"coding agent"',
    '"agent harness"',
    '"LLM inference" open source',
    '"open source AI" tool',
    '"NixOS" AI',
    'privacy AI developer',
)

DEFAULT_X_MIRRORS = (
    "https://xcancel.com",
    "https://nitter.net",
)


def _csv_env(name: str, default: tuple[str, ...]) -> list[str]:
    raw = os.environ.get(name, "")
    values = [item.strip() for item in raw.split(",") if item.strip()] if raw else list(default)
    return list(dict.fromkeys(values))


def _fetch(url: str, *, accept: str = "text/html,application/atom+xml,application/rss+xml;q=0.9,*/*;q=0.5") -> tuple[bytes, str]:
    request = urllib.request.Request(
        url,
        headers={
            "User-Agent": USER_AGENT,
            "Accept": accept,
            "Accept-Language": "en-US,en;q=0.8",
        },
    )
    with urllib.request.urlopen(request, timeout=FETCH_TIMEOUT) as response:
        content_type = response.headers.get("Content-Type", "")
        payload = response.read(MAX_RESPONSE_BYTES + 1)
    if len(payload) > MAX_RESPONSE_BYTES:
        raise RuntimeError(f"response too large: {url}")
    return payload, content_type


def _strip_markup(value: str) -> str:
    value = re.sub(r"<[^>]+>", " ", value or "")
    return re.sub(r"\s+", " ", html.unescape(value)).strip()


def _node_text(entry: ET.Element, name: str) -> str:
    node = entry.find(f"{{*}}{name}")
    return "" if node is None or node.text is None else node.text.strip()


def _feed_entries(payload: bytes, *, surface: str, feed_url: str) -> list[dict[str, Any]]:
    root = ET.fromstring(payload)
    entries = list(root.findall("{*}entry")) + list(root.findall("{*}channel/{*}item"))
    out: list[dict[str, Any]] = []
    for entry in entries:
        title = _node_text(entry, "title")
        identifier = _node_text(entry, "id") or _node_text(entry, "guid")
        updated = _node_text(entry, "updated") or _node_text(entry, "published") or _node_text(entry, "pubDate")
        summary = _node_text(entry, "content") or _node_text(entry, "summary") or _node_text(entry, "description")
        link = ""
        for node in entry.findall("{*}link"):
            href = (node.attrib.get("href") or "").strip()
            rel = (node.attrib.get("rel") or "alternate").strip()
            if href and rel in {"alternate", ""}:
                link = href
                break
            if href and not link:
                link = href
            if node.text and not link:
                link = node.text.strip()
        if not link:
            link = identifier if identifier.startswith("http") else ""
        if not link:
            continue
        out.append(
            {
                "surface": surface,
                "feed": feed_url,
                "id": identifier,
                "url": link,
                "title": _strip_markup(title)[:500],
                "summary": _strip_markup(summary)[:900],
                "updated": updated,
            }
        )
    return out


def _canonical_reddit_url(url: str) -> str:
    try:
        parsed = urllib.parse.urlsplit(url)
    except Exception:
        return url
    host = parsed.netloc.lower()
    if host.endswith("reddit.com"):
        path = re.sub(r"/+", "/", parsed.path.rstrip("/")) or "/"
        return urllib.parse.urlunsplit(("https", "www.reddit.com", path, "", ""))
    return urllib.parse.urlunsplit((parsed.scheme or "https", parsed.netloc, parsed.path, parsed.query, ""))


def _canonical_x_url(url: str) -> str:
    try:
        parsed = urllib.parse.urlsplit(url)
    except Exception:
        return url
    match = re.match(r"^/([^/]+)/status/(\d+)", parsed.path)
    if match:
        return f"https://x.com/{match.group(1)}/status/{match.group(2)}"
    if parsed.netloc.lower() in {"x.com", "www.x.com", "twitter.com", "www.twitter.com"}:
        return urllib.parse.urlunsplit(("https", "x.com", parsed.path.rstrip("/"), "", ""))
    return urllib.parse.urlunsplit((parsed.scheme or "https", parsed.netloc, parsed.path, parsed.query, ""))


def _dedupe(items: list[dict[str, Any]], canonicalizer, limit: int) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    seen: set[str] = set()
    for item in items:
        canonical = canonicalizer(str(item.get("url") or ""))
        marker = canonical or str(item.get("id") or "")
        if not marker or marker in seen:
            continue
        seen.add(marker)
        item = dict(item)
        item["canonicalUrl"] = canonical
        out.append(item)
        if len(out) >= limit:
            break
    return out


def _intake_path(name: str) -> Path:
    return STATE_ROOT / "unknown-frontier-ai" / "intake" / f"{name}-latest.json"


def reddit_rss_intake(target: int) -> dict[str, Any]:
    target = max(1, target)
    seeds = _csv_env("VESPER_REDDIT_SEEDS", DEFAULT_REDDIT_SEEDS)
    comment_seeds = _csv_env("VESPER_REDDIT_COMMENT_SEEDS", tuple(seeds[:6]))
    feeds: list[tuple[str, str]] = []

    groups = [seeds[index:index + 5] for index in range(0, len(seeds), 5)]
    for index, group in enumerate(groups):
        if group:
            joined = "+".join(group)
            feeds.append(("reddit-new", f"https://www.reddit.com/r/{joined}/new.rss?limit=100"))
        if index < len(comment_seeds):
            sub = comment_seeds[index]
            feeds.append(("reddit-comments", f"https://www.reddit.com/r/{sub}/comments.rss?limit=100"))
    for sub in comment_seeds[len(groups):]:
        feeds.append(("reddit-comments", f"https://www.reddit.com/r/{sub}/comments.rss?limit=100"))

    raw: list[dict[str, Any]] = []
    errors: list[dict[str, str]] = []
    fetched_feeds: list[str] = []
    for surface, url in feeds:
        try:
            payload, _ = _fetch(url, accept="application/atom+xml,application/rss+xml,text/xml;q=0.9,*/*;q=0.5")
            raw.extend(_feed_entries(payload, surface=surface, feed_url=url))
            fetched_feeds.append(url)
        except Exception as exc:
            errors.append({"url": url, "error": str(exc)[-500:]})
        if len(_dedupe(raw, _canonical_reddit_url, target)) >= target:
            break

    candidates = _dedupe(raw, _canonical_reddit_url, target)
    subreddit_pattern = re.compile(r"(?<![\w/])r/([A-Za-z0-9_]{2,40})")
    discovered: list[str] = []
    known = {item.lower() for item in seeds}
    for item in candidates:
        text = f"{item.get('title', '')} {item.get('summary', '')}"
        for match in subreddit_pattern.findall(text):
            if match.lower() not in known and match not in discovered:
                discovered.append(match)

    result = {
        "source": "reddit-rss",
        "generatedAt": now().isoformat(timespec="seconds"),
        "target": target,
        "rawEntries": len(raw),
        "canonicalCandidates": len(candidates),
        "feedsFetched": fetched_feeds,
        "errors": errors,
        "discoveredSubreddits": discovered[:40],
        "candidates": candidates,
    }
    atomic_json(_intake_path("reddit"), result)
    return result


class _StatusLinkParser(HTMLParser):
    def __init__(self, base_url: str):
        super().__init__(convert_charrefs=True)
        self.base_url = base_url
        self.items: list[dict[str, Any]] = []
        self._current_href = ""
        self._current_text: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag != "a":
            return
        values = {key: value or "" for key, value in attrs}
        href = values.get("href", "")
        if re.search(r"/[^/]+/status/\d+", href):
            self._current_href = urllib.parse.urljoin(self.base_url + "/", href)
            self._current_text = []

    def handle_data(self, data: str) -> None:
        if self._current_href:
            self._current_text.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "a" and self._current_href:
            self.items.append(
                {
                    "url": self._current_href,
                    "title": re.sub(r"\s+", " ", " ".join(self._current_text)).strip()[:500],
                    "summary": "",
                }
            )
            self._current_href = ""
            self._current_text = []


def _x_search_mirror(mirror: str, query: str) -> tuple[list[dict[str, Any]], str]:
    params = urllib.parse.urlencode({"f": "tweets", "q": query})
    rss_url = f"{mirror.rstrip('/')}/search/rss?{params}"
    try:
        payload, _ = _fetch(rss_url, accept="application/atom+xml,application/rss+xml,text/xml;q=0.9,*/*;q=0.5")
        entries = _feed_entries(payload, surface="x-mirror-rss", feed_url=rss_url)
        if entries:
            for item in entries:
                item["mirror"] = mirror
                item["query"] = query
            return entries, "rss"
    except Exception:
        pass

    html_url = f"{mirror.rstrip('/')}/search?{params}"
    payload, _ = _fetch(html_url)
    parser = _StatusLinkParser(mirror.rstrip("/"))
    parser.feed(payload.decode("utf-8", errors="replace"))
    for item in parser.items:
        item.update({"surface": "x-mirror-html", "mirror": mirror, "query": query, "feed": html_url})
    return parser.items, "html"


def x_mirror_intake(target: int) -> dict[str, Any]:
    target = max(1, target)
    mirrors = [item.rstrip("/") for item in _csv_env("VESPER_X_MIRRORS", DEFAULT_X_MIRRORS)]
    queries = _csv_env("VESPER_X_QUERIES", DEFAULT_X_QUERIES)
    raw: list[dict[str, Any]] = []
    errors: list[dict[str, str]] = []
    health: dict[str, dict[str, Any]] = {mirror: {"successes": 0, "failures": 0, "modes": []} for mirror in mirrors}

    for query in queries:
        query_succeeded = False
        for mirror in mirrors:
            try:
                items, mode = _x_search_mirror(mirror, query)
                if not items:
                    raise RuntimeError("empty search result")
                raw.extend(items)
                health[mirror]["successes"] += 1
                if mode not in health[mirror]["modes"]:
                    health[mirror]["modes"].append(mode)
                query_succeeded = True
                break
            except Exception as exc:
                health[mirror]["failures"] += 1
                errors.append({"mirror": mirror, "query": query, "error": str(exc)[-500:]})
        if len(_dedupe(raw, _canonical_x_url, target)) >= target:
            break
        if not query_succeeded:
            time.sleep(0.2)

    candidates = _dedupe(raw, _canonical_x_url, target)
    result = {
        "source": "x-mirror",
        "generatedAt": now().isoformat(timespec="seconds"),
        "target": target,
        "rawEntries": len(raw),
        "canonicalCandidates": len(candidates),
        "mirrors": health,
        "errors": errors,
        "queries": queries,
        "candidates": candidates,
    }
    atomic_json(_intake_path("x"), result)
    return result


def compact_intake(value: dict[str, Any], max_chars: int = 76000) -> str:
    compact = dict(value)
    candidates = compact.get("candidates") or []
    if isinstance(candidates, list):
        compact["candidates"] = [
            {
                "url": item.get("canonicalUrl") or item.get("url"),
                "title": item.get("title", ""),
                "summary": item.get("summary", ""),
                "updated": item.get("updated", ""),
                "surface": item.get("surface", ""),
                "query": item.get("query", ""),
                "mirror": item.get("mirror", ""),
            }
            for item in candidates
            if isinstance(item, dict)
        ]
    return json.dumps(compact, ensure_ascii=False, indent=2)[:max_chars]
