//! The run page: where a run says what happens in it
//! (`.kiro/specs/run-dashboard`, task 2.1). The batch runner tells; whoever
//! wants to show the run listens. Without a listener, telling does nothing,
//! and a run is what it was before.
//!
//! One listener for the process, set for the length of a run. The driver's
//! own tests run one test per process, so two runs never share it.

use super::runner::{Job, MutantResult, WorkerReport};
use std::sync::{Arc, RwLock};

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
