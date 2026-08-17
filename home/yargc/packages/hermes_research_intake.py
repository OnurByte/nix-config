from __future__ import annotations

import html
import json
import os
import re
import time
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET
from datetime import datetime
from html.parser import HTMLParser
from pathlib import Path
from typing import Any, Callable

from hermes_automation_common import STATE_ROOT, atomic_json, load_json, now

USER_AGENT = os.environ.get(
    "VESPER_RESEARCH_USER_AGENT",
    "VesperHermesResearch/1.0 (+local personal research; contact: configured-by-user)",
)
FETCH_TIMEOUT = max(3, int(os.environ.get("VESPER_RESEARCH_FETCH_TIMEOUT", "10")))
MAX_RESPONSE_BYTES = max(256_000, int(os.environ.get("VESPER_RESEARCH_MAX_RESPONSE_BYTES", "2500000")))

# Protected defaults are durable anchors, not an allowlist. Runtime env vars may
# replace them; load_source_registry() reconciles protection against that set.
CENTRAL_REDDIT_ANCHORS = (
    "MoneroMeansMoney",
    "Monero",
    "vibecoding",
    "ClaudeCode",
    "codex",
    "opencodeCLI",
    "cursor",
    "privacy",
    "NixOS",
    "Tor",
    "netsec",
)

# Explicit negative preference: do not let source discovery re-promote these.
IGNORED_REDDIT_SOURCES = ("LocalLLaMA",)

DEFAULT_REDDIT_SEEDS = (
    "selfhosted",
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
    "vibecoding",
    "ClaudeCode",
    "codex",
    "opencodeCLI",
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
    '"coding agent" workflow',
    '"vibe coding" workflow',
    '"agent harness" coding',
    '"Claude Code" skill',
    '"Codex CLI" agent',
    '"OpenCode" agent',
    '"SKILL.md" coding agent',
    '"AGENTS.md" coding agent',
    '"MCP" coding agent',
    '"context engineering" coding agent',
    '"Monero" privacy tool',
    '"Monero" atomic swap',
    '"Cuprate" Monero',
    '"Tor" privacy research',
    '"NixOS" agent workflow',
)

DEFAULT_X_MIRRORS = (
    "https://xcancel.com",
    "https://nitter.net",
)

SOURCE_REGISTRY_PATH = STATE_ROOT / "unknown-frontier-ai" / "source-registry.json"
TIER_PRIORITY = {"promoted": 4, "trusted": 3, "probation": 2, "retired": 0}
SOURCE_BUDGET_RATIOS = {"anchor": 0.45, "dynamic": 0.30, "explore": 0.25}


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


def _ignored_reddit_names() -> set[str]:
    return {
        _clean_reddit_name(item).lower()
        for item in _csv_env("VESPER_REDDIT_IGNORED", IGNORED_REDDIT_SOURCES)
        if _clean_reddit_name(item)
    }


def _source_key(kind: str, name: str) -> str:
    clean = _clean_reddit_name(name) if kind == "reddit" else _clean_x_name(name)
    return f"{kind}:{clean.lower()}"


def _configured_anchors(kind: str) -> list[str]:
    if kind == "reddit":
        ignored = _ignored_reddit_names()
        return [
            clean
            for item in _csv_env("VESPER_REDDIT_ANCHORS", CENTRAL_REDDIT_ANCHORS)
            if (clean := _clean_reddit_name(item)) and clean.lower() not in ignored
        ]
    if kind == "x":
        return [
            clean
            for item in _csv_env("VESPER_X_ANCHORS", CENTRAL_X_ANCHORS)
            if (clean := _clean_x_name(item))
        ]
    return []


def _empty_registry() -> dict[str, Any]:
    return {"version": 2, "updatedAt": "", "sources": {}}


def _parse_time(value: Any) -> datetime | None:
    if not value:
        return None
    try:
        parsed = datetime.fromisoformat(str(value))
    except Exception:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.astimezone()
    return parsed


def _apply_lifecycle(entry: dict[str, Any], stamp: datetime) -> bool:
    if entry.get("retiredReason") == "user-excluded":
        changed = entry.get("tier") != "retired" or entry.get("protected") is not False or float(entry.get("score") or 0.0) != 0.0
        entry["tier"] = "retired"
        entry["protected"] = False
        entry["score"] = 0.0
        return changed

    if entry.get("protected"):
        if entry.get("tier") != "anchor":
            entry["tier"] = "anchor"
            return True
        return False

    changed = False
    tier = str(entry.get("tier") or "probation")
    hits = max(0, int(entry.get("hits") or 0))
    failures = max(0, int(entry.get("failures") or 0))
    last_useful = _parse_time(entry.get("lastUseful"))
    age_days = max(0, (stamp - last_useful).days) if last_useful is not None else None

    desired = tier
    if hits >= 4 and (age_days is None or age_days <= 45):
        desired = "promoted"
    elif hits >= 2 and (age_days is None or age_days <= 60):
        desired = "trusted"
    elif tier != "retired":
        desired = "probation"

    if hits == 0 and failures >= 8:
        desired = "retired"
        entry["retiredReason"] = "repeated-fetch-failure"
    elif age_days is not None and age_days >= 120:
        desired = "retired"
        entry["retiredReason"] = "stale-no-recent-use"

    if desired != tier:
        entry["tier"] = desired
        changed = True
    return changed


def load_source_registry() -> dict[str, Any]:
    value = load_json(SOURCE_REGISTRY_PATH, _empty_registry())
    if not isinstance(value, dict):
        value = _empty_registry()
    sources = value.get("sources")
    if not isinstance(sources, dict):
        sources = {}
        value["sources"] = sources

    stamp_dt = now()
    stamp = stamp_dt.isoformat(timespec="seconds")
    current_anchor_keys: set[str] = set()
    changed = False

    for kind in ("reddit", "x"):
        for name in _configured_anchors(kind):
            key = _source_key(kind, name)
            current_anchor_keys.add(key)
            entry = sources.get(key)
            if not isinstance(entry, dict):
                sources[key] = {
                    "kind": kind,
                    "name": name,
                    "tier": "anchor",
                    "protected": True,
                    "score": 10.0,
                    "hits": 0,
                    "observations": 0,
                    "failures": 0,
                    "firstSeen": stamp,
                    "lastSeen": stamp,
                    "lastUseful": "",
                    "origin": "central-config",
                }
                changed = True
                continue
            entry["kind"] = kind
            entry["name"] = name
            if entry.get("tier") != "anchor" or entry.get("protected") is not True:
                entry["tier"] = "anchor"
                entry["protected"] = True
                changed = True
            if float(entry.get("score") or 0.0) < 10.0:
                entry["score"] = 10.0
                changed = True

    ignored_reddit = _ignored_reddit_names()
    for key, entry in list(sources.items()):
        if not isinstance(entry, dict):
            continue
        removed_anchor = (
            entry.get("protected")
            and entry.get("origin") == "central-config"
            and key not in current_anchor_keys
        )
        explicitly_ignored = (
            entry.get("kind") == "reddit"
            and str(entry.get("name") or "").lower() in ignored_reddit
        )
        if removed_anchor or explicitly_ignored:
            entry["protected"] = False
            entry["tier"] = "retired"
            entry["score"] = 0.0
            entry["retiredReason"] = "user-excluded" if explicitly_ignored else "removed-from-central-config"
            changed = True
        if _apply_lifecycle(entry, stamp_dt):
            changed = True

    value["version"] = 2
    if changed:
        value["updatedAt"] = stamp
        atomic_json(SOURCE_REGISTRY_PATH, value)
    return value


def _save_source_registry(value: dict[str, Any]) -> None:
    value["version"] = 2
    value["updatedAt"] = now().isoformat(timespec="seconds")
    atomic_json(SOURCE_REGISTRY_PATH, value)


def discover_source(kind: str, raw_name: str, *, origin: str) -> None:
    name = _clean_reddit_name(raw_name) if kind == "reddit" else _clean_x_name(raw_name)
    if not name:
        return
    if kind == "reddit" and name.lower() in _ignored_reddit_names():
        return

    registry = load_source_registry()
    sources = registry["sources"]
    key = _source_key(kind, name)
    stamp = now().isoformat(timespec="seconds")
    entry = sources.get(key)
    is_anchor = key in {_source_key(kind, item) for item in _configured_anchors(kind)}
    if not isinstance(entry, dict):
        sources[key] = {
            "kind": kind,
            "name": name,
            "tier": "anchor" if is_anchor else "probation",
            "protected": is_anchor,
            "score": 10.0 if is_anchor else 0.25,
            "hits": 0,
            "observations": 0,
            "failures": 0,
            "firstSeen": stamp,
            "lastSeen": stamp,
            "lastUseful": "",
            "origin": "central-config" if is_anchor else origin,
        }
    else:
        if entry.get("tier") == "retired" and not is_anchor:
            entry["tier"] = "probation"
            entry["score"] = max(0.25, min(1.0, float(entry.get("score") or 0.0)))
            entry["retiredReason"] = ""
        entry["lastSeen"] = stamp
        if is_anchor:
            entry["tier"] = "anchor"
            entry["protected"] = True
            entry["score"] = max(10.0, float(entry.get("score") or 0.0))
        if not entry.get("origin"):
            entry["origin"] = origin
    _save_source_registry(registry)


def _note_source_fetch(kind: str, raw_name: str, *, ok: bool) -> None:
    name = _clean_reddit_name(raw_name) if kind == "reddit" else _clean_x_name(raw_name)
    if not name:
        return
    if kind == "reddit" and name.lower() in _ignored_reddit_names():
        return
    discover_source(kind, name, origin="fetch")
    registry = load_source_registry()
    entry = registry["sources"].get(_source_key(kind, name))
    if not isinstance(entry, dict):
        return
    entry["observations"] = int(entry.get("observations") or 0) + 1
    entry["lastSeen"] = now().isoformat(timespec="seconds")
    if not ok:
        entry["failures"] = int(entry.get("failures") or 0) + 1
    _apply_lifecycle(entry, now())
    _save_source_registry(registry)


def _dynamic_sources(kind: str, limit: int) -> list[str]:
    registry = load_source_registry()
    ignored = _ignored_reddit_names() if kind == "reddit" else set()
    entries = [
        item
        for item in registry.get("sources", {}).values()
        if isinstance(item, dict)
        and item.get("kind") == kind
        and not item.get("protected")
        and item.get("tier") != "retired"
        and str(item.get("name") or "").lower() not in ignored
    ]
    entries.sort(
        key=lambda item: (
            TIER_PRIORITY.get(str(item.get("tier") or "probation"), 0),
            float(item.get("score") or 0.0),
            int(item.get("hits") or 0),
            -int(item.get("failures") or 0),
            str(item.get("lastUseful") or item.get("lastSeen") or ""),
        ),
        reverse=True,
    )
    return [str(item.get("name")) for item in entries[: max(0, limit)] if item.get("name")]


def _names_from_urls(source: str, urls: list[str]) -> set[str]:
    names: set[str] = set()
    if source == "reddit":
        ignored = _ignored_reddit_names()
        for url in urls:
            match = re.search(r"(?:^|/)r/([A-Za-z0-9_]{2,40})(?:/|$)", urllib.parse.urlsplit(url).path, re.I)
            if match:
                name = _clean_reddit_name(match.group(1))
                if name and name.lower() not in ignored:
                    names.add(name)
    elif source == "x":
        for url in urls:
            path = urllib.parse.urlsplit(url).path
            match = re.match(r"^/([A-Za-z0-9_]{1,15})(?:/status/\d+|/?$)", path)
            if match:
                names.add(_clean_x_name(match.group(1)))
    return {name for name in names if name}


def _useful_source_names(source: str, report: dict[str, Any]) -> set[str]:
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
    return _names_from_urls(source, urls)


def _candidate_source_names(source: str, report: dict[str, Any]) -> set[str]:
    patch = report.get("statePatch") or {}
    candidate_sources = patch.get("candidateSources") if isinstance(patch, dict) else []
    names: set[str] = set()
    if not isinstance(candidate_sources, list):
        return names
    for item in candidate_sources:
        text = item if isinstance(item, str) else json.dumps(item, ensure_ascii=False)
        if source == "reddit":
            ignored = _ignored_reddit_names()
            for match in re.findall(r"(?:^|\s|/)r/([A-Za-z0-9_]{2,40})", text, re.I):
                name = _clean_reddit_name(match)
                if name and name.lower() not in ignored:
                    names.add(name)
        elif source == "x":
            for match in re.findall(r"@([A-Za-z0-9_]{1,15})", text):
                names.add(_clean_x_name(match))
    return {name for name in names if name}


def reinforce_source_registry(source: str, report: dict[str, Any]) -> None:
    if source not in {"reddit", "x"}:
        return

    # candidateSources are hints only: probation without useful-hit credit.
    useful_names = _useful_source_names(source, report)
    candidate_names = _candidate_source_names(source, report) - useful_names
    for name in candidate_names:
        discover_source(source, name, origin=f"{source}-candidate")
    for name in useful_names:
        discover_source(source, name, origin=f"{source}-scout")

    if not useful_names:
        return

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
        _apply_lifecycle(entry, now())
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
        "redditAnchors": _configured_anchors("reddit"),
        "xAnchors": _configured_anchors("x"),
        "dynamicReddit": _dynamic_sources("reddit", 12),
        "dynamicX": _dynamic_sources("x", 16),
    }


def _fetch(
    url: str,
    *,
    accept: str = "text/html,application/atom+xml,application/rss+xml;q=0.9,*/*;q=0.5",
) -> tuple[bytes, str]:
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
        out.append({
            "surface": surface,
            "feed": feed_url,
            "id": identifier,
            "url": link,
            "title": _strip_markup(title)[:500],
            "summary": _strip_markup(summary)[:900],
            "updated": updated,
        })
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


def _dedupe(
    items: list[dict[str, Any]],
    canonicalizer: Callable[[str], str],
    limit: int | None = None,
) -> list[dict[str, Any]]:
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
        if limit is not None and len(out) >= limit:
            break
    return out


def _pool_quotas(target: int) -> dict[str, int]:
    target = max(1, target)
    anchor = max(1, round(target * SOURCE_BUDGET_RATIOS["anchor"]))
    dynamic = max(1, round(target * SOURCE_BUDGET_RATIOS["dynamic"]))
    if anchor + dynamic >= target:
        dynamic = max(1, target - anchor - 1) if target >= 3 else max(0, target - anchor)
    explore = max(0, target - anchor - dynamic)
    total = anchor + dynamic + explore
    if total < target:
        explore += target - total
    elif total > target:
        anchor = max(0, anchor - (total - target))
    return {"anchor": anchor, "dynamic": dynamic, "explore": explore}


def _round_robin(
    items: list[dict[str, Any]],
    limit: int,
    *,
    key_fields: tuple[str, ...],
) -> list[dict[str, Any]]:
    if limit <= 0:
        return []
    buckets: dict[str, list[dict[str, Any]]] = {}
    order: list[str] = []
    for item in items:
        key = ""
        for field in key_fields:
            value = str(item.get(field) or "").strip()
            if value:
                key = f"{field}:{value.lower()}"
                break
        if not key:
            key = f"url:{item.get('canonicalUrl') or item.get('url') or len(order)}"
        if key not in buckets:
            buckets[key] = []
            order.append(key)
        buckets[key].append(item)

    out: list[dict[str, Any]] = []
    while len(out) < limit:
        progressed = False
        for key in order:
            bucket = buckets.get(key) or []
            if not bucket:
                continue
            out.append(bucket.pop(0))
            progressed = True
            if len(out) >= limit:
                break
        if not progressed:
            break
    return out


def _select_candidate_pools(
    pools: dict[str, list[dict[str, Any]]],
    target: int,
    *,
    canonicalizer: Callable[[str], str],
    key_fields: tuple[str, ...],
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    quotas = _pool_quotas(target)
    prepared = {name: _dedupe(items, canonicalizer) for name, items in pools.items()}
    selected: list[dict[str, Any]] = []

    for name in ("anchor", "dynamic", "explore"):
        chosen = _round_robin(prepared.get(name, []), quotas[name], key_fields=key_fields)
        for item in chosen:
            value = dict(item)
            value["budgetPool"] = name
            selected.append(value)

    selected = _dedupe(selected, canonicalizer)
    if len(selected) < target:
        selected_markers = {item.get("canonicalUrl") for item in selected}
        leftovers: list[dict[str, Any]] = []
        for name in ("explore", "dynamic", "anchor"):
            for item in prepared.get(name, []):
                marker = item.get("canonicalUrl")
                if marker and marker in selected_markers:
                    continue
                value = dict(item)
                value["budgetPool"] = name
                leftovers.append(value)
        selected.extend(_round_robin(leftovers, target - len(selected), key_fields=key_fields))
        selected = _dedupe(selected, canonicalizer, target)

    actual_counts = {"anchor": 0, "dynamic": 0, "explore": 0}
    for item in selected:
        pool = str(item.get("budgetPool") or "")
        if pool in actual_counts:
            actual_counts[pool] += 1

    return selected[:target], {
        "ratios": SOURCE_BUDGET_RATIOS,
        "quotas": quotas,
        "selected": actual_counts,
        "available": {name: len(items) for name, items in prepared.items()},
    }


def _intake_path(name: str) -> Path:
    return STATE_ROOT / "unknown-frontier-ai" / "intake" / f"{name}-latest.json"


def _fetch_reddit_feed(surface: str, url: str, source_name: str | None) -> list[dict[str, Any]]:
    payload, _ = _fetch(url, accept="application/atom+xml,application/rss+xml,text/xml;q=0.9,*/*;q=0.5")
    items = _feed_entries(payload, surface=surface, feed_url=url)
    for item in items:
        if source_name:
            item["sourceName"] = source_name
        else:
            match = re.search(r"/r/([A-Za-z0-9_]{2,40})/", urllib.parse.urlsplit(str(item.get("url") or "")).path, re.I)
            if match:
                item["sourceName"] = _clean_reddit_name(match.group(1))
    return items


def reddit_rss_intake(target: int) -> dict[str, Any]:
    target = max(1, target)
    anchors = _configured_anchors("reddit")
    general = [
        clean
        for item in _csv_env("VESPER_REDDIT_SEEDS", DEFAULT_REDDIT_SEEDS)
        if (clean := _clean_reddit_name(item)) and clean.lower() not in _ignored_reddit_names()
    ]
    dynamic = [_clean_reddit_name(item) for item in _dynamic_sources("reddit", 12)]
    comment_anchors = [
        clean
        for item in _csv_env("VESPER_REDDIT_COMMENT_SEEDS", DEFAULT_REDDIT_COMMENT_ANCHORS)
        if (clean := _clean_reddit_name(item)) and clean.lower() not in _ignored_reddit_names()
    ]

    for name in anchors:
        discover_source("reddit", name, origin="central-config")

    pools: dict[str, list[dict[str, Any]]] = {"anchor": [], "dynamic": [], "explore": []}
    errors: list[dict[str, str]] = []
    fetched_feeds: list[str] = []
    anchor_feeds_completed = 0

    for sub in anchors:
        url = f"https://www.reddit.com/r/{sub}/new.rss?limit=50"
        try:
            pools["anchor"].extend(_fetch_reddit_feed("reddit-anchor-new", url, sub))
            fetched_feeds.append(url)
            _note_source_fetch("reddit", sub, ok=True)
            anchor_feeds_completed += 1
        except Exception as exc:
            errors.append({"url": url, "error": str(exc)[-500:]})
            _note_source_fetch("reddit", sub, ok=False)

    for sub in comment_anchors:
        url = f"https://www.reddit.com/r/{sub}/comments.rss?limit=100"
        try:
            pools["anchor"].extend(_fetch_reddit_feed("reddit-anchor-comments", url, sub))
            fetched_feeds.append(url)
            _note_source_fetch("reddit", sub, ok=True)
            anchor_feeds_completed += 1
        except Exception as exc:
            errors.append({"url": url, "error": str(exc)[-500:]})
            _note_source_fetch("reddit", sub, ok=False)

    anchor_lower = {item.lower() for item in anchors}
    for sub in list(dict.fromkeys(item for item in dynamic if item and item.lower() not in anchor_lower)):
        url = f"https://www.reddit.com/r/{sub}/new.rss?limit=40"
        try:
            pools["dynamic"].extend(_fetch_reddit_feed("reddit-dynamic-new", url, sub))
            fetched_feeds.append(url)
            _note_source_fetch("reddit", sub, ok=True)
        except Exception as exc:
            errors.append({"url": url, "error": str(exc)[-500:]})
            _note_source_fetch("reddit", sub, ok=False)

    dynamic_lower = {item.lower() for item in dynamic}
    exploration = [
        item for item in general
        if item and item.lower() not in anchor_lower and item.lower() not in dynamic_lower
    ]
    for index in range(0, len(exploration), 5):
        group = exploration[index:index + 5]
        if not group:
            continue
        url = f"https://www.reddit.com/r/{'+'.join(group)}/new.rss?limit=100"
        try:
            pools["explore"].extend(_fetch_reddit_feed("reddit-explore-new", url, None))
            fetched_feeds.append(url)
        except Exception as exc:
            errors.append({"url": url, "error": str(exc)[-500:]})

    candidates, budget = _select_candidate_pools(
        pools,
        target,
        canonicalizer=_canonical_reddit_url,
        key_fields=("sourceName", "feed"),
    )

    subreddit_pattern = re.compile(r"(?<![\w/])r/([A-Za-z0-9_]{2,40})")
    discovered: list[str] = []
    known = {item.lower() for item in anchors + general + dynamic}
    ignored = _ignored_reddit_names()
    for item in candidates:
        text = f"{item.get('title', '')} {item.get('summary', '')}"
        for match in subreddit_pattern.findall(text):
            clean = _clean_reddit_name(match)
            if clean and clean.lower() not in known and clean.lower() not in ignored and clean not in discovered:
                discovered.append(clean)
                discover_source("reddit", clean, origin="reddit-rss-mention")

    result = {
        "source": "reddit-rss",
        "generatedAt": now().isoformat(timespec="seconds"),
        "target": target,
        "rawEntries": sum(len(items) for items in pools.values()),
        "canonicalCandidates": len(candidates),
        "anchors": anchors,
        "ignored": sorted(ignored),
        "dynamicSources": dynamic,
        "anchorFeedsCompleted": anchor_feeds_completed,
        "budget": budget,
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
            self.items.append({
                "url": self._current_href,
                "title": re.sub(r"\s+", " ", " ".join(self._current_text)).strip()[:500],
                "summary": "",
            })
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
        entries = _feed_entries(payload, surface="x-profile-rss", feed_url=rss_url)
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
        item.update({"surface": "x-profile-html", "mirror": mirror, "sourceAccount": account, "feed": html_url})
    return parser.items, "html"


def _x_fetch_account(
    account: str,
    mirrors: list[str],
    health: dict[str, dict[str, Any]],
    errors: list[dict[str, str]],
) -> list[dict[str, Any]]:
    for mirror in mirrors:
        try:
            items, mode = _x_profile_mirror(mirror, account)
            if not items:
                raise RuntimeError("empty profile result")
            health[mirror]["successes"] += 1
            if mode not in health[mirror]["modes"]:
                health[mirror]["modes"].append(mode)
            _note_source_fetch("x", account, ok=True)
            return items
        except Exception as exc:
            health[mirror]["failures"] += 1
            errors.append({"mirror": mirror, "account": account, "error": str(exc)[-500:]})
    _note_source_fetch("x", account, ok=False)
    return []


def x_mirror_intake(target: int) -> dict[str, Any]:
    target = max(1, target)
    mirrors = [item.rstrip("/") for item in _csv_env("VESPER_X_MIRRORS", DEFAULT_X_MIRRORS)]
    queries = _csv_env("VESPER_X_QUERIES", DEFAULT_X_QUERIES)
    anchors = _configured_anchors("x")
    dynamic = [_clean_x_name(item) for item in _dynamic_sources("x", 16)]

    for name in anchors:
        discover_source("x", name, origin="central-config")

    pools: dict[str, list[dict[str, Any]]] = {"anchor": [], "dynamic": [], "explore": []}
    errors: list[dict[str, str]] = []
    health: dict[str, dict[str, Any]] = {
        mirror: {"successes": 0, "failures": 0, "modes": []} for mirror in mirrors
    }
    completed_accounts: list[str] = []

    for account in anchors:
        items = _x_fetch_account(account, mirrors, health, errors)
        if items:
            pools["anchor"].extend(items)
            completed_accounts.append(account)

    anchor_lower = {item.lower() for item in anchors}
    for account in list(dict.fromkeys(item for item in dynamic if item and item.lower() not in anchor_lower)):
        items = _x_fetch_account(account, mirrors, health, errors)
        if items:
            pools["dynamic"].extend(items)
            completed_accounts.append(account)

    for query in queries:
        query_succeeded = False
        for mirror in mirrors:
            try:
                items, mode = _x_search_mirror(mirror, query)
                if not items:
                    raise RuntimeError("empty search result")
                health[mirror]["successes"] += 1
                if mode not in health[mirror]["modes"]:
                    health[mirror]["modes"].append(mode)
                pools["explore"].extend(items)
                query_succeeded = True
                break
            except Exception as exc:
                health[mirror]["failures"] += 1
                errors.append({"mirror": mirror, "query": query, "error": str(exc)[-500:]})
        if not query_succeeded:
            time.sleep(0.2)

    candidates, budget = _select_candidate_pools(
        pools,
        target,
        canonicalizer=_canonical_x_url,
        key_fields=("sourceAccount", "query", "mirror"),
    )

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
        "rawEntries": sum(len(items) for items in pools.values()),
        "canonicalCandidates": len(candidates),
        "anchors": anchors,
        "dynamicSources": dynamic,
        "accountsCompleted": completed_accounts,
        "budget": budget,
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
                "budgetPool": item.get("budgetPool", ""),
                "sourceName": item.get("sourceName", ""),
                "sourceAccount": item.get("sourceAccount", ""),
                "query": item.get("query", ""),
                "mirror": item.get("mirror", ""),
            }
            for item in candidates
            if isinstance(item, dict)
        ]
    return json.dumps(compact, ensure_ascii=False, indent=2)[:max_chars]
