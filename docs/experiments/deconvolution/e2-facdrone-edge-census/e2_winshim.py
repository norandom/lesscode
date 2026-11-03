"""pytest plugin `-p e2_winshim`: make Facdrone's embedded PostgreSQL fixture work on Windows.

Two Windows gaps in the specimen, both worked around here without touching Facdrone (plan.md
amendments 2 and 5):

1. `facdrone_testing.fixtures.pytest_configure` calls `os.geteuid`, absent on Windows. The
   first shim returned 0, which took Facdrone's "PostgreSQL refuses root" branch and skipped
   every DB-backed test. This version returns 1000 so the embedded server is started.
2. `facdrone_data.testing.pg_server.start` passes `-c listen_addresses=''` inside `pg_ctl -o`.
   Without a shell to strip the quotes, Windows PostgreSQL 17.9 tries to resolve the host name
   `''` and fails ("could not create any TCP/IP sockets", probe 2026-11-03). The unquoted empty
   value starts the server on its Unix socket. `start` below is the upstream function with only
   that option changed; both the module attribute and the name bound in `fixtures` are patched.
"""

from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path

if not hasattr(os, "geteuid"):
    os.geteuid = lambda: 1000

_PATCHED = False


def _patched_start():
    from facdrone_data.testing import pg_server as ps

    root = Path(tempfile.mkdtemp(prefix="facdrone-pg-"))
    server = ps.EmbeddedPostgres(root, ps.BIN_DIR)
    try:
        server._run("initdb", "-D", str(server.data), "-U", "postgres", "--auth=trust",
                    "--encoding=UTF8", "--locale=C", "--no-sync")
        options = f"-k {root} -c listen_addresses= -c fsync=off -c full_page_writes=off"
        # Second Windows gap: the started postgres inherits captured pipes, so capture_output
        # blocks forever after `pg_ctl -w start` returns. Detach the handles; `-w` still waits.
        subprocess.run([str(server.bin_dir / "pg_ctl"), "-D", str(server.data), "-o", options,
                        "-l", str(root / "server.log"), "-w", "start"],
                       check=True, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL)
        server._run("createdb", "-h", str(root), "-U", "postgres", ps.DATABASE)
    except BaseException:
        server.stop()
        raise
    return server


def _patch() -> None:
    global _PATCHED
    if _PATCHED or os.name != "nt":
        return
    from facdrone_data.testing import pg_server
    from facdrone_testing import fixtures

    pg_server.start = _patched_start
    fixtures.start_pg_server = _patched_start
    _PATCHED = True


def pytest_configure(config) -> None:  # tryfirst: run before facdrone_testing.fixtures' hook
    _patch()


pytest_configure.tryfirst = True  # type: ignore[attr-defined]
