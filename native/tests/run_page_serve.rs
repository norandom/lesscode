//! The listener of the run page, asked with a plain client
//! (`.kiro/specs/run-dashboard`, task 3.1).

use lesscode_core::adapters::driver::page_serve::{Served, LOCAL};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PAGE: &str = "<p>the page</p>";

/// A listener whose snapshot the test sets.
fn served() -> (Served, Arc<Mutex<(u64, String)>>) {
    let state = Arc::new(Mutex::new((1, String::from("{\"n\":1}"))));
    let read = Arc::clone(&state);
    let latest = Arc::new(move || read.lock().unwrap().clone());
    let page = |events: &str| format!("{PAGE}<!-- {events} -->");
    (Served::start(LOCAL, page, latest).unwrap(), state)
}

fn set(state: &Mutex<(u64, String)>, number: u64) {
    *state.lock().unwrap() = (number, format!("{{\"n\":{number}}}"));
}

/// `http://127.0.0.1:PORT/SECRET/` as the host and the path.
fn parts(address: &str) -> (String, String) {
    let rest = address.strip_prefix("http://").unwrap();
    let (host, path) = rest.split_once('/').unwrap();
    (host.to_owned(), format!("/{path}"))
}

fn ask(host: &str, request: &str) -> TcpStream {
    let mut stream = TcpStream::connect(host).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    stream
}

/// The whole answer to a request for `path`.
fn get(host: &str, path: &str) -> String {
    let mut stream = ask(
        host,
        &format!("GET {path} HTTP/1.1\r\nHost: {host}\r\n\r\n"),
    );
    let mut answer = String::new();
    let _ = stream.read_to_string(&mut answer);
    answer
}

/// An open stream of snapshots, after its head.
fn follow(host: &str, path: &str) -> BufReader<TcpStream> {
    let request = format!("GET {path}events HTTP/1.1\r\nHost: {host}\r\n\r\n");
    let mut stream = BufReader::new(ask(host, &request));
    let mut line = String::new();
    stream.read_line(&mut line).unwrap();
    assert!(line.starts_with("HTTP/1.1 200"), "{line}");
    while line.trim_end() != "" {
        line.clear();
        stream.read_line(&mut line).unwrap();
    }
    stream
}

/// The next snapshot of a stream, or `None` when the stream has ended.
fn next(stream: &mut BufReader<TcpStream>) -> Option<String> {
    let mut line = String::new();
    loop {
        line.clear();
        if stream.read_line(&mut line).ok()? == 0 {
            return None;
        }
        if let Some(data) = line.trim_end().strip_prefix("data: ") {
            return Some(data.to_owned());
        }
    }
}

#[test]
fn the_page_and_the_stream_are_given_with_the_secret() {
    let (served, state) = served();
    let (host, path) = parts(served.address());
    assert!(host.starts_with("127.0.0.1:"), "{host}");
    assert_eq!(path.len(), 34, "a secret of 32 characters: {path}");
    let page = get(&host, &path);
    assert!(page.starts_with("HTTP/1.1 200 OK\r\n"), "{page}");
    assert!(page.contains("Cache-Control: no-store"));
    assert!(
        page.ends_with(&format!("{PAGE}<!-- {path}events -->")),
        "{page}"
    );
    let mut stream = follow(&host, &path);
    assert_eq!(next(&mut stream).as_deref(), Some("{\"n\":1}"));
    set(&state, 2);
    assert_eq!(next(&mut stream).as_deref(), Some("{\"n\":2}"));
}

#[test]
fn nothing_is_given_without_the_secret_or_with_a_wrong_one() {
    let (served, _state) = served();
    let (host, path) = parts(served.address());
    let wrong = format!("/{}/", "0".repeat(32));
    let longer = format!("{}x/", path.trim_end_matches('/'));
    let post = ask(
        &host,
        &format!("POST {path} HTTP/1.1\r\nHost: {host}\r\n\r\n"),
    );
    let mut posted = String::new();
    let _ = BufReader::new(post).read_to_string(&mut posted);
    assert!(!posted.contains(PAGE), "only reading is answered: {posted}");
    for asked in ["/", "/events", wrong.as_str(), longer.as_str(), "/../"] {
        let answer = get(&host, asked);
        assert!(answer.starts_with("HTTP/1.1 404"), "{asked}: {answer}");
        assert!(
            answer.ends_with("\r\n\r\n"),
            "no content for {asked}: {answer}"
        );
    }
    let other = served_twice_differ(&path);
    assert!(other, "two listeners do not share a secret");
}

fn served_twice_differ(first: &str) -> bool {
    let (second, _state) = served();
    parts(second.address()).1 != first
}

#[test]
fn two_streams_at_once_get_every_snapshot() {
    let (served, state) = served();
    let (host, path) = parts(served.address());
    let (mut one, mut two) = (follow(&host, &path), follow(&host, &path));
    for number in 2..=4 {
        set(&state, number);
        let expected = format!("{{\"n\":{number}}}");
        // Each stream may still owe the snapshot before; read up to this one.
        for stream in [&mut one, &mut two] {
            let mut got = next(stream).unwrap();
            while got != expected {
                got = next(stream).unwrap();
            }
        }
    }
}

#[test]
fn a_client_that_goes_away_disturbs_neither_the_run_nor_the_others() {
    let (served, state) = served();
    let (host, path) = parts(served.address());
    let gone = follow(&host, &path);
    let mut stays = follow(&host, &path);
    assert_eq!(next(&mut stays).as_deref(), Some("{\"n\":1}"));
    drop(gone);
    // A request that never finishes its head is dropped too.
    let half = ask(&host, "GET /");
    for number in 2..=6 {
        set(&state, number);
        assert_eq!(next(&mut stays), Some(format!("{{\"n\":{number}}}")));
    }
    drop(half);
    let began = Instant::now();
    drop(served);
    assert!(
        began.elapsed() < Duration::from_secs(8),
        "{:?}",
        began.elapsed()
    );
}

#[test]
fn the_last_snapshot_is_sent_and_the_listener_ends_with_the_run() {
    let (served, state) = served();
    let (host, path) = parts(served.address());
    let mut stream = follow(&host, &path);
    assert_eq!(next(&mut stream).as_deref(), Some("{\"n\":1}"));
    set(&state, 9);
    drop(served);
    let mut rest = Vec::new();
    while let Some(snapshot) = next(&mut stream) {
        rest.push(snapshot);
    }
    assert_eq!(rest.last().map(String::as_str), Some("{\"n\":9}"));
    assert!(
        TcpStream::connect(&host).is_err() || get(&host, &path).is_empty(),
        "nothing is served after the run"
    );
}

#[test]
fn a_listener_that_cannot_start_is_an_error_and_nothing_more() {
    let taken = TcpListener::bind(LOCAL).unwrap();
    let address = taken.local_addr().unwrap().to_string();
    let latest = Arc::new(|| (0, String::new()));
    let failed = Served::start(&address, |_| String::new(), latest.clone());
    assert!(failed.is_err_and(|why| why.contains(&address)));
    // Every address but this machine's own is refused.
    let Err(why) = Served::start("0.0.0.0:0", |_| String::new(), latest) else {
        panic!("an address others can reach was taken");
    };
    assert!(why.contains("this machine"), "{why}");
}
