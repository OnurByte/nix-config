from __future__ import annotations

import json
import tempfile
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from hermes_automation_common import load_registry
from hermes_automation_contract import validate_registry
from hermes_automation_scheduler import WATCHDOG_TASKS, cron_create_argv, cron_edit_argv
from hermes_automation_tasks import TASKS
import hermes_research_intake as research_intake
import hermes_research_link_registry as link_registry
import hermes_research_web as research_web
from hermes_research_intake import (
    CENTRAL_REDDIT_ANCHORS,
    CENTRAL_X_ANCHORS,
    IGNORED_REDDIT_SOURCES,
    _canonical_reddit_url,
    _canonical_x_url,
    _pool_quotas,
    _select_candidate_pools,
)
from hermes_research_web import canonical_web_url, is_onion_url
from hermes_tasks_daily import (
    FRONTIER_CANDIDATE_BUDGET,
    FRONTIER_DEEP_READ_BUDGET,
    FRONTIER_SOURCES,
    FRONTIER_TOTAL_CANDIDATE_TARGET,
    FRONTIER_TOTAL_DEEP_READ_TARGET,
)


class HermesAutomationContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.registry = load_registry()

    def test_registry_contract(self) -> None:
        self.assertEqual([], validate_registry(self.registry, task_names=TASKS, watchdog_names=WATCHDOG_TASKS))

    def test_expected_capabilities_are_declared(self) -> None:
        expected = {
            "unknown-frontier-github",
            "unknown-frontier-reddit",
            "unknown-frontier-x",
            "unknown-frontier-web",
            "unknown-frontier-synthesis",
            "free-ai-radar",
            "agenda",
            "morning-check",
            "upstream-edge-radar",
            "vesper-health-watch",
            "cron-skill-integrity-watch",
            "second-brain-dream",
            "user-pain-miner",
            "project-archaeologist",
            "skill-evolution-review",
            "ai-usage-economist",
        }
        self.assertTrue(expected.issubset(self.registry))

    def test_frontier_sources_match_declarative_scouts(self) -> None:
        self.assertEqual(("github", "reddit", "x", "web"), FRONTIER_SOURCES)
        for source in FRONTIER_SOURCES:
            self.assertIn(f"unknown-frontier-{source}", TASKS)

    def test_frontier_coverage_contract(self) -> None:
        self.assertGreaterEqual(FRONTIER_TOTAL_CANDIDATE_TARGET, 200)
        self.assertLessEqual(FRONTIER_TOTAL_CANDIDATE_TARGET, 1000)
        self.assertEqual(FRONTIER_TOTAL_CANDIDATE_TARGET, sum(FRONTIER_CANDIDATE_BUDGET.values()))
        self.assertGreaterEqual(FRONTIER_TOTAL_DEEP_READ_TARGET, 24)
        self.assertLessEqual(FRONTIER_TOTAL_DEEP_READ_TARGET, 60)
        self.assertEqual(FRONTIER_TOTAL_DEEP_READ_TARGET, sum(FRONTIER_DEEP_READ_BUDGET.values()))
        for source in FRONTIER_SOURCES:
            self.assertGreater(FRONTIER_CANDIDATE_BUDGET[source], 0)
            self.assertGreater(FRONTIER_DEEP_READ_BUDGET[source], 0)

    def test_interest_profile_and_central_sources(self) -> None:
        for subreddit in ("MoneroMeansMoney", "Monero", "vibecoding", "ClaudeCode", "codex", "opencodeCLI", "cursor"):
            self.assertIn(subreddit, CENTRAL_REDDIT_ANCHORS)
        self.assertNotIn("LocalLLaMA", CENTRAL_REDDIT_ANCHORS)
        self.assertIn("LocalLLaMA", IGNORED_REDDIT_SOURCES)
        for account in ("Teknium", "thdxr", "XOpenSource", "eigenwallet", "SimpleXChat", "akaclandestine", "DailyDarkWeb"):
            self.assertIn(account, CENTRAL_X_ANCHORS)
        web_urls = {item["url"] for item in research_web.CENTRAL_WEB_ANCHORS}
        self.assertIn("https://monero.forum/", web_urls)
        self.assertIn(
            "https://opbible7nans45sg33cbyeiwqmlp5fu7lklu6jd6f3mivrjeqadco5yd.onion/opsec/",
            web_urls,
        )

    def test_source_registry_learns_only_from_useful_evidence(self) -> None:
        previous_path = research_intake.SOURCE_REGISTRY_PATH
        with tempfile.TemporaryDirectory() as tmp:
            research_intake.SOURCE_REGISTRY_PATH = Path(tmp) / "source-registry.json"
            try:
                initial = research_intake.load_source_registry()
                anchor = initial["sources"]["reddit:moneromeansmoney"]
                self.assertTrue(anchor["protected"])
                self.assertEqual("anchor", anchor["tier"])
                self.assertNotIn("reddit:localllama", initial["sources"])

                hint_only = {"candidates": [], "sources": [], "statePatch": {"candidateSources": ["@newbuilder"]}}
                research_intake.reinforce_source_registry("x", hint_only)
                hinted = research_intake.load_source_registry()["sources"]["x:newbuilder"]
                self.assertEqual("probation", hinted["tier"])
                self.assertEqual(0, hinted["hits"])

                useful = {
                    "candidates": [{"title": "useful candidate", "urls": ["https://x.com/newbuilder/status/123"]}],
                    "sources": [],
                    "statePatch": {},
                }
                research_intake.reinforce_source_registry("x", useful)
                research_intake.reinforce_source_registry("x", useful)
                learned = research_intake.load_source_registry()["sources"]["x:newbuilder"]
                self.assertEqual("trusted", learned["tier"])
                self.assertEqual(2, learned["hits"])
                research_intake.reinforce_source_registry("x", useful)
                research_intake.reinforce_source_registry("x", useful)
                promoted = research_intake.load_source_registry()["sources"]["x:newbuilder"]
                self.assertEqual("promoted", promoted["tier"])
                self.assertEqual(4, promoted["hits"])

                research_intake.discover_source("reddit", "LocalLLaMA", origin="test")
                after_ignore = research_intake.load_source_registry()
                self.assertNotIn("reddit:localllama", after_ignore["sources"])
            finally:
                research_intake.SOURCE_REGISTRY_PATH = previous_path

    def test_web_links_use_one_standard_record_shape(self) -> None:
        previous_path = research_intake.SOURCE_REGISTRY_PATH
        previous_audit = link_registry.WEB_GC_AUDIT_PATH
        previous_anchors = research_web.CENTRAL_WEB_ANCHORS
        with tempfile.TemporaryDirectory() as tmp:
            research_intake.SOURCE_REGISTRY_PATH = Path(tmp) / "source-registry.json"
            link_registry.WEB_GC_AUDIT_PATH = Path(tmp) / "link-gc.json"
            try:
                records = link_registry.web_link_records()
                self.assertEqual(2, len([item for item in records if item["seed"]]))
                required = {
                    "id", "kind", "url", "label", "topic", "seed", "tier",
                    "score", "hits", "observations", "failures", "origin",
                    "firstSeen", "lastSeen", "lastUseful",
                }
                for item in records:
                    self.assertEqual(required, set(item))
                    self.assertEqual("web", item["kind"])
                urls = {item["url"] for item in records}
                self.assertIn("https://monero.forum/", urls)
                self.assertIn(
                    "https://opbible7nans45sg33cbyeiwqmlp5fu7lklu6jd6f3mivrjeqadco5yd.onion/opsec/",
                    urls,
                )
            finally:
                research_intake.SOURCE_REGISTRY_PATH = previous_path
                link_registry.WEB_GC_AUDIT_PATH = previous_audit
                research_web.CENTRAL_WEB_ANCHORS = previous_anchors

    def test_web_link_gc_removes_bad_seed_after_84_hours(self) -> None:
        previous_path = research_intake.SOURCE_REGISTRY_PATH
        previous_audit = link_registry.WEB_GC_AUDIT_PATH
        previous_anchors = research_web.CENTRAL_WEB_ANCHORS
        with tempfile.TemporaryDirectory() as tmp:
            research_intake.SOURCE_REGISTRY_PATH = Path(tmp) / "source-registry.json"
            link_registry.WEB_GC_AUDIT_PATH = Path(tmp) / "link-gc.json"
            try:
                registry = research_web._ensure_web_registry()
                target_key = next(
                    key for key, item in registry["sources"].items()
                    if isinstance(item, dict) and "monero.forum" in str(item.get("name") or "")
                )
                entry = registry["sources"][target_key]
                entry["firstSeen"] = "2026-01-01T00:00:00+00:00"
                entry["lastSeen"] = "2026-01-04T12:00:00+00:00"
                entry["lastUseful"] = ""
                entry["hits"] = 0
                entry["observations"] = link_registry.WEB_GC_MIN_OBSERVATIONS
                entry["failures"] = 0
                research_intake._save_source_registry(registry)

                result = link_registry.prune_web_links()
                self.assertGreaterEqual(result["badWindowHours"], 84)
                self.assertEqual(1, result["removedCount"])
                self.assertTrue(result["removed"][0]["seed"])
                self.assertEqual("84h-no-useful-output", result["removed"][0]["reason"])
                after = research_intake.load_source_registry()["sources"]
                self.assertNotIn(target_key, after)
                self.assertTrue(link_registry.WEB_GC_AUDIT_PATH.is_file())
            finally:
                research_intake.SOURCE_REGISTRY_PATH = previous_path
                link_registry.WEB_GC_AUDIT_PATH = previous_audit
                research_web.CENTRAL_WEB_ANCHORS = previous_anchors

    def test_deleted_seed_can_return_only_as_learned_source(self) -> None:
        previous_path = research_intake.SOURCE_REGISTRY_PATH
        previous_audit = link_registry.WEB_GC_AUDIT_PATH
        previous_anchors = research_web.CENTRAL_WEB_ANCHORS
        with tempfile.TemporaryDirectory() as tmp:
            research_intake.SOURCE_REGISTRY_PATH = Path(tmp) / "source-registry.json"
            link_registry.WEB_GC_AUDIT_PATH = Path(tmp) / "link-gc.json"
            try:
                audit = [{
                    "id": "old",
                    "kind": "web",
                    "url": "https://monero.forum/",
                    "label": "Monero Forum",
                    "topic": "monero-privacy",
                    "seed": True,
                    "reason": "84h-no-useful-output",
                    "deletedAt": "2026-08-10T00:00:00+03:00",
                }]
                link_registry.WEB_GC_AUDIT_PATH.parent.mkdir(parents=True, exist_ok=True)
                link_registry.WEB_GC_AUDIT_PATH.write_text(json.dumps(audit))
                link_registry.prune_web_links()
                registry = research_web._ensure_web_registry()
                self.assertFalse(any("monero.forum" in str(item.get("name") or "") for item in registry["sources"].values() if isinstance(item, dict)))

                research_web._discover_web_source("https://monero.forum/", origin="web-scout")
                restored = research_intake.load_source_registry()["sources"]
                item = next(item for item in restored.values() if isinstance(item, dict) and "monero.forum" in str(item.get("name") or ""))
                self.assertEqual("probation", item["tier"])
                self.assertEqual("web-scout", item["origin"])
            finally:
                research_intake.SOURCE_REGISTRY_PATH = previous_path
                link_registry.WEB_GC_AUDIT_PATH = previous_audit
                research_web.CENTRAL_WEB_ANCHORS = previous_anchors

    def test_old_localllama_anchor_is_retired_on_registry_migration(self) -> None:
        previous_path = research_intake.SOURCE_REGISTRY_PATH
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "source-registry.json"
            path.write_text(json.dumps({
                "version": 1,
                "updatedAt": "",
                "sources": {
                    "reddit:localllama": {
                        "kind": "reddit", "name": "LocalLLaMA", "tier": "anchor", "protected": True,
                        "score": 10.0, "hits": 3, "failures": 0,
                        "firstSeen": "2026-01-01T00:00:00+00:00", "lastSeen": "2026-01-01T00:00:00+00:00",
                        "lastUseful": "2026-01-01T00:00:00+00:00", "origin": "central-config"
                    }
                }
            }))
            research_intake.SOURCE_REGISTRY_PATH = path
            try:
                migrated = research_intake.load_source_registry()["sources"]["reddit:localllama"]
                self.assertFalse(migrated["protected"])
                self.assertEqual("retired", migrated["tier"])
                self.assertEqual("user-excluded", migrated["retiredReason"])
            finally:
                research_intake.SOURCE_REGISTRY_PATH = previous_path

    def test_candidate_pool_budget_preserves_exploration(self) -> None:
        quotas = _pool_quotas(100)
        self.assertEqual(100, sum(quotas.values()))
        self.assertGreater(quotas["anchor"], quotas["explore"])
        self.assertGreater(quotas["dynamic"], 0)
        self.assertGreater(quotas["explore"], 0)

        def item(pool: str, source: str, index: int) -> dict[str, str]:
            return {"url": f"https://example.com/{pool}/{source}/{index}", "sourceName": source}

        pools = {
            "anchor": [item("anchor", "huge-anchor", i) for i in range(200)] + [item("anchor", "other-anchor", i) for i in range(10)],
            "dynamic": [item("dynamic", "learned", i) for i in range(80)],
            "explore": [item("explore", f"query-{i % 5}", i) for i in range(80)],
        }
        selected, budget = _select_candidate_pools(pools, 100, canonicalizer=lambda value: value, key_fields=("sourceName",))
        self.assertEqual(100, len(selected))
        self.assertGreater(budget["selected"]["dynamic"], 0)
        self.assertGreater(budget["selected"]["explore"], 0)

    def test_web_url_contract_and_tor_identity(self) -> None:
        onion = "https://opbible7nans45sg33cbyeiwqmlp5fu7lklu6jd6f3mivrjeqadco5yd.onion/opsec/?utm_source=test#x"
        canonical = canonical_web_url(onion)
        self.assertEqual("https://opbible7nans45sg33cbyeiwqmlp5fu7lklu6jd6f3mivrjeqadco5yd.onion/opsec/", canonical)
        self.assertTrue(is_onion_url(canonical))
        self.assertFalse(is_onion_url("https://monero.forum/"))
        self.assertEqual("", canonical_web_url("http://127.0.0.1/admin"))
        self.assertEqual("", canonical_web_url("http://localhost/admin"))
        self.assertEqual("", canonical_web_url("file:///etc/passwd"))

    def test_research_skill_references_and_evals_exist(self) -> None:
        skill = HERE.parent / "skills" / "hermes-research-radar"
        expected = {
            skill / "SKILL.md",
            skill / "references" / "research-pipeline.md",
            skill / "references" / "source-governance.md",
            skill / "references" / "central-sources.md",
            skill / "references" / "reddit-rss.md",
            skill / "references" / "x-research.md",
            skill / "references" / "web-tor.md",
            skill / "references" / "research-evolution.md",
            skill / "evals" / "evals.json",
        }
        self.assertTrue(all(path.is_file() for path in expected))
        skill_text = (skill / "SKILL.md").read_text(errors="replace")
        self.assertIn("200-1000", skill_text)
        self.assertIn("vibe coding", skill_text.lower())
        self.assertIn("unknown-frontier-web", skill_text)
        self.assertIn("84 hours", skill_text)
        central_text = (skill / "references" / "central-sources.md").read_text(errors="replace")
        self.assertIn("monero.forum", central_text)
        self.assertIn(".onion/opsec/", central_text)
        self.assertIn("seed", central_text.lower())
        web_tor = (skill / "references" / "web-tor.md").read_text(errors="replace")
        self.assertIn("SOCKS5 hostname", web_tor)
        self.assertIn("84 hours", web_tor)
        evals = json.loads((skill / "evals" / "evals.json").read_text())
        self.assertEqual("hermes-research-radar", evals["skill_name"])
        self.assertGreaterEqual(len(evals["evals"]), 8)

    def test_social_mirror_urls_are_canonicalized(self) -> None:
        expected = "https://x.com/example/status/123456"
        for url in (
            "https://x.com/example/status/123456",
            "https://twitter.com/example/status/123456?s=20",
            "https://xcancel.com/example/status/123456#m",
            "https://nitter.net/example/status/123456",
        ):
            self.assertEqual(expected, _canonical_x_url(url))
        self.assertEqual(
            "https://www.reddit.com/r/programming/comments/abc123/title",
            _canonical_reddit_url("https://old.reddit.com/r/programming/comments/abc123/title/?utm_source=rss"),
        )

    def test_daily_frontier_pipeline_is_staggered(self) -> None:
        def minute_of_day(name: str) -> int:
            minute, hour, *_ = self.registry[name]["schedule"].split()
            return int(hour) * 60 + int(minute)

        order = [
            "unknown-frontier-github", "unknown-frontier-reddit", "unknown-frontier-x",
            "unknown-frontier-web", "free-ai-radar", "unknown-frontier-synthesis", "agenda", "morning-check",
        ]
        values = [minute_of_day(name) for name in order]
        self.assertEqual(values, sorted(values))
        self.assertGreaterEqual(minute_of_day("unknown-frontier-synthesis") - minute_of_day("unknown-frontier-web"), 25)

    def test_all_cron_jobs_are_script_only_at_hermes_layer(self) -> None:
        script = Path("/home/test/.hermes/scripts/vesper-test.sh")
        create = cron_create_argv("hermes", "vesper:test", "0 9 * * *", "Run", "local", script)
        edit = cron_edit_argv("hermes", "abc", "vesper:test", "0 9 * * *", "Run", "local", script)
        self.assertIn("--no-agent", create)
        self.assertIn("--no-agent", edit)
        self.assertIn("--script", create)
        self.assertIn("--script", edit)

    def test_dispatch_jobs_do_not_claim_cron_delivery(self) -> None:
        for name, spec in self.registry.items():
            if spec.get("mode") == "dispatch":
                self.assertEqual("local", spec.get("deliver"), name)

    def test_watchdogs_are_zero_token_alert_jobs(self) -> None:
        for name in ("vesper-health-watch", "cron-skill-integrity-watch"):
            spec = self.registry[name]
            self.assertEqual("watchdog", spec["mode"])
            self.assertEqual("telegram", spec["deliver"])
            self.assertIn(spec["task"], WATCHDOG_TASKS)


if __name__ == "__main__":
    unittest.main()
