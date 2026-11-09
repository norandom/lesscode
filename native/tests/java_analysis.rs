//! Integration tests for Java AST & effect analyzer.

use lesscode_core::adapter_traits::{AnalysisRequest, FrameSpec, StateFrameAnalyzer};
use lesscode_core::adapters::java::JavaEffectAnalyzer;

#[test]
fn scans_clean_pure_java_without_gaps() {
    let analyzer = JavaEffectAnalyzer::new();
    let source = r#"
package com.example;

public class MathHelper {
    public static int add(int a, int b) {
        return a + b;
    }
}
"#;
    let (write_sites, gaps) = analyzer.scan_source("MathHelper.java", source);
    assert!(gaps.is_empty(), "clean Java should have no gaps");
    assert!(write_sites.is_empty(), "pure function has no write sites");
}

#[test]
fn detects_unsupported_dynamic_forms() {
    let analyzer = JavaEffectAnalyzer::new();
    let source = r#"
package com.example;

public class DynamicDanger {
    public void reflection() throws Exception {
        Class<?> c = Class.forName("com.example.Hidden");
        ClassLoader loader = c.getClassLoader();
    }

    public native void nativeCall();

    public void launchThread() {
        new Thread(() -> {}).start();
    }
}
"#;
    let (_write_sites, gaps) = analyzer.scan_source("DynamicDanger.java", source);
    assert!(
        gaps.len() >= 4,
        "should detect reflection, classloader, native, and thread"
    );
    assert!(gaps.iter().any(|g| g.reason.contains("reflection")));
    assert!(gaps.iter().any(|g| g.reason.contains("class loading")));
    assert!(gaps.iter().any(|g| g.reason.contains("Native JNI")));
    assert!(gaps.iter().any(|g| g.reason.contains("thread dispatch")));
}

#[test]
fn detects_write_sites_and_enforces_frame_spec() {
    let analyzer = JavaEffectAnalyzer::new();
    let source = r#"
package com.example;

public class Counter {
    private int count = 0;

    public void increment() {
        this.count++;
        count += 1;
    }
}
"#;
    let (write_sites, gaps) = analyzer.scan_source("Counter.java", source);
    assert!(gaps.is_empty());
    assert!(!write_sites.is_empty(), "should detect mutation sites");

    let req = AnalysisRequest {
        scope_name: "counter".into(),
        source_paths: vec![],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["Counter.java".into()],
            permitted_shell_locations: vec!["shell.java".into()],
            failure_invariants: vec!["AuditLogAbsence".into()],
        },
        effect_profile: "java-junit-v1".into(),
    };
    let result = analyzer.analyze(&req);
    assert!(result.is_valid);
}

#[test]
fn analyzer_flags_unauthorized_write_sites() {
    let analyzer = JavaEffectAnalyzer::new();
    let temp_dir = std::env::temp_dir().join("java_test_scope");
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("Stateful.java");
    std::fs::write(
        &file_path,
        "public class Stateful { int x = 10; void mutate() { x += 5; } }",
    )
    .unwrap();

    let req = AnalysisRequest {
        scope_name: "stateful".into(),
        source_paths: vec![file_path.to_str().unwrap().into()],
        frame_spec: FrameSpec {
            allowed_write_sites: vec!["Other.java".into()], // file_path not allowed
            permitted_shell_locations: vec!["shell.java".into()],
            failure_invariants: vec![],
        },
        effect_profile: "java-junit-v1".into(),
    };
    let result = analyzer.analyze(&req);
    assert!(!result.is_valid);
    assert!(!result.violations.is_empty());
    let _ = std::fs::remove_dir_all(&temp_dir);
}
