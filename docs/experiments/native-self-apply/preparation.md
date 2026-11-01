# Task 7.1 preparation receipt

Prepared NS01 on Windows with CPython 3.12.10. The isolated baseline and candidate
contain the same **924 files**, copied from the actual working tree, including
existing uncommitted rule source and tests. Original production/test bytes were
unchanged. See [receipt.json](receipt.json), [source-inventory.json](source-inventory.json)
and [the source archive](baseline.zip). No candidate has been written.

Native preparation passed with a separately copied executable. Its SHA-256 is
`5b41c7bf7e3e6d6b015783884dd2b488c8e981490c1f297f4e2de1facc32aa4d`.
The package version string alone is insufficient provenance: the receipt also
records HEAD, actual native source/configuration hashes and executable identity.
The native baseline identity is
`f655aec4a68aa7b97e9c25fd791d42a861cc5cadd2c71e22eb68bbc8d0a170e0`.

The owned whole-module baseline has 1,599 native logical statements, 2,342 physical
lines, 437 decisions and 183 functions. Before any candidate edit, the frozen
target requires at least 25 fewer production statements, with maxima of 2,342
physical lines, 437 decisions and 183 functions. The earlier estimate of 36
removable statements uses CPython AST counting and is a distinct definition.
The native executable remains authoritative for the target.

The preliminary byte-identical comparison deliberately had no passing evidence:
it returned exit 2, `incomplete`. It calibrated counts; it did not establish
behavior or adoption. Dynamic call context remains unavailable, with located
relationships retained in [baseline-counts.json](baseline-counts.json). These
are context findings, including both sides and nested diagnostic details, not
dead-code declarations. Task 7.2 must review relevant ownership relationships
before admitting its result. Preparation itself does not certify semantic scope.
The broad wider snapshot includes prior research; category totals keep those
counts separate from production benefit. No performance claim follows from the
time taken to calibrate that wider snapshot.

The existing pinned uv environment passed **171 tests in 42.284 seconds**, exit
0, `OK`; [raw log](environment-probe.log). The relevant original unchanged tests
loaded source through the isolated baseline's `src` path. This is an environment
probe, not the future external execution-tree procedure or paired candidate
evidence. The full project regression suite was intentionally left for task 7.2.

## Reconstruct and verify

The original local directories are:

```text
.runs/native-self-apply/ns01/baseline
.runs/native-self-apply/ns01/candidate
.runs/native-self-apply/ns01/external/lesscode-core.exe
.runs/native-self-apply/ns01/external/frozen-checks
```

To reproduce inputs in a fresh directory, extract `baseline.zip` twice, into
separate new baseline and candidate roots. Verify every extracted regular file
against `source-inventory.json`; the path sets must match exactly. Keep the
executable and checks in a sibling external directory. Verify the executable
against the receipt; rebuilding from pinned native source/toolchain can establish
source reproducibility but does not automatically establish the same executable
hash. Use a new trial if the evaluator identity changes.

Copy `manifest.json`, `inventory.json`, `check-definition.json`,
`expected-outcomes.json` and the archived checks/resources into the external
directory. Run the frozen executable:

```text
lesscode-core.exe prepare --manifest EXTERNAL/manifest.json --baseline BASELINE --artifact-root EXTERNAL --out FRESH/frozen.json
```

Compare the resulting JSON value with [frozen.json](frozen.json); the same
snapshot, manifest and definitions must produce the same frozen record. Do not
overwrite the retained freeze. Runtime analyzer resources are separately listed
in [tool-inventory.json](tool-inventory.json), rather than included in source
benefit or silently fetched. Task 7.2 copies the existing installed resources
into its external execution trees, verifies their hashes, and freezes any new
independent runner before candidate edits. Re-freeze check definitions if adding
the runner changes their declared contract; run both source versions anew.

This task added documentation, JSON evidence and one source archive, with zero
production changes, zero test changes and zero new production dependencies.
Required maturity evidence remains open; Rust and whole-project maturity are
explicit exclusions. A rejected candidate remains an acceptable future outcome.
