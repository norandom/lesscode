//! The run page: starting the default browser, only when the person asked
//! for it in this invocation (`.kiro/specs/run-dashboard`, task 3.2). The
//! platform's own opener is started directly, never through a shell.

use std::process::{Command, Stdio};

/// Starts a browser on an address, or says why it could not.
pub type Opener = fn(&str) -> Result<(), String>;

#[cfg(windows)]
const PROGRAM: (&str, &[&str]) = ("rundll32", &["url.dll,FileProtocolHandler"]);
#[cfg(target_os = "macos")]
const PROGRAM: (&str, &[&str]) = ("open", &[]);
#[cfg(all(unix, not(target_os = "macos")))]
const PROGRAM: (&str, &[&str]) = ("xdg-open", &[]);

/// The platform's opener on `address`. It does not wait for the browser.
pub fn open(address: &str) -> Result<(), String> {
    let (program, before) = PROGRAM;
    let mut child = Command::new(program)
        .args(before)
        .arg(address)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{program}: {e}"))?;
    // The opener hands over to the browser and ends; nobody waits on it here.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}
