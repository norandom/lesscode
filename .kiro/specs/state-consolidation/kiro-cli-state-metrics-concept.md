# Concept: state metrics for `kiro-cli code check`

Written 2026-11-18 for the owner, who maintains both projects. A proposal for `openidea-kiro`, in that
tool's own terms (observations, limits, rules, ratchet). Nothing in `openidea-kiro` was changed. The
numbers come from the four examples under `docs/examples` and were counted by hand; they can serve as
the first contract fixtures.

## Why a new rule and not a setting

`CODE_SCATTERED_STATE` finds one subject compared with literal values in more than three files. Its
own documentation names what it cannot see: state carried in booleans, in a number, or in a value
that reaches the comparison through a variable. All four examples are of that kind, and all four sit
in one or two files. No threshold of the existing rule reaches them.

The division of work stays as the README of `openidea-kiro` has it: a finding there names a
candidate; a lesscode step shows whether the replacement kept its protection.

## What is measured

A **holder** is a place that keeps part of a program's standing between calls. Which syntax is a
holder is data in the language profile, like the rest of the measurement:

| Kind | Rust | Python, TypeScript, JavaScript |
| --- | --- | --- |
| flag | a field of type `bool` or `Option<…>` | an attribute or property assigned `True`/`False`/`None`, `true`/`false`/`null`/`undefined` |
| cell | a field or local of a lock, atomic or interior-mutable type (`Mutex`, `RwLock`, `Cell`, `RefCell`, `Atomic*`, `OnceCell`) | an attribute assigned in a method other than the constructor |
| shared local | a local written in one closure and read in another | a variable written in one nested function and read in another (`nonlocal`, closure capture) |

Four metrics, each an observation with a definition version, like the existing ones:

| Scope | Metric | Definition | Built-in limit to start from |
| --- | --- | --- | --- |
| type | `flags` | holders of kind flag declared in the type | 2, warning |
| type | `cells` | holders of kind cell declared in the type | 1, warning |
| routine | `shared_locals` | holders of kind shared local declared in the routine | 0, warning |
| state (new scope: one holder) | `deciders` | routines with a condition, a `match` or a loop test on a value read from the holder | 2, information |

A flag has two values, so a type with `flags` = n can stand in 2^n ways. The limit of 2 says: beyond
four combinations, name the states.

One rule that reads across routines:

**`CODE_STATE_SPREAD`** (warning). Two or more holders of one type are written in the same routine,
in at least two routines, or are tested one after the other in one routine. The hint is `collect:`
keep them in one value and make every change of it a method of that value.

## How it fits the ratchet

- `flags`, `cells` and `shared_locals` are judged like routine metrics: a value above its limit that
  rose blocks; a breach that was there before is marked pre-existing.
- The coverage line gets one more net figure next to statements and lines: **decisions**, the sum of
  cyclomatic complexity minus the number of routines over the changed files. A step that untangles
  state should leave it about where it was (R162: 24 and 24, 30 and 31, 14 to 12). A large fall
  means behaviour was removed, not untangled.

## What the examples give, counted by hand

| Example | Metric | Before | After |
| --- | --- | --- | --- |
| Order flags | `flags` of `Order` | 3 | 0 |
| Order flags | `deciders` of the order's state, most-asked holder | 4 | 2 |
| Batch progress | `cells` of `Shared` | 3 | 1 |
| Batch progress | `CODE_STATE_SPREAD` on `Shared` | reported: `fatal` and `queue` tested one after the other | not reported |
| Clipboard lease | `cells` of `NativeClipboardState` | 3 | 2 |
| Clipboard lease | `cells` of `Lease` | 1 | 0 |
| Clipboard lease | `CODE_STATE_SPREAD` | reported: `owner` and `valid` written together in three routines | not reported |
| Reply side channel | `shared_locals` of the adapter's `one` | 1 | 0 |

`operation` in the clipboard state is a second cell that guards something else; `cells` = 2 after the
change is a true reading and a pre-existing breach of a limit of 1.

## What it cannot see, to be said in its documentation

- Which holders form one machine. `CODE_STATE_SPREAD` is a pointer from co-writing and neighbouring
  tests; two unrelated flags changed in one routine are reported too.
- States that mean something. Only a person or a written transition table says that.
- A holder reached under another name in another file: a clone handed to a thread, a field read
  through a parameter. The lesscode finder follows some of these names (R158); a first version here
  need not.
- State in a plain field of an enumeration type that is assigned in many places, such as a session's
  state in twenty assignments. That is a fifth metric, `writers` per holder, left out of this first
  cut because none of the four examples measures it.

## What would show the concept is wrong

Calibration on the five repositories of the existing calibration record. If `flags` > 2 or `cells` >
1 is reported for most types, the limits say nothing and must move or the metric must narrow (for
example, count only flags that are tested together). The four examples must give the table above.
