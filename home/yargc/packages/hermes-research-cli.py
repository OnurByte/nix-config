#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys

from hermes_research_adhoc import ADHOC_MAX_PAGES, run_adhoc_research, source_records


def _run_parser(prog: str) -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog=prog)
    p.add_argument("query")
    p.add_argument("--pages", type=int, default=600, help=f"candidate inspection target (50..{ADHOC_MAX_PAGES})")
    p.add_argument("--deep-reads", type=int, default=None, help="full-page/deep-read target; default scales with --pages")
    p.add_argument("--max-workers", type=int, default=None, help="parallel surface workers (1..4)")
    return p


def _print_sources(as_json: bool) -> int:
    records = source_records()
    if as_json:
        print(json.dumps({"schemaVersion": 1, "count": len(records), "sources": records}, ensure_ascii=False, indent=2))
        return 0
    print("kind\ttier\tscore\thits\tfail\tlabel\turl")
    for item in records:
        print(
            f"{item['kind']}\t{item['tier']}\t{item['score']:.2f}\t{item['hits']}\t{item['failures']}\t"
            f"{item['label']}\t{item['url']}"
        )
    return 0


def main() -> int:
    argv = sys.argv[1:]
    if not argv:
        print('usage: vesper-research [run] "query" [--pages N] | sources [--json]', file=sys.stderr)
        return 2

    if argv[0] == "sources":
        p = argparse.ArgumentParser(prog="vesper-research sources")
        p.add_argument("--json", action="store_true")
        args = p.parse_args(argv[1:])
        return _print_sources(args.json)

    if argv[0] == "run":
        argv = argv[1:]
    args = _run_parser("vesper-research").parse_args(argv)

    try:
        report = run_adhoc_research(args.query, pages=args.pages, deep_reads=args.deep_reads, max_workers=args.max_workers)
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        return 1
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
