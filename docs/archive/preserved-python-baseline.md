# Preserved Python Baseline & Commit Reference Index

This document permanently records the git commit references, hashes, and recovery procedures for the original Python baseline implementation (`src/lesscode_baseline/` and associated test infrastructure) upon its formal retirement in favor of the unified native Rust substrate (`lesscode-core`).

All files, history, and work-in-progress remain fully preserved in Git history and can be inspected or recovered at any time using the references below.

---

## 1. Preserved Commit References

| Commit Hash | Subject / Milestone | Content / Key Files Preserved |
|---|---|---|
| **`fd0b5b9`** | `wip(rules): preserve rule evaluation checks and round 21 sql divergence artifacts` | Final state of owner WIP: `src/lesscode_baseline/rules/evaluate.py`, `tests/test_rule_evaluation.py`, and `sql-divergences-round21.json` |
| **`73c39e3`** | `docs(research): record R53 multi-language deconvolution synthesis report` | R53 COBESY research synthesis across Python, TypeScript, Java, and Rust |
| **`3295408`** | `docs(roadmap): record completion of Java and Rust deconvolution unification` | Final four-language roadmap completion status in `.kiro/steering/roadmap.md` |
| **`d62980d`** | `docs: add multi-language deconvolution architecture and extension guide` | Comprehensive architecture documentation at `docs/multi-language-deconvolution.md` |
| **`e31b650`** | `feat(rust-adapter): complete task 6 full system verification, budget audit, and GO verdict` | Native Rust deconvolution adapter completion and GO certification |
| **`a1b72a4`** | `feat(java-adapter): complete task 6 full system verification, budget audit, and GO verdict` | Java deconvolution adapter completion and GO certification |
| **`607c72f`** | `feat(typescript-adapter): complete task 6 full system verification, budget audit, and GO verdict` | TypeScript deconvolution adapter completion and GO certification |
| **`3c9597d`** | `feat(native-closure): complete task 7 full system verification, budget audit, and GO verdict` | Native closure porting of state boundary deconvolution into Rust substrate |
| **`f619117`** | `docs(research): record R46 e6 report envelope adoption maturity gate certification` | State-boundary deconvolution dual-specimen adoption (E5/E6) in Python |

---

## 2. Git History Lookup and Recovery Procedures

### Inspecting Preserved Code Without Modifying Working Tree
To view any preserved Python file exactly as it was at commit `fd0b5b9`:
```bash
git show fd0b5b9:src/lesscode_baseline/rules/evaluate.py
git show fd0b5b9:tests/test_rule_evaluation.py
git show fd0b5b9:src/lesscode_baseline/state_boundary/frame.py
```

### Viewing Historical Changes
To view the full commit history and diffs of the Python baseline:
```bash
git log fd0b5b9 -- src/lesscode_baseline/
git log -p fd0b5b9 -- src/lesscode_baseline/rules/evaluate.py
```

### Restoring a File or Subsystem From History
If any part of the Python baseline is needed for reference, benchmarking, or research comparison:
```bash
# Restore a single file:
git checkout fd0b5b9 -- src/lesscode_baseline/rules/evaluate.py

# Restore the entire baseline directory into a local folder:
git checkout fd0b5b9 -- src/lesscode_baseline/
```

---

## 3. Native Rust Supersession Mapping

All capabilities of the Python baseline have been completely ported and expanded in the native Rust substrate:

| Retired Python Module | Native Rust Implementation | Nextest Binary |
|---|---|---|
| `src/lesscode_baseline/state_boundary/frame.py` | `native/src/state_frame.rs` | `state_frame` |
| `src/lesscode_baseline/state_boundary/pure_verifier.py` | `native/src/pure_verifier.rs` | `pure_verifier` |
| `src/lesscode_baseline/state_boundary/mutant_generator.py` | `native/src/mutant_generator.rs` | `mutant_generator` |
| `src/lesscode_baseline/state_boundary/stream_monitor.py` | `native/src/stream_monitor.rs` | `stream_monitor` |
| `src/lesscode_baseline/surface.py` | `native/src/surface.rs` | `surface` |
| `src/lesscode_baseline/recording.py` | `native/src/recording.rs` | `recording` |
| `src/lesscode_baseline/regenerator.py` | `native/src/regenerator.rs` | `regenerator` |
| `src/lesscode_baseline/metrics.py`, `measures.py` | `native/src/metrics.rs`, `measures.rs` | `measures`, `budgets` |
| `src/lesscode_baseline/cli.py`, `evaluation.py`, `report.py` | `native/src/main.rs`, `compare.rs` | `cli`, `workflow` |
| (New multi-language adapters) | `native/src/adapters/` (`python`, `typescript`, `java`, `rust`) | `*_adapter`, `*_e2e` |

The native Rust substrate provides strict deterministic performance, single-binary distribution, zero runtime dependencies, and strict memory/token budgeting.
