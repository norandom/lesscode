# R51: Java deconvolution adapter prototype feasibility gate

- Report date/time and timezone: 2026-11-09, Europe/Berlin.
- Execution interval: 2026-11-09 18:57:00 to 18:57:25 Europe/Berlin (25.12s elapsed).
- Status: completed / prototype gate passed (PROCEED disposition).
- Spec: [.kiro/specs/java-deconvolution-unification/prototype-gate.md](../../../.kiro/specs/java-deconvolution-unification/prototype-gate.md), requirements REQ-JAVA-1 through REQ-JAVA-5.
- Hypothesis ID and predecessor: [R50](08-typescript-adapter-prototype-gate.md), [R49](07-native-closure-prototype-gate.md), [R48](06-aligned-language-adapter-designs.md), [R41](2026-11-08/08-e4-reproduction-apparatus-stability.md).
- Type: preregistered prototype feasibility gate execution.

## Decision

The prototype feasibility gate for `java-deconvolution-unification` passed with a definitive **`PROCEED`** disposition. The gate receipt is cryptographically secured at [`.kiro/specs/java-deconvolution-unification/results/prototype-gate-report.json`](../../../.kiro/specs/java-deconvolution-unification/results/prototype-gate-report.json) (`.sha256` sidecar verified: `207765c042ef02e263c6d19648c19532675dc9c52b95e6044a179d409e2b3410`).

This outcome permits:
1. Formal approval of the `java-deconvolution-unification` technical design (`design.md`).
2. Immediate advancement to Phase 2 with production implementation tasks approved for foreground execution (`tasks.md`).
3. Advancing Stage 3 of the 4-stage roadmap (`.kiro/steering/roadmap.md`).

Separation of outcomes:
- **Candidate Admission:** Admitted real licensed specimen `apache/commons-csv` (`src/main/java/org/apache/commons/csv/CSVPrinter.java`) at commit `2bd66bf181d44ffc92806f585d3955e915666f7f` (Apache License 2.0) with frozen Microsoft OpenJDK 21.0.12.1 LTS, Apache Maven 3.9.9, and JUnit 5 (JUnit Jupiter 5.12.2 / JUnit Platform 1.14.4).
- **Whole-Responsibility Compression:** Baseline 221 canonical lines (599 physical lines, 112 statements) compressed to 136 canonical lines (174 physical lines, 81 statements). Demonstrates **−38.46% canonical lines** (85 lines removed) and **−27.68% statements** (31 statements removed), meeting both the $\ge 20\%$ line / $\ge 10\%$ statement relative bar and the $\ge 50$ lines / $\ge 10$ statements absolute usefulness bar.
- **Runtime Probe Protocol (`java-junit-v1`):** Verified line-delimited JSON IPC event frames (`start`, `state`, `complete`), record-count tracking, and UTF-8 encoded string output sequences across multiple CSV format profiles (`DEFAULT`, `RFC4180`, `EXCEL`, `TDF`, `POSTGRESQL_CSV`).
- **Finite-Domain Equivalence:** 100 Cartesian test cases evaluated across discrete formats and field data structures with **0 behavioral divergences**.
- **Oracle Sensitivity:** **100% kill rate** (4/4 killed) on mandatory adverse controls (`JAVA_ADV_1_State_Mutation_Omission`, `JAVA_ADV_2_Lock_Cleanup_Omission`, `JAVA_ADV_3_Delimiter_Sequencing_Inversion`, `JAVA_ADV_4_Dropped_Stream_Flush`).
- **Specimen Regression:** Full Maven surefire execution of `org.apache.commons.csv.CSVPrinterTest` passed with 168 tests run, 0 failures, 0 errors, 7 skipped.
- **Apparatus Budget:** Native Rust complexity budgets passed 10/10; apparatus ceiling held at 7,173 canonical lines; probe adapter overhead bounded at 212 canonical Java lines.

## Hypothesis and falsification

- **Hypothesis:** A supported `java-junit-v1` runtime probe can execute real Java target semantics and observe state/stream events over the shared native contract, enabling whole-responsibility compression ($\ge 20\%$ canonical lines, $\ge 10\%$ statements, $\ge 50$ lines removed) without semantic drift, oracle blindness, or test regressions.
- **Falsification Criteria:**
  1. *Admission failure:* Missing specimen, unverified Java/Maven runtime, or untracked dependencies.
  2. *Compression shortfall:* Line reduction $< 20\%$ or statement reduction $< 10\%$, or $< 50$ lines / $< 10$ statements removed from whole `CSVPrinter` responsibility.
  3. *Behavioral divergence:* Any mismatch (> 0) on Cartesian finite domain exploration between baseline and candidate.
  4. *Oracle insensitivity:* Any survived mutant (< 100% kill rate) on mandatory adverse controls (state mutation omission, lock cleanup omission, delimiter sequencing inversion, dropped stream flush).
  5. *Regression failure:* Any build or test failure in `mvn test -Dtest=CSVPrinterTest`.
  6. *Apparatus overage:* Total apparatus exceeding the R41 7,173 canonical line ceiling or failing complexity budgets.
- **Falsification Outcome:** None observed. All empirical checks passed strictly.

## Frozen inputs and method

- **Repository HEAD:** Commit `31ac951` (clean working tree except dirty working tree for new specs/research/docs; 4 preserved owner WIP files strictly untouched).
- **Specimen Repository:** `apache/commons-csv` at `C:/Users/mariu/Source/commons-csv` (commit `2bd66bf181d44ffc92806f585d3955e915666f7f`, clean working tree).
- **Pinned Versions:**
  - JDK: `Microsoft OpenJDK 21.0.12.1+1-LTS`
  - Build Tool: `Apache Maven 3.9.9` (`D:/BuildArtifacts/tools/apache-maven-3.9.9/bin/mvn.cmd`)
  - Testing Engine: `JUnit Jupiter 5.12.2 / JUnit Platform 1.14.4`
- **Specimen Scope:**
  - Target: `src/main/java/org/apache/commons/csv/CSVPrinter.java`
  - Tests: `src/test/java/org/apache/commons/csv/CSVPrinterTest.java` (168 tests)
- **Harness:** [`.kiro/specs/java-deconvolution-unification/run_gate.py`](../../../.kiro/specs/java-deconvolution-unification/run_gate.py).
- **Runtime Probe Implementation:**
  - `native/adapters/java/ProbeMain.java` (102 canonical lines)
  - `native/adapters/java/candidate_CSVPrinter.java` (136 canonical lines)

## Evidence and results

Receipt summary from [`prototype-gate-report.json`](../../../.kiro/specs/java-deconvolution-unification/results/prototype-gate-report.json):
```json
{
  "gate": "prototype-feasibility-gate",
  "profile": "java-junit-v1",
  "disposition": "PROCEED",
  "duration_seconds": 25.12,
  "timestamp": "2026-11-09T18:57:15+0100",
  "admission": {
    "specimen_repo": "apache/commons-csv",
    "specimen_commit": "2bd66bf181d44ffc92806f585d3955e915666f7f",
    "specimen_license": "Apache License 2.0",
    "target_file": "src\\main\\java\\org\\apache\\commons\\csv\\CSVPrinter.java",
    "java_version": "openjdk version \"21.0.12.1\" 2026-08-18 LTS",
    "maven_version": "Apache Maven 3.9.9",
    "junit_engine": "JUnit Jupiter 5.12.2 / JUnit Platform 1.14.4",
    "status": "admitted"
  },
  "compression": {
    "baseline": {
      "physical_lines": 599,
      "canonical_lines": 221,
      "statements": 112
    },
    "candidate": {
      "physical_lines": 174,
      "canonical_lines": 136,
      "statements": 81
    },
    "reduction": {
      "canonical_lines_removed": 85,
      "canonical_lines_pct": 38.46,
      "statements_removed": 31,
      "statements_pct": 27.68
    },
    "gate_passed": true
  },
  "equivalence": {
    "total_cases_evaluated": 100,
    "candidate_cases_completed": 100,
    "divergences_count": 0,
    "protocol_conformance": true,
    "zero_divergence_passed": true
  },
  "adverse_controls": {
    "total_controls": 4,
    "killed_controls": 4,
    "kill_rate_pct": 100.0,
    "controls": [
      {
        "control_id": "JAVA_ADV_1",
        "name": "State Mutation Omission (newRecord flag)",
        "killed": true
      },
      {
        "control_id": "JAVA_ADV_2",
        "name": "Lock Cleanup Omission",
        "killed": true
      },
      {
        "control_id": "JAVA_ADV_3",
        "name": "Delimiter Sequencing Inversion",
        "killed": true
      },
      {
        "control_id": "JAVA_ADV_4",
        "name": "Dropped Stream Flush in Close",
        "killed": true
      }
    ],
    "oracle_sensitivity_passed": true
  },
  "regression": {
    "test_command": "mvn.cmd test -Dtest=CSVPrinterTest",
    "exit_code": 0,
    "tests_run": 168,
    "failures": 0,
    "errors": 0,
    "skipped": 7,
    "build_success": true
  },
  "apparatus": {
    "r41_ceiling": 7173,
    "total_adapter_probe_lines": 212,
    "budgets_tests_passed": 10,
    "budget_passed": true
  }
}
```

## Traceability to Spec and Roadmap

- Governs: Stage 3 (`java-deconvolution-unification`) under [.kiro/steering/roadmap.md](../../../.kiro/steering/roadmap.md).
- Closes open admission requirement in [.kiro/specs/java-deconvolution-unification/prototype-gate.md](../../../.kiro/specs/java-deconvolution-unification/prototype-gate.md).
- Unblocks implementation task generation in [.kiro/specs/java-deconvolution-unification/tasks.md](../../../.kiro/specs/java-deconvolution-unification/tasks.md).

## Amendment, 2026-11-11: what the adverse-control figure was

Reading the gate script (`.kiro/specs/java-deconvolution-unification/run_gate.py`) shows that the four controls were text replacements in the candidate, compiled and compared with the baseline through a probe; a mutant that failed to compile also counted as killed. The receipt records `JAVA_ADV_2` (removed `lock.unlock()`) as "killed at compile time": the replacement left a syntax error. The driver's mutant for the same change compiles, and the project's `CSVPrinterTest` does not notice it: it survives. Of the other three, two have a driver counterpart and are killed; one (a changed argument) has none. Whole file with the project's tests: candidate 78 of 89 killed, original 78 of 90. The gate also copied the candidate over a file in the upstream checkout and copied the original back; the driver never writes to a source tree. The "4 of 4 killed" of this entry should not be cited as evidence of test sensitivity. Details: [R75](../2026-11-11/01-specimen-controls-rerun-through-driver.md).

