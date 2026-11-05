"""Tests for the guard and atomic receipt apparatus (deconvolution-loop task 1.2).

The guard's exit code (0 proceed, 3 stop) depends on host load; the tests assert
the note shape and ordering, not the exit code.
"""
import datetime
import json
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import receipt_atomic  # noqa: E402

WORKER = (
    "import sys; sys.path.insert(0, sys.argv[1]); import receipt_atomic as r, datetime\n"
    "for i in range(10):\n"
    "    t = datetime.datetime.now(datetime.UTC).isoformat()\n"
    "    r.add_command(sys.argv[2], {'label': f'{sys.argv[3]}-{i}', 'cmd': ['dummy'], 'cwd': None, 'env': {},\n"
    "        'start_utc': t, 'end_utc': t, 'seconds': 0.0, 'exit': 0, 'adaptive': None})\n"
)


def test_concurrent_appends_keep_all_entries(tmp_path):
    rec = tmp_path / "receipt.json"
    procs = [subprocess.Popen([sys.executable, "-c", WORKER, str(HERE), str(rec), tag]) for tag in ("a", "b")]
    assert [p.wait(timeout=120) for p in procs] == [0, 0]
    data = json.loads(rec.read_text())
    labels = [c["label"] for c in data["commands"]]
    assert len(labels) == 20 and len(set(labels)) == 20
    assert set(data) == {"commands", "facts", "notes"}


def test_guard_note_precedes_heavy_step(tmp_path):
    rec = tmp_path / "receipt.json"
    r = subprocess.run([sys.executable, str(HERE / "guard.py"), "dummy-heavy", "--receipt", str(rec),
                        "--max-wait", "1"], capture_output=True, text=True, timeout=300)
    assert r.returncode in (0, 3), r.stderr  # exit code depends on host load
    receipt_atomic.run(rec, "dummy-heavy", [sys.executable, "-c", "print('heavy')"], probe=False,
                       logs=tmp_path / "logs")
    data = json.loads(rec.read_text())
    notes = [n for n in data["notes"] if n["key"] == "dummy-heavy-guard"]
    assert len(notes) == 1
    value = notes[0]["value"]
    assert value["thresholds"] == {"max_wait_s": 1.0, "cpu_other_max": 0.35, "ram_floor_gb": 8.0, "ram_min_gb": 6.0}
    assert "probe" in value and value["decision"] in ("run", "wait again", "stop")
    cmd = data["commands"][0]
    assert cmd["label"] == "dummy-heavy"
    assert datetime.datetime.fromisoformat(notes[0]["time_utc"]) < datetime.datetime.fromisoformat(cmd["start_utc"])


def test_format_compatible_with_receipt3(tmp_path):
    rec = tmp_path / "receipt.json"
    receipt_atomic.add_note(rec, "k", {"x": 1})
    receipt_atomic.add_fact(rec, {"v": 1})
    receipt_atomic.run(rec, "c", [sys.executable, "-c", "pass"], probe=False, logs=tmp_path / "logs")
    data = json.loads(rec.read_text())
    assert list(data) == ["commands", "facts", "notes"]
    assert set(data["commands"][0]) == {"label", "cmd", "cwd", "env", "start_utc", "end_utc", "seconds", "exit", "adaptive"}
    assert set(data["notes"][0]) == {"time_utc", "key", "value"}
    assert "time_utc" in data["facts"][0]
