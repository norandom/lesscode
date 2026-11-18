//! Compare observable state and the Boolean returned after every event.
use super::{check, Machine, Result};
use lesscode_core::{adapters::driver::process, receipt::sha256_hex};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path, time::Duration};

const Z3_SHA: &str = "2bffeee10ba02a510d07be9c3fdf9c22f10ccc9724dc84201c5a40196f1a1e8f";
const BOUND: usize = 4;
struct Alphabet {
    states: BTreeMap<String, usize>,
    events: Vec<String>,
}
impl Alphabet {
    fn new(a: &Machine, b: &Machine) -> Result<Self> {
        check(a.events.len() == 2, "comparison is bounded to two events")?;
        check(
            a.events.keys().eq(b.events.keys()),
            "event alphabets differ",
        )?;
        Ok(Self {
            states: a
                .states
                .union(&b.states)
                .cloned()
                .enumerate()
                .map(|(i, s)| (s, i))
                .collect(),
            events: a.events.keys().cloned().collect(),
        })
    }
    fn steps(&self, m: &Machine, arm: &str, n: usize) -> String {
        let mut s = format!(
            "(define-fun {arm}0 () Int {})\n",
            self.states[&m.initial.value]
        );
        for i in 0..n {
            let prior = format!("{arm}{i}");
            let mut accepts = Vec::new();
            let mut dest = prior.clone();
            for (j, event) in self.events.iter().enumerate() {
                let rule = &m.events[event];
                let guard = format!(
                    "(and (= e{i} {j}) (= {prior} {}))",
                    self.states[&rule.from.value]
                );
                dest = format!("(ite {guard} {} {dest})", self.states[&rule.to.value]);
                accepts.push(guard);
            }
            s.push_str(&format!(
                "(define-fun {arm}_ok{i} () Bool (or {}))\n",
                accepts.join(" ")
            ));
            s.push_str(&format!("(define-fun {arm}{} () Int {dest})\n", i + 1));
        }
        s
    }
    fn query(&self, a: &Machine, b: &Machine, n: usize) -> String {
        let mut s = String::from("(set-logic QF_LIA)\n");
        for i in 0..n {
            s.push_str(&format!(
                "(declare-const e{i} Int)\n(assert (and (<= 0 e{i}) (< e{i} {})))\n",
                self.events.len()
            ));
        }
        s += &self.steps(a, "a", n);
        s += &self.steps(b, "b", n);
        let mut diff = vec!["(distinct a0 b0)".to_string()];
        for i in 0..n {
            diff.push(format!("(distinct a_ok{i} b_ok{i})"));
            diff.push(format!("(distinct a{} b{})", i + 1, i + 1));
        }
        format!("{s}(assert (or {}))\n(check-sat)\n", diff.join(" "))
    }
    fn witness(&self, answer: &str, n: usize) -> Result<Vec<String>> {
        let (sat, values) = answer.split_once('\n').ok_or("missing model bindings")?;
        check(sat.trim() == "sat", "model unavailable")?;
        let mut rest = values
            .trim()
            .strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .ok_or("bad model")?
            .trim();
        let mut found = BTreeMap::new();
        while !rest.is_empty() {
            let (binding, tail) = rest
                .strip_prefix('(')
                .and_then(|s| s.split_once(')'))
                .ok_or("bad event binding")?;
            let words: Vec<_> = binding.split_whitespace().collect();
            let [name, digit] = words.as_slice() else {
                return Err("bad event binding".into());
            };
            let event = self
                .events
                .get(digit.parse::<usize>()?)
                .ok_or("out of domain event")?;
            check(
                found.insert(*name, event.clone()).is_none(),
                "duplicate event binding",
            )?;
            rest = tail.trim();
        }
        check(found.len() == n, "wrong witness length")?;
        (0..n)
            .map(|i| {
                found
                    .remove(format!("e{i}").as_str())
                    .ok_or_else(|| "missing event".into())
            })
            .collect()
    }
}

fn call(z3: &Path, out: &Path, label: &str, args: &[&str]) -> Result<String> {
    let argv: Vec<_> = std::iter::once(z3.to_string_lossy().into_owned())
        .chain(args.iter().map(|s| s.to_string()))
        .collect();
    let p = process::run(&argv, out, &BTreeMap::new(), Duration::from_secs(10));
    fs::write(out.join(format!("{label}.txt")), &p.output)?;
    save(
        out,
        &format!("{label}.command.json"),
        &json!({"argv":argv,"cwd":out,"exit":p.exit_code,"seconds":p.seconds,"timeout":p.timed_out,"error":p.launch_error}),
    )?;
    check(p.passed(), &format!("solver call failed: {label}"))?;
    Ok(p.output.trim().into())
}
fn save(out: &Path, name: &str, value: &impl serde::Serialize) -> Result<()> {
    Ok(fs::write(
        out.join(name),
        serde_json::to_vec_pretty(value)?,
    )?)
}
fn solve(z3: &Path, out: &Path, label: &str, smt: &str) -> Result<String> {
    let filename = format!("{label}.smt2");
    fs::write(out.join(&filename), smt)?;
    call(z3, out, label, &["-smt2", &filename])
}
fn trace(machine: &Machine, events: &[String]) -> Vec<Value> {
    let mut state = machine.initial.value.clone();
    let mut obs = vec![json!({"state":state})];
    for event in events {
        let (next, accepted) = machine.step(&state, event);
        state = next;
        obs.push(json!({"event":event,"accepted":accepted,"state":state}));
    }
    obs
}
pub fn compare(a: &Machine, b: &Machine, z3: &Path, out: &Path) -> Result<bool> {
    let alphabet = Alphabet::new(a, b)?;
    let z3 = z3.canonicalize()?;
    check(
        sha256_hex(&fs::read(&z3)?) == Z3_SHA,
        "unexpected Z3 binary",
    )?;
    fs::create_dir(out)?;
    let out = &out.canonicalize()?;
    save(out, "left.json", a)?;
    save(out, "right.json", b)?;
    let version = call(&z3, out, "version", &["-version"])?;
    let mut verdicts = Vec::new();
    let mut witness = None;
    let mut histories = vec![vec![]];
    let mut census = Vec::new();
    for n in 0..=BOUND {
        let query = alphabet.query(a, b, n);
        let verdict = solve(&z3, out, &format!("length-{n}"), &query)?;
        check(
            matches!(verdict.as_str(), "sat" | "unsat"),
            "solver returned unknown or invalid output",
        )?;
        let mut differences = 0;
        for h in &histories {
            let left = trace(a, h);
            let right = trace(b, h);
            differences += usize::from(left != right);
            census.push(json!({"events":h,"left":left,"right":right}));
        }
        check(
            (verdict == "sat") == (differences != 0),
            "solver disagrees with enumerated model traces",
        )?;
        if witness.is_none() && verdict == "sat" {
            let events = if n == 0 {
                Vec::new()
            } else {
                let names = (0..n)
                    .map(|i| format!("e{i}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let answer = solve(
                    &z3,
                    out,
                    "shortest",
                    &format!("{query}(get-value ({names}))\n"),
                )?;
                alphabet.witness(&answer, n)?
            };
            let left = trace(a, &events);
            let right = trace(b, &events);
            check(left != right, "solver witness fails model replay")?;
            witness = Some(json!({"events":events,"left":left,"right":right}));
        }
        verdicts.push(json!({"length":n,"solver":verdict,"histories":histories.len(),"different":differences}));
        if n < BOUND {
            histories = histories
                .into_iter()
                .flat_map(|h| {
                    alphabet.events.iter().map(move |event| {
                        let mut next = h.clone();
                        next.push(event.clone());
                        next
                    })
                })
                .collect();
        }
    }
    let different = witness.is_some();
    save(out, "histories.json", &census)?;
    let result = json!({"events":alphabet.events,"state_ids":alphabet.states,"z3":version,"z3_sha256":Z3_SHA,
        "histories":census.len(),"lengths":verdicts,"witness":witness});
    save(out, "result.json", &result)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(different)
}
