# Parity fixtures of the agnostic driver

Used by `native/tests/driver_parity.rs`. Added with the owner's permission of 2026-11-10 (task 3.4 of
`.kiro/specs/agnostic-annotation-driver/`).

## `scopes/`: the five files of the prototype gate

Each file is an unmodified copy at the commit the gate used. The bytes are the ones the gate ran on (a Windows
checkout, so every file has CRLF line endings); `.gitattributes` marks the directory `-text` so that Git never
converts them. The mutant hashes in the references depend on these exact bytes.

| Fixture | Origin | Commit | Licence |
| --- | --- | --- | --- |
| `mmhmem/src/setup/ui/update.rs` | https://github.com/norandom/mmhmem | `f32596f8c3117aa05a0d4db924a4de18d80f42d1` | No licence file in the repository; the owner's own code, included with the owner's permission |
| `mvibe/mvibe_proxy/ratelimit.py` | https://github.com/norandom/mvibe | `b56d0e37a716e77c6237f9aa5fe230745f76742c` | Beer-ware, `mvibe/LICENSE` |
| `OpenUltraCode/src/state.ts`, `OpenUltraCode/src/verification.ts` | https://github.com/norandom/OpenUltraCode | `37a4e55f1b9397332acc6434783099a631a04918` | MIT, `OpenUltraCode/LICENSE` |
| `commons-csv/src/main/java/org/apache/commons/csv/Lexer.java` | https://github.com/apache/commons-csv | `2bd66bf181d44ffc92806f585d3955e915666f7f` | Apache-2.0, `commons-csv/LICENSE.txt` and `commons-csv/NOTICE.txt` |

These files are test data. They are not compiled, not shipped in the binary and not covered by the project's
complexity budgets.

## `model-packs/`: the two packs a model wrote at the gate

Byte-identical copies of `docs/experiments/annotation-driver/results/attempt2/d2/packs/{java,typescript}.json`
(gate attempt 2, R64). They are copied here because the Linux check runs in a container that mounts only
`native/` and `tools/`; a test must not read outside those.

## `reference/`: what the Python prototype produced

One file per gate scope, written by
`docs/experiments/annotation-driver/native-spike/make_reference.py` from the frozen prototype
(`engine.py`, `validator.py`, `run_controls.py`, the prototype's packs) run on the files above. Each holds the
eligible sites, every mutant (site, operator, SHA-256 of the mutated text), the control cases with their
annotations and outcomes, and the hashes of the prototype files and scope files it was made from.

To regenerate (needs the prototype's Python environment; nothing in the product does):

```powershell
cd docs\experiments\annotation-driver
uv run --no-sync python native-spike\make_reference.py `
  --scopes-dir ..\..\..\native\tests\fixtures\driver\scopes `
  --out-dir ..\..\..\native\tests\fixtures\driver\reference
```

A regenerated reference that differs is a finding, not a routine update: record it in the research log.
