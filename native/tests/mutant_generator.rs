//! Test suite for adverse mutant generator and sensitivity accounting.
//!
//! Governed by:
//! - REQ-NAT-4.3 through REQ-NAT-4.5
//! - `.kiro/specs/native-deconvolution-closure/tasks.md`

use lesscode_core::adapter_traits::*;
use lesscode_core::mutant_generator::*;

#[test]
fn standard_mandatory_controls_inventory() {
    let controls = NativeMutantGenerator::standard_mandatory_controls();
    assert_eq!(controls.len(), 4);
    let ids: Vec<&str> = controls.iter().map(|c| c.id.as_str()).collect();
    assert!(ids.contains(&"E3_AuditedAbsence"));
    assert!(ids.contains(&"E4_F1_Silent_None"));
    assert!(ids.contains(&"E4_F3_Buffer_Drop"));
    assert!(ids.contains(&"E4_F7_Error_Order"));
}

#[test]
fn evaluate_controls_all_killed_reports_pass() {
    let controls = NativeMutantGenerator::standard_mandatory_controls();
    let batch = NativeMutantGenerator::evaluate_controls(&controls, |ctrl| {
        Ok(Some(format!("Killed with witness for {}", ctrl.id)))
    });

    assert!(batch.all_killed);
    assert_eq!(batch.killed_count, 4);
    assert_eq!(batch.denominator, 4);
    for res in &batch.results {
        assert!(matches!(res.outcome, MutantOutcome::Killed { .. }));
    }
}

#[test]
fn evaluate_controls_survived_fails_gate() {
    let controls = NativeMutantGenerator::standard_mandatory_controls();
    let batch = NativeMutantGenerator::evaluate_controls(&controls, |ctrl| {
        if ctrl.id == "E4_F3_Buffer_Drop" {
            Ok(None) // Survived!
        } else {
            Ok(Some(format!("Killed {}", ctrl.id)))
        }
    });

    assert!(!batch.all_killed);
    assert_eq!(batch.killed_count, 3);
    assert_eq!(batch.denominator, 4);

    let survived = batch
        .results
        .iter()
        .find(|r| r.spec.id == "E4_F3_Buffer_Drop")
        .unwrap();
    assert_eq!(survived.outcome, MutantOutcome::Survived);
}
