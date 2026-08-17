from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import hermes_automation_common
from hermes_automation_common import load_registry
from hermes_automation_contract import validate_registry
from hermes_automation_scheduler import WATCHDOG_TASKS, cron_create_argv, cron_edit_argv
from hermes_automation_tasks import TASKS
from hermes_tasks_daily import FRONTIER_SOURCES


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
            "cron-retention",
            "second-brain-dream",
            "user-pain-miner",
            "project-archaeologist",
            "skill-evolution-review",
            "ai-usage-economist",
            "weekly-intelligence-review",
        }
        self.assertTrue(expected.issubset(self.registry))

    def test_frontier_sources_match_declarative_scouts(self) -> None:
        self.assertEqual(("github", "reddit", "x"), FRONTIER_SOURCES)
        for source in FRONTIER_SOURCES:
            self.assertIn(f"unknown-frontier-{source}", TASKS)

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

    def test_oneshot_preloads_requested_skills_and_toolsets(self) -> None:
        completed = subprocess.CompletedProcess(["hermes"], 0, stdout="ok", stderr="")
        with patch.object(hermes_automation_common, "hermes_bin", return_value="hermes"), patch.object(
            hermes_automation_common.subprocess,
            "run",
            return_value=completed,
        ) as run:
            hermes_automation_common._invoke(
                "research",
                toolsets=["web", "x_search"],
                skills=["hermes-research-radar"],
                timeout=60,
            )
        command = run.call_args.args[0]
        self.assertIn("--toolsets", command)
        self.assertEqual("web,x_search", command[command.index("--toolsets") + 1])
        self.assertIn("--skills", command)
        self.assertEqual("hermes-research-radar", command[command.index("--skills") + 1])


if __name__ == "__main__":
    unittest.main()
