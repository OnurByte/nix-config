from __future__ import annotations

import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

from hermes_research_adhoc import ADHOC_SURFACES, ADHOC_WAVE_PAGES, research_plan


class HermesAdhocResearchTests(unittest.TestCase):
    def test_3000_page_plan_uses_every_surface(self) -> None:
        plan = research_plan("monero opsec", 3000)
        self.assertEqual(3000, plan["candidateTarget"])
        self.assertEqual(set(ADHOC_SURFACES), set(plan["surfaceBudget"]))
        self.assertEqual(3000, sum(plan["surfaceBudget"].values()))
        for surface in ADHOC_SURFACES:
            self.assertGreater(plan["surfaceBudget"][surface], 0)
            self.assertGreater(plan["surfaceDeepReadBudget"][surface], 0)
            self.assertGreater(len(plan["waves"][surface]), 0)
            self.assertEqual(
                plan["surfaceBudget"][surface],
                sum(item["candidateTarget"] for item in plan["waves"][surface]),
            )
            self.assertTrue(all(item["candidateTarget"] <= ADHOC_WAVE_PAGES for item in plan["waves"][surface]))

    def test_large_research_is_not_clamped_to_daily_1000(self) -> None:
        plan = research_plan("coding agents", 3000, deep_reads=180)
        self.assertEqual(3000, plan["candidateTarget"])
        self.assertEqual(180, plan["deepReadTarget"])
        self.assertEqual(180, sum(plan["surfaceDeepReadBudget"].values()))

    def test_opsec_seed_is_declared_in_home_configuration(self) -> None:
        text = (HERE.parent / "hermes.nix").read_text(errors="replace")
        self.assertIn("VESPER_REDDIT_SEEDS", text)
        self.assertIn("opsec", text.lower())
        self.assertIn("VESPER_REDDIT_COMMENT_SEEDS", text)

    def test_adhoc_reference_exists(self) -> None:
        ref = HERE.parent / "skills" / "hermes-research-radar" / "references" / "adhoc-research.md"
        self.assertTrue(ref.is_file())
        text = ref.read_text(errors="replace")
        self.assertIn("3000", text)
        self.assertIn("GitHub", text)
        self.assertIn("Reddit", text)
        self.assertIn("X/Twitter", text)
        self.assertIn("Tor onion", text)


if __name__ == "__main__":
    unittest.main()
