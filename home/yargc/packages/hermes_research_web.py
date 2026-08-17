from __future__ import annotations

import hashlib
import ipaddress
import json
import os
import re
import shutil
import subprocess
import urllib.parse
from datetime import datetime
from html.parser import HTMLParser
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, now
from hermes_research_intake import (
    SOURCE_REGISTRY_PATH,
    TIER_PRIORITY,
    _apply_lifecycle,
    _save_source_registry,
    _select_candidate_pools,
    load_source_registry,
)

CENTRAL_WEB_ANCHORS: tuple[dict[str, str], ...] = (
    {
        "name": "OP Bible OPSEC",
        "url": "https://opbible7nans45sg33cbyeiwqmlp5fu7lklu6jd6f3mivrjeqadco5yd.onion/opsec/",
        "topic": "privacy-opsec",
    },
    {
        "name": "Monero Forum",
        "url": "https://monero.forum/",
        "topic": "monero-privacy",
    },
)

TOR_SOCKS_HOST = os.environ.get("VESPER_TOR_SOCKS_HOST", "127.0.0.1")
TOR_SOCKS_PORT = max(1, min(65535, int(os.environ.get("VESPER_TOR_SOCKS_PORT", "9050"))))
WEB_FETCH_TIMEOUT = max(5, int(os.environ.get("VESPER_WEB_FETCH_TIMEOUT", "35")))
WEB_MAX_RESPONSE_BYTES = max(256_000, int(os.environ.get("VESPER_WEB_MAX_RESPONSE_BYTES", "2500000")))
WEB_DEEP_CONTENT_CHARS = max(2000, int(os.environ.get("VESPER_WEB_DEEP_CONTENT_CHARS", "9000")))
WEB_DYNAMIC_SOURCE_LIMIT = max(0, min(24, int(os.environ.get("VESPER_WEB_DYNAMIC_SOURCE_LIMIT", "10"))))
WEB_USER_AGENT = os.environ.get(
    "VESPER_RESEARCH_USER_AGENT",
    "VesperHermesResearch/1.0 (+local personal research; contact: configured-by-user)",
)
WEB_STATE_ROOT = STATE_ROOT / "unknown-frontier-ai" / "web"

_POSITIVE_TERMS = {
    "monero": 7,
    "xmr": 6,
    "privacy": 6,
    "opsec": 7,
    "tor": 5,
    "onion": 5,
    "security": 4,
    "vulnerability": 5,
    "exploit": 4,
    "wallet": 4,
    "atomic swap": 6,
    "swap": 3,
    "payment": 3,
    "private": 3,
    "simplex": 5,
    "grapheneos": 4,
    "cuprate": 6,
    "coding agent": 6,
    "codex": 5,
    "claude code": 5,
    "opencode": 5,
    "hermes": 5,
    "mcp": 4,
    "agent": 2,
    "research": 2,
}
_NEGATIVE_TERMS = {
    "price": -8,
    "trading": -8,
    "chart": -7,
    "going short": -8,
    "market cap": -7,
    "bull": -4,
    "bear": -4,
    "casino": -6,
}
_PLATFORM_HOSTS = {
    "github.com",
    "www.github.com",
    "x.com",
    "www.x.com",
    "twitter.com",
    "www.twitter.com",
    "reddit.com",
    "www.reddit.com",
    "old.reddit.com",
    "youtube.com",
    "www.youtube.com",
    "youtu.be",
    "t.me",
}
_TRACKING_PARAMS = {
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "ref",
    "source",
}


class _PageParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.title_parts: list[str] = []
        self.text_parts: list[str] = []
        self.links: list[tuple[str, str]] = []
        self._skip_depth = 0
        self._in_title = False
        self._href = ""
        self._anchor_text: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        tag = tag.lower()
        if tag in {"script", "style", "noscript", "svg", "canvas"}:
            self._skip_depth += 1
            return
        if self._skip_depth:
            return
        if tag == "title":
            self._in_title = True
        if tag == "a":
            values = {key.lower(): value or "" for key, value in attrs}
            self._href = values.get("href", "").strip()
            self._anchor_text = []

    def handle_endtag(self, tag: str) -> None:
        tag = tag.lower()
        if tag in {"script", "style", "noscript", "svg", "canvas"}:
            if self._skip_depth:
                self._skip_depth -= 1
            return
        if self._skip_depth:
            return
        if tag == "title":
            self._in_title = False
        if tag == "a" and self._href:
            text = re.sub(r"\s+", " ", " ".join(self._anchor_text)).strip()
            self.links.append((self._href, text[:500]))
            self._href = ""
            self._anchor_text = []

    def handle_data(self, data: str) -> None:
        if self._skip_depth:
            return
        text = re.sub(r"\s+", " ", data).strip()
        if not text:
            return
        self.text_parts.append(text)
        if self._in_title:
            self.title_parts.append(text)
        if self._href:
            self._anchor_text.append(text)


def _is_onion_host(host: str) -> bool:
    labels = host.lower().rstrip(".").split(".")
    if len(labels) < 2 or labels[-1] != "onion":
        return False
    service = labels[-2]
    return bool(re.fullmatch(r"[a-z2-7]{56}", service))


def _host_is_safe(host: str) -> bool:
    host = host.lower().rstrip(".")
    if not host or host in {"localhost", "localhost.localdomain"} or host.endswith(".local"):
        return False
    if _is_onion_host(host):
        return True
    try:
        value = ipaddress.ip_address(host.strip("[]"))
    except ValueError:
        return True
    return not (
        value.is_private
        or value.is_loopback
        or value.is_link_local
        or value.is_multicast
        or value.is_reserved
        or value.is_unspecified
    )


def canonical_web_url(url: str, *, base: str | None = None) -> str:
    value = urllib.parse.urljoin(base, url) if base else url
    try:
        parsed = urllib.parse.urlsplit(value.strip())
    except Exception:
        return ""
    scheme = parsed.scheme.lower()
    if scheme not in {"http", "https"} or not parsed.hostname or parsed.username or parsed.password:
        return ""
    host = parsed.hostname.lower().rstrip(".")
    if not _host_is_safe(host):
        return ""
    try:
        port = parsed.port
    except ValueError:
        return ""
    if port not in {None, 80, 443}:
        return ""
    netloc = host
    if port is not None and not ((scheme == "http" and port == 80) or (scheme == "https" and port == 443)):
        netloc = f"{host}:{port}"
    path = re.sub(r"/{2,}", "/", parsed.path or "/")
    query_pairs = [
        (key, val)
        for key, val in urllib.parse.parse_qsl(parsed.query, keep_blank_values=True)
        if key.lower() not in _TRACKING_PARAMS and not key.lower().startswith("utm_")
    ]
    query = urllib.parse.urlencode(query_pairs, doseq=True)
    return urllib.parse.urlunsplit((scheme, netloc, path, query, ""))


def is_onion_url(url: str) -> bool:
    try:
        host = urllib.parse.urlsplit(url).hostname or ""
    except Exception:
        return False
    return _is_onion_host(host)


def _source_identity(url: str, *, keep_path: bool = False) -> str:
    canonical = canonical_web_url(url)
    if not canonical:
        return ""
    parsed = urllib.parse.urlsplit(canonical)
    path = parsed.path if keep_path else "/"
    if keep_path and not path.endswith("/"):
        path += "/" if "." not in path.rsplit("/", 1)[-1] else ""
    return urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, path or "/", "", ""))


def _web_key(identity: str) -> str:
    return "web:" + hashlib.sha256(identity.encode()).hexdigest()[:20]


def _configured_web_anchors() -> list[dict[str, str]]:
    anchors: list[dict[str, str]] = []
    for item in CENTRAL_WEB_ANCHORS:
        url = canonical_web_url(item["url"])
        if url:
            anchors.append({"name": item["name"], "url": url, "topic": item["topic"]})
    extra = os.environ.get("VESPER_WEB_ANCHORS", "").strip()
    if extra:
        for raw in extra.split(","):
            url = canonical_web_url(raw.strip())
            if url and all(existing["url"] != url for existing in anchors):
                anchors.append({"name": urllib.parse.urlsplit(url).hostname or url, "url": url, "topic": "user-anchor"})
    return anchors


def _ensure_web_registry() -> dict[str, Any]:
    registry = load_source_registry()
    sources = registry.setdefault("sources", {})
    stamp = now().isoformat(timespec="seconds")
    changed = False
    for anchor in _configured_web_anchors():
        identity = _source_identity(anchor["url"], keep_path=True)
        key = _web_key(identity)
        entry = sources.get(key)
        if not isinstance(entry, dict):
            sources[key] = {
                "kind": "web",
                "name": identity,
                "label": anchor["name"],
                "topic": anchor["topic"],
                "tier": "anchor",
                "protected": True,
                "score": 10.0,
                "hits": 0,
                "observations": 0,
                "failures": 0,
                "firstSeen": stamp,
                "lastSeen": stamp,
                "lastUseful": "",
                "origin": "central-web-config",
            }
            changed = True
            continue
        entry["kind"] = "web"
        entry["name"] = identity
        entry["label"] = anchor["name"]
        entry["topic"] = anchor["topic"]
        if entry.get("tier") != "anchor" or entry.get("protected") is not True:
            entry["tier"] = "anchor"
            entry["protected"] = True
            changed = True
        if float(entry.get("score") or 0.0) < 10.0:
            entry["score"] = 10.0
            changed = True
    if changed:
        _save_source_registry(registry)
    return registry


def _discover_web_source(url: str, *, origin: str) -> None:
    canonical = canonical_web_url(url)
    if not canonical:
        return
    parsed = urllib.parse.urlsplit(canonical)
    if parsed.hostname and parsed.hostname.lower() in _PLATFORM_HOSTS:
        return
    identity = _source_identity(canonical)
    if not identity:
        return
    registry = _ensure_web_registry()
    sources = registry["sources"]
    key = _web_key(identity)
    if any(
        isinstance(item, dict) and item.get("kind") == "web" and item.get("protected") and _source_identity(str(item.get("name") or "")) == identity
        for item in sources.values()
    ):
        return
    stamp = now().isoformat(timespec="seconds")
    entry = sources.get(key)
    if not isinstance(entry, dict):
        sources[key] = {
            "kind": "web",
            "name": identity,
            "label": parsed.hostname or identity,
            "topic": "discovered",
            "tier": "probation",
            "protected": False,
            "score": 0.25,
            "hits": 0,
            "observations": 0,
            "failures": 0,
            "firstSeen": stamp,
            "lastSeen": stamp,
            "lastUseful": "",
            "origin": origin,
        }
    else:
        if entry.get("tier") == "retired" and entry.get("retiredReason") != "user-excluded":
            entry["tier"] = "probation"
            entry["score"] = max(0.25, min(1.0, float(entry.get("score") or 0.0)))
            entry["retiredReason"] = ""
        entry["lastSeen"] = stamp
    _save_source_registry(registry)


def _note_web_fetch(identity: str, *, ok: bool) -> None:
    registry = _ensure_web_registry()
    sources = registry["sources"]
    key = _web_key(identity)
    entry = sources.get(key)
    if not isinstance(entry, dict):
        _discover_web_source(identity, origin="web-fetch")
        registry = _ensure_web_registry()
        entry = registry["sources"].get(key)
    if not isinstance(entry, dict):
        return
    entry["observations"] = int(entry.get("observations") or 0) + 1
    entry["lastSeen"] = now().isoformat(timespec="seconds")
    if not ok:
        entry["failures"] = int(entry.get("failures") or 0) + 1
    _apply_lifecycle(entry, now())
    _save_source_registry(registry)


def _dynamic_web_sources(limit: int = WEB_DYNAMIC_SOURCE_LIMIT) -> list[dict[str, Any]]:
    registry = _ensure_web_registry()
    entries = [
        item
        for item in registry.get("sources", {}).values()
        if isinstance(item, dict)
        and item.get("kind") == "web"
        and not item.get("protected")
        and item.get("tier") != "retired"
        and canonical_web_url(str(item.get("name") or ""))
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
    return entries[: max(0, limit)]


def _curl_fetch(url: str, timeout: int = WEB_FETCH_TIMEOUT) -> dict[str, Any]:
    canonical = canonical_web_url(url)
    if not canonical:
        raise RuntimeError(f"unsafe or unsupported URL: {url}")
    curl = shutil.which("curl")
    if not curl:
        raise RuntimeError("curl executable not found")
    marker = "__VESPER_CURL_META__"
    command = [
        curl,
        "--silent",
        "--show-error",
        "--location",
        "--compressed",
        "--fail-with-body",
        "--connect-timeout",
        "12",
        "--max-time",
        str(timeout),
        "--max-filesize",
        str(WEB_MAX_RESPONSE_BYTES),
        "--proto",
        "=http,https",
        "--proto-redir",
        "=http,https",
        "--user-agent",
        WEB_USER_AGENT,
        "--header",
        "Accept: text/html,application/xhtml+xml,text/plain,application/json,application/xml;q=0.9,*/*;q=0.2",
    ]
    transport = "direct"
    if is_onion_url(canonical):
        command.extend(["--socks5-hostname", f"{TOR_SOCKS_HOST}:{TOR_SOCKS_PORT}"])
        transport = "tor-socks5h"
    command.extend([
        "--write-out",
        f"\n{marker}%{{http_code}}\t%{{content_type}}\t%{{url_effective}}",
        canonical,
    ])
    completed = subprocess.run(command, capture_output=True, timeout=timeout + 8, check=False)
    stdout = completed.stdout.decode("utf-8", errors="replace")
    stderr = completed.stderr.decode("utf-8", errors="replace")
    if completed.returncode != 0:
        raise RuntimeError(f"curl rc={completed.returncode}: {(stderr or stdout)[-1200:]}")
    body, sep, meta = stdout.rpartition("\n" + marker)
    if not sep:
        raise RuntimeError("curl response metadata missing")
    parts = meta.split("\t", 2)
    status = int(parts[0]) if parts and parts[0].isdigit() else 0
    content_type = parts[1] if len(parts) > 1 else ""
    effective = canonical_web_url(parts[2]) if len(parts) > 2 else canonical
    if status >= 400:
        raise RuntimeError(f"HTTP {status} for {canonical}")
    textual = content_type.startswith("text/") or any(
        token in content_type for token in ("json", "xml", "xhtml", "html")
    )
    return {
        "url": canonical,
        "effectiveUrl": effective or canonical,
        "status": status,
        "contentType": content_type,
        "transport": transport,
        "text": body if textual else "",
        "bytes": len(completed.stdout),
        "textual": textual,
    }


def _parse_document(fetch: dict[str, Any]) -> dict[str, Any]:
    text = str(fetch.get("text") or "")
    content_type = str(fetch.get("contentType") or "")
    if "html" not in content_type.lower() and "xhtml" not in content_type.lower() and "<html" not in text[:1000].lower():
        clean = re.sub(r"\s+", " ", text).strip()
        return {"title": "", "content": clean, "links": []}
    parser = _PageParser()
    try:
        parser.feed(text)
    except Exception:
        pass
    title = re.sub(r"\s+", " ", " ".join(parser.title_parts)).strip()[:500]
    content = re.sub(r"\s+", " ", " ".join(parser.text_parts)).strip()
    return {"title": title, "content": content, "links": parser.links}


def fetch_document(url: str, *, max_chars: int = 50000) -> dict[str, Any]:
    fetch = _curl_fetch(url)
    parsed = _parse_document(fetch)
    return {
        "url": fetch["url"],
        "effectiveUrl": fetch["effectiveUrl"],
        "status": fetch["status"],
        "contentType": fetch["contentType"],
        "transport": fetch["transport"],
        "title": parsed["title"],
        "content": str(parsed["content"])[: max(1, max_chars)],
        "links": [
            {"url": canonical_web_url(href, base=str(fetch["effectiveUrl"])), "text": text}
            for href, text in parsed["links"][:500]
            if canonical_web_url(href, base=str(fetch["effectiveUrl"]))
        ],
    }


def _candidate_score(item: dict[str, Any]) -> int:
    haystack = f"{item.get('title', '')} {item.get('url', '')} {item.get('summary', '')}".lower()
    score = 0
    for term, weight in _POSITIVE_TERMS.items():
        if term in haystack:
            score += weight
    for term, weight in _NEGATIVE_TERMS.items():
        if term in haystack:
            score += weight
    if item.get("isOnion"):
        score += 2
    if item.get("topic") in {"privacy-opsec", "monero-privacy"}:
        score += 2
    return score


def _page_candidates(
    source_name: str,
    source_url: str,
    topic: str,
    document: dict[str, Any],
    *,
    pool: str,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    base = str(document.get("effectiveUrl") or source_url)
    source_host = urllib.parse.urlsplit(source_url).hostname or ""
    internal: list[dict[str, Any]] = []
    external: list[dict[str, Any]] = []
    root = {
        "url": canonical_web_url(source_url),
        "title": str(document.get("title") or source_name),
        "summary": str(document.get("content") or "")[:1200],
        "content": str(document.get("content") or "")[:WEB_DEEP_CONTENT_CHARS],
        "sourceName": source_name,
        "sourceOrigin": source_url,
        "surface": "web-core-root" if pool == "anchor" else "web-dynamic-root",
        "topic": topic,
        "isOnion": is_onion_url(source_url),
        "transport": str(document.get("transport") or ""),
    }
    root["heuristicScore"] = _candidate_score(root)
    internal.append(root)

    for link in document.get("links") or []:
        if not isinstance(link, dict):
            continue
        url = canonical_web_url(str(link.get("url") or ""), base=base)
        if not url or url == root["url"]:
            continue
        host = urllib.parse.urlsplit(url).hostname or ""
        item = {
            "url": url,
            "title": str(link.get("text") or host or url)[:500],
            "summary": "",
            "sourceName": source_name,
            "sourceOrigin": source_url,
            "surface": "web-core-link" if pool == "anchor" else "web-dynamic-link",
            "topic": topic,
            "isOnion": is_onion_url(url),
            "transport": "tor-socks5h" if is_onion_url(url) else "direct",
        }
        item["heuristicScore"] = _candidate_score(item)
        if host.lower() == source_host.lower():
            internal.append(item)
        else:
            item["surface"] = "web-external-link"
            external.append(item)
    return internal, external


def _fetch_source(source_name: str, source_url: str, topic: str, *, pool: str) -> tuple[list[dict[str, Any]], list[dict[str, Any]], dict[str, Any]]:
    identity = _source_identity(source_url, keep_path=(pool == "anchor"))
    try:
        document = fetch_document(source_url, max_chars=WEB_DEEP_CONTENT_CHARS * 2)
        document["transport"] = "tor-socks5h" if is_onion_url(source_url) else "direct"
        _note_web_fetch(identity, ok=True)
        internal, external = _page_candidates(source_name, source_url, topic, document, pool=pool)
        return internal, external, {"source": source_url, "ok": True, "transport": document["transport"], "status": document.get("status", 0)}
    except Exception as exc:
        _note_web_fetch(identity, ok=False)
        return [], [], {
            "source": source_url,
            "ok": False,
            "transport": "tor-socks5h" if is_onion_url(source_url) else "direct",
            "error": str(exc)[-1200:],
        }


def _prefetch_onion_content(candidates: list[dict[str, Any]], limit: int) -> tuple[int, list[dict[str, str]]]:
    fetched = 0
    errors: list[dict[str, str]] = []
    for item in candidates:
        if fetched >= max(0, limit):
            break
        if not item.get("isOnion") or item.get("content"):
            continue
        url = str(item.get("url") or "")
        try:
            document = fetch_document(url, max_chars=WEB_DEEP_CONTENT_CHARS)
            item["title"] = str(document.get("title") or item.get("title") or url)
            item["content"] = str(document.get("content") or "")[:WEB_DEEP_CONTENT_CHARS]
            item["summary"] = item["content"][:1200]
            item["transport"] = str(document.get("transport") or "tor-socks5h")
            item["prefetched"] = True
            fetched += 1
        except Exception as exc:
            errors.append({"url": url, "error": str(exc)[-800:]})
    return fetched, errors


def web_registry_summary() -> dict[str, Any]:
    registry = _ensure_web_registry()
    entries = [
        item for item in registry.get("sources", {}).values()
        if isinstance(item, dict) and item.get("kind") == "web"
    ]
    counts: dict[str, int] = {}
    for item in entries:
        tier = str(item.get("tier") or "unknown")
        counts[tier] = counts.get(tier, 0) + 1
    return {
        "counts": counts,
        "anchors": [item["url"] for item in _configured_web_anchors()],
        "dynamic": [str(item.get("name")) for item in _dynamic_web_sources()],
    }


def web_core_intake(target: int, *, deep_fetch_limit: int = 10) -> dict[str, Any]:
    target = max(1, target)
    anchors = _configured_web_anchors()
    dynamics = _dynamic_web_sources()
    pools: dict[str, list[dict[str, Any]]] = {"anchor": [], "dynamic": [], "explore": []}
    source_health: list[dict[str, Any]] = []

    for anchor in anchors:
        internal, external, health = _fetch_source(anchor["name"], anchor["url"], anchor["topic"], pool="anchor")
        pools["anchor"].extend(internal)
        pools["explore"].extend(external)
        source_health.append(health)

    for entry in dynamics:
        url = canonical_web_url(str(entry.get("name") or ""))
        if not url:
            continue
        label = str(entry.get("label") or urllib.parse.urlsplit(url).hostname or url)
        topic = str(entry.get("topic") or "learned")
        internal, external, health = _fetch_source(label, url, topic, pool="dynamic")
        pools["dynamic"].extend(internal)
        pools["explore"].extend(external)
        source_health.append(health)

    for pool in pools.values():
        pool.sort(key=lambda item: (int(item.get("heuristicScore") or 0), str(item.get("title") or "")), reverse=True)

    candidates, budget = _select_candidate_pools(
        pools,
        target,
        canonicalizer=canonical_web_url,
        key_fields=("sourceName", "sourceOrigin"),
    )

    # External domains are source hypotheses only; useful-hit credit is deferred
    # until the scout actually returns evidence-bearing candidates from them.
    discovered: list[str] = []
    for item in candidates:
        if item.get("budgetPool") != "explore":
            continue
        url = canonical_web_url(str(item.get("url") or ""))
        if not url:
            continue
        host = (urllib.parse.urlsplit(url).hostname or "").lower()
        if host in _PLATFORM_HOSTS:
            continue
        identity = _source_identity(url)
        if identity and identity not in discovered:
            discovered.append(identity)
            _discover_web_source(identity, origin="web-link-edge")

    onion_prefetched, prefetch_errors = _prefetch_onion_content(candidates, deep_fetch_limit)
    errors = [item for item in source_health if not item.get("ok")]
    errors.extend({"source": item["url"], "ok": False, "transport": "tor-socks5h", "error": item["error"]} for item in prefetch_errors)
    result = {
        "source": "web-core",
        "generatedAt": now().isoformat(timespec="seconds"),
        "target": target,
        "canonicalCandidates": len(candidates),
        "anchors": anchors,
        "dynamicSources": [str(item.get("name")) for item in dynamics],
        "sourceHealth": source_health,
        "errors": errors,
        "budget": budget,
        "onionPrefetched": onion_prefetched,
        "discoveredSources": discovered[:60],
        "webRegistry": web_registry_summary(),
        "candidates": candidates,
    }
    atomic_json(WEB_STATE_ROOT / "latest.json", result)
    return result


def _urls_from_report(report: dict[str, Any]) -> set[str]:
    urls: set[str] = set()
    for candidate in report.get("candidates") or []:
        if not isinstance(candidate, dict):
            continue
        for raw in candidate.get("urls") or []:
            if isinstance(raw, str):
                url = canonical_web_url(raw)
                if url:
                    urls.add(url)
    for source in report.get("sources") or []:
        raw = source.get("url") if isinstance(source, dict) else source
        if isinstance(raw, str):
            url = canonical_web_url(raw)
            if url:
                urls.add(url)
    return urls


def _candidate_source_urls(report: dict[str, Any]) -> set[str]:
    patch = report.get("statePatch") or {}
    values = patch.get("candidateSources") if isinstance(patch, dict) else []
    urls: set[str] = set()
    if not isinstance(values, list):
        return urls
    for value in values:
        text = value if isinstance(value, str) else json.dumps(value, ensure_ascii=False)
        for raw in re.findall(r"https?://[^\s\]\[()<>\"']+", text):
            url = canonical_web_url(raw.rstrip(".,;:"))
            if url:
                urls.add(url)
    return urls


def reinforce_web_registry(report: dict[str, Any]) -> None:
    useful_urls = _urls_from_report(report)
    candidate_urls = _candidate_source_urls(report) - useful_urls
    for url in candidate_urls:
        _discover_web_source(url, origin="web-scout-candidate")
    if not useful_urls:
        return

    registry = _ensure_web_registry()
    sources = registry["sources"]
    stamp = now().isoformat(timespec="seconds")
    anchor_identities = {_source_identity(item["url"], keep_path=True) for item in _configured_web_anchors()}
    useful_identities: set[str] = set()
    for url in useful_urls:
        parsed = urllib.parse.urlsplit(url)
        if parsed.hostname and parsed.hostname.lower() in _PLATFORM_HOSTS:
            continue
        matching_anchor = next((identity for identity in anchor_identities if url.startswith(identity)), "")
        identity = matching_anchor or _source_identity(url)
        if identity:
            useful_identities.add(identity)
            if not matching_anchor:
                _discover_web_source(identity, origin="web-scout")

    registry = _ensure_web_registry()
    sources = registry["sources"]
    for identity in useful_identities:
        key = _web_key(identity)
        entry = sources.get(key)
        if not isinstance(entry, dict):
            continue
        entry["hits"] = int(entry.get("hits") or 0) + 1
        entry["score"] = round(float(entry.get("score") or 0.0) + 1.0, 3)
        entry["lastUseful"] = stamp
        entry["lastSeen"] = stamp
        _apply_lifecycle(entry, now())
    _save_source_registry(registry)


def compact_web_intake(value: dict[str, Any], max_chars: int = 100000) -> str:
    compact = {key: val for key, val in value.items() if key != "candidates"}
    candidates: list[dict[str, Any]] = []
    for item in value.get("candidates") or []:
        if not isinstance(item, dict):
            continue
        candidates.append({
            "url": item.get("url"),
            "title": item.get("title", ""),
            "summary": item.get("summary", ""),
            "content": str(item.get("content") or "")[:WEB_DEEP_CONTENT_CHARS],
            "topic": item.get("topic", ""),
            "surface": item.get("surface", ""),
            "budgetPool": item.get("budgetPool", ""),
            "sourceName": item.get("sourceName", ""),
            "sourceOrigin": item.get("sourceOrigin", ""),
            "transport": item.get("transport", ""),
            "isOnion": bool(item.get("isOnion")),
            "prefetched": bool(item.get("prefetched")),
            "heuristicScore": item.get("heuristicScore", 0),
        })
    compact["candidates"] = candidates
    return json.dumps(compact, ensure_ascii=False, indent=2)[:max_chars]
