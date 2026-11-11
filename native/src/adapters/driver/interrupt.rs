//! Being stopped from outside (Requirement 5.7): Ctrl-C, a termination
//! signal, a closing console. The handler only sets a flag. The loop that
//! waits for a project command sees the flag, kills that command's process
//! tree, and the runner restores the planted file and ends the run as void.
//!
//! This is the one place in the crate that calls the operating system
//! directly; the two functions it declares are part of every target's
//! system library, so no dependency is added.

use std::sync::atomic::{AtomicBool, Ordering};

static REQUESTED: AtomicBool = AtomicBool::new(false);

/// Whether a stop was requested since `install`.
pub fn requested() -> bool {
    REQUESTED.load(Ordering::SeqCst)
}

/// What a void run reports when it was stopped.
pub const STOPPED: &str = "the run was stopped from outside; planted files were restored";

#[cfg(windows)]
mod system {
    use super::REQUESTED;
    use std::sync::atomic::Ordering;

    unsafe extern "system" {
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(u32) -> i32>,
            add: i32,
        ) -> i32;
    }

    /// Ctrl-C and Ctrl-Break are 0 and 1; closing, logoff and shutdown follow.
    unsafe extern "system" fn handler(event: u32) -> i32 {
        REQUESTED.store(true, Ordering::SeqCst);
        if event >= 2 {
            // The process is ended when this returns; leave the waiting
            // loops time to kill what they started.
            std::thread::sleep(std::time::Duration::from_secs(4));
        }
        1
    }

    pub fn install() {
        // SAFETY: the handler is a plain function that lives as long as the
        // program and only stores to an atomic (and sleeps on its own thread).
        unsafe {
            SetConsoleCtrlHandler(Some(handler), 1);
        }
    }
}

#[cfg(unix)]
mod system {
    use super::REQUESTED;
    use std::sync::atomic::Ordering;

    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;

    unsafe extern "C" {
        fn signal(signum: i32, handler: extern "C" fn(i32)) -> usize;
    }

    extern "C" fn handler(_signal: i32) {
        // Storing to an atomic is all a signal handler may safely do here.
        REQUESTED.store(true, Ordering::SeqCst);
    }

    pub fn install() {
        // SAFETY: `signal` is given a function that lives as long as the
        // program and is async-signal-safe.
        unsafe {
            signal(SIGINT, handler);
            signal(SIGTERM, handler);
        }
    }
}

/// Start listening. Called once, by `deconvolute run`.
pub fn install() {
    system::install();
}
