# Regenerator as a recorded single-turn program (deconvolution-loop task 4)

- Written: 2026-11-06, Europe/Berlin, during task 4 of
  [deconvolution-loop](../../../../.kiro/specs/deconvolution-loop/tasks.md). Requirements 6.1, 6.7, 6.8
  and 11.2 of that spec; design rows RoundController and Regenerator.
- Purpose: replace the interactive opus agent of E3 step 1 with a program of fixed signature that makes
  exactly one request to a model endpoint per turn, records every input and output by content hash, and
  writes the returned files into a fresh isolated candidate tree. The step ends there. The edit-scope check,
  the structure check, the measures and the acceptance arm are task 5.1 (integration); none of them run here.
- Files: `regen_turn.py` (the program), `test_regen_turn.py` (pytest, no network),
  `results/request-e3-turn1.json` (the E3 scope request, feedback `null`), `results/receipt.json`
  (receipt_atomic format), `results/turns/<label>/` (prompt and response texts with sha256 sidecars),
  `results/logs/<label>.log`.

## Signature

`TurnRequest` (JSON on disk, dataclass in code), paths relative to the lesscode root or absolute:

| Field | Meaning |
| --- | --- |
| `scope_root` | tree the scope files are read from (the frozen copy `.runs/e2-facdrone/source`) |
| `scope_files[]` | the only files that may change; sent verbatim, each fenced under its path |
| `surface_path` | the derived surface (`public-surface.json`), sent verbatim |
| `definitions` | the E3 definitions text (public API, pure core, effect shell, observation levels, write site, edge) |
| `targets` | the measure targets as a JSON object (T2 and T3 of the E3 plan, with the original's baseline) |
| `contract` | the structural contract text (declaration format, pure rule, write-site rule, bounds, patched names) |
| `acceptance_command` | text only; the loop runs it, the program never does |
| `feedback` | `null`; or `{"kind": "failed", "first_divergence", "measure_gaps"}`; or `{"kind": "accepted_scored", "first_divergence", "measure_gaps", "surviving_faults", "counterexamples"}` |

`TurnResult`: `files[] {path, content}`, `structure {pure[], shell[]}` with `file.py::qualname` entries,
`change_log` (text), `meta` (the recorded turn facts). The model is asked for exactly this object through
the endpoint's JSON-schema output format (`output_config.format`, `additionalProperties: false`).

The turn rule (req 6.7) is enforced in `build_prompt`: with `feedback` null the user message has no feedback
section at all; `failed` renders exactly two fields (first divergence, measure gaps against the targets);
`accepted_scored` renders those two plus surviving faults and realised counterexamples. Any other kind or
any extra key is a `ValueError` before anything is sent.

## Prompt layout

- System (fixed text `SYSTEM`, 1,730 chars): the regenerator role; the rules (only the scope files may
  change, every surface name keeps its kind, signature and behaviour, patched names stay module globals,
  SQL text and parameters unchanged, declare every function pure or shell, meet the targets, feedback is
  the only information about a previous round); answer with one JSON object.
- User (87,447 chars for the E3 scope): `## Scope files` with each file fenced as ` ```python ` under
  `### <path>`; `## Derived public surface` (the JSON verbatim); `## Definitions`; `## Measure targets`
  (JSON); `## Structural contract`; `## Acceptance command (run by the loop, not by you)`; and, only when
  feedback is present, `## Feedback from the previous round`.
- Request: `model`, `max_tokens` 64000, `system`, one user message, `output_config {effort: high, format:
  json_schema}`. No `temperature`, `top_p`, `thinking` budget, prefill, `tool_choice` or `fallbacks`.
  `--thinking-display summarized` adds `thinking {type: adaptive, display: summarized}`, which is what
  omitting `thinking` already means on this model plus the opt-in to stream the thinking text (used on the
  retry, see below). Streaming through `client.messages.stream(...)`, then `get_final_message()`.

## What is recorded (req 6.8, 11.2)

- Receipt `results/receipt.json` through `receipt_atomic.py`: the outer `run` entry (command, cwd,
  recorded env overrides, which are empty, start, end, seconds, exit); fact `t4-extras` (anthropic SDK
  version and where it is installed); fact `<label>-inputs` (sha256 of `regen_turn.py`, the request JSON,
  the surface file and the five frozen scope files; candidate root; endpoint style, base-URL host and the
  sha256 of the model name; feedback kind; prompt hashes); note `<label>-turn` (system prompt sha256,
  user prompt sha256, full request body sha256 over the exact kwargs with `sort_keys`, response body
  sha256 over `msg.to_json()`, model as sent and as returned, message id, request id, `seed: null` with the
  note "endpoint exposes no seed; determinism not claimed", counted input tokens, usage, stop reason and
  stop details, elapsed seconds, stream event count); note `<label>-candidate` (per written file the path,
  bytes and sha256; structure counts; sha256 of `result.json`); or note `<label>-failed` with the reason,
  exit code and the stream progress at the failure. Failed turns stay in the receipt (req 11.3).
- Texts under `results/turns/<label>/`: `system.txt`, `user.txt`, `response.json` (`msg.to_json()`),
  `result.json` (written files with hashes, the structure declaration, the change log), each with a
  `.sha256` sidecar. The structure declaration and change log are deliberately kept here and not written
  into the candidate tree, so that only the scope files can differ from the frozen copy. `user.txt` and
  `response.json` hold the specimen's source and its regenerated form; they stay on disk and are not
  committed (as the large golden traces), their sidecars, `system.txt` and `result.json` are.
- Never recorded: the key. The program reads `LESSCODE_MODEL_ENV` (default `~/.lesscode/model.env`,
  `NAME=value # comment` lines) itself; `Endpoint` exposes the key only through `api_key()` and redacts it
  in `repr`, `str` and `public()`. The receipt's env field holds only `--env` overrides, and none are passed.
- Exit codes: 0 ok; 2 usage; 3 failed turn (API or transport error, `stop_reason` other than `end_turn`,
  invalid JSON); 4 the candidate copy's scope files differ from the frozen ones (checked before the call);
  5 counted input above 120,000 tokens (checked before generation); 6 a returned path outside the scope
  (checked before any file is written). A missing or malformed endpoint file raises `ValueError` and exits
  1 through the interpreter; it is not a turn outcome.
- Hash semantics (found by the 2026-11-06 review): every sha256 in the receipt and in the `.sha256`
  sidecars is computed over the text as sent or received (UTF-8, LF). For the three recorded turns the
  texts under `results/turns/` were written through `write_text` without `newline`, so the bytes on disk
  carry CRLF and a raw byte hash of those files differs from the sidecar; reading the file as text and
  re-encoding reproduces the sidecar value. From the apparatus revision recorded in fact
  `t4-apparatus-after-review` onwards, `_save` writes with `newline="\n"` so the bytes equal the hashed
  text. The request body itself is not stored, only its hash; `request_kwargs(endpoint, system, user,
  max_tokens, effort, display)` rebuilds it byte for byte from the saved prompts and the recorded
  parameters (the reviewer did so for both recorded requests and matched the receipt).

## How to run

Native Windows, the e2-facdrone venv interpreter, `PYTHONDONTWRITEBYTECODE=1`, `PYTHONIOENCODING=utf-8`.
The SDK lives in `.runs/regen-extras` (`uv pip install --python <venv python> --target .runs/regen-extras
anthropic`; appended to `sys.path` so the venv's own packages win).

```
<python> regen_turn.py prompt --request results/request-e3-turn1.json --out <dir>      # dry: prompts + hashes, no call
<python> ..\receipt_atomic.py results\receipt.json run <label> --no-probe --cwd <this dir> -- ^
  <python> regen_turn.py run --request results\request-e3-turn1.json --cand D:\Source\lesscode\.runs\e4-regen\cand-t4 ^
           --receipt results\receipt.json --label <label> [--thinking-display summarized]
<python> -m pytest -q -p no:cacheprovider test_regen_turn.py
```

The candidate root is created if absent as `shutil.copytree` of the frozen copy (no symlinks; `.git`,
`__pycache__`, `.pytest_cache`, `.venv` ignored) and the five scope files are verified by sha256 against
the frozen ones before the call. Written files keep the line-ending convention of the file they replace
(the frozen copy is CRLF).

## Smoke on the frozen E3 scope (2026-11-06)

Dry prompt (`prompt`, no call): system sha256 `5fab7bf2…ae41` (1,730 chars), user sha256 `36257f4a…e402`
(87,447 chars, 2,595 lines); all five files, the surface JSON, definitions, targets, contract and the
acceptance command present; no feedback section. Recorded as note `t4-dry-prompt`. Counted input before
each generation: 36,115 tokens (cap 120,000). Same request file and prompts for all three turns.

Three logged turns; the third produced the candidate tree. **The task's done criterion is met by
`t4-turn2`** (note `t4-smoke-outcome-2`): one logged turn on the E3 scope yielded a candidate tree and the
receipt holds the prompt, response, model identifier and seed hashes.

| Label | Request | stop_reason | Usage (in / out, thinking) | Elapsed | Outcome |
| --- | --- | --- | --- | --- | --- |
| `t4-turn1` | max_tokens 64000, effort high, thinking display omitted | none | none | 310.2 s | failed, exit 3: `ReadError: WinError 10054` (connection reset by the remote host), no response received; `results/turns/t4-turn1/` holds the prompts only |
| `t4-turn1-retry` | max_tokens 64000, effort high, `--thinking-display summarized` | `max_tokens` | 36,557 / 64,000, thinking 64,000 | 556.4 s (10,839 events) | failed, exit 3: one thinking block, no text block, no JSON; request `2d7feed0…`, response `989ccff4…`, message `msg_011CgmMZ3za6qworyk9Jd1eh` |
| `t4-turn2` | max_tokens 128000, effort medium, `--thinking-display summarized` (controller decision, note `t4-turn2-decision`) | `end_turn` | 36,557 / 84,992, thinking 63,504 | 668.4 s (14,231 events; command 673.3 s) | **candidate written**; request `2ab4b6f0…`, response `05e7b990…`, model `claude-opus-5-5` sent and returned, message `msg_011CgmNUMVFLc4Q8ZgkDAGNC`, seed null; structure 36 pure / 17 shell; `result.json` `396a5179…` |

The retry's one change (`--thinking-display summarized`) was made so the stream would carry thinking deltas
instead of staying silent; the first turn's reset at ~5 minutes is consistent with an idle-connection cut but
the cause is not established. The retry showed the real limit: at effort `high` the model spent the whole
64,000-token output budget thinking. `t4-turn2` raised `max_tokens` to the model maximum and lowered the
effort; nothing else changed (signature, prompts, model, schema, candidate root, no feedback section).

Candidate `.runs/e4-regen/cand-t4` after `t4-turn2` (note `t4-turn2-candidate`; all five CRLF like the
frozen files):

| File | Lines | Bytes | sha256 | py_compile |
| --- | ---: | ---: | --- | --- |
| `decision_evidence_publication.py` | 427 (was 442) | 15,423 | `4d328cc04f6e2c99…` | ok |
| `decision_evidence_store.py` | 298 (was 313) | 12,644 | `27eba0f0334da302…` | ok |
| `decision_evidence_reader.py` | 82 (was 102) | 3,339 | `bdefe6b697747bb9…` | ok |
| `decision_input_slice_store.py` | 225 (was 239) | 8,706 | `842cb8d5f27ea5c5…` | ok |
| `decision_slice_publisher.py` | 182 (was 178) | 7,054 | `fbf61509fcd5ff91…` | ok |
| total | 1,214 (was 1,274) | | | |

`py_compile` (note `t4-turn2-pycompile`): all five compile under CPython 3.13.16 with `cfile` outside the
candidate tree; no `.pyc` or `__pycache__` exists in the tree afterwards. This is information for task 5.1,
not a gate; the physical line count is a raw figure, not the T2 measure.

`request_id` is `null` in the turn notes: `get_final_message()` in anthropic 2.6.0 does not carry
`_request_id` on the streamed message. Apparatus hashes: `regen_turn.py` `f0f53f67…` for `t4-turn1` (fact
`t4-turn1-inputs`) and `978f5edc…` for the retry and `t4-turn2` (the patch added stream-progress recording
and the `--thinking-display` flag). Request JSON `20e48d54…`, surface `5335c53d…`. Line budget:
`regen_turn.py` is 337 lines against the task's 300 (overrun reported, not compacted; the 22-line system
brief and the schema are inside the module); tests 138 lines.

### The model's change log (t4-turn2, quoted in full from `results/turns/t4-turn2/result.json`)

The log is one paragraph in the response; it is broken here at each per-file marker, with no word changed.

> decision_evidence_publication.py: seal_evidence and seal_evidence_with_run_slices now share one private _bound (replacing _captured_payload and _seal_captured); it decodes the capture, converts inputs to run-slice references only when a namespace is given, builds each route identity directly (same scope/execution/step_id/as_of/versions values as the former replace chain) and seals with schema_version 1 or 2. Error order kept: run-slice conversion before as_of parsing. _valid_slice_scalar inlined into slice_bytes with identical predicate. deduplicated_inputs no longer uses set.add (dict membership instead) so the pure core has no write-site match. compact_capture replaces every input via dataclass replace (equal values, identical encoded capture). bind_evidence and stored_bound_payload use conditional expressions; stored_evidence_payload pops inside the comprehension (same key order). Function docstrings became comments.
> 
> decision_evidence_store.py: _check_record/_check_source_identity/_check_transform/_check_ic/_check_input/_validate_records folded into one _validate(value, adapter=_BATCH) with identical check order and messages; batch-only stage/route checks run only for DecisionEvidenceBatch, so decode_consumed_evidence reuses it. _json removed (_dump has a compact default separator); the nested timestamp default became module-level _timestamp bound with functools.partial. _records is an iterative preorder walk (same yield order). SQL text and parameters unchanged.
> 
> decision_evidence_reader.py: _failure and _resolve_input inlined into _resolve_batch (identical messages and chaining); read_resolved_evidence only routes batches with run slices to _resolve_batch. read_decision_evidence stays a module global looked up at call time.
> 
> decision_input_slice_store.py: _rebuild and _refusal inlined into _resolve/chain_length with identical messages; _walk loops on base_id instead of break; _row_digest inlines its canonical text; SliceResolver.resolve rebinds fresh caches on namespace change instead of clearing in place (not preserved: external references to the old cache dicts no longer see them emptied). _load remains a module global used by _walk.
> 
> decision_slice_publisher.py: _referenced_id inlined into referenced_slices with identical message; row/retained build rows with inline expressions in the same evaluation order. All SQL statements unchanged.
> 
> Deliberately not preserved: function/class docstrings (now comments, __doc__ differs; ConsumedEvidence/_Captured/SealedEvidence auto-generated dataclass __doc__), private helper names removed (_json, _check_*, _validate_records, _captured_payload, _seal_captured, _valid_slice_scalar, _failure, _resolve_input, _rebuild, _refusal, _referenced_id), none of which appear in the public surface.

## Not claimed

No acceptance run, no measures, no structure check, no edit-scope check (task 5.1 runs them on
`cand-t4`). No determinism (the endpoint exposes no seed). No statement about whether the candidate passes
T1 to T3, nor about the change log's own claims (unchanged SQL, identical messages); they are the model's
assertions until 5.1 checks them. The first failure's cause (transport) is a hypothesis, not a finding.
