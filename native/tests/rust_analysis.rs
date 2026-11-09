//! Integration tests for Rust static effect analyzer and frame validator.

use lesscode_core::adapter_traits::{AnalysisRequest, FrameSpec};
use lesscode_core::adapters::rust::RustEffectAnalyzer;

#[test]
fn scans_clean_pure_rust_without_gaps() {
    let analyzer = RustEffectAnalyzer::new();
    let source = r#"
pub fn add(a: i32, b: i32) -> i32 {
    let c = a + b;
    c * 2
}
"#;
    let (sites, gaps) = analyzer.scan_source("src/lib.rs", source);
    assert_eq!(sites.len(), 0);
    assert_eq!(gaps.len(), 0);
}

#[test]
fn detects_write_sites_and_enforces_frame_spec() {
    let analyzer = RustEffectAnalyzer::new();
    let source = r#"
pub fn compute() {
    let mut vec = Vec::new();
    vec.push(1);
    vec.insert(0, 2);
    let mut x = 10;
    x += 5;
}
"#;
    let (sites, gaps) = analyzer.scan_source("src/compute.rs", source);
    assert!(sites.len() >= 3);
    assert_eq!(gaps.len(), 0);
}

#[test]
fn detects_unsupported_dynamic_forms() {
    let analyzer = RustEffectAnalyzer::new();
    let source = r#"
pub fn tricky() {
    unsafe {
        let ptr: *const i32 = std::ptr::null();
    }
    let d: Box<dyn std::fmt::Debug> = Box::new(42);
    let _ = std::thread::spawn(|| {});
    let _cell = std::cell::RefCell::new(1);
}
"#;
    let (_, gaps) = analyzer.scan_source("src/tricky.rs", source);
    assert!(gaps.len() >= 4);
    assert!(gaps.iter().any(|g| g.reason.contains("Unsafe")));
    assert!(gaps.iter().any(|g| g.reason.contains("Raw pointers")));
    assert!(gaps.iter().any(|g| g.reason.contains("Dynamic trait")));
    assert!(gaps.iter().any(|g| g.reason.contains("thread")));
}

#[test]
fn analyzer_flags_unauthorized_write_sites() {
    let analyzer = RustEffectAnalyzer::new();
    let temp_dir = std::env::temp_dir().join(format!("rust-analysis-test-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("state.rs");
    std::fs::write(&file_path, "pub fn mutate() { let mut v = vec![]; v.push(1); }").unwrap();

    let request = AnalysisRequest {
        scope_name: "test".into(),
        source_paths: vec![file_path.to_str().unwrap().into()],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["authorized.rs".into()],
            permitted_shell_locations: vec![],
            failure_invariants: vec![],
        },
        effect_profile: "rust-nextest-v1".into(),
    };

    let result = lesscode_core::adapter_traits::StateFrameAnalyzer::analyze(&analyzer, &request);
    assert!(!result.is_valid);
    assert_eq!(result.violations.len(), 1);
    assert!(result.violations[0].contains("Unauthorized write site"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
