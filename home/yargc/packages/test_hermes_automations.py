from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from hermes_automation_common import load_registry
from hermes_automation_contract import validate_registry
from hermes_automation_scheduler import WATCHDOG_TASKS, cron_create_argv, cron_edit_argv
from hermes_automation_tasks import TASKS
from hermes_research_intake import _canonical_reddit_url, _canonical_x_url
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
        self.assertEqual(
            [],
            validate_registry(self.registry, task_names=TASKS, watchdog_names=WATCHDOG_TASKS),
        )

    def test_expected_capabilities_are_declared(self) -> None:
        expected = {
            "unknown-frontier-github",
            "unknown-frontier-reddit",
            "unknown-frontier-x",
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
        self.assertEqual(("github", "reddit", "x"), FRONTIER_SOURCES)
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

    def test_research_skill_references_exist(self) -> None:
        skill = HERE.parent / "skills" / "hermes-research-radar"
        expected = {
            skill / "SKILL.md",
            skill / "references" / "research-pipeline.md",
            skill / "references" / "source-governance.md",
            skill / "references" / "reddit-rss.md",
            skill / "references" / "x-research.md",
        }
        self.assertTrue(all(path.is_file() for path in expected))
        skill_text = (skill / "SKILL.md").read_text(errors="replace")
        self.assertIn("200-1000", skill_text)
        self.assertIn("X / Twitter", (skill / "references" / "x-research.md").read_text(errors="replace"))
        self.assertIn("RSS", (skill / "references" / "reddit-rss.md").read_text(errors="replace"))

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
            "unknown-frontier-github",
            "unknown-frontier-reddit",
            "unknown-frontier-x",
            "free-ai-radar",
            "unknown-frontier-synthesis",
            "agenda",
            "morning-check",
        ]
        values = [minute_of_day(name) for name in order]
        self.assertEqual(values, sorted(values))
        self.assertGreaterEqual(
            minute_of_day("unknown-frontier-synthesis") - minute_of_day("unknown-frontier-x"),
            20,
        )

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
