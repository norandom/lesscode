# Example: a cell that carried one fact past a function

Source: this repository. `tools/guide-optimizer/src/adapter.rs` and `model/src/repair.rs`, before at
commit `1eb106f`, after at `3db7caf` (in release 2.1.0). Research records:
[R154](../../research/2026-11-18/20-reply-machine-one-place.md),
[R155](../../research/2026-11-18/21-reply-rule-in-the-loop.md),
[R156](../../research/2026-11-18/22-find-step-reply-machine.md).

## What the code does

`with_repairs` asks a model, shows the reply to a check, and asks again when the check says no. A
reply that was cut off, or that the model refused to give, must never be accepted, whatever its text
says.

## Before

`with_repairs` handed the check only the reply's text. The adapter needed to know how the reply had
stopped, so it carried that fact around the loop in a cell: one closure wrote it, the other took it
out.

```rust
// adapter.rs
// The check sees only the text; the call notes how the reply stopped.
let stop = Cell::new(None);
let mut call = |asked: &CallRequest| {
    self.model
        .chat(asked)
        .inspect(|reply| stop.set(Some(reply.stop.clone())))
};
let mut check = |text: &str| {
    let verdict = match stop.take() {
        Some(Stop::CutOff) => failed("the reply was cut off before the pack was complete".into()),
        Some(Stop::Refused) => failed("the request was refused".into()),
        _ => judge(task, &pack_text(text)),
    };
    // ...
};
let outcome = with_repairs(&mut call, &request, &mut check, self.repairs);
```

The rule "a reply that is not whole is rejected" lived in this one caller. The loop itself, in
another file, decided again on the same stop reason when it chose whether to ask once more. The two
other callers of the loop, `lesscode-model check` and `measure`, had no such rule and judged the text
alone.

## After

The check receives the whole reply, and the rule is in the loop, once, for every caller.

```rust
// adapter.rs
let mut call = |asked: &CallRequest| self.model.chat(asked);
let mut check = |reply: &Reply| {
    let verdict = judge(task, &pack_text(&reply.text));
    // ...
};

// repair.rs, inside with_repairs
let Verdict { passed, reasons } = match reply.stop {
    Stop::CutOff => unjudged(CUT_OFF),
    Stop::Refused => unjudged(REFUSED),
    _ => check(&reply),
};
```

## What changed, counted

| Measure | Before | After |
| --- | --- | --- |
| Cells that carry the stop reason | 1 | 0 |
| Functions that decide on the stop reason | 2 | 1 |
| Files that decide on the stop reason | 2 | 1 |
| Conditions on the stop reason | 4 | 4 |
| Callers of the loop that reject a reply that is not whole | 1 of 3 | 3 of 3 |

The conditions did not get fewer. They are in one function where they were in two files.

## How it was checked

- Every history of three replies, 125 in all, was run through the old and the new adapter. The trial
  records are the same, apart from one sentence of wording.
- A check that looks at the text alone accepted a reply that was not whole in 62 of those 125
  histories under the old loop. Under the new loop it does in none. This is a change of behaviour and
  it is meant: it is in the release notes of 2.1.0.

## What our own commands saw

| Command | Before | After |
| --- | --- | --- |
| Research finder: machines in the four files | 1 | 0 |
| `deconvolute analyze`: branch conditions, four files | 13 | 13 |
| `deconvolute analyze`: state writes, four files | 13 | 13 |
| `deconvolute measure-scope`: call edges | 33 | 33 |

The released commands do not see this change. The finder does, and it is a research program, not a
released command. `analyze` counts no arm of a Rust `match` as a condition, and a closure writing a
cell is not a state write it knows.

## What this does not show

- The histories have three replies each. Longer ones were not run.
- The comparison used scripted replies. No model was called.
