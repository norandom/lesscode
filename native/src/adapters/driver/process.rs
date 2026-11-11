//! Running a project's command with a time limit. Output is read while the
//! command runs, and at the limit, or when the driver is told to stop, the
//! whole process tree is killed.

use super::interrupt;
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// How much of each output stream is kept.
const KEEP_BYTES: usize = 1 << 20;

#[derive(Debug, Clone)]
pub struct Finished {
    /// None when the command could not be started or was killed.
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub seconds: f64,
    /// Standard output followed by standard error, each cut at one mebibyte.
    pub output: String,
    /// Why the command could not be started, if it could not.
    pub launch_error: Option<String>,
}

impl Finished {
    pub fn passed(&self) -> bool {
        self.exit_code == Some(0) && !self.timed_out
    }
}

/// Read a stream to its end on another thread, keeping the first part.
fn drain(stream: Option<impl Read + Send + 'static>) -> mpsc::Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut kept = Vec::new();
        let mut buffer = [0u8; 8192];
        let Some(mut stream) = stream else { return };
        while let Ok(read) = stream.read(&mut buffer) {
            if read == 0 {
                break;
            }
            let room = KEEP_BYTES.saturating_sub(kept.len());
            kept.extend_from_slice(&buffer[..read.min(room)]);
        }
        let _ = sender.send(kept);
    });
    receiver
}

/// Kill the command and everything it started.
fn kill_tree(child: &mut Child) {
    let pid = child.id().to_string();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/PID", &pid, "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-KILL", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
    let _ = child.wait();
}

/// Run `command` in `cwd`; no shell is involved.
pub fn run(
    command: &[String],
    cwd: &Path,
    env: &BTreeMap<String, String>,
    limit: Duration,
) -> Finished {
    let started = Instant::now();
    let not_started = |why: String| Finished {
        exit_code: None,
        timed_out: false,
        seconds: 0.0,
        output: String::new(),
        launch_error: Some(why),
    };
    let Some((program, arguments)) = command.split_first() else {
        return not_started("the command is empty".into());
    };
    if interrupt::requested() {
        return not_started(interrupt::STOPPED.into());
    }
    let mut process = Command::new(program);
    process.args(arguments).current_dir(cwd).envs(env);
    process
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut process, 0);
    let mut child = match process.spawn() {
        Ok(child) => child,
        Err(e) => return not_started(format!("{program}: {e}")),
    };
    let (stdout, stderr) = (drain(child.stdout.take()), drain(child.stderr.take()));
    let (exit_code, timed_out) = loop {
        match child.try_wait() {
            Ok(Some(status)) => break (status.code(), false),
            Ok(None) if started.elapsed() < limit && !interrupt::requested() => {
                std::thread::sleep(Duration::from_millis(20))
            }
            _ => {
                kill_tree(&mut child);
                break (None, true);
            }
        }
    };
    // A surviving grandchild could hold a pipe open; do not wait for it forever.
    let wait = Duration::from_secs(if timed_out { 5 } else { 30 });
    let mut output = stdout.recv_timeout(wait).unwrap_or_default();
    output.extend(stderr.recv_timeout(wait).unwrap_or_default());
    Finished {
        exit_code,
        timed_out,
        seconds: started.elapsed().as_secs_f64(),
        output: String::from_utf8_lossy(&output).into_owned(),
        launch_error: None,
    }
}
