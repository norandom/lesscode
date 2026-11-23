//! The run page: a listener on this machine's own address for the length
//! of a run (`.kiro/specs/run-dashboard`, task 3.1). It answers three
//! things: the page, the stream of snapshots, and nothing for anything
//! else. The address carries a secret; a request without it gets no content.
//!
//! Standard library only. Nothing here can change what a run returns: a
//! listener that cannot start is an `Err` the caller reports and goes on.

use std::collections::hash_map::RandomState;
use std::hash::BuildHasher;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

/// This machine only, on a port the system chooses.
pub const LOCAL: &str = "127.0.0.1:0";

/// How often a stream looks for a new snapshot, and the listener for a
/// new request.
const LOOK: Duration = Duration::from_millis(50);

/// How long a request may take to arrive, and a send to be taken.
const PATIENCE: Duration = Duration::from_secs(5);

/// The number of the newest snapshot, and the snapshot as one line of text.
pub type Latest = dyn Fn() -> (u64, String) + Send + Sync;

struct Shared {
    secret: String,
    page: String,
    latest: Arc<Latest>,
    stop: AtomicBool,
}

/// The listener lives while this does. Dropping it sends the newest
/// snapshot down every open stream, closes them and waits for that.
pub struct Served {
    address: String,
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

/// Sixteen bytes nobody else on the machine can guess. The standard
/// library keys each `RandomState` from the system's randomness.
fn secret() -> String {
    let part = || RandomState::new().hash_one(std::process::id());
    format!("{:016x}{:016x}", part(), part())
}

impl Served {
    /// Listens on `listen`, which must be an address of this machine.
    /// `page` gets the path of the stream and gives the page's text.
    pub fn start(
        listen: &str,
        page: impl FnOnce(&str) -> String,
        latest: Arc<Latest>,
    ) -> Result<Self, String> {
        let listener = TcpListener::bind(listen).map_err(|e| format!("{listen}: {e}"))?;
        let local = listener.local_addr().map_err(|e| e.to_string())?;
        if !local.ip().is_loopback() {
            return Err(format!("{local} is not an address of this machine alone"));
        }
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let secret = secret();
        let shared = Arc::new(Shared {
            page: page(&format!("/{secret}/events")),
            secret,
            latest,
            stop: AtomicBool::new(false),
        });
        let address = format!("http://{local}/{}/", shared.secret);
        let theirs = Arc::clone(&shared);
        let thread = spawn(move || accept(&listener, &theirs));
        Ok(Self {
            address,
            shared,
            thread: Some(thread),
        })
    }

    /// Where a browser finds the page.
    pub fn address(&self) -> &str {
        &self.address
    }
}

impl Drop for Served {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn accept(listener: &TcpListener, shared: &Arc<Shared>) {
    let mut open: Vec<JoinHandle<()>> = Vec::new();
    while !shared.stop.load(Ordering::Acquire) {
        match listener.accept() {
            Ok((stream, _)) => {
                let theirs = Arc::clone(shared);
                open.push(spawn(move || answer(stream, &theirs)));
            }
            Err(_) => sleep(LOOK),
        }
        open.retain(|thread| !thread.is_finished());
    }
    for thread in open {
        let _ = thread.join();
    }
}

/// The path of a request for reading, once its head has arrived whole.
fn asked(stream: &TcpStream) -> Option<String> {
    let mut head = BufReader::new(stream.take(16 * 1024));
    let mut first = String::new();
    head.read_line(&mut first).ok()?;
    let mut line = String::new();
    while head.read_line(&mut line).ok()? > 2 {
        line.clear();
    }
    let mut words = first.split_whitespace();
    (words.next() == Some("GET")).then(|| words.next().map(str::to_owned))?
}

fn answer(mut stream: TcpStream, shared: &Shared) {
    let ready = stream
        .set_nonblocking(false)
        .and_then(|()| stream.set_read_timeout(Some(PATIENCE)))
        .and_then(|()| stream.set_write_timeout(Some(PATIENCE)));
    let (Ok(()), Some(path)) = (ready, asked(&stream)) else {
        return;
    };
    let within = path
        .strip_prefix('/')
        .and_then(|rest| rest.strip_prefix(shared.secret.as_str()));
    match within {
        Some("/") => {
            let body = &shared.page;
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
        Some("/events") => follow(&mut stream, shared),
        // No secret, no answer worth reading.
        _ => {
            let _ = stream.write_all(
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            );
        }
    }
}

/// One long answer that carries every new snapshot, until the page goes
/// away or the run ends. The last snapshot is sent before it closes.
fn follow(stream: &mut TcpStream, shared: &Shared) {
    let head = b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n";
    if stream.write_all(head).is_err() {
        return;
    }
    let mut sent = None;
    loop {
        let ending = shared.stop.load(Ordering::Acquire);
        let (sequence, text) = (shared.latest)();
        if sent != Some(sequence) {
            let told = write!(stream, "data: {text}\n\n").and_then(|()| stream.flush());
            if told.is_err() {
                return;
            }
            sent = Some(sequence);
        }
        if ending {
            return;
        }
        sleep(LOOK);
    }
}
