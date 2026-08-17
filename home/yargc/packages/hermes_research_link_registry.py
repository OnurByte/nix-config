from __future__ import annotations

import json
import os
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any

from hermes_automation_common import STATE_ROOT, atomic_json, now
from hermes_research_intake import _save_source_registry
import hermes_research_web as research_web

WEB_GC_BAD_HOURS = max(24.0, float(os.environ.get("VESPER_WEB_GC_BAD_HOURS", "84")))
WEB_GC_MIN_OBSERVATIONS = max(2, int(os.environ.get("VESPER_WEB_GC_MIN_OBSERVATIONS", "3")))
WEB_GC_AUDIT_PATH = STATE_ROOT / "unknown-frontier-ai" / "web" / "link-gc.json"


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


def _load_audit() -> list[dict[str, Any]]:
    try:
        value = json.loads(WEB_GC_AUDIT_PATH.read_text(errors="replace")) if WEB_GC_AUDIT_PATH.is_file() else []
    except Exception:
        value = []
    return value if isinstance(value, list) else []


def _append_gc_audit(events: list[dict[str, Any]]) -> None:
    if not events:
        return
    current = _load_audit()
    current.extend(events)
    atomic_json(WEB_GC_AUDIT_PATH, current[-200:])


def _deleted_seed_urls() -> set[str]:
    urls: set[str] = set()
    for item in _load_audit():
        if not isinstance(item, dict) or not item.get("seed"):
            continue
        url = research_web.canonical_web_url(str(item.get("url") or ""))
        if url:
            urls.add(url)
    return urls


def _apply_seed_suppression() -> set[str]:
    """Prevent a deleted built-in seed from being re-created every process.

    A suppressed seed can still come back if the researcher rediscovers it as a
    learned source and it later produces useful evidence; only automatic
    central-config recreation is suppressed.
    """
    deleted = _deleted_seed_urls()
    if not deleted:
        return set()
    active = tuple(
        item for item in research_web.CENTRAL_WEB_ANCHORS
        if research_web.canonical_web_url(item["url"]) not in deleted
    )
    research_web.CENTRAL_WEB_ANCHORS = active
    return deleted


def _normalize_entry(key: str, entry: dict[str, Any]) -> bool:
    raw_url = str(entry.get("url") or entry.get("name") or "")
    url = research_web.canonical_web_url(raw_url)
    if not url:
        return False
    seed = str(entry.get("origin") or "") == "central-web-config"
    standard_internal = {
        "id": key,
        "kind": "web",
        "url": url,
        "name": url,  # compatibility with older web-registry code
        "label": str(entry.get("label") or url),
        "topic": str(entry.get("topic") or "discovered"),
        "seed": seed,
        "score": float(entry.get("score") or 0.0),
        "hits": max(0, int(entry.get("hits") or 0)),
        "observations": max(0, int(entry.get("observations") or 0)),
        "failures": max(0, int(entry.get("failures") or 0)),
        "firstSeen": str(entry.get("firstSeen") or ""),
        "lastSeen": str(entry.get("lastSeen") or ""),
        "lastUseful": str(entry.get("lastUseful") or ""),
        "origin": str(entry.get("origin") or "unknown"),
        "retiredReason": str(entry.get("retiredReason") or ""),
    }
    changed = False
    for field, value in standard_internal.items():
        if entry.get(field) != value:
            entry[field] = value
            changed = True
    return changed


def _poor_since(entry: dict[str, Any]) -> datetime | None:
    # A useful hit resets the quality clock. Until a source has ever produced a
    # useful result, its first observation/creation starts the clock.
    return _parse_time(entry.get("lastUseful")) or _parse_time(entry.get("firstSeen"))


def _quality_delete_reason(entry: dict[str, Any], stamp: datetime) -> str:
    if str(entry.get("retiredReason") or "") == "user-excluded":
        return ""
    observations = max(0, int(entry.get("observations") or 0))
    if observations < WEB_GC_MIN_OBSERVATIONS:
        return ""
    since = _poor_since(entry)
    if since is None or stamp - since < timedelta(hours=WEB_GC_BAD_HOURS):
        return ""
    failures = max(0, int(entry.get("failures") or 0))
    hits = max(0, int(entry.get("hits") or 0))
    if failures >= max(2, observations // 2):
        return "84h-low-quality-fetch"
    if hits == 0:
        return "84h-no-useful-output"
    return "84h-no-recent-useful-output"


def prune_web_links() -> dict[str, Any]:
    """Garbage-collect low-quality learned *and seed* web links.

    Nothing is immortal. A built-in seed is only a bootstrap hint. If it has at
    least a few observations and spends 84 hours without a useful hit, it can be
    removed just like a learned source. Explicit user exclusions remain as
    tombstones because their purpose is to prevent autonomous resurrection.
    """
    suppressed = _apply_seed_suppression()
    registry = research_web._ensure_web_registry()
    sources = registry.setdefault("sources", {})
    stamp = now()
    changed = False
    removed: list[dict[str, Any]] = []

    # If _ensure_web_registry recreated a previously deleted built-in seed before
    # suppression took effect in an older process/state, remove that automatic
    # recreation silently. A learned rediscovery uses a different origin and is
    # allowed to live again.
    for key, entry in list(sources.items()):
        if not isinstance(entry, dict) or entry.get("kind") != "web":
            continue
        changed = _normalize_entry(str(key), entry) or changed
        url = research_web.canonical_web_url(str(entry.get("url") or entry.get("name") or ""))
        if url in suppressed and str(entry.get("origin") or "") == "central-web-config" and int(entry.get("hits") or 0) == 0:
            del sources[key]
            changed = True

    for key, entry in list(sources.items()):
        if not isinstance(entry, dict) or entry.get("kind") != "web":
            continue
        changed = _normalize_entry(str(key), entry) or changed
        reason = _quality_delete_reason(entry, stamp)
        if not reason:
            continue
        removed.append({
            "id": str(key),
            "kind": "web",
            "url": str(entry.get("url") or entry.get("name") or ""),
            "label": str(entry.get("label") or ""),
            "topic": str(entry.get("topic") or ""),
            "seed": bool(entry.get("seed")),
            "tier": str(entry.get("tier") or ""),
            "score": float(entry.get("score") or 0.0),
            "hits": int(entry.get("hits") or 0),
            "observations": int(entry.get("observations") or 0),
            "failures": int(entry.get("failures") or 0),
            "origin": str(entry.get("origin") or ""),
            "reason": reason,
            "deletedAt": stamp.isoformat(timespec="seconds"),
        })
        del sources[key]
        changed = True

    if changed:
        _save_source_registry(registry)
    _append_gc_audit(removed)
    # Apply newly-created seed tombstones immediately so the rest of this process
    # (web_core_intake) does not fetch a link that was just deleted.
    _apply_seed_suppression()
    active = [
        entry for entry in sources.values()
        if isinstance(entry, dict) and entry.get("kind") == "web" and entry.get("tier") != "retired"
    ]
    return {
        "badWindowHours": WEB_GC_BAD_HOURS,
        "minObservations": WEB_GC_MIN_OBSERVATIONS,
        "removed": removed,
        "removedCount": len(removed),
        "activeCount": len(active),
    }


def web_link_records(*, include_retired: bool = False) -> list[dict[str, Any]]:
    # Listing reflects the same GC decision the next scout run will use.
    prune_web_links()
    registry = research_web._ensure_web_registry()
    sources = registry.setdefault("sources", {})
    changed = False
    records: list[dict[str, Any]] = []
    for key, entry in sources.items():
        if not isinstance(entry, dict) or entry.get("kind") != "web":
            continue
        changed = _normalize_entry(str(key), entry) or changed
        if not include_retired and entry.get("tier") == "retired":
            continue
        records.append({
            "id": str(entry.get("id") or key),
            "kind": "web",
            "url": str(entry.get("url") or ""),
            "label": str(entry.get("label") or ""),
            "topic": str(entry.get("topic") or ""),
            "seed": bool(entry.get("seed")),
            "tier": str(entry.get("tier") or ""),
            "score": float(entry.get("score") or 0.0),
            "hits": int(entry.get("hits") or 0),
            "observations": int(entry.get("observations") or 0),
            "failures": int(entry.get("failures") or 0),
            "origin": str(entry.get("origin") or ""),
            "firstSeen": str(entry.get("firstSeen") or ""),
            "lastSeen": str(entry.get("lastSeen") or ""),
            "lastUseful": str(entry.get("lastUseful") or ""),
        })
    if changed:
        _save_source_registry(registry)
    records.sort(key=lambda item: (not item["seed"], item["tier"] != "promoted", item["label"].lower(), item["url"]))
    return records
