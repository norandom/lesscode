# Hand-written semantic mutants of the original lifecycle scope (task 6.1, for 6.4)

Written 2026-11-06, before any candidate exists. Each patch is a unified diff against one frozen file of
`.runs/e2-facdrone/source` (CRLF content lines kept byte for byte), makes one behaviour change that the
public surface or the recorded corpus can observe, and was verified on a scratch copy of the single file
(`git apply --check`, then `git apply`, sha256 of the result, `compile()`): receipt label
`mutants-apply-check`. The frozen tree was only read. The predictions below are fixed now and are read in
6.4 as caught or missed with the catching evidence (requirement 8.4); 6.4 ports each one to the candidate
where the code moved and documents the port.

Rule for the catching evidence, fixed before the runs: a mutant is **caught** if the 6.4 arm on the mutated
copy exits nonzero (a seed test fails, or the replay reports a Level 1 divergence); the first failing test
node or the first divergence (`sequence op k path`) is the evidence. A patch is applied to a copy whose
seven files are hash-verified against the frozen original before and after (restore by copy).

| Id | File, line | Change | Predicted catcher |
| --- | --- | --- | --- |
| M1 | `lifecycle_store.py:148` (`overdue_flags`) | off-by-one in the overdue window: `f.overdue(now, after)` becomes `f.overdue(now, after - timedelta(days=1))`; a flag counts as overdue one day early | **likely survives: alphabet gap.** Seed tests: `test_a_flag_closes_only_by_a_fresh_census_or_a_retirement` judges a 30-day-old flag against 14 days (overdue on both sides). Corpus: the only `overdue_flags` operations are S03 ops 4 and 6 (age 0 and 30 days against 14), G02 op 1 (age 0 against 14) and G10 to G12 op 1 (no open flag). No sequence observes a flag whose age lies in (after − 1 day, after]. The alphabet can express the catcher (`flag`, `tick 1`, `overdue_flags days=1`), so 6.5/6.7 may replay it as a new sequence under requirement 9.6 |
| M2 | `lifecycle_store.py:212` (`transition`) | the revision bump is dropped: the applied event is written at `r.expected_revision` instead of `r.expected_revision + 1` | **caught by seed tests and by the corpus.** Seed test: `tests/unit/test_lifecycle_store_db.py::TestTheTableOnTheDatabase::test_enroll_admit_flag_readmit_and_retire_append_events` (the enrolment lands at revision 0, the admit at expected 1 is refused with `expected revision 1, standing is at 0`, or the applied-row unique index refuses the second revision-0 row); also `test_the_enrollment_command_is_the_registry_route_from_absent` expects `absent -> candidate at revision 1` in the CLI output. Corpus: first divergence S01 op 1 (`enroll`), path `result.revision` (1 recorded, 0 replayed) and `result.event_id` |
| M3 | `lifecycle_monitor_runner.py:209-225` (`MonitorRunner._emit`) | reordered event append: the `lifecycle.not_run` records are emitted before the `lifecycle.reading` records instead of after | **caught by the corpus only.** Seed tests: predicted to pass, since `test_watch_unmeasured_and_not_run_are_recorded_without_the_alerting_event` asserts membership (`"lifecycle.not_run" in events`), not order. Corpus: the recorded `emitted` list keeps emission order per signal; S09 op 2 (`run` with windows none/thin/breach) is the one recorded run that emits both a not-run window and readings; first divergence S09 op 2, path `result.emitted.carry@3[0].event` (`lifecycle.reading` recorded, `lifecycle.not_run` replayed) |
| M4 | `lifecycle_store.py:105` (`standing`) | changed standing rule: the standing's `census` reference is taken from the latest applied event whatever its authority, instead of only from a census event | **caught by the corpus only.** Seed tests: predicted to pass; the only assertion on `standing.census` (`test_enroll_admit_flag_readmit_and_retire_append_events:106`) follows a census admit, where both sides agree, and the seams' `monitored()` reads `census` only for admitted standings. Corpus: S02 op 7 (`standing` after an enrolment: `census` None recorded, `enrollment:operator` replayed) and S06 op 3 (`enrollment:op`); first divergence S02 op 7, path `result.census` |

Patch hashes (sha256 of the patch file; the mutated file's sha256 is what 6.4 must see after applying it):

| Id | Patch sha256 | Mutated file sha256 | Frozen file sha256 |
| --- | --- | --- | --- |
| M1 | `89699566b9e908d3e79321d597ada0811253e23086e448e3394502a7bcd035a3` | `b92700bc175706b0dd5200635259b823e7173b0f0ed60cb6234810b51425ac79` | `35d9f27a…8d974` (`lifecycle_store.py`) |
| M2 | `01ee894d458d422eece64931a0bf27715ac569a0f146fe6cd97c4b977536f181` | `da7640cd3362ffc6274ec94e55728f7874038c0fe47f93653fa8a95643bc4950` | `35d9f27a…8d974` |
| M3 | `7dceff53347386a809c870381d3d0c1426300d8ca1abf4f95422953bec7d57eb` | `175ddfd1441b35c149f4939f2cb9758ef93ad6284306b87b14df28b802b3fcf1` | `fdd35e1c…b2f5` (`lifecycle_monitor_runner.py`) |
| M4 | `b0526f6dd762ecb124c721ce89fadf89c3cc1a7f334677f4984b46a69b963dea` | `1c1aefe91eb9e95bd5a6ffe3d13f5f55e15207dc4a15be8e942a63a0284c10d9` | `35d9f27a…8d974` |

Why these four: M2 is the classic fault every layer sees (a control for the arm); M3 and M4 are faults
the seed tests are predicted to miss and the recording to catch, which is the protection the corpus is
supposed to add; M1 is a fault the whole acceptance is predicted to miss, an honest alphabet gap stated
before the run. All four lie in code the alphabet reaches (`transition`, `standing`, `overdue_flags`,
`MonitorRunner.run`), not in the declared residue.

How 6.4 applies them: copy the frozen file into the mutable copy (`.runs/e4-lifecycle/orig-mut`), `git apply
mutants/M<k>.patch` with cwd at the copy's root, verify the file's sha256 equals the mutated hash above, run
the arm, restore by copying the frozen file back and verify the seven hashes. On the candidate, port the
change to where the construct moved; if the construct no longer exists (for example the emission loop was
restructured so that no reorder is expressible), report the port as not possible and the mutant as not
run on that tree, with the reason.
