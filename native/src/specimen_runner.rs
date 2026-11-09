//! Counted specimen trace runner and process supervisor.
//!
//! Governed by:
//! - REQ-NAT-5.1 through REQ-NAT-5.5
//! - `.kiro/specs/native-deconvolution-closure/design.md`

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::adapter_traits::{RunEvent, RunRequest, RunResult, SpecimenTraceRunner};
pub use crate::receipt::sha256_hex;

/// SHA-256 digest hex calculation.
pub fn sha256_hex_bytes(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// Helper to prepare an isolated scratch directory.
pub fn create_isolated_workspace(prefix: &str) -> Result<PathBuf, String> {
    let base = Path::new("D:/BuildArtifacts/lesscode");
    let target_dir = if base.exists() {
        base.join(format!("{prefix}-{}", std::process::id()))
    } else {
        std::env::temp_dir().join(format!("{prefix}-{}", std::process::id()))
    };

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Cannot create workspace {}: {e}", target_dir.display()))?;
    Ok(target_dir)
}

/// Parses line-delimited JSON events from protocol stdout bytes.
pub fn parse_protocol_events(raw_bytes: &[u8]) -> Vec<RunEvent> {
    let mut events = Vec::new();
    let text = String::from_utf8_lossy(raw_bytes);

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(ev) = serde_json::from_str::<RunEvent>(trimmed) {
                events.push(ev);
            }
        }
    }

    events
}

fn spawn_process(request: &RunRequest) -> Result<std::process::Child, String> {
    let mut cmd = Command::new(&request.command);
    cmd.args(&request.args);
    for (k, v) in &request.env {
        cmd.env(k, v);
    }
    if let Some(ref cwd) = request.cwd {
        cmd.current_dir(cwd);
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd.spawn().map_err(|e| format!("Failed to spawn process: {e}"))
}

fn collect_output(
    mut child: std::process::Child,
    status: std::process::ExitStatus,
    max_output_bytes: usize,
) -> RunResult {
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_end(&mut stdout_bytes);
    }
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_end(&mut stderr_bytes);
    }

    if stdout_bytes.len() > max_output_bytes {
        stdout_bytes.truncate(max_output_bytes);
    }

    let events = parse_protocol_events(&stdout_bytes);
    let exit_code = status.code().unwrap_or(-1);

    RunResult {
        exit_code,
        events,
        stdout_bytes,
        stderr_bytes,
        success: status.success(),
        failure_reason: None,
    }
}

/// Native specimen trace runner implementing SpecimenTraceRunner trait.
#[derive(Debug, Default, Clone)]
pub struct NativeSpecimenRunner;

impl NativeSpecimenRunner {
    pub fn new() -> Self {
        Self
    }

    fn run_process(&self, request: &RunRequest) -> RunResult {
        let start = Instant::now();
        let timeout = Duration::from_millis(request.timeout_ms);

        let mut child = match spawn_process(request) {
            Ok(c) => c,
            Err(e) => {
                return RunResult {
                    exit_code: -1,
                    events: Vec::new(),
                    stdout_bytes: Vec::new(),
                    stderr_bytes: e.clone().into_bytes(),
                    success: false,
                    failure_reason: Some(e),
                };
            }
        };

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return collect_output(child, status, request.max_output_bytes);
                }
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        return RunResult {
                            exit_code: -1,
                            events: Vec::new(),
                            stdout_bytes: Vec::new(),
                            stderr_bytes: b"Process killed due to deadline expiration".to_vec(),
                            success: false,
                            failure_reason: Some(format!(
                                "Process exceeded deadline of {}ms",
                                request.timeout_ms
                            )),
                        };
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    let _ = child.kill();
                    return RunResult {
                        exit_code: -1,
                        events: Vec::new(),
                        stdout_bytes: Vec::new(),
                        stderr_bytes: e.to_string().into_bytes(),
                        success: false,
                        failure_reason: Some(format!("Error polling child process: {e}")),
                    };
                }
            }
        }
    }
}

impl SpecimenTraceRunner for NativeSpecimenRunner {
    fn run(&self, request: &RunRequest) -> RunResult {
        if request.command.is_empty() {
            return RunResult {
                exit_code: -1,
                events: Vec::new(),
                stdout_bytes: Vec::new(),
                stderr_bytes: Vec::new(),
                success: false,
                failure_reason: Some("Empty command".into()),
            };
        }

        self.run_process(request)
    }
}
