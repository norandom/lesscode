"""Identify actual source bytes without Git checkout or writes.

Globs match the entire relative POSIX path, case sensitively on every OS.
``*``, ``?`` and shell character classes match within one segment; a standalone
``**`` matches zero or more segments. Dot names are ordinary names. Exclusions
are explicit ScopeRules, never inherited Git ignore or analyzer configuration.
The content identity is SHA-256 of UTF-8 compact JSON arrays containing sorted
relative path, byte length and lowercase content SHA-256. Modes and source root
do not enter this portable byte identity. Scope identity hashes canonical JSON
of the declared rules separately. Capture creates no disposable evaluation copy.
"""

from fnmatch import fnmatchcase
from functools import lru_cache
import hashlib
import json
from pathlib import Path
import stat

from .contracts import ContractError, Failure, FileRecord, ScopeRules, Snapshot

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
