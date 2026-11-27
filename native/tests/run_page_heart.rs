//! A sign of life while nothing else changes (`.kiro/specs/run-dashboard`,
//! task 2.2). The interval is the design's five seconds in a run; the tests
//! use a short one and measure against it.

use lesscode_core::adapters::driver::config::RunConfig;
use lesscode_core::adapters::driver::registry::embedded;
use lesscode_core::dashboard::watch::{Heart, BEAT};
use lesscode_core::step::plan_page::PlanWatch;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

const EVERY: Duration = Duration::from_millis(100);

fn watch(name: &str) -> Arc<PlanWatch> {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("run_page_heart_{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("project/src")).unwrap();
    fs::write(
        dir.join("project/src/app.py"),
        "def one(a):\n    return a\n",
    )
    .unwrap();
    let config = json!({"language": "python", "source_root": "project", "files": ["src/app.py"], "test": ["true"]});
    fs::write(dir.join("run.json"), config.to_string()).unwrap();
    let config = RunConfig::read(&dir.join("run.json")).unwrap();
    let pack = embedded(&config.language).unwrap();
    Arc::new(PlanWatch::new(&config, pack, &dir.join("work"), 1).unwrap())
}

#[test]
fn the_interval_of_a_run_is_five_seconds() {
    assert_eq!(BEAT, Duration::from_secs(5));
}

#[test]
fn signs_of_life_come_while_nothing_else_happens_and_never_far_apart() {
    let watch = watch("alive");
    let heart = watch.heart(EVERY);
    // Look often, and keep the moment of each sign of life that is new.
    let (mut seen, mut last) = (Vec::new(), watch.snapshot().sequence);
    let began = Instant::now();
    while began.elapsed() < Duration::from_millis(1_000) {
        sleep(Duration::from_millis(10));
        let now = watch.snapshot().sequence;
        if now != last {
            seen.push(began.elapsed());
            last = now;
        }
    }
    drop(heart);
    assert!(
        seen.len() >= 5,
        "about ten in a second, at least five: {seen:?}"
    );
    let widest = seen.windows(2).map(|pair| pair[1] - pair[0]).max().unwrap();
    assert!(
        widest <= EVERY * 3,
        "never three intervals without one: {widest:?} in {seen:?}"
    );
    let shown = watch.snapshot();
    assert!(
        shown.seconds > 0.5,
        "the run's time moves with them: {}",
        shown.seconds
    );
    assert!(!shown.ended);
}

#[test]
fn none_comes_after_the_heart_is_dropped() {
    let watch = watch("still");
    let heart = watch.heart(EVERY);
    sleep(EVERY * 3);
    drop(heart);
    let after = watch.snapshot().sequence;
    sleep(EVERY * 4);
    assert_eq!(watch.snapshot().sequence, after);
}

#[test]
fn dropping_does_not_wait_for_a_whole_interval() {
    let beats = Arc::new(AtomicU64::new(0));
    let counted = Arc::clone(&beats);
    let heart = Heart::start(Duration::from_secs(60), move || {
        counted.fetch_add(1, Ordering::Relaxed);
    });
    let began = Instant::now();
    drop(heart);
    assert!(
        began.elapsed() < Duration::from_secs(2),
        "{:?}",
        began.elapsed()
    );
    assert_eq!(beats.load(Ordering::Relaxed), 0);
}
