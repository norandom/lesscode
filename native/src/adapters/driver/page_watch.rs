//! The run page: where a run says what happens in it
//! (`.kiro/specs/run-dashboard`, task 2.1). The batch runner tells; whoever
//! wants to show the run listens. Without a listener, telling does nothing,
//! and a run is what it was before.
//!
//! One listener for the process, set for the length of a run. The driver's
//! own tests run one test per process, so two runs never share it.

use super::runner::{Job, MutantResult, WorkerReport};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

/// A sign of life at least this often while the tool lives, also while a
/// copy builds and nothing else changes.
pub const BEAT: Duration = Duration::from_secs(5);

/// Calls `beat` every `every` until it is dropped. Dropping waits for the
/// last call to end, so nothing is told after that.
pub struct Heart {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Heart {
    pub fn start(every: Duration, beat: impl Fn() + Send + 'static) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let asked = Arc::clone(&stop);
        // Short naps, so that dropping does not wait a whole interval.
        let nap = every.min(Duration::from_millis(50));
        let thread = spawn(move || {
            let mut waited = Duration::ZERO;
            while !asked.load(Ordering::Acquire) {
                sleep(nap);
                waited += nap;
                if waited >= every && !asked.load(Ordering::Acquire) {
                    beat();
                    waited = Duration::ZERO;
                }
            }
        });
        Self {
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for Heart {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub enum Event<'a> {
    /// The faults that will be run or found from an earlier run.
    Planned(&'a [Job]),
    /// The unchanged tests ended in a copy; `stopped` says why it drops out.
    Unchanged(&'a WorkerReport),
    /// A fault was judged, or its outcome was found from an earlier run.
    Classed(&'a MutantResult),
}

pub trait Watch: Send + Sync {
    fn tell(&self, event: Event<'_>);
}

static LISTENER: RwLock<Option<Arc<dyn Watch>>> = RwLock::new(None);

/// Tell the listener, if there is one.
pub fn tell(event: Event<'_>) {
    let listener = LISTENER
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(watch) = listener.as_ref() {
        watch.tell(event);
    }
}

/// A listener is set while this lives.
pub struct Watching(());

impl Watching {
    pub fn begin(watch: Arc<dyn Watch>) -> Self {
        *LISTENER
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(watch);
        Self(())
    }
}

impl Drop for Watching {
    fn drop(&mut self) {
        *LISTENER
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }
}
