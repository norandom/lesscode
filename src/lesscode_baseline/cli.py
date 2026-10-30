"""Command-line startup for the local evaluator."""

import argparse

from . import __version__

def main(argv: list[str] | None = None) -> int:
    """Show CLI usage or the installed package version."""
    parser = argparse.ArgumentParser(
        prog="lesscode-baseline",
        description="Local evidence for supplied Python code simplifications.",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    parser.parse_args(argv)
    parser.print_help()
    return 0
