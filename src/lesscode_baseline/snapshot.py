"""Identify source bytes and create consistent disposable evaluation copies.

Globs match the entire relative POSIX path, case sensitively on every OS.
``*``, ``?`` and shell character classes match within one segment; a standalone
``**`` matches zero or more segments. Dot names are ordinary names. Exclusions
are explicit ScopeRules, never inherited Git ignore or analyzer configuration.
The content identity is SHA-256 of UTF-8 compact JSON arrays containing sorted
relative path, byte length and lowercase content SHA-256. Modes and source root
do not enter this portable byte identity. Scope identity hashes canonical JSON
of the declared rules separately. ``capture_scope`` only identifies source;
``capture`` and ``disposable_workspace`` create copies without editing originals.
"""

from fnmatch import fnmatchcase
from functools import lru_cache
from contextlib import contextmanager
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import shutil
import stat
from typing import Iterator

from .contracts import ContractError, Failure, FileRecord, ScopeRules, Snapshot

def capture(root: Path, scope: ScopeRules, destination: Path) -> Snapshot | Failure:
    """Copy selected bytes into a new directory, preserving manifest mode flags.

    The returned root is the copy; its byte identity equals the supplied source.
    Copying and re-reading source manifests detects concurrent changes, but is
    not an atomic filesystem snapshot or a hostile-code isolation boundary.
    """
    owned = None
    affected = None
    try:
        source, target = _copy_paths(root, destination)
        before = capture_scope(source, scope)
        if isinstance(before, Failure):
            return before
        root_identity = _object_identity(source.lstat())
        owned = _OwnedDirectory.create(target)
        for record in before.files:
            affected = record.path
            path = source / record.path
            _safe_entry(source, path)
            first = path.lstat()
            content = path.read_bytes()
            last = path.lstat()
            if (_file_identity(first) != _file_identity(last)
                    or _unsafe(last) or not stat.S_ISREG(last.st_mode)
                    or stat.S_IMODE(last.st_mode) != record.mode
                    or len(content) != record.size_bytes
                    or hashlib.sha256(content).hexdigest() != record.content_hash):
                raise OSError("source changed during copy")
            output = target / record.path
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_bytes(content)
            output.chmod(record.mode)
        after = capture_scope(source, scope)
        if isinstance(after, Failure):
            return after
        if _object_identity(source.lstat()) != root_identity or after != before:
            affected = next((a.path if a else b.path for a, b in file_changes(before, after)), None)
            raise OSError("source changed during copy")
        # Verify the actual copied bytes rather than assuming successful writes.
        for record in before.files:
            affected = record.path
            output = target / record.path
            if hashlib.sha256(output.read_bytes()).hexdigest() != record.content_hash:
                raise OSError("copied bytes do not match source fingerprint")
        result = Snapshot(str(target), before.content_hash, before.scope_hash,
                          before.revision, before.files, before.exclusions)
        owned = None  # Successful caller now owns the persistent copy.
        return result
    except (OSError, ContractError, ValueError) as exc:
        return Failure("snapshot_copy", str(exc), affected, None, None, False, None)
    finally:
        if owned is not None:
            owned.cleanup()


@dataclass(frozen=True)
class DisposableCopies:
    """In-process lifecycle handle; public snapshot JSON remains unchanged."""

    measurement: Snapshot
    checks: Path


@contextmanager
def disposable_workspace(root: Path, scope: ScopeRules, destination: Path, *,
                         retain: bool = False) -> Iterator[DisposableCopies | Failure]:
    """Create independent measurement/check trees and clean only the owned root.

    Measurement files lose write bits; check files gain owner write permission.
    This protects against ordinary relative test writes, not privileged or
    malicious commands. ``retain=True`` explicitly preserves diagnostic copies,
    including partial capture failures. Cleanup failures raise visibly.
    """
    owned = None
    try:
        _, target = _copy_paths(root, destination)
        owned = _OwnedDirectory.create(target)
        measurement = capture(root, scope, target / "measurement")
        if isinstance(measurement, Failure):
            result = measurement
        else:
            check = capture(Path(measurement.root), scope, target / "checks")
            if isinstance(check, Failure):
                result = check
            else:
                for record in measurement.files:
                    (target / "checks" / record.path).chmod(record.mode | stat.S_IWUSR)
                    (target / "measurement" / record.path).chmod(record.mode & ~0o222)
                result = DisposableCopies(measurement, target / "checks")
    except (OSError, ContractError, ValueError) as exc:
        result = Failure("snapshot_copy", str(exc), None, None, None, False, None)
    try:
        yield result
    finally:
        if owned is not None and not retain:
            owned.cleanup()


def _object_identity(info) -> tuple:
    return info.st_dev, info.st_ino


def _file_identity(info) -> tuple:
    return (*_object_identity(info), info.st_size, info.st_mode,
            info.st_mtime_ns, info.st_ctime_ns)


def _safe_entry(root: Path, path: Path) -> None:
    for entry in (path, *path.parents):
        info = entry.lstat()
        if _unsafe(info):
            raise OSError("symlink/reparse point cannot be copied")
        if entry == root:
            return
    raise OSError("copy path escapes source root")


def _copy_paths(root: Path, destination: Path) -> tuple[Path, Path]:
    root, destination = Path(root).absolute(), Path(destination).absolute()
    info = root.lstat()
    if _unsafe(info) or not stat.S_ISDIR(info.st_mode):
        raise OSError("source root must be a real directory")
    parent_info = destination.parent.lstat()
    if _unsafe(parent_info) or not stat.S_ISDIR(parent_info.st_mode):
        raise OSError("destination parent must be a real directory")
    try:
        destination.lstat()
    except FileNotFoundError:
        pass
    else:
        raise OSError("destination must be new and cannot be a link/reparse point")
    source = root.resolve(strict=True)
    target = destination.parent.resolve(strict=True) / destination.name
    if target.is_relative_to(source) or source.is_relative_to(target):
        raise OSError("source and destination cannot overlap")
    return source, target


@dataclass(frozen=True)
class _OwnedDirectory:
    path: Path
    identity: tuple

    @classmethod
    def create(cls, path: Path):
        path.mkdir()  # Exclusive creation; never adopt a pre-existing directory.
        return cls(path, _object_identity(path.lstat()))

    def cleanup(self) -> None:
        """Refuse replacement roots and links before removing an owned tree."""
        info = self.path.lstat()
        if (_unsafe(info) or self.path.resolve(strict=True) != self.path
                or _object_identity(info) != self.identity):
            raise OSError("refusing cleanup of replaced or redirected workspace")
        entries = []
        def inspect(directory):
            for entry in directory.iterdir():
                info = entry.lstat()
                if _unsafe(info):
                    raise OSError("refusing cleanup of workspace containing links/reparse points")
                entries.append((entry, info.st_mode))
                if stat.S_ISDIR(info.st_mode):
                    inspect(entry)
        inspect(self.path)
        for entry, mode in entries:
            entry.chmod(stat.S_IMODE(mode) | stat.S_IWUSR)
        shutil.rmtree(self.path)

def matches(path: str, pattern: str) -> bool:
    """Match portable whole-path globs, including zero-segment globstars."""
    parts, globs = path.split("/"), pattern.split("/")

    @lru_cache(maxsize=None)
    def match(i: int, j: int) -> bool:
        if j == len(globs):
            return i == len(parts)
        if globs[j] == "**":
            return match(i, j + 1) or (i < len(parts) and match(i + 1, j))
        return i < len(parts) and fnmatchcase(parts[i], globs[j]) and match(i + 1, j + 1)

    return match(0, 0)


def _hash(value: object) -> str:
    return hashlib.sha256(json.dumps(value, ensure_ascii=False, sort_keys=True,
                                    separators=(",", ":")).encode("utf-8")).hexdigest()


def _category(path: str, scope: ScopeRules) -> str:
    if not any(matches(path, p) for p in scope.include):
        return "excluded"
    for category in ("excluded", "generated", "vendor", "tests", "product"):
        patterns = scope.exclude if category == "excluded" else getattr(scope, category)
        if any(matches(path, p) for p in patterns):
            if category == "product" and not path.endswith(".py"):
                return "support"
            return category
    return "support"


def capture_scope(root: Path, scope: ScopeRules) -> Snapshot | Failure:
    """Read selected tracked/untracked files; a Failure never masquerades as a snapshot.

    ``revision`` is None: byte identity is authoritative and Git is not required.
    Excluded paths are listed but never read or hashed. Every traversed entry is
    checked for case collisions and links before use, including excluded entries.
    """
    root = Path(root).absolute()
    affected: str | None = None
    try:
        scope = ScopeRules.from_dict(scope.to_dict())
        root_stat = root.lstat()
        if _unsafe(root_stat) or not stat.S_ISDIR(root_stat.st_mode):
            raise OSError("source root must be a real directory, not a link/reparse point")
        root = root.resolve(strict=True)
        files: list[FileRecord] = []
        exclusions: list[str] = []
        seen: dict[str, str] = {}

        def visit(directory: Path) -> None:
            nonlocal affected
            for path in sorted(directory.iterdir(), key=lambda p: p.name):
                affected = path.relative_to(root).as_posix()
                info = path.lstat()
                folded = affected.casefold()
                if folded in seen:
                    raise OSError(f"case collision: {seen[folded]} and {affected}")
                seen[folded] = affected
                if _unsafe(info):
                    raise OSError("symlink/reparse point cannot be captured")
                if stat.S_ISDIR(info.st_mode):
                    visit(path)
                    continue
                if not stat.S_ISREG(info.st_mode):
                    raise OSError("input is not a regular file")
                category = _category(affected, scope)
                if category == "excluded":
                    exclusions.append(affected)
                    continue
                content = path.read_bytes()
                record = FileRecord(affected, category, len(content),
                                    hashlib.sha256(content).hexdigest(), stat.S_IMODE(info.st_mode))
                files.append(FileRecord.from_dict(record.to_dict()))

        visit(root)
        files.sort(key=lambda f: f.path)
        identity = _hash([[f.path, f.size_bytes, f.content_hash] for f in files])
        snapshot = Snapshot(str(root), identity, _hash(scope.to_dict()), None,
                            tuple(files), tuple(sorted(exclusions)))
        return Snapshot.from_dict(snapshot.to_dict())
    except (OSError, ContractError, UnicodeError, ValueError) as exc:
        # Invalid portable names cannot themselves enter affected_path's contract.
        try:
            failure = Failure("snapshot_input", str(exc), affected, None, None, False, None)
            return Failure.from_dict(failure.to_dict())
        except ContractError:
            return Failure("snapshot_input", f"{affected!r}: {exc}", None, None, None, False, None)


def _unsafe(info) -> bool:
    return stat.S_ISLNK(info.st_mode) or bool(
        getattr(info, "st_file_attributes", 0) & stat.FILE_ATTRIBUTE_REPARSE_POINT)


def file_changes(before: Snapshot, after: Snapshot) -> tuple[tuple[FileRecord | None, FileRecord | None], ...]:
    """Sorted old/new records for additions, removals, content/mode/class changes.

    Existing FileRecord categories retain separate test/product/support evidence.
    A missing side means addition/removal; no rename is inferred from identical bytes.
    """
    old, new = {f.path: f for f in before.files}, {f.path: f for f in after.files}
    return tuple((old.get(path), new.get(path)) for path in sorted(old.keys() | new.keys())
                 if old.get(path) != new.get(path))
