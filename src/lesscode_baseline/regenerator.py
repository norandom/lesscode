"""Probabilistic single-turn/two-tier regenerator controller (ProbabilisticRegenerator).

Language-agnostic by contract: ``TurnRequest.language`` drives prompt labels
and code fences, and every Python-specific analysis step (syntax, surface,
measures) is delegated behind the extractor seams in ``surface`` and
``measures``; this module never imports ``ast``.
"""
from __future__ import annotations

import argparse
import sys

__all__ = [
    "EXIT_OK",
    "EXIT_USAGE",
    "EXIT_FAILED_TURN",
    "EXIT_COPY_MISMATCH",
    "EXIT_INPUT_TOO_LARGE",
    "EXIT_OUT_OF_SCOPE",
    "main",
]

# Preserved exit codes (frozen regen_turn.py contract; see design.md Error Handling).
EXIT_OK = 0
EXIT_USAGE = 2
EXIT_FAILED_TURN = 3
EXIT_COPY_MISMATCH = 4
EXIT_INPUT_TOO_LARGE = 5
EXIT_OUT_OF_SCOPE = 6


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="lesscode_baseline.regenerator",
        description="Single-turn/two-tier regenerator controller (Slice A; slice-B adoption gated by G1-R).",
    )
    sub = parser.add_subparsers(dest="command")
    for name in ("prompt", "run", "refine"):
        sub.add_parser(name, help=f"{name} subcommand (implemented by tasks 5.1-5.4)")
    args = parser.parse_args(argv)
    if args.command is None:
        parser.print_usage(sys.stderr)
        return EXIT_USAGE
    print(f"regenerator: {args.command} is implemented by tasks 5.1-5.4", file=sys.stderr)
    return EXIT_USAGE


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
