//! R166: which shapes of `SharedFrame` its own methods reach. Placed beside
//! `shared_frame.rs` in a copy of the project and declared there as a child
//! module, so that it reads the private fields. It runs every sequence of
//! calls up to a length on a fresh value and writes down the shape after
//! each call.

use super::*;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::atomic::Ordering;

const SIDE: u32 = 4;
const LENGTH: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Shape {
    dirty: bool,
    damage: bool,
    full: bool,
    wake: bool,
}

fn shape(frame: &SharedFrame) -> Shape {
    let inner = frame.inner.lock().unwrap();
    Shape {
        dirty: frame.dirty.load(Ordering::Acquire),
        damage: inner.damage.is_some(),
        full: inner.full,
        wake: frame.wake.get().is_some(),
    }
}

type Call = (&'static str, fn(&SharedFrame));

fn pixels() -> Vec<u8> {
    vec![7u8; (SIDE * SIDE * 4) as usize]
}

/// The whole calls.
const A: &[Call] = &[
    ("update_rect inside", |f| f.update_rect(1, 1, 2, 1, &[9u8; 8], 8)),
    ("update_rect empty", |f| f.update_rect(1, 1, 0, 0, &[], 0)),
    ("update_full", |f| f.update_full(SIDE, SIDE, &pixels())),
    ("update_full too short", |f| f.update_full(SIDE, SIDE, &[0u8; 3])),
    ("take_damage", |f| drop(f.take_damage())),
    ("take_full", |f| drop(f.take_full())),
    ("set_wake", |f| f.set_wake(Arc::new(super::super::native_layer::Wake::default()))),
];

/// The parts the whole calls are made of; all public.
const B: &[Call] = &[
    ("write a rect, no mark_dirty", |f| f.begin_write().update_rect(1, 1, 2, 1, &[9u8; 8], 8)),
    ("write full, no mark_dirty", |f| drop(f.begin_write().update_full(SIDE, SIDE, &pixels()))),
    ("mark_dirty", |f| f.mark_dirty()),
];

/// Exists in test builds only.
const C: &[Call] = &[("publish (test only)", |f| drop(f.publish()))];

struct Seen {
    /// The shortest sequence that reached each shape.
    reached: BTreeMap<Shape, Vec<&'static str>>,
    /// Shapes first reached at each length.
    new_at: Vec<usize>,
    /// Calls that took `dirty` from set to not set.
    cleared_dirty: Vec<&'static str>,
    sequences: u64,
}

fn walk(calls: &[Call]) -> Seen {
    let mut seen = Seen { reached: BTreeMap::new(), new_at: vec![0; LENGTH + 1], cleared_dirty: Vec::new(), sequences: 0 };
    seen.reached.insert(shape(&SharedFrame::new(SIDE, SIDE)), Vec::new());
    seen.new_at[0] = 1;
    let mut choice = vec![0usize; LENGTH];
    loop {
        // One sequence of full length; every prefix of it is a shorter sequence.
        let frame = SharedFrame::new(SIDE, SIDE);
        let mut before = shape(&frame);
        for (step, &at) in choice.iter().enumerate() {
            let (name, call) = calls[at];
            call(&frame);
            let after = shape(&frame);
            if before.dirty && !after.dirty && !seen.cleared_dirty.contains(&name) {
                seen.cleared_dirty.push(name);
            }
            // Keep the shortest way there, whatever order the sequences come in.
            if seen.reached.get(&after).is_none_or(|how| how.len() > step + 1) {
                seen.reached.insert(after, choice[..=step].iter().map(|&c| calls[c].0).collect());
            }
            before = after;
        }
        seen.sequences += 1;
        // The next sequence, counting in the base of the number of calls.
        let mut digit = LENGTH;
        loop {
            if digit == 0 {
                seen.new_at = vec![0; LENGTH + 1];
                for how in seen.reached.values() {
                    seen.new_at[how.len()] += 1;
                }
                return seen;
            }
            digit -= 1;
            choice[digit] += 1;
            if choice[digit] < calls.len() {
                break;
            }
            choice[digit] = 0;
        }
    }
}

fn report(label: &str, calls: &[Call], out: &mut String) {
    let seen = walk(calls);
    let yes = |b: bool| if b { "true" } else { "false" };
    let without_wake: std::collections::BTreeSet<_> = seen.reached.keys().map(|s| (s.dirty, s.damage, s.full)).collect();
    writeln!(out, "  {{\"set\": \"{label}\", \"calls\": {}, \"sequences\": {}, \"length\": {LENGTH},", calls.len(), seen.sequences).unwrap();
    writeln!(out, "   \"reached_with_wake\": {}, \"reached_without_wake\": {},", seen.reached.len(), without_wake.len()).unwrap();
    writeln!(out, "   \"new_shapes_at_length\": {:?},", seen.new_at).unwrap();
    writeln!(out, "   \"calls_that_cleared_dirty\": {:?},", seen.cleared_dirty).unwrap();
    writeln!(out, "   \"shapes\": [").unwrap();
    let last = seen.reached.len() - 1;
    for (i, (s, how)) in seen.reached.iter().enumerate() {
        writeln!(
            out,
            "    {{\"dirty\": {}, \"damage\": {}, \"full\": {}, \"wake\": {}, \"by\": {:?}}}{}",
            yes(s.dirty), yes(s.damage), yes(s.full), yes(s.wake), how, if i == last { "" } else { "," }
        )
        .unwrap();
    }
    writeln!(out, "   ]}}").unwrap();
}

#[test]
fn reached_shapes() {
    let b: Vec<Call> = A.iter().chain(B).copied().collect();
    let c: Vec<Call> = b.iter().chain(C).copied().collect();
    let mut out = String::from("[\n");
    report("A", A, &mut out);
    out.push_str("  ,\n");
    report("B", &b, &mut out);
    out.push_str("  ,\n");
    report("C", &c, &mut out);
    out.push_str("]\n");
    let path = std::env::var("LESSCODE_WALK_OUT").expect("LESSCODE_WALK_OUT names the output file");
    std::fs::write(path, out).unwrap();
}
