// The lease part of src-tauri/src/commands/native_clipboard.rs in drift-rdp, before the change.
// The rest of the file differs only in how `guard` is called.
use crate::rdp::{
    clipboard::ClipboardContext, clipboard_payload::ClipboardPayload, session::SessionManager,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
use tauri::{Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Default)]
pub struct NativeClipboardState {
    owner: Mutex<Option<Arc<Lease>>>,
    operation: Mutex<()>,
    claim_sequence: AtomicU64,
}
struct Lease {
    id: String,
    session: String,
    epoch: String,
    valid: AtomicBool,
}
impl NativeClipboardState {
    fn reserve(&self) -> u64 {
        let mut owner = self.owner.lock().unwrap();
        let ticket = self.claim_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(old) = owner.take() {
            old.valid.store(false, Ordering::SeqCst);
        }
        ticket
    }
    fn claim(&self, ticket: u64, session: String, epoch: String) -> Result<String, String> {
        let mut owner = self.owner.lock().unwrap();
        if self.claim_sequence.load(Ordering::SeqCst) != ticket {
            return Err("Clipboard owner changed during setup.".into());
        }
        if let Some(old) = owner.take() {
            old.valid.store(false, Ordering::SeqCst);
        }
        let id = uuid::Uuid::new_v4().to_string();
        *owner = Some(Arc::new(Lease {
            id: id.clone(),
            session,
            epoch,
            valid: AtomicBool::new(true),
        }));
        Ok(id)
    }
    fn lease(&self, id: &str) -> Result<Arc<Lease>, String> {
        self.owner
            .lock()
            .unwrap()
            .as_ref()
            .filter(|owner| owner.id == id)
            .cloned()
            .ok_or_else(|| "Clipboard owner changed. Retry in the active session.".into())
    }
    fn release(&self, id: &str) {
        let mut owner = self.owner.lock().unwrap();
        if owner.as_ref().is_some_and(|owner| owner.id == id)
            && let Some(old) = owner.take()
        {
            old.valid.store(false, Ordering::SeqCst);
        }
    }
}
fn guard(lease: &Lease, deadline: Instant) -> Result<(), String> {
    if !lease.valid.load(Ordering::SeqCst) || Instant::now() >= deadline {
        Err("Clipboard operation expired or its session changed.".into())
    } else {
        Ok(())
    }
}
