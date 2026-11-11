//! `deconvolute run` stopped from outside while a mutant's test is running
//! (Requirement 5.7): the project command is killed, the planted file is
//! restored, and the run ends as void. The signal is a real one: SIGTERM on
//! Unix, Ctrl-Break on Windows.

use serde_json::{json, Value};
use std::fs;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const APP: &str = "def step(state, a):\n    if a > 3:\n        raise ValueError(\"big\")\n    state.total = a\n    return a\n";

fn toy() -> String {
    let test_binary = std::env::current_exe().unwrap();
    let profile = test_binary.parent().unwrap().parent().unwrap();
    let name = format!("toy_check{}", std::env::consts::EXE_SUFFIX);
    let path = profile.join("examples").join(name);
    assert!(path.is_file(), "not run: {} was not built", path.display());
    path.to_string_lossy().into_owned()
}

#[cfg(unix)]
fn stop(child: &Child) {
    let status = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .expect("kill is available");
    assert!(status.success());
}

#[cfg(windows)]
fn stop(child: &Child) {
    extern "system" {
        fn GenerateConsoleCtrlEvent(event: u32, group: u32) -> i32;
    }
    // SAFETY: a plain system call with two integers. Ctrl-Break (1) goes to
    // the process group the driver was started in, not to this test.
    let sent = unsafe { GenerateConsoleCtrlEvent(1, child.id()) };
    assert_ne!(sent, 0, "not run: this process has no console to send from");
}

fn start(command: &mut Command) -> Child {
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(command, 0x0000_0200); // its own group
    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("run lesscode-core binary")
}

#[test]
fn a_stopped_run_kills_the_command_restores_the_copy_and_is_void() {
    let dir = std::env::temp_dir().join(format!("lc-test-interrupt-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("project/src")).unwrap();
    fs::write(dir.join("project/src/app.py"), APP).unwrap();
    // The stand-in sleeps for a minute on any mutant; limits are far longer.
    let test = json!([
        toy(),
        "--file",
        "src/app.py",
        "--print",
        "TOY RAN",
        "--expect",
        "if a > 3:",
        "--expect",
        "state.total = a",
        "--expect",
        "raise ValueError",
        "--expect",
        "return a",
        "--sleep-if-missing-ms",
        "60000"
    ]);
    let config = json!({"language": "python", "source_root": "project", "files": ["src/app.py"],
                        "test": test, "timeout_floor_s": 200, "timeout_ceiling_s": 300});
    fs::write(dir.join("run.json"), config.to_string()).unwrap();

    let receipt_path = dir.join("receipt.json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_lesscode-core"));
    command.args(["deconvolute", "run", "--config"]);
    command
        .arg(dir.join("run.json"))
        .arg("--work")
        .arg(dir.join("work"));
    command.arg("--out").arg(&receipt_path);
    let mut child = start(&mut command);

    // Wait until a mutant is planted: the restore journal exists only then.
    let journal = dir.join("work/w1/journal.json");
    let waited = Instant::now();
    while !journal.exists() {
        assert!(
            waited.elapsed() < Duration::from_secs(30),
            "no mutant was planted"
        );
        assert!(
            child.try_wait().unwrap().is_none(),
            "the run ended before a mutant ran"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(300));
    stop(&child);

    let stopped = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if stopped.elapsed() > Duration::from_secs(20) {
            let _ = child.kill();
            panic!("the driver did not end within 20 seconds of the stop");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(status.code(), Some(3), "a stopped run is void");
    let receipt: Value = serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    let void = receipt["void"].as_str().unwrap();
    assert!(void.contains("stopped from outside"), "{void}");
    assert!(!journal.exists(), "the journal entry was not cleared");
    assert_eq!(
        fs::read_to_string(dir.join("work/w1/tree/src/app.py")).unwrap(),
        APP,
        "the copy was not restored"
    );
    assert_eq!(
        fs::read_to_string(dir.join("project/src/app.py")).unwrap(),
        APP
    );
    let classed = receipt["mutants"].as_array().unwrap();
    assert!(
        classed.iter().all(|m| m["class"] != "timeout"),
        "the interrupted mutant must not be classed: {classed:?}"
    );
    let _ = fs::remove_dir_all(&dir);
}
