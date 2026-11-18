// The lease part of src-tauri/src/commands/native_clipboard.rs in drift-rdp, after the change.
// The rest of the file differs only in how `guard` is called.
use crate::rdp::{
    clipboard::ClipboardContext, clipboard_payload::ClipboardPayload, session::SessionManager,
};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Default)]
pub struct NativeClipboardState {
    owner: Mutex<Owner>,
    operation: Mutex<()>,
}
/// Who owns the native clipboard. Every change of owner is a method of this one value.
#[derive(Default)]
struct Owner {
    tickets: u64,
    lease: Option<Lease>,
}
#[derive(Clone)]
struct Lease {
    id: String,
    session: String,
    epoch: String,
}
impl Owner {
    /// Ends the current lease and returns the ticket the next claim must show.
    fn reserve(&mut self) -> u64 {
        self.tickets += 1;
        self.lease = None;
        self.tickets
    }
    fn claim(&mut self, ticket: u64, session: String, epoch: String) -> Result<String, String> {
        if self.tickets != ticket {
            return Err("Clipboard owner changed during setup.".into());
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.lease = Some(Lease {
            id: id.clone(),
            session,
            epoch,
        });
        Ok(id)
    }
    /// The lease with this id, while it is still the owner.
    fn lease(&self, id: &str) -> Option<&Lease> {
        self.lease.as_ref().filter(|lease| lease.id == id)
    }
    fn release(&mut self, id: &str) {
        if self.lease(id).is_some() {
            self.lease = None;
        }
    }
}
impl NativeClipboardState {
    fn reserve(&self) -> u64 {
        self.owner.lock().unwrap().reserve()
    }
    fn claim(&self, ticket: u64, session: String, epoch: String) -> Result<String, String> {
        self.owner.lock().unwrap().claim(ticket, session, epoch)
    }
    fn lease(&self, id: &str) -> Result<Lease, String> {
        self.owner
            .lock()
            .unwrap()
            .lease(id)
            .cloned()
            .ok_or_else(|| "Clipboard owner changed. Retry in the active session.".into())
    }
    fn release(&self, id: &str) {
        self.owner.lock().unwrap().release(id)
    }
    /// A lease is good while it is still the owner and its time is not up.
    fn guard(&self, lease: &Lease, deadline: Instant) -> Result<(), String> {
        if self.owner.lock().unwrap().lease(&lease.id).is_none() || Instant::now() >= deadline {
            Err("Clipboard operation expired or its session changed.".into())
        } else {
            Ok(())
        }
    }
}
