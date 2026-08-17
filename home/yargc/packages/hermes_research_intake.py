from __future__ import annotations

import html
import json
import os
import re
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from html.parser import HTMLParser
from pathlib import Path
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, load_json, now

USER_AGENT = os.environ.get(
    "VESPER_RESEARCH_USER_AGENT",
    "VesperHermesResearch/1.0 (+local personal research; contact: configured-by-user)",
)
FETCH_TIMEOUT = max(3, int(os.environ.get("VESPER_RESEARCH_FETCH_TIMEOUT", "10")))
MAX_RESPONSE_BYTES = max(256_000, int(os.environ.get("VESPER_RESEARCH_MAX_RESPONSE_BYTES", "2500000")))

CENTRAL_REDDIT_ANCHORS = (
    "MoneroMeansMoney",
    "Monero",
    "LocalLLaMA",
    "privacy",
    "NixOS",
    "selfhosted",
    "Tor",
    "netsec",
)

DEFAULT_REDDIT_SEEDS = (
    "MachineLearning",
    "programming",
    "opensource",
    "linux",
    "rust",
    "golang",
    "cybersecurity",
    "webdev",
)

DEFAULT_REDDIT_COMMENT_ANCHORS = (
    "MoneroMeansMoney",
    "Monero",
    "LocalLLaMA",
    "privacy",
)

CENTRAL_X_ANCHORS = (
    "Teknium",
    "thdxr",
    "XOpenSource",
    "ZixuanLi_",
    "eigenwallet",
    "kyc_rip",
    "XBToshi",
    "schmidt1024",
    "XMRHub_org",
    "CR1337",
    "linuxuser1996",
    "Examare1",
    "ZcashLabs",
    "akaclandestine",
    "DailyDarkWeb",
    "SimpleXChat",
)

DEFAULT_X_QUERIES = (
    '"AI agent" code',
    '"coding agent"',
    '"agent harness"',
    '"LLM inference" open source',
    '"open source AI" tool',
    '"NixOS" AI',
    'privacy AI developer',
    'Monero privacy tool',
    'Tor privacy research',
)

DEFAULT_X_MIRRORS = (
    "https://xcancel.com",
    "https://nitter.net",
)

SOURCE_REGISTRY_PATH = STATE_ROOT / "unknown-frontier-ai" / "source-registry.json"
TIER_PRIORITY = {"promoted": 3, "trusted": 2, "probation": 1}


def _csv_env(name: str, default: tuple[str, ...]) -> list[str]:
    raw = os.environ.get(name, "")
    values = [item.strip() for item in raw.split(",") if item.strip()] if raw else list(default)
    return list(dict.fromkeys(values))


def _clean_reddit_name(value: str) -> str:
    value = value.strip().lstrip("/")
    if value.lower().startswith("r/"):
        value = value[2:]
    return re.sub(r"[^A-Za-z0-9_]", "", value)[:40]


def _clean_x_name(value: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "", value.strip().lstrip("@"))[:15]


def _source_key(kind: str, name: str) -> str:
    clean = _clean_reddit_name(name) if kind == "reddit" else _clean_x_name(name)
    return f"{kind}:{clean.lower()}"


def _empty_registry() -> dict[str, Any]:
    return {"version": 1, "updatedAt": "", "sources": {}}


def load_source_registry() -> dict[str, Any]:
    value = load_json(SOURCE_REGISTRY_PATH, _empty_registry())
    if not isinstance(value, dict):
        value = _empty_registry()
    sources = value.get("sources")
    if not isinstance(sources, dict):
        sources = {}
        value["sources"] = sources

    changed = False
    stamp = now().isoformat(timespec="seconds")
    for kind, anchors in (("reddit", CENTRAL_REDDIT_ANCHORS), ("x", CENTRAL_X_ANCHORS)):
        for raw_name in anchors:
            name = _clean_reddit_name(raw_name) if kind == "reddit" else _clean_x_name(raw_name)
            key = _source_key(kind, name)
            entry = sources.get(key)
            if not isinstance(entry, dict):
                entry = {
                    "kind": kind,
                    "name": name,
                    "tier": "anchor",
                    "protected": True,
                    "score": 10.0,
                    "hits": 0,
                    "failures": 0,
                    "firstSeen": stamp,
                    "lastSeen": stamp,
                    "lastUseful": "",
                    "origin": "central-config",
                }
                sources[key] = entry
                changed = True
            else:
                if entry.get("tier") != "anchor" or entry.get("protected") is not True:
                    entry["tier"] = "anchor"
                    entry["protected"] = True
                    changed = True
                entry["kind"] = kind
                entry["name"] = name
                entry["score"] = max(10.0, float(entry.get("score") or 0.0))
    if changed:
        value["updatedAt"] = stamp
        atomic_json(SOURCE_REGISTRY_PATH, value)
    return value


def _save_source_registry(value: dict[str, Any]) -> None:
    value["updatedAt"] = now().isoformat(timespec="seconds")
    atomic_json(SOURCE_REGISTRY_PATH, value)


def discover_source(kind: str, raw_name: str, *, origin: str) -> None:
    name = _clean_reddit_name(raw_name) if kind == "reddit" else _clean_x_name(raw_name)
    if not name:
        return
    registry = load_source_registry()
    sources = registry["sources"]
    key = _source_key(kind, name)
    stamp = now().isoformat(timespec="seconds")
    entry = sources.get(key)
    if not isinstance(entry, dict):
        sources[key] = {
            "kind": kind,
            "name": name,
            "tier": "probation",
            "protected": False,
            "score": 0.25,
            "hits": 0,
            "failures": 0,
            "firstSeen": stamp,
            "lastSeen": stamp,
            "lastUseful": "",
            "origin": origin,
        }
    else:
        entry["lastSeen"] = stamp
        if not entry.get("origin"):
            entry["origin"] = origin
    _save_source_registry(registry)


def _note_source_fetch(kind: str, raw_name: str, *, ok: bool) -> None:
    name = _clean_reddit_name(raw_name) if kind == "reddit" else _clean_x_name(raw_name)
    if not name:
        return
    registry = load_source_registry()
    sources = registry["sources"]
    key = _source_key(kind, name)
    entry = sources.get(key)
    if not isinstance(entry, dict):
        discover_source(kind, name, origin="fetch")
        registry = load_source_registry()
        sources = registry["sources"]
        entry = sources.get(key)
    if not isinstance(entry, dict):
        return
    entry["lastSeen"] = now().isoformat(timespec="seconds")
    if not ok:
        entry["failures"] = int(entry.get("failures") or 0) + 1
    _save_source_registry(registry)


def _dynamic_sources(kind: str, limit: int) -> list[str]:
    registry = load_source_registry()
    entries = [
        item
        for item in registry.get("sources", {}).values()
        if isinstance(item, dict) and item.get("kind") == kind and not item.get("protected")
    ]
    entries.sort(
        key=lambda item: (
            TIER_PRIORITY.get(str(item.get("tier") or "probation"), 0),
            float(item.get("score") or 0.0),
            int(item.get("hits") or 0),
            str(item.get("lastUseful") or item.get("lastSeen") or ""),
        ),
        reverse=True,
    )
    return [str(item.get("name")) for item in entries[: max(0, limit)] if item.get("name")]


def _source_names_from_report(source: str, report: dict[str, Any]) -> set[str]:
    names: set[str] = set()
    urls: list[str] = []
    for candidate in report.get("candidates") or []:
        if not isinstance(candidate, dict):
            continue
        for url in candidate.get("urls") or []:
            if isinstance(url, str):
                urls.append(url)
    for item in report.get("sources") or []:
        if isinstance(item, dict) and isinstance(item.get("url"), str):
            urls.append(str(item["url"]))
        elif isinstance(item, str):
            urls.append(item)

    if source == "reddit":
        for url in urls:
            match = re.search(r"(?:^|/)r/([A-Za-z0-9_]{2,40})(?:/|$)", urllib.parse.urlsplit(url).path, re.I)
            if match:
                names.add(_clean_reddit_name(match.group(1)))
    elif source == "x":
        for url in urls:
            path = urllib.parse.urlsplit(url).path
            match = re.match(r"^/([A-Za-z0-9_]{1,15})(?:/status/\d+|/?$)", path)
            if match:
                names.add(_clean_x_name(match.group(1)))

    patch = report.get("statePatch") or {}
    candidate_sources = patch.get("candidateSources") if isinstance(patch, dict) else []
    if isinstance(candidate_sources, list):
        for item in candidate_sources:
            text = item if isinstance(item, str) else json.dumps(item, ensure_ascii=False)
            if source == "reddit":
                for match in re.findall(r"(?:^|\s|/)r/([A-Za-z0-9_]{2,40})", text, re.I):
                    names.add(_clean_reddit_name(match))
            elif source == "x":
                for match in re.findall(r"@([A-Za-z0-9_]{1,15})", text):
                    names.add(_clean_x_name(match))
    return {name for name in names if name}


def reinforce_source_registry(source: str, report: dict[str, Any]) -> None:
    if source not in {"reddit", "x"}:
        return
    useful_names = _source_names_from_report(source, report)
    if not useful_names:
        return
    for name in useful_names:
        discover_source(source, name, origin=f"{source}-scout")

    registry = load_source_registry()
    sources = registry["sources"]
    stamp = now().isoformat(timespec="seconds")
    for name in useful_names:
        entry = sources.get(_source_key(source, name))
        if not isinstance(entry, dict):
            continue
        entry["hits"] = int(entry.get("hits") or 0) + 1
        entry["score"] = round(float(entry.get("score") or 0.0) + 1.0, 3)
        entry["lastUseful"] = stamp
        entry["lastSeen"] = stamp
        if not entry.get("protected"):
            hits = int(entry.get("hits") or 0)
            score = float(entry.get("score") or 0.0)
            if hits >= 4 and score >= 4.0:
                entry["tier"] = "promoted"
            elif hits >= 2 and score >= 2.0:
                entry["tier"] = "trusted"
            else:
                entry["tier"] = "probation"
    _save_source_registry(registry)


def source_registry_summary() -> dict[str, Any]:
    registry = load_source_registry()
    entries = [item for item in registry.get("sources", {}).values() if isinstance(item, dict)]
    counts: dict[str, int] = {}
    for item in entries:
        tier = str(item.get("tier") or "unknown")
        counts[tier] = counts.get(tier, 0) + 1
    return {
        "counts": counts,
        "redditAnchors": list(_csv_env("VESPER_REDDIT_ANCHORS", CENTRAL_REDDIT_ANCHORS)),
        "xAnchors": list(_csv_env("VESPER_X_ANCHORS", CENTRAL_X_ANCHORS)),
        "dynamicReddit": _dynamic_sources("reddit", 12),
        "dynamicX": _dynamic_sources("x", 16),
    }


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
        value = dict(item)
        value["canonicalUrl"] = canonical
        out.append(value)
        if len(out) >= limit:
            break
    return out


def _intake_path(name: str) -> Path:
    return STATE_ROOT / "unknown-frontier-ai" / "intake" / f"{name}-latest.json"


def reddit_rss_intake(target: int) -> dict[str, Any]:
    target = max(1, target)
    anchors = [_clean_reddit_name(item) for item in _csv_env("VESPER_REDDIT_ANCHORS", CENTRAL_REDDIT_ANCHORS)]
    general = [_clean_reddit_name(item) for item in _csv_env("VESPER_REDDIT_SEEDS", DEFAULT_REDDIT_SEEDS)]
    dynamic = [_clean_reddit_name(item) for item in _dynamic_sources("reddit", 12)]
    comment_anchors = [_clean_reddit_name(item) for item in _csv_env("VESPER_REDDIT_COMMENT_SEEDS", DEFAULT_REDDIT_COMMENT_ANCHORS)]

    for name in anchors:
        discover_source("reddit", name, origin="central-config")

    feeds: list[tuple[str, str, str | None]] = []
    for sub in anchors:
        feeds.append(("reddit-anchor-new", f"https://www.reddit.com/r/{sub}/new.rss?limit=50", sub))
    for sub in comment_anchors:
        feeds.append(("reddit-anchor-comments", f"https://www.reddit.com/r/{sub}/comments.rss?limit=100", sub))

    secondary = list(dict.fromkeys([item for item in dynamic + general if item and item.lower() not in {a.lower() for a in anchors}]))
    for index in range(0, len(secondary), 5):
        group = secondary[index:index + 5]
        if group:
            feeds.append(("reddit-discovery-new", f"https://www.reddit.com/r/{'+'.join(group)}/new.rss?limit=100", None))

    raw: list[dict[str, Any]] = []
    errors: list[dict[str, str]] = []
    fetched_feeds: list[str] = []
    anchor_feeds_completed = 0
    for surface, url, source_name in feeds:
        try:
            payload, _ = _fetch(url, accept="application/atom+xml,application/rss+xml,text/xml;q=0.9,*/*;q=0.5")
            entries = _feed_entries(payload, surface=surface, feed_url=url)
            raw.extend(entries)
            fetched_feeds.append(url)
            if source_name:
                _note_source_fetch("reddit", source_name, ok=True)
                anchor_feeds_completed += 1
        except Exception as exc:
            errors.append({"url": url, "error": str(exc)[-500:]})
            if source_name:
                _note_source_fetch("reddit", source_name, ok=False)
        if source_name is None and len(_dedupe(raw, _canonical_reddit_url, target)) >= target:
            break

    candidates = _dedupe(raw, _canonical_reddit_url, target)
    subreddit_pattern = re.compile(r"(?<![\w/])r/([A-Za-z0-9_]{2,40})")
    discovered: list[str] = []
    known = {item.lower() for item in anchors + general + dynamic}
    for item in candidates:
        text = f"{item.get('title', '')} {item.get('summary', '')}"
        for match in subreddit_pattern.findall(text):
            clean = _clean_reddit_name(match)
            if clean and clean.lower() not in known and clean not in discovered:
                discovered.append(clean)
                discover_source("reddit", clean, origin="reddit-rss-mention")

    result = {
        "source": "reddit-rss",
        "generatedAt": now().isoformat(timespec="seconds"),
        "target": target,
        "rawEntries": len(raw),
        "canonicalCandidates": len(candidates),
        "anchors": anchors,
        "dynamicSources": dynamic,
        "anchorFeedsCompleted": anchor_feeds_completed,
        "feedsFetched": fetched_feeds,
        "errors": errors,
        "discoveredSubreddits": discovered[:40],
        "sourceRegistry": source_registry_summary(),
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


def _x_profile_mirror(mirror: str, account: str) -> tuple[list[dict[str, Any]], str]:
    account = _clean_x_name(account)
    rss_url = f"{mirror.rstrip('/')}/{account}/rss"
    try:
        payload, _ = _fetch(rss_url, accept="application/atom+xml,application/rss+xml,text/xml;q=0.9,*/*;q=0.5")
        entries = _feed_entries(payload, surface="x-anchor-rss", feed_url=rss_url)
        if entries:
            for item in entries:
                item["mirror"] = mirror
                item["sourceAccount"] = account
            return entries, "rss"
    except Exception:
        pass

    html_url = f"{mirror.rstrip('/')}/{account}"
    payload, _ = _fetch(html_url)
    parser = _StatusLinkParser(mirror.rstrip("/"))
    parser.feed(payload.decode("utf-8", errors="replace"))
    for item in parser.items:
        item.update({"surface": "x-anchor-html", "mirror": mirror, "sourceAccount": account, "feed": html_url})
    return parser.items, "html"


def x_mirror_intake(target: int) -> dict[str, Any]:
    target = max(1, target)
    mirrors = [item.rstrip("/") for item in _csv_env("VESPER_X_MIRRORS", DEFAULT_X_MIRRORS)]
    queries = _csv_env("VESPER_X_QUERIES", DEFAULT_X_QUERIES)
    anchors = [_clean_x_name(item) for item in _csv_env("VESPER_X_ANCHORS", CENTRAL_X_ANCHORS)]
    dynamic = [_clean_x_name(item) for item in _dynamic_sources("x", 16)]

    for name in anchors:
        discover_source("x", name, origin="central-config")

    raw: list[dict[str, Any]] = []
    errors: list[dict[str, str]] = []
    health: dict[str, dict[str, Any]] = {mirror: {"successes": 0, "failures": 0, "modes": []} for mirror in mirrors}
    anchor_accounts_completed: list[str] = []

    for account in list(dict.fromkeys(anchors + dynamic)):
        success = False
        for mirror in mirrors:
            try:
                items, mode = _x_profile_mirror(mirror, account)
                if not items:
                    raise RuntimeError("empty profile result")
                raw.extend(items)
                health[mirror]["successes"] += 1
                if mode not in health[mirror]["modes"]:
                    health[mirror]["modes"].append(mode)
                _note_source_fetch("x", account, ok=True)
                anchor_accounts_completed.append(account)
                success = True
                break
            except Exception as exc:
                health[mirror]["failures"] += 1
                errors.append({"mirror": mirror, "account": account, "error": str(exc)[-500:]})
        if not success:
            _note_source_fetch("x", account, ok=False)

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
    discovered: list[str] = []
    known = {item.lower() for item in anchors + dynamic}
    mention_pattern = re.compile(r"@([A-Za-z0-9_]{1,15})")
    for item in candidates:
        canonical = str(item.get("canonicalUrl") or item.get("url") or "")
        match = re.match(r"^/([A-Za-z0-9_]{1,15})/status/\d+", urllib.parse.urlsplit(canonical).path)
        names = [match.group(1)] if match else []
        names.extend(mention_pattern.findall(f"{item.get('title', '')} {item.get('summary', '')}"))
        for raw_name in names:
            clean = _clean_x_name(raw_name)
            if clean and clean.lower() not in known and clean not in discovered:
                discovered.append(clean)
                discover_source("x", clean, origin="x-mirror-edge")

    result = {
        "source": "x-mirror",
        "generatedAt": now().isoformat(timespec="seconds"),
        "target": target,
        "rawEntries": len(raw),
        "canonicalCandidates": len(candidates),
        "anchors": anchors,
        "dynamicSources": dynamic,
        "anchorAccountsCompleted": anchor_accounts_completed,
        "mirrors": health,
        "errors": errors,
        "queries": queries,
        "discoveredAccounts": discovered[:60],
        "sourceRegistry": source_registry_summary(),
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
                "sourceAccount": item.get("sourceAccount", ""),
            }
            for item in candidates
            if isinstance(item, dict)
        ]
    return json.dumps(compact, ensure_ascii=False, indent=2)[:max_chars]
