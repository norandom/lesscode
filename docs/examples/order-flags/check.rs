// Runs every sequence of up to six events through both versions and compares what a caller sees.
// rustc --edition 2024 --test check.rs
mod before {
    include!("before.rs");
}
mod after {
    include!("after.rs");
}

#[test]
fn both_versions_answer_alike() {
    let mut compared = 0;
    for length in 0..=6u32 {
        for code in 0..3usize.pow(length) {
            let (mut old, mut new) = (before::Order::default(), after::Order::default());
            let mut rest = code;
            for _ in 0..length {
                let (was, now) = match rest % 3 {
                    0 => (old.pay(), new.apply(after::Event::Pay)),
                    1 => (old.ship(), new.apply(after::Event::Ship)),
                    _ => (old.cancel(), new.apply(after::Event::Cancel)),
                };
                rest /= 3;
                assert_eq!(was.err(), now.err());
                if let Ok(next) = now {
                    new = next;
                }
                assert_eq!(old.label(), new.label());
                compared += 1;
            }
        }
    }
    assert_eq!(compared, 6 * 729 + 5 * 243 + 4 * 81 + 3 * 27 + 2 * 9 + 3);
}
