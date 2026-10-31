"""Local plan/run entry points; startup requires no analyzers or LLM service."""

import argparse
import json
from pathlib import Path
import sys

from . import __version__

def _parser():
    parser = argparse.ArgumentParser(prog="lesscode-baseline",
        description="Local evidence for supplied Python code simplifications.", allow_abbrev=False)
    parser.add_argument("--version", action="version", version=f"%(prog)s {__version__}")
    commands = parser.add_subparsers(dest="command")
    plan = commands.add_parser("plan", help="freeze and inspect a comparison without running project checks", allow_abbrev=False)
    for name in ("profile", "baseline", "candidate", "out"):
        plan.add_argument("--" + name, required=True, type=Path)
    plan.add_argument("--tool-paths", type=Path, help="setup_toolchain entrypoints.json; paths relative to this file")
    run = commands.add_parser("run", help="execute a frozen plan into a new directory", allow_abbrev=False)
    for name in ("plan", "out"):
        run.add_argument("--" + name, required=True, type=Path)
    run.add_argument("--retain-workspaces", action="store_true")
    for side in ("baseline", "candidate"):
        run.add_argument("--" + side + "-evidence", action="append", type=Path, default=[],
                         help="producer-supplied optional evidence envelope; repeat for multiple imports")
    return parser


def decision_exit_code(decision):
    return {"accepted": 0, "rejected": 1, "invalid": 2,
            "incomplete": 3, "baseline_ineligible": 3}[decision]


def _expose(prepared):
    # JSON escapes control characters in supplied argv/env; each record is one line.
    print("Resolved project commands (cwd is relative to each fresh disposable source copy):")
    for side in ("baseline", "candidate"):
        snapshot = getattr(prepared.plan, side)
        print(json.dumps({"side": side, "source": snapshot.root}, ensure_ascii=True))
        for command in prepared.plan.resolved_commands:
            print(json.dumps({"side": side, **command.to_dict()}, ensure_ascii=True, sort_keys=True))
    for tool in prepared.tools:
        if tool.status == "unavailable":
            print(json.dumps({"unavailable_tool": tool.name, "reason": tool.reason}, ensure_ascii=True))
    sys.stdout.flush()


def _tool_paths(path):
    if path is None:
        return None
    document = json.loads(path.read_bytes())
    if not isinstance(document, dict) or any(not isinstance(k, str) or not isinstance(v, str)
                                             for k, v in document.items()):
        raise ValueError("tool paths must be a JSON object of path strings")
    return {key: str((path.resolve().parent / value).resolve()) for key, value in document.items()}


def main(argv: list[str] | None = None) -> int:
    parser = _parser()
    try:
        args = parser.parse_args(argv)
    except SystemExit as exc:
        return int(exc.code)
    if args.command is None:
        parser.print_help()
        return 0
    # Lazy imports keep --help/--version independent of runtime probes.
    from . import config, evaluation
    from .contracts import ContractError, Failure, PreparedPlan, dumps, loads
    from .report import PersistenceError
    try:
        if args.command == "plan":
            prepared = config.prepare(args.profile, args.baseline, args.candidate, args.out,
                                      tool_paths=_tool_paths(args.tool_paths))
            if isinstance(prepared, Failure):
                print("Input error: " + prepared.message, file=sys.stderr)
                return 2
            _expose(prepared)
            # Exclusive creation also closes the race after prepare's existence check.
            try:
                with args.out.open("x", encoding="utf-8", newline="\n") as handle:
                    handle.write(dumps(prepared) + "\n")
            except FileExistsError:
                raise ValueError("output must be new") from None
            except OSError as exc:
                raise PersistenceError(args.out.resolve(), str(exc)) from exc
            print("Plan: " + str(args.out.resolve()))
            return 0
        prepared = loads(PreparedPlan, args.plan.read_bytes())
        if args.out.exists():
            raise ValueError("output must be new")
        _expose(prepared)
        evidence = tuple(evaluation.OptionalEvidenceInput(side, path.resolve(), "producer_supplied")
                         for side in ("baseline", "candidate")
                         for path in getattr(args, side + "_evidence"))
        result = evaluation.evaluate(prepared, args.out, retain_workspaces=args.retain_workspaces,
                                     optional_evidence=evidence)
        print("Decision: " + result.decision)
        print("Summary: " + str(args.out.resolve() / "summary.md"))
        print("Report: " + str(args.out.resolve() / "report.json"))
        return decision_exit_code(result.decision)
    except PersistenceError as exc:
        print(f"Persistence failure: {exc}; retained diagnostics: {exc.directory}", file=sys.stderr)
        return 4
    except (ContractError, ValueError, FileNotFoundError, IsADirectoryError, FileExistsError) as exc:
        print("Input error: " + str(exc), file=sys.stderr)
        return 2
    except KeyboardInterrupt:
        print("Interrupted; retained diagnostics: " + str(args.out.resolve()), file=sys.stderr)
        return 3
    except Exception as exc:
        print(f"Internal/storage failure: {exc}; retained diagnostics: {args.out.resolve()}", file=sys.stderr)
        return 4
