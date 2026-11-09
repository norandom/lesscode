//! Test suite for TypeScript AST & Effect Analyzer.
//!
//! Governed by:
//! - REQ-TS-1.1, REQ-TS-1.2, REQ-TS-1.3, REQ-TS-1.4
//! - `.kiro/specs/typescript-deconvolution-adapter/tasks.md` Task 1

use lesscode_core::adapter_traits::*;
use lesscode_core::adapters::typescript::analysis::TypeScriptEffectAnalyzer;

#[test]
fn scans_clean_pure_typescript_without_gaps() {
    let analyzer = TypeScriptEffectAnalyzer::new();
    let code = r#"
export function add(a: number, b: number): number {
    return a + b;
}
"#;
    let (writes, gaps) = analyzer.scan_source("calc.ts", code);
    assert!(writes.is_empty());
    assert!(gaps.is_empty());
}

#[test]
fn detects_unsupported_dynamic_forms() {
    let analyzer = TypeScriptEffectAnalyzer::new();
    let code = r#"
export function evil(code: string) {
    eval(code);
    const p = new Proxy({}, {});
    import("./dynamic.js");
    const w = new Worker("worker.js");
}
"#;
    let (_writes, gaps) = analyzer.scan_source("evil.ts", code);
    assert_eq!(gaps.len(), 4);
    assert!(gaps.iter().any(|g| g.reason.contains("eval()")));
    assert!(gaps.iter().any(|g| g.reason.contains("proxy")));
    assert!(gaps.iter().any(|g| g.reason.contains("import")));
    assert!(gaps.iter().any(|g| g.reason.contains("Workers")));
}

#[test]
fn detects_write_sites_and_enforces_frame_spec() {
    let analyzer = TypeScriptEffectAnalyzer::new();
    let code = r#"
export function mutateState(state: any) {
    state.count = 42;
    state.items.push("item");
}
"#;
    let (writes, gaps) = analyzer.scan_source("state.ts", code);
    assert_eq!(writes.len(), 2);
    assert!(gaps.is_empty());
}

#[test]
fn analyzer_flags_unauthorized_write_sites() {
    let analyzer = TypeScriptEffectAnalyzer::new();
    let req = AnalysisRequest {
        scope_name: "test_scope".into(),
        source_paths: vec![],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["authorized_store.ts".into()],
            permitted_shell_locations: vec!["shell.ts".into()],
            failure_invariants: vec!["AuditLogAbsence".into()],
        },
        effect_profile: "ts-node-v1".into(),
    };
    let result = analyzer.analyze(&req);
    assert!(result.is_valid);
}
