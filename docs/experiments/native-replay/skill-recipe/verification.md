# Task 6.4 skill recipe verification

On 2026-11-01, Windows x64, the conditional [native recipe](../../../../.agents/skills/lesscode-discovery/references/native-workflow.md) exercised the compiled CLI against unchanged task 6.2 ATS records. Requirements 4.1, 4.2 and 4.5; design CLI/library contract, Scope contract and File Structure Plan. This is documentation/instructions integration; no runtime behavior or artificial test was added. RED is not applicable to this authorized non-behavioral task.

The command block was run from `D:/Source/lesscode` with `$ats` and `$core` exactly as documented and `$run = '.runs/native-replay/skill-recipe-6.4'`. Each process outcome was captured immediately:

| Invocation | Exit | Raw output |
| --- | --- | --- |
| `prepare` | 0 | [prepare.log](prepare.log): `Prepared` fresh frozen path |
| `compare` candidate | 0 | [candidate.log](candidate.log): `Accepted` |
| `compare` mutant | 1 | [mutant.log](mutant.log): `RejectedBehavior`; declared behavioral failure; observed mismatches |

The full command arguments appear in the recipe. Both comparisons passed repeated explicit artifact roots (`$ats/baseline-raw` and `$ats`); no implicit search was used. The executable's observed SHA-256 is retained in [receipt.json](receipt.json), with raw outputs and exit codes. No claim of authenticated execution follows from local hashes.

The freshly generated freeze, candidate JSON/Markdown and mutant JSON/Markdown were each byte-identical to the corresponding durable [task 6.2 records](../ats-run/recorded/). [output-sha256.json](output-sha256.json) retains fresh and expected hashes plus exact durable paths; [equality.json](equality.json) additionally records full parsed JSON equality for both outcomes and the freeze. Large duplicate artifacts are not copied into this verification folder. Fresh output remains at the recorded ignored run path. The linked durable reports preserve the hypothesis, metrics, check domains, rejection explanations and limitations without duplicating their schemas here.

All 28 input files (the complete ATS recorded tree plus the replay runner, original corpus harness and oracle) were hashed before and after. [inputs-before.json](inputs-before.json) and [inputs-after.json](inputs-after.json) match exactly; the receipt records `inputs_unchanged: true`. Recorded source, frozen definitions, checks and raw evidence were unchanged. The original ATS behavioral corpus was not re-executed by this documentation task: the existing paired observations were admitted again. For fresh behavioral execution follow [the task 6.2 runner](../ats-run/README.md).

The observed accepted result retains 8820 matched fictional reporting cases, exact return/exception observations and subclass dispatch checks. The adverse case retains 2251 observed mismatches. Whole-module accounting and its separately frozen task 6.2 source target reproduce exactly; unresolved internal call layers remain unavailable. External services and non-reporting behavior remain unverified. The recorded inventory is null and dependency changes empty, so this replay demonstrates neither inventory admission nor framework selection; the skill links implemented inventory-admission examples and requires shared snapshot-bound inventory review for subsequent proposals. These manually selected trials do not establish the discovery skill's effectiveness.

Skill validation command (exit 0, raw output `Skill is valid!`):

```powershell
uv.exe run --no-project --no-python-downloads --python .venv-task11/Scripts/python.exe --with PyYAML==6.0.3 python -X utf8 C:/Users/mariu/.codex/skills/.system/skill-creator/scripts/quick_validate.py .agents/skills/lesscode-discovery
```

The first attempt without `--with` failed with `ModuleNotFoundError: No module named 'yaml'`. The second, with PyYAML but without `-X utf8`, failed reading existing UTF-8 punctuation with Windows cp1252 (`UnicodeDecodeError`, byte `0x9d`). The final command used an ephemeral uv dependency and explicit UTF-8, preserving the pinned environment and skill text. Frontmatter validation does not validate the workflow's discovery quality. Independent forward review remains parent-owned.
