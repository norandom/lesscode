"""Product-only staging, deterministic size and pinned analyzer evidence.

Physical lines are ``len(original_bytes.splitlines())``: CRLF, LF and CR
separate lines, with a final unterminated line counted. Compression frames
sorted relative UTF-8 paths and original bytes, each preceded by an unsigned
64-bit big-endian length. No contents are decoded or newline-normalized.
"""

from contextlib import contextmanager
from dataclasses import replace
import gzip
import hashlib
from io import BytesIO
import json
import math
import os
from pathlib import Path
import platform
import stat
import statistics
import struct
import tempfile
from typing import Iterator
import zlib

from .contracts import (ClonePair, ComplexityPayload, DuplicationPayload, Failure,
                        FileComplexity, FunctionComplexity, Limits, MetricEvidence,
                        ResolvedCommand, Snapshot, SourceSpan)
from .process import execute

_LIZARD_VERSION = "1.17.31"
_JSCPD_VERSION = "4.0.5"
_DUPLICATION_DEFINITION = "jscpd-python-weak-minlines5-mintokens50-aggregate-pair-span-v1"

# Use the pinned tokenizer's maps, not Python lexical guesses. getTokensCount
# uses original token positions after filtering; it is not the filtered length.
_DUPLICATION_INVENTORY = r"""
const fs = require('fs'), path = require('path');
try {
    const entry = process.argv[1], inventory = process.argv[2];
    const pkg = path.dirname(path.dirname(entry));
    const manifest = require(path.join(pkg, 'package.json'));
    if (manifest.version !== '4.0.5') throw Error('wrong jscpd version: ' + manifest.version);
    const {Tokenizer, tokenize} = require(require.resolve('@jscpd/tokenizer', {paths:[pkg]}));
    const {weak} = require(require.resolve('@jscpd/core', {paths:[pkg]}));
    const files = JSON.parse(fs.readFileSync(inventory, 'utf8')).map(name => {
        const bytes = fs.readFileSync(name);
        // The actual CLI decodes UTF-8. Refuse replacement decoding of a
        // Python coding-cookie file rather than measuring changed characters.
        new TextDecoder('utf-8', {fatal:true}).decode(bytes);
        const data = bytes.toString();
        if (/jscpd:ignore-(?:start|end)/.test(data)) throw Error('source suppression: ' + name);
        const maps = new Tokenizer().generateMaps(name, data, 'python', {mode:weak, minTokens:50});
        const meaningful = tokenize(data, 'python').filter(t=>weak(t)).some(t=>t.value.length > 0);
        return {path:name, maps:meaningful ? maps.length : 0,
                tokens:maps.reduce((n,m)=>n+m.getTokensCount(),0)};
    });
    console.log(JSON.stringify({version:manifest.version, files}));
} catch (error) {
    console.error(String(error)); process.exit(1);
}
"""


def measure_duplication(snapshot: Snapshot, interpreter: Path, node: Path, entry: Path,
                        limits: Limits, logs: Path) -> MetricEvidence:
    """Run pinned jscpd against an isolated, syntax-validated product manifest.

    The fixed min-lines setting can omit short nonempty files. Such omissions
    remain unavailable; only an independent inventory of no meaningful tokens can
    explain an absent source. Pair-span ratio can exceed unique-source coverage.
    A zero-only scope is unavailable when the vendor emits no fresh JSON report.
    """
    snapshot = Snapshot.from_dict(snapshot.to_dict())
    limits = Limits.from_dict(limits.to_dict())
    reason, artifact, payload = None, None, None
    with stage_product(snapshot) as staged:
        if isinstance(staged, Failure):
            reason = staged.message
        else:
            root = Path(staged.root)
            inventory = root.parent / "inventory.json"
            paths = [f.path for f in staged.files]
            inventory.write_text(json.dumps(paths), encoding="utf-8")
            syntax = ResolvedCommand("duplication-syntax", (str(interpreter), "-I", "-S", "-c",
                _SYNTAX_WORKER, str(inventory)), ".", {}, limits, "selected Python")
            try:
                syntax_report, artifact = _duplication_worker(syntax, root, logs)
                if (not isinstance(syntax_report, dict) or set(syntax_report) != {"python", "validated"}
                        or syntax_report["validated"] != paths or not isinstance(syntax_report["python"], str)
                        or not syntax_report["python"]):
                    raise ValueError("syntax inventory/runtime mismatch")
                if not entry.is_absolute() or not entry.is_file():
                    raise ValueError("missing absolute jscpd entry path")
                # Windows OpenSSL needs the OS directory to initialize its
                # random provider. Do not inherit NODE_OPTIONS or project env.
                environment = {"SystemRoot": os.environ["SystemRoot"]} if os.name == "nt" else {}
                worker = ResolvedCommand("duplication-inventory", (node.as_posix(), "-e",
                    _DUPLICATION_INVENTORY, entry.as_posix(), inventory.as_posix()),
                    ".", environment, limits, "jscpd " + _JSCPD_VERSION)
                token_inventory, artifact = _duplication_worker(worker, root, logs)
                bounds = _duplication_bounds(staged)
                config = root.parent / "jscpd.json"
                # Explicit external config and output prevent project discovery.
                config.write_text(json.dumps({"format":["python"], "mode":"weak",
                    "minLines":5, "minTokens":50, "maxLines":bounds[0], "maxSize":bounds[1],
                    "ignore":[], "gitignore":False, "noSymlinks":True}), encoding="utf-8")
                Path(logs).mkdir(parents=True, exist_ok=True)
                output = Path(tempfile.mkdtemp(prefix="duplication-", dir=Path(logs).resolve()))
                report_path = output / "jscpd-report.json"
                cli = ResolvedCommand("metrics-duplication", (node.as_posix(), entry.as_posix(),
                    "--config", config.as_posix(), "--format", "python", "--mode", "weak",
                    "--min-lines", "5", "--min-tokens", "50", "--max-lines", str(bounds[0]),
                    "--max-size", bounds[1], "--reporters", "json", "--output", output.as_posix(),
                    "--silent", root.as_posix()), ".", environment, limits, "jscpd " + _JSCPD_VERSION)
                result = execute(cli, root, logs)
                artifact = result.stderr_artifact
                stderr = (Path(logs) / artifact).read_text(encoding="utf-8", errors="replace")
                if result.status != "completed" or result.exit_code != 0 or stderr:
                    raise ValueError(f"execution {result.status}; exit {result.exit_code}: {stderr[:2000]}")
                if not report_path.is_file():
                    raise ValueError("missing fresh jscpd report")
                artifact = report_path.relative_to(Path(logs).resolve()).as_posix()
                report = json.loads(report_path.read_text(encoding="utf-8"))
                payload = _duplication_payload(report, token_inventory, staged)
            except (ValueError, TypeError, KeyError, UnicodeError) as exc:
                artifact = getattr(exc, "artifact", artifact)
                reason = f"duplication {exc}"
    return MetricEvidence("jscpd_pair_span_ratio", "unavailable" if reason else "available", payload,
        "ratio", _DUPLICATION_DEFINITION, "jscpd", _JSCPD_VERSION,
        snapshot.scope_hash, reason, artifact)


class _DuplicationFailure(ValueError):
    def __init__(self, message: str, artifact: str):
        super().__init__(message)
        self.artifact = artifact


def _duplication_worker(command: ResolvedCommand, root: Path, logs: Path) -> tuple[object, str]:
    result = execute(command, root, logs)
    stderr = (Path(logs) / result.stderr_artifact).read_text(encoding="utf-8", errors="replace")
    if result.status != "completed" or result.exit_code != 0 or stderr:
        raise _DuplicationFailure(f"{command.command_id} execution {result.status}; exit {result.exit_code}: {stderr[:2000]}", result.stderr_artifact)
    try:
        report = json.loads((Path(logs) / result.stdout_artifact).read_text(encoding="utf-8"))
    except (ValueError, UnicodeError) as exc:
        raise _DuplicationFailure("malformed worker report: " + str(exc), result.stdout_artifact) from exc
    return report, result.stdout_artifact


def _duplication_bounds(snapshot: Snapshot) -> tuple[int, str]:
    data = [_content(Path(snapshot.root), record) for record in snapshot.files]
    lines = max((content.count(b"\n") + 1 for content in data), default=0) + 1
    size = max(map(len, data), default=0) + 1
    # Both bounds flow through JavaScript numbers and bytes.parse. Refuse an
    # unrepresentable value rather than leave the vendor's silent skip enabled.
    if lines > 2**53 - 1 or size > 2**53 - 1:
        raise ValueError("unsupported duplication bounds")
    return lines, str(size) + "b"


def _duplication_payload(report: object, inventory: object, snapshot: Snapshot) -> DuplicationPayload:
    def integer(value):
        if type(value) is not int or value < 0:
            raise ValueError("invalid nonnegative integer statistic")
        return value

    paths = {record.path: record for record in snapshot.files}
    if (not isinstance(inventory, dict) or set(inventory) != {"version", "files"}
            or inventory["version"] != _JSCPD_VERSION or not isinstance(inventory["files"], list)):
        raise ValueError("invalid pinned tokenizer inventory/version")
    tokens, zero = {}, []
    for file in inventory["files"]:
        if not isinstance(file, dict) or set(file) != {"path", "maps", "tokens"}:
            raise ValueError("invalid tokenizer file inventory")
        name = file["path"]
        if name not in paths or name in tokens:
            raise ValueError("tokenizer inventory differs from staged manifest")
        tokens[name] = integer(file["tokens"])
        maps = integer(file["maps"])
        if maps == 0:
            if tokens[name] != 0:
                raise ValueError("zero-map inventory has nonzero tokens")
            zero.append(name)
        elif maps != 1:
            raise ValueError("unexpected Python token map population")
    if set(tokens) != set(paths):
        raise ValueError("tokenizer inventory differs from staged manifest")

    def normalize(name):
        if not isinstance(name, str):
            raise ValueError("invalid analyzer path")
        name = name.replace("\\", "/")
        root = Path(snapshot.root).as_posix() + "/"
        if name.startswith(root):
            name = name[len(root):]
        elif name.startswith("./"):
            name = name[2:]
        if name not in paths:
            raise ValueError("analyzer path absent from staged manifest")
        return name

    if not isinstance(report, dict) or not isinstance(report.get("duplicates"), list):
        raise ValueError("invalid jscpd report schema")
    statistics_report = report["statistics"]
    total = statistics_report["total"]
    formats = statistics_report["formats"]
    if not isinstance(formats, dict) or set(formats) - {"python"}:
        raise ValueError("unexpected analyzer formats")
    scanned = {}
    python_format = formats.get("python", {})
    if not isinstance(python_format, dict) or not isinstance(python_format.get("sources", {}), dict):
        raise ValueError("invalid scanned source schema")
    for name, value in python_format.get("sources", {}).items():
        name = normalize(name)
        if name in scanned:
            raise ValueError("duplicate scanned path")
        scanned[name] = integer(value["tokens"])
    if (not set(paths) - set(zero) <= set(scanned)
            or any(scanned[n] != tokens[n] for n in scanned)):
        raise ValueError("scanned source/token inventory differs from staged manifest")
    denominator = integer(total["tokens"])
    numerator = integer(total["duplicatedTokens"])
    if denominator != sum(tokens.values()) or integer(total["sources"]) != len(scanned):
        raise ValueError("aggregate denominator/source inventory mismatch")
    clones = []
    pair_tokens = 0
    for clone in report["duplicates"]:
        if clone["format"] != "python":
            raise ValueError("unexpected clone format")
        spans = []
        for key in ("firstFile", "secondFile"):
            file = clone[key]
            name = normalize(file["name"])
            span = SourceSpan.from_dict({"path":name, "start_line":file["start"], "end_line":file["end"]})
            if name not in scanned or span.end_line > len(_content(Path(snapshot.root), paths[name]).splitlines()):
                raise ValueError("clone span outside scanned source")
            if file["startLoc"]["line"] != span.start_line or file["endLoc"]["line"] != span.end_line:
                raise ValueError("clone location mismatch")
            start, end = integer(file["startLoc"]["position"]), integer(file["endLoc"]["position"])
            if end < start:
                raise ValueError("invalid clone token span")
            if key == "firstFile":
                pair_tokens += end - start
            spans.append(span)
        spans.sort(key=lambda s: (s.path, s.start_line, s.end_line))
        clones.append(ClonePair(*spans))
    if numerator != pair_tokens or integer(total["clones"]) != len(clones):
        raise ValueError("aggregate pair-span statistics mismatch")
    clones.sort(key=lambda c: (c.first.path, c.first.start_line, c.first.end_line,
                               c.second.path, c.second.start_line, c.second.end_line))
    payload = DuplicationPayload("duplication", numerator, denominator,
        numerator / denominator if denominator else None,
        None if denominator else "zero token denominator", "weak", total["percentageTokens"],
        tuple(clones), tuple(sorted(scanned)), tuple(sorted(zero)))
    return DuplicationPayload.from_dict(payload.to_dict())
_COMPLEXITY_DEFINITION = "lizard-python-ccn-files-functions-median-nearest-rank-p95-v1"

# -I retains the selected tool environment's installed packages while excluding
# project imports, PYTHONPATH and user site configuration. Compile/AST inventory
# is independent of Lizard, which accepts invalid syntax and can omit functions.
_COMPLEXITY_WORKER = """
import ast, importlib.metadata, json, pathlib, sys, tokenize

def main():
    paths = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'))
    expected, lines, sources = {}, {}, {}
    for name in paths:
        data = pathlib.Path(name).read_bytes()
        try:
            compile(data, name, 'exec', dont_inherit=True)
            tree = ast.parse(data, filename=name)
        except (SyntaxError, ValueError) as exc:
            raise ValueError('syntax validation: ' + name + ': ' + str(exc)) from exc
        expected[name] = sorted([[node.name, node.lineno] for node in ast.walk(tree)
                                 if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))])
        lines[name] = len(data.splitlines())
        with tokenize.open(name) as source:
            sources[name] = source.read()
    import lizard
    version = importlib.metadata.version('lizard')
    if version != '1.17.31':
        raise ValueError('wrong Lizard version: ' + version)
    files, functions = [], []
    for name in paths:
        # Use the pinned reader's exact comment normalization, including spaces,
        # repeated #, forgiveness suffixes and backslash-continued comments.
        reader = lizard.get_reader_for(name)(lizard.FileInfoBuilder(name))
        source = lizard.auto_read(name)
        if source.replace('\\r\\n', '\\n').replace('\\r', '\\n') != sources[name]:
            raise ValueError('unsupported analyzer source decoding: ' + name)
        for token in reader.generate_tokens(source):
            comment = reader.get_comment_from_token(token)
            if comment is not None and (comment.strip().startswith('#lizard forgive')
                                        or 'GENERATED CODE' in comment):
                raise ValueError('source suppression: ' + name)
        info = lizard.analyze_file(name)
        if info.filename != name:
            raise ValueError('analyzer file identity mismatch: ' + name)
        files.append({'path': name, 'nloc': info.nloc})
        for function in info.function_list:
            functions.append({'path': name, 'name': function.name, 'long_name': function.long_name,
                              'start_line': function.start_line, 'end_line': function.end_line,
                              'ccn': function.cyclomatic_complexity, 'nloc': function.nloc,
                              'token_count': function.token_count})
    print(json.dumps({'version': version, 'files': files, 'functions': functions,
                      'expected_functions': expected, 'physical_lines': lines}))

try:
    main()
except Exception as exc:
    print(str(exc), file=sys.stderr)
    sys.exit(1)
"""


def measure_complexity(snapshot: Snapshot, interpreter: Path, limits: Limits,
                       logs: Path) -> MetricEvidence:
    """Measure all product files using the explicitly installed pinned Lizard.

    Function records sort by descending CCN, then source identity, so the full
    inventory also identifies hotspots without a separate arbitrary top-N list.
    A file or function omission, stderr warning or malformed report invalidates
    the entire scope; unavailable evidence always retains a diagnostic reason.
    """
    snapshot = Snapshot.from_dict(snapshot.to_dict())
    limits = Limits.from_dict(limits.to_dict())
    reason: str | None = None
    artifact: str | None = None
    payload: ComplexityPayload | None = None
    with stage_product(snapshot) as staged:
        if isinstance(staged, Failure):
            reason = staged.message
        else:
            paths = [record.path for record in staged.files]
            inventory = Path(staged.root).parent / "inventory.json"
            inventory.write_text(json.dumps(paths), encoding="utf-8")
            command = ResolvedCommand("metrics-complexity", (str(interpreter), "-I", "-X", "utf8", "-c",
                _COMPLEXITY_WORKER, str(inventory)), ".", {}, limits, "Lizard " + _LIZARD_VERSION)
            result = execute(command, Path(staged.root), logs)
            artifact = result.stdout_artifact
            stderr = (Path(logs) / result.stderr_artifact).read_text(encoding="utf-8", errors="replace")
            if result.status != "completed" or result.exit_code != 0 or stderr:
                artifact = result.stderr_artifact
                reason = (f"complexity execution {result.status}; exit {result.exit_code}: "
                          + stderr[:2000])
            else:
                try:
                    report = json.loads((Path(logs) / result.stdout_artifact).read_text(encoding="utf-8"))
                    payload = _complexity_payload(report, staged)
                except (ValueError, TypeError, KeyError, UnicodeError) as exc:
                    reason = f"complexity report: {exc}"
    return MetricEvidence("complexity", "unavailable" if reason else "available", payload,
        "CCN", _COMPLEXITY_DEFINITION, "lizard", _LIZARD_VERSION,
        snapshot.scope_hash, reason, artifact)


def _complexity_payload(report: object, snapshot: Snapshot) -> ComplexityPayload:
    if (not isinstance(report, dict) or set(report) != {
            "version", "files", "functions", "expected_functions", "physical_lines"}
            or report["version"] != _LIZARD_VERSION):
        raise ValueError("invalid worker schema or pinned Lizard version")
    if not isinstance(report["files"], list) or not isinstance(report["functions"], list):
        raise ValueError("file/function inventories must be lists")
    files = tuple(sorted((FileComplexity.from_dict(f) for f in report["files"]), key=lambda f: f.path))
    functions = tuple(sorted((FunctionComplexity.from_dict(f) for f in report["functions"]),
                             key=lambda f: (-f.ccn, f.path, f.start_line, f.end_line, f.long_name)))
    paths = [record.path for record in snapshot.files]
    if [f.path for f in files] != paths:
        raise ValueError("analyzer file inventory differs from staged manifest")
    expected, lines = report["expected_functions"], report["physical_lines"]
    if (not isinstance(expected, dict) or not isinstance(lines, dict)
            or set(expected) != set(paths) or set(lines) != set(paths)):
        raise ValueError("independent syntax inventory differs from staged manifest")
    records = {record.path: record for record in snapshot.files}
    by_file: dict[str, list[FunctionComplexity]] = {path: [] for path in paths}
    for function in functions:
        if function.path not in by_file:
            raise ValueError("function path absent from staged manifest")
        by_file[function.path].append(function)
    for file in files:
        # Recompute line bounds from the verified staged bytes instead of trusting
        # analyzer locations or an internally consistent but truncated report.
        bound = len(_content(Path(snapshot.root), records[file.path]).splitlines())
        if type(lines[file.path]) is not int or lines[file.path] != bound or file.nloc > bound:
            raise ValueError("invalid file line count: " + file.path)
        actual = by_file[file.path]
        # Python function identifiers cannot contain dots. Lizard qualifies
        # nested function names with enclosing function names; AST names do not.
        if sorted([[f.name.rsplit(".", 1)[-1], f.start_line] for f in actual]) != expected[file.path]:
            raise ValueError("analyzer function inventory differs from syntax inventory: " + file.path)
        if any(f.end_line > bound or f.nloc > f.end_line - f.start_line + 1 for f in actual):
            raise ValueError("invalid function line count: " + file.path)
    values = sorted(f.ccn for f in functions)
    distribution = ((min(values), max(values), statistics.median(values), values[math.ceil(.95 * len(values)) - 1])
                    if values else (None, None, None, None))
    payload = ComplexityPayload("complexity", files, functions, len(functions), *distribution)
    return ComplexityPayload.from_dict(payload.to_dict())

_DEFINITIONS = (
    ("source_bytes", "bytes", "product-original-bytes-v1"),
    ("physical_lines", "lines", "product-bytes-splitlines-final-unterminated-v1"),
    ("compressed_source_bytes", "bytes",
     "product-sorted-utf8-path-bytes-u64be-lengths-gzip-level9-mtime0-v1"),
)

# -I -S prevents Python startup/project configuration from being inherited.
# compile validates bytes with the chosen runtime but never executes source.
_SYNTAX_WORKER = """
import json, pathlib, platform, sys
paths = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'))
for name in paths:
    compile(pathlib.Path(name).read_bytes(), name, 'exec', dont_inherit=True)
print(json.dumps({'python': platform.python_version(), 'validated': paths}))
"""


def _real_path(root: Path, path: Path) -> None:
    for entry in (path, *path.parents):
        info = entry.lstat()
        if (stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0)
                & stat.FILE_ATTRIBUTE_REPARSE_POINT):
            raise OSError("input contains symlink/reparse point")
        if entry == root:
            if not stat.S_ISDIR(info.st_mode):
                raise OSError("input root must be a directory")
            return
    raise OSError("input path escapes root")


def _content(root: Path, record) -> bytes:
    path = root / record.path
    _real_path(root, path)
    if not stat.S_ISREG(path.lstat().st_mode):
        raise OSError("input must be a regular file")
    data = path.read_bytes()
    if len(data) != record.size_bytes or hashlib.sha256(data).hexdigest() != record.content_hash:
        raise OSError("input bytes differ from snapshot")
    return data


@contextmanager
def stage_product(snapshot: Snapshot) -> Iterator[Snapshot | Failure]:
    """Yield an owned, isolated tree containing only verified product inputs.

    Callers must pass explicit analyzer options disabling implicit ignores and
    auto-configuration. No repository config or ignore file is copied. The
    source scope identity is retained so all metrics bind to the fixed scope.
    Temporary state is removed on success, input failure and caller exceptions.
    """
    snapshot = Snapshot.from_dict(snapshot.to_dict())
    with tempfile.TemporaryDirectory(prefix="lesscode-metrics-") as temporary:
        target = Path(temporary) / "product"
        target.mkdir()
        affected = None
        try:
            source = Path(snapshot.root)
            _real_path(source, source)
            files = tuple(sorted((r for r in snapshot.files if r.category == "product"),
                                 key=lambda r: r.path))
            for record in files:
                affected = record.path
                if Path(record.path).suffix != ".py":
                    raise OSError("unsupported product syntax: Python .py files required")
                data = _content(source, record)
                output = target / record.path
                output.parent.mkdir(parents=True, exist_ok=True)
                output.write_bytes(data)
                _content(target, record)
            staged = replace(snapshot, root=str(target), files=files, exclusions=())
        except OSError as exc:
            staged = Failure("metrics_input", f"input {affected or snapshot.root}: {exc}",
                             affected, None, None, False, None)
        yield staged


def measure_sizes(snapshot: Snapshot, interpreter: Path, limits: Limits,
                  logs: Path) -> tuple[MetricEvidence, ...]:
    """Validate Python through bounded execution, then measure original bytes.

    ``interpreter`` is the caller-selected absolute Python executable; execution
    uses no inherited environment. Logs are retained at the supplied artifact
    root. Their paths in evidence are relative to that root. Persistence errors
    propagate instead of promising evidence that was never stored.
    """
    snapshot = Snapshot.from_dict(snapshot.to_dict())
    limits = Limits.from_dict(limits.to_dict())
    runtime = (f"{platform.python_implementation()} {platform.python_version()}; "
               f"zlib {zlib.ZLIB_VERSION}/{zlib.ZLIB_RUNTIME_VERSION}")
    reason, artifact, values = None, None, (None, None, None)
    with stage_product(snapshot) as staged:
        if isinstance(staged, Failure):
            reason = staged.message
        else:
            # Inventory sits outside the product tree; it cannot be measured by
            # analyzers and avoids platform argv limits for large manifests.
            inventory = Path(staged.root).parent / "inventory.json"
            paths = [record.path for record in staged.files]
            inventory.write_text(json.dumps(paths), encoding="utf-8")
            command = ResolvedCommand("metrics-syntax", (str(interpreter), "-I", "-S", "-c",
                _SYNTAX_WORKER, str(inventory)), ".", {}, limits, "selected Python")
            execution = execute(command, Path(staged.root), logs)
            artifact = execution.stderr_artifact
            if execution.status != "completed" or execution.exit_code != 0:
                reason = f"Python syntax validation {execution.status}; exit {execution.exit_code}"
            else:
                try:
                    report = json.loads((Path(logs) / execution.stdout_artifact).read_text(encoding="utf-8"))
                    if (not isinstance(report, dict) or set(report) != {"python", "validated"}
                            or report["validated"] != paths or not isinstance(report["python"], str)
                            or not report["python"]):
                        raise ValueError("invalid syntax worker inventory/runtime")
                    runtime += "; syntax Python " + report["python"]
                except (ValueError, UnicodeError) as exc:
                    reason = f"Python syntax validation report: {exc}"
                    artifact = execution.stdout_artifact
            if reason is None:
                try:
                    values = _size_values(staged)
                except OSError as exc:
                    reason = f"measurement input: {exc}"
    return tuple(MetricEvidence(key, "unavailable" if reason else "available",
        None if reason else value, unit, definition, "lesscode-size", runtime,
        snapshot.scope_hash, reason, artifact)
        for (key, unit, definition), value in zip(_DEFINITIONS, values))


def _size_values(snapshot: Snapshot) -> tuple[int, int, int]:
    raw_bytes, lines = 0, 0
    compressed = BytesIO()
    with gzip.GzipFile(fileobj=compressed, mode="wb", filename="", compresslevel=9, mtime=0) as output:
        for record in snapshot.files:
            data = _content(Path(snapshot.root), record)
            raw_bytes += len(data)
            lines += len(data.splitlines())
            path = record.path.encode("utf-8")
            output.write(struct.pack(">Q", len(path)))
            output.write(path)
            output.write(struct.pack(">Q", len(data)))
            output.write(data)
    return raw_bytes, lines, len(compressed.getvalue())
