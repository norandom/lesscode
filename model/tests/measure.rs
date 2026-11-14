//! Requirement 10: the indicator of whether a model is good enough. The
//! verdict rule and the counting with scripted replies, the built-in task
//! set against the core's checks, and `lesscode-model measure` through the
//! binary against a local stub endpoint (task 4.3).

mod binary;
mod stub_endpoint;

use binary::{EndpointFile, Ran, run_in};
use lesscode_model::measure::{Counts, Measured, Verdict, measure, verdict};
use lesscode_model::repair::{self, REPAIRS};
use lesscode_model::taskset::{BUILT_IN, Task, TaskSet, built_in};
use lesscode_model::{CallError, CallRequest, Reply, Sent, Stop};
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use stub_endpoint::{Answer, chat_reply, ok, start};

// --- the verdict rule ---------------------------------------------------------

fn counts(trials: u32, not_run: u32, first_passed: u32, passed: u32) -> Counts {
    Counts {
        trials,
        not_run,
        first_passed,
        passed,
        ..Counts::default()
    }
}

#[test]
fn three_quarters_of_the_first_replies_are_good_enough_direct() {
    assert_eq!(
        verdict(&counts(20, 0, 15, 15), false),
        Verdict::GoodEnoughDirect
    );
    assert_eq!(
        verdict(&counts(20, 0, 20, 20), false),
        Verdict::GoodEnoughDirect
    );
    assert_ne!(
        verdict(&counts(20, 0, 14, 20), false),
        Verdict::GoodEnoughDirect
    );
}

#[test]
fn below_that_nine_tenths_within_the_repairs_are_good_enough_with_the_repair_loop() {
    assert_eq!(
        verdict(&counts(20, 0, 14, 18), false),
        Verdict::GoodEnoughWithRepair
    );
    assert_eq!(
        verdict(&counts(20, 0, 0, 20), false),
        Verdict::GoodEnoughWithRepair
    );
    assert_eq!(
        verdict(&counts(20, 0, 14, 17), false),
        Verdict::NotGoodEnoughAsIs
    );
    assert_eq!(
        verdict(&counts(20, 0, 0, 0), false),
        Verdict::NotGoodEnoughAsIs
    );
}

#[test]
fn the_quick_mode_has_its_three_outcomes() {
    assert_eq!(verdict(&counts(5, 0, 1, 5), true), Verdict::GoodEnoughSoFar);
    assert_eq!(verdict(&counts(5, 0, 4, 4), true), Verdict::Undecided);
    assert_eq!(verdict(&counts(5, 0, 0, 3), true), Verdict::Undecided);
    assert_eq!(verdict(&counts(5, 0, 2, 2), true), Verdict::NotGoodEnough);
    assert_eq!(verdict(&counts(5, 0, 0, 0), true), Verdict::NotGoodEnough);
    // One trial that did not run: not all five passed, so not "so far".
    assert_eq!(verdict(&counts(5, 1, 4, 4), true), Verdict::Undecided);
}

#[test]
fn more_than_a_fifth_of_the_trials_not_run_gives_no_verdict_and_says_why() {
    let Verdict::NoVerdict { why } = verdict(&counts(20, 5, 15, 15), false) else {
        panic!("no verdict was expected");
    };
    assert!(why.contains("5 of 20 trials did not run"), "{why}");
    assert!(matches!(
        verdict(&counts(5, 2, 3, 3), true),
        Verdict::NoVerdict { .. }
    ));
    assert!(matches!(
        verdict(&counts(20, 20, 0, 0), false),
        Verdict::NoVerdict { .. }
    ));
    // A fifth not run still gives a verdict, over the trials that ran.
    assert_eq!(
        verdict(&counts(20, 4, 12, 12), false),
        Verdict::GoodEnoughDirect
    );
    assert_eq!(
        verdict(&counts(20, 4, 11, 15), false),
        Verdict::GoodEnoughWithRepair
    );
    assert_eq!(
        verdict(&counts(20, 4, 11, 14), false),
        Verdict::NotGoodEnoughAsIs
    );
}

#[test]
fn only_the_three_verdicts_that_say_good_enough_end_with_exit_code_0() {
    let exits: Vec<u8> = [
        Verdict::GoodEnoughDirect,
        Verdict::GoodEnoughWithRepair,
        Verdict::GoodEnoughSoFar,
        Verdict::NotGoodEnoughAsIs,
        Verdict::NotGoodEnough,
        Verdict::Undecided,
        Verdict::NoVerdict { why: String::new() },
    ]
    .iter()
    .map(Verdict::exit)
    .collect();
    assert_eq!(exits, [0, 0, 0, 4, 4, 4, 3]);
}

// --- counting, with scripted replies ---------------------------------------

fn request(user: &str) -> CallRequest {
    CallRequest {
        purpose: "test".into(),
        system: String::new(),
        user: user.into(),
        max_output_tokens: 100,
        temperature: None,
        structure: None,
    }
}

fn reply(text: &str) -> Result<Reply, CallError> {
    Ok(Reply {
        text: text.into(),
        stop: Stop::Complete,
        input_tokens: 11,
        output_tokens: 7,
        reasoning_tokens: None,
        cached_input_tokens: None,
        structured: None,
        seconds: 0.5,
        waits: vec![],
        sent: Sent {
            host: "stub".into(),
            model: "stub-model".into(),
            max_output_tokens: 100,
            temperature: None,
            reasoning_level: None,
        },
    })
}

fn refused() -> Result<Reply, CallError> {
    Err(CallError::Refused {
        host: "stub".into(),
        status: 500,
        said: "broken".into(),
        retry_after: None,
    })
}

fn two_tasks() -> TaskSet {
    TaskSet {
        name: "two".into(),
        check_name: "the text is good".into(),
        tasks: vec![
            Task {
                id: "a".into(),
                request: request("task a"),
            },
            Task {
                id: "b".into(),
                request: request("task b"),
            },
        ],
    }
}

/// Runs `measure` over the two tasks with `script` as the replies in order;
/// returns the counts and the first line of every request that was made.
fn measured(script: Vec<Result<Reply, CallError>>, trials: u32) -> (Counts, Vec<String>) {
    let mut script: VecDeque<_> = script.into();
    let mut asked = Vec::new();
    let mut call = |request: &CallRequest| {
        asked.push(request.user.lines().next().unwrap_or_default().to_owned());
        script
            .pop_front()
            .expect("the script has a reply for every call")
    };
    let mut check = |_: &Task, text: &str| repair::Verdict {
        passed: text == "good",
        reasons: vec!["the text is not good".into()],
    };
    let counts = measure(&mut call, &two_tasks(), &mut check, trials, REPAIRS);
    (counts, asked)
}

#[test]
fn every_trial_goes_through_the_check_with_the_repairs_allowed_and_is_counted() {
    let (counts, asked) = measured(
        vec![
            reply("good"),
            reply("bad"),
            reply("good"),
            reply("bad"),
            reply("bad"),
            reply("bad"),
            reply("good"),
        ],
        4,
    );
    assert_eq!(
        asked,
        [
            "task a", "task b", "task b", "task a", "task a", "task a", "task b"
        ]
    );
    assert_eq!(
        counts,
        Counts {
            trials: 4,
            not_run: 0,
            first_passed: 2,
            passed: 3,
            calls: 7,
            output_tokens: 49,
            seconds: 3.5,
        }
    );
}

#[test]
fn a_trial_whose_call_failed_is_not_run_and_does_not_count_against_the_model() {
    // Ten trials: the second fails at once, the third after a rejected reply.
    let mut script = vec![reply("good"), refused(), reply("bad"), refused()];
    script.extend((0..7).map(|_| reply("good")));
    let (counts, _) = measured(script, 10);
    assert_eq!((counts.trials, counts.not_run), (10, 2));
    assert_eq!((counts.first_passed, counts.passed), (8, 8));
    assert_eq!(counts.calls, 9, "the calls that were answered");
    assert_eq!(verdict(&counts, false), Verdict::GoodEnoughDirect);
}

#[test]
fn once_no_verdict_is_possible_the_trials_left_are_not_started() {
    let (counts, asked) = measured((0..5).map(|_| refused()).collect(), 20);
    assert_eq!(asked.len(), 5, "five of twenty is more than a fifth");
    assert_eq!((counts.trials, counts.not_run, counts.calls), (20, 20, 0));
}

#[test]
fn the_report_names_task_set_check_model_and_trials_beside_the_verdict() {
    let report = Measured {
        task_set: "two".into(),
        check: "the text is good".into(),
        model: "stub-model".into(),
        host: "stub".into(),
        repairs: REPAIRS,
        quick: false,
        counts: Counts {
            trials: 20,
            not_run: 0,
            first_passed: 10,
            passed: 16,
            calls: 40,
            output_tokens: 4000,
            seconds: 80.0,
        },
    };
    assert_eq!(
        report.lines(),
        [
            "task set: two",
            "check: the text is good",
            "model: stub-model at stub",
            "trials: 20, 0 not run",
            "first reply passed: 10 of 20 (50 %)",
            "passed within 2 repairs: 16 of 20 (80 %)",
            "per accepted answer: 2.5 calls, 250 output tokens, 5.0 s",
            "verdict: not good enough as is",
            "  an optimiser around the instruction is indicated; measure its result with this same command",
        ]
    );
    let json = report.json();
    assert_eq!(json["verdict"], "not good enough as is");
    assert_eq!(
        (json["trials"].as_u64(), json["passed"].as_u64()),
        (Some(20), Some(16))
    );
    assert_eq!(json["task_set"], "two");
}

// --- the built-in set against the core's checks -------------------------------

fn answer(path: &str, content: &str, pure: &[&str], shell: &[&str]) -> String {
    json!({
        "files": [{"path": path, "content": content}],
        "structure": {"pure": pure, "shell": shell},
        "change_log": "rewritten",
    })
    .to_string()
}

const PRICES: &str = "def total_price(items):\n    return sum(item[\"price\"] * item[\"count\"] for item in items)\n";

const SHIPPING: &str = "FREE_FROM = 50\n\n\ndef _rate_for(weight):\n    return 4 if weight <= 1 else 7 if weight <= 5 else 12\n\n\ndef shipping_fee(order):\n    return 0 if order[\"total\"] >= FREE_FROM else _rate_for(order[\"weight\"])\n";

const SHIPPING_INLINED: &str = "FREE_FROM = 50\n\n\ndef shipping_fee(order):\n    weight = order[\"weight\"]\n    return 0 if order[\"total\"] >= FREE_FROM else 4 if weight <= 1 else 7 if weight <= 5 else 12\n";

const DISCOUNT: &str = "RATES = {\"gold\": (0.1, 0.2), \"silver\": (0.05, 0.1)}\n\n\ndef discount(customer, amount):\n    return amount * RATES.get(customer[\"tier\"], (0, 0))[amount > 100]\n";

const DISCOUNT_TIDIED: &str = "def discount(customer, amount):\n    if customer[\"tier\"] == \"gold\":\n        rate = 0.2 if amount > 100 else 0.1\n    elif customer[\"tier\"] == \"silver\":\n        rate = 0.1 if amount > 100 else 0.05\n    else:\n        rate = 0\n    return amount * rate\n";

const ORDERS: &str = "def order_total(lines):\n    return sum(line[\"price\"] * line[\"count\"] for line in lines)\n\n\ndef _save(session, order):\n    session.add(order)\n    session.commit()\n\n\ndef record_order(session, order):\n    order[\"total\"] = order_total(order[\"lines\"])\n    _save(session, order)\n\n\ndef record_refund(session, order, amount):\n    order[\"total\"] = order[\"total\"] - amount\n    order[\"refunded\"] = True\n    _save(session, order)\n";

const ORDERS_AS_THEY_WERE: &str = "def record_order(session, order):\n    order[\"total\"] = sum(line[\"price\"] * line[\"count\"] for line in order[\"lines\"])\n    session.add(order)\n    session.commit()\n\n\ndef record_refund(session, order, amount):\n    order[\"total\"] = order[\"total\"] - amount\n    order[\"refunded\"] = True\n    session.add(order)\n    session.commit()\n";

const ORDERS_SHELL: [&str; 3] = [
    "orders.py::_save",
    "orders.py::record_order",
    "orders.py::record_refund",
];

/// An answer that passes, for each task of the built-in set in its order.
fn good_answers() -> [String; 4] {
    [
        answer("src/prices.py", PRICES, &["prices.py::total_price"], &[]),
        answer(
            "src/shipping.py",
            SHIPPING,
            &["shipping.py::_rate_for", "shipping.py::shipping_fee"],
            &[],
        ),
        answer("src/discount.py", DISCOUNT, &["discount.py::discount"], &[]),
        answer(
            "src/orders.py",
            ORDERS,
            &["orders.py::order_total"],
            &ORDERS_SHELL,
        ),
    ]
}

#[test]
fn the_built_in_set_has_four_tasks_and_a_good_answer_to_each_passes_the_cores_checks() {
    let (set, mut core) = built_in().unwrap();
    assert_eq!(set.name, BUILT_IN);
    assert!(
        set.check_name.contains("evaluate_tier1_static"),
        "{}",
        set.check_name
    );
    let ids: Vec<&str> = set.tasks.iter().map(|task| task.id.as_str()).collect();
    assert_eq!(
        ids,
        ["1-total-price", "2-shipping-fee", "3-discount", "4-orders"]
    );
    for (task, good) in set.tasks.iter().zip(good_answers()) {
        assert!(task.request.user.contains("## Definitions"), "{}", task.id);
        let verdict = core.judge(task, &good);
        assert!(verdict.passed, "{}: {:?}", task.id, verdict.reasons);
    }
}

#[test]
fn a_careless_answer_to_a_graded_task_is_rejected_by_the_check_the_task_is_there_for() {
    let (set, mut core) = built_in().unwrap();
    let mut rejected_by = |task: usize, careless: String| {
        let verdict = core.judge(&set.tasks[task], &careless);
        assert!(!verdict.passed, "{}", set.tasks[task].id);
        verdict.reasons.join("; ")
    };

    let reason = rejected_by(
        1,
        answer(
            "src/shipping.py",
            SHIPPING_INLINED,
            &["shipping.py::shipping_fee"],
            &[],
        ),
    );
    assert!(
        reason.starts_with("surface: ") && reason.contains("_rate_for"),
        "{reason}"
    );

    let reason = rejected_by(
        2,
        answer(
            "src/discount.py",
            DISCOUNT_TIDIED,
            &["discount.py::discount"],
            &[],
        ),
    );
    assert!(
        reason.starts_with("static_targets: ") && reason.contains("written_stmts_max"),
        "{reason}"
    );

    let reason = rejected_by(
        3,
        answer(
            "src/orders.py",
            ORDERS_AS_THEY_WERE,
            &[],
            &ORDERS_SHELL[1..],
        ),
    );
    assert!(
        reason.starts_with("static_targets: ") && reason.contains("written_writes_max"),
        "{reason}"
    );

    let reason = rejected_by(
        3,
        answer(
            "src/orders.py",
            ORDERS,
            &["orders.py::order_total", "orders.py::record_order"],
            &["orders.py::_save", "orders.py::record_refund"],
        ),
    );
    assert!(
        reason.starts_with("structure: ") && reason.contains("param session"),
        "{reason}"
    );

    // Each task judged with the answer to another one: out of scope.
    let reason = rejected_by(0, good_answers()[3].clone());
    assert!(reason.contains("paths outside scope"), "{reason}");
}

// --- through the binary ------------------------------------------------------

/// A directory that is removed when the test ends.
struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "lesscode-model-measure-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Folder(path)
    }

    /// Every file below, with its size.
    fn files(&self) -> Vec<(PathBuf, u64)> {
        fn below(dir: &Path, found: &mut Vec<(PathBuf, u64)>) {
            for entry in std::fs::read_dir(dir).unwrap().map(Result::unwrap) {
                match entry.file_type().unwrap().is_dir() {
                    true => below(&entry.path(), found),
                    false => found.push((entry.path(), entry.metadata().unwrap().len())),
                }
            }
        }
        let mut found = Vec::new();
        below(&self.0, &mut found);
        found.sort();
        found
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn answers(texts: &[&str]) -> Vec<Answer> {
    texts
        .iter()
        .map(|text| ok(chat_reply(text, "stop")))
        .collect()
}

/// Runs `measure` with `args` against a stub with `script`, in an empty
/// directory; returns what it said, the requests the stub saw and the
/// files in the directory afterwards.
fn measure_with(name: &str, script: Vec<Answer>, args: &[&str]) -> (Ran, usize, usize) {
    let stub = start(script);
    let file = EndpointFile::new(name, &stub.address);
    let project = Folder::new(name);
    let mut all = vec!["measure", "--endpoint-env", file.path()];
    all.extend(args);
    let ran = run_in(Some(&project.0), &all, "");
    ran.shows_no_key();
    (ran, stub.seen().len(), project.files().len())
}

#[test]
fn the_command_reports_the_built_in_set_and_ends_with_exit_code_0_when_good_enough() {
    let good = good_answers();
    let out = Folder::new("out");
    let out_file = out.0.join("measured.json");
    let (ran, seen, files) = measure_with(
        "direct",
        answers(&[&good[0], &good[1], &good[2], "not an answer", &good[3]]),
        &["--trials", "4", "--out", out_file.to_str().unwrap()],
    );

    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    let lines: Vec<&str> = ran.out.lines().collect();
    assert_eq!(lines[0], "task set: built-in");
    assert!(
        lines[1].starts_with("check: the core's parsing and static checks"),
        "{}",
        lines[1]
    );
    assert!(
        lines[2].starts_with("model: stub-model at 127.0.0.1:"),
        "{}",
        lines[2]
    );
    assert_eq!(lines[3], "trials: 4, 0 not run");
    assert_eq!(lines[4], "first reply passed: 3 of 4 (75 %)");
    assert_eq!(lines[5], "passed within 2 repairs: 4 of 4 (100 %)");
    assert!(
        lines[6].starts_with("per accepted answer: 1.2 calls, 9 output tokens, "),
        "{}",
        lines[6]
    );
    assert_eq!(lines[7], "verdict: good enough direct");
    assert_eq!(lines.len(), 8);
    assert_eq!(seen, 5);
    assert_eq!(files, 0, "nothing is written where the command is run");

    let written: Value =
        serde_json::from_str(&std::fs::read_to_string(&out_file).unwrap()).unwrap();
    assert_eq!(written["verdict"], "good enough direct");
    assert_eq!(written["task_set"], "built-in");
    assert_eq!(written["model"], "stub-model");
    assert_eq!(
        (written["trials"].as_u64(), written["first_passed"].as_u64()),
        (Some(4), Some(3))
    );
    assert_eq!(written["calls"], 5);
}

#[test]
fn twenty_trials_are_run_unless_another_number_is_stated() {
    let good = good_answers();
    let script: Vec<&str> = (0..20).map(|trial| good[trial % 4].as_str()).collect();
    let (ran, seen, _) = measure_with("twenty", answers(&script), &[]);
    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(ran.out.contains("trials: 20, 0 not run"), "{}", ran.out);
    assert!(
        ran.out.contains("first reply passed: 20 of 20 (100 %)"),
        "{}",
        ran.out
    );
    assert_eq!(seen, 20);
}

#[test]
fn a_model_that_is_not_good_enough_as_is_ends_with_exit_code_4_and_the_optimiser_is_named() {
    let (ran, seen, files) = measure_with("as-is", answers(&["not an answer"]), &["--trials", "4"]);
    assert_eq!(ran.code, Some(4), "{}\n{}", ran.out, ran.err);
    assert!(
        ran.out.contains("passed within 2 repairs: 0 of 4 (0 %)"),
        "{}",
        ran.out
    );
    assert!(
        ran.out
            .contains("per accepted answer: no answer was accepted"),
        "{}",
        ran.out
    );
    assert!(
        ran.out.contains("verdict: not good enough as is"),
        "{}",
        ran.out
    );
    assert!(
        ran.out
            .contains("an optimiser around the instruction is indicated"),
        "{}",
        ran.out
    );
    assert_eq!(seen, 12, "three calls for each of four trials");
    assert_eq!(files, 0);
}

#[test]
fn the_quick_mode_runs_five_trials_and_has_its_own_verdicts() {
    let good = good_answers();
    let (ran, seen, _) = measure_with(
        "quick-good",
        answers(&[&good[0], &good[1], &good[2], &good[3], &good[0]]),
        &["--quick"],
    );
    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert!(
        ran.out.contains("trials: 5 (quick), 0 not run"),
        "{}",
        ran.out
    );
    assert!(
        ran.out.contains("verdict: good enough so far"),
        "{}",
        ran.out
    );
    assert_eq!(seen, 5);

    let (ran, seen, _) = measure_with("quick-bad", answers(&["not an answer"]), &["--quick"]);
    assert_eq!(ran.code, Some(4), "{}\n{}", ran.out, ran.err);
    assert!(
        ran.out.ends_with("verdict: not good enough\n"),
        "{}",
        ran.out
    );
    assert_eq!(seen, 15);

    // The first three trials pass, the last two never do.
    let mut script = vec![good[0].as_str(), good[1].as_str(), good[2].as_str()];
    script.push("not an answer");
    let (ran, _, _) = measure_with("quick-undecided", answers(&script), &["--quick"]);
    assert_eq!(ran.code, Some(4), "{}\n{}", ran.out, ran.err);
    assert!(ran.out.ends_with("verdict: undecided\n"), "{}", ran.out);
}

#[test]
fn an_endpoint_that_fails_gives_no_verdict_with_exit_code_3_and_says_why() {
    let broken = (500, "", r#"{"error": {"message": "broken"}}"#.to_owned());
    let (ran, seen, files) = measure_with("no-verdict", vec![broken], &["--quick"]);
    assert_eq!(ran.code, Some(3), "{}\n{}", ran.out, ran.err);
    assert!(
        ran.out.contains("trials: 5 (quick), 5 not run"),
        "{}",
        ran.out
    );
    assert!(ran.out.contains("verdict: no verdict"), "{}", ran.out);
    assert!(ran.out.contains("5 of 5 trials did not run"), "{}", ran.out);
    assert!(
        ran.err.contains("refused the call (status 500)"),
        "{}",
        ran.err
    );
    assert_eq!(seen, 2, "after two of five no verdict is possible");
    assert_eq!(files, 0);
}

#[test]
fn a_task_set_supplied_by_the_caller_takes_the_place_of_the_built_in_one() {
    let set = Folder::new("supplied-set");
    let task = set.0.join("halve");
    std::fs::create_dir_all(task.join("src")).unwrap();
    std::fs::write(
        task.join("src").join("halve.py"),
        "def halve(number):\n    half = number / 2\n    return half\n",
    )
    .unwrap();
    std::fs::write(task.join("public-surface.json"), r#"{"modules": {}}"#).unwrap();
    std::fs::write(
        task.join("task.json"),
        r#"{"definitions": "halve(number) is half the number.", "scope_files": ["src/halve.py"],
            "targets": {"written_stmts_max": 2}}"#,
    )
    .unwrap();
    let before = set.files();
    let halved = answer(
        "src/halve.py",
        "def halve(number):\n    return number / 2\n",
        &["halve.py::halve"],
        &[],
    );

    let (ran, seen, files) = measure_with(
        "supplied",
        answers(&[&halved]),
        &["--trials", "2", "--task-set", set.0.to_str().unwrap()],
    );
    assert_eq!(ran.code, Some(0), "{}\n{}", ran.out, ran.err);
    assert_eq!(
        ran.out.lines().next(),
        Some(format!("task set: {}", set.0.display()).as_str())
    );
    assert!(
        ran.out.contains("first reply passed: 2 of 2 (100 %)"),
        "{}",
        ran.out
    );
    assert_eq!(seen, 2);
    assert_eq!(files, 0);
    assert_eq!(set.files(), before, "the supplied set is only read");
}

#[test]
fn an_unusable_command_line_task_set_or_endpoint_file_ends_with_exit_code_2() {
    let empty = Folder::new("empty-set");
    for args in [
        &["--quick", "--trials", "7"][..],
        &["--trials", "0"][..],
        &["--trials", "many"][..],
        &["--record", "calls.jsonl"][..],
        &["--task-set", empty.0.to_str().unwrap()][..],
    ] {
        let (ran, seen, files) = measure_with("usage", answers(&["unused"]), args);
        assert_eq!(ran.code, Some(2), "{args:?}: {}", ran.err);
        assert!(
            ran.err.contains("usage: lesscode-model measure"),
            "{args:?}: {}",
            ran.err
        );
        assert_eq!((seen, files), (0, 0), "{args:?}");
    }

    let missing = std::env::temp_dir().join("lesscode-model-measure-no-such-file.env");
    let ran = run_in(
        None,
        &["measure", "--endpoint-env", missing.to_str().unwrap()],
        "",
    );
    assert_eq!(ran.code, Some(2), "{}", ran.err);
    assert!(
        ran.err.contains("lesscode-model-measure-no-such-file.env"),
        "{}",
        ran.err
    );
    assert_eq!(ran.out, "");
}
