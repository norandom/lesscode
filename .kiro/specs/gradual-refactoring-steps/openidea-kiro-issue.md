# Issue text for openidea-kiro: executed evidence per acceptance criterion

Written 2026-11-11 for the owner to file (task 6.4, Requirement 8.3). Not filed by the tooling or by an
agent. Until openidea-kiro agrees to a format, lesscode writes it as `step-evidence-provisional-v0` with
`"provisional": true`.

---

**Title:** Accept executed evidence per acceptance criterion from an external tool

**What I would like**

`kiro-cli check` audits the form of a spec: EARS, terms, contradictions. It cannot know whether a criterion
was ever shown by a run. lesscode judges small refactoring steps, each with a spec of its own under
`.kiro/specs/<step>/`, and for each criterion that the step ties to a command it records that the command
ran, on which commit and with which exit code. I would like `kiro-cli` to be able to read such a file, so
that a spec can be reported as "criterion 1.2: shown by a run on commit X" or "no evidence", and so that
evidence for another state of the code is recognised as stale.

**What lesscode writes today** (one file per step, beside the spec; example from a real step)

```json
{
  "receipt": "step-evidence-provisional-v0",
  "provisional": true,
  "step": "one-exit",
  "spec": "one-exit",
  "base": "9ee2fc32e31f9f18c78d5836c0d4dcfbad1dcc56",
  "revision": "e64b7b3587c36992dceb7551cccd042762e7c968",
  "hashes": {
    "scope": {"native/src/recording.rs": "da63888288777b19c3007ac1c67834ddac8e1098"},
    "tests": "9a4d1c8d1011ab977eaad773992e02ec44c0cfdbef83918d0a6e6c68e014168e",
    "declaration": "dca471ab06bd2075b18f544e7dcab80e6a866bf4"
  },
  "criteria": [
    {"id": "1.1",
     "command": ["cargo", "nextest", "run", "--locked", "--manifest-path", "native/Cargo.toml",
                 "--test", "recording", "verdict_"],
     "exit_code": 0, "passed": true}
  ]
}
```

- `spec` is the feature name; `criteria[].id` is `<requirement>.<criterion>` as numbered in
  `requirements.md`.
- `revision` is the commit the command ran on, in a copy of the repository. `hashes.scope` are git blob
  hashes of the files the step may change, `hashes.tests` is one hash over the committed test files,
  `hashes.declaration` the blob hash of the step declaration.
- The commands are the project's own. lesscode adds no test framework.

**Questions to settle before the format is fixed**

1. Where should the file live and what should it be called, so that `kiro-cli` finds it without
   configuration? Today: a path the step declaration names, by convention `.kiro/specs/<spec>/evidence.json`.
2. Is `<requirement>.<criterion>` the right identifier, or does openidea-kiro have a stable criterion ID
   that survives renumbering?
3. Should evidence name the tests that ran, not only the command and its exit code? lesscode does not parse
   test runner output; it could record a command's output hash.
4. How should staleness be decided: by the commit, by the hashes of the files, or both? lesscode's own guard
   uses the hashes, so that a commit which only adds the evidence file does not make it stale.
5. Should a criterion without evidence be a finding, and at which severity? Some criteria cannot be shown by
   a command.
6. One file per spec with all criteria, or one line per run in an append-only file?

**What lesscode does not ask for**

No change to how specs are written, and no judgement by `kiro-cli` about the code. lesscode does not judge
the consistency or traceability of a spec either; the two tools would meet in this one file.
