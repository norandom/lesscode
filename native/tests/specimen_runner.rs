//! Test suite for specimen runner and supervisor lifecycle.
//!
//! Governed by:
//! - REQ-NAT-5.1 through REQ-NAT-5.5
//! - `.kiro/specs/native-deconvolution-closure/tasks.md`

use lesscode_core::adapter_traits::*;
use lesscode_core::specimen_runner::*;

#[test]
fn isolated_workspace_creation() {
    let ws = create_isolated_workspace("specimen-runner-test").unwrap();
    assert!(ws.exists());
    let _ = std::fs::remove_dir_all(&ws);
}

#[test]
fn protocol_event_parsing() {
    let raw = br#"
    Some unstructured process output
    {"seq":1,"step":"init","kind":"log","payload":{"type":"string","value":"started"}}
    Interleaved log noise
    {"seq":2,"step":"eval","kind":"write","payload":{"type":"int64","value":42}}
    "#;

    let events = parse_protocol_events(raw);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].seq, 1);
    assert_eq!(events[0].kind, "log");
    assert_eq!(events[1].seq, 2);
    assert_eq!(events[1].kind, "write");
}

#[test]
fn process_supervisor_executes_and_captures() {
    let runner = NativeSpecimenRunner::new();
    let (command, args) = if cfg!(windows) {
        (
            "powershell".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                r#"Write-Output '{"seq":1,"step":"run","kind":"complete","payload":{"type":"string","value":"done"}}'"#.to_string(),
            ],
        )
    } else {
        (
            "sh".to_string(),
            vec![
                "-c".to_string(),
                r#"echo '{"seq":1,"step":"run","kind":"complete","payload":{"type":"string","value":"done"}}'"#.to_string(),
            ],
        )
    };
    let req = RunRequest {
        command,
        args,
        env: vec![],
        cwd: None,
        timeout_ms: 30_000,
        max_output_bytes: 4096,
    };

    let result = runner.run(&req);
    assert!(result.success);
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].step, "run");
}

#[test]
fn process_supervisor_terminates_on_deadline_expiration() {
    let runner = NativeSpecimenRunner::new();
    let (command, args) = if cfg!(windows) {
        (
            "powershell".to_string(),
            vec![
                "-NoProfile".to_string(),
                "-Command".to_string(),
                "Start-Sleep -Seconds 5".to_string(),
            ],
        )
    } else {
        (
            "sh".to_string(),
            vec!["-c".to_string(), "sleep 5".to_string()],
        )
    };
    let req = RunRequest {
        command,
        args,
        env: vec![],
        cwd: None,
        timeout_ms: 150, // 150ms timeout triggers deadline kill
        max_output_bytes: 4096,
    };

    let result = runner.run(&req);
    assert!(!result.success);
    assert!(result.failure_reason.is_some());
    assert!(result.failure_reason.unwrap().contains("exceeded deadline"));
}
