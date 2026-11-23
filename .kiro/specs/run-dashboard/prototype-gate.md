# Prototype gate: the run page

Registered 2026-11-23, 15:20 Europe/Berlin, before the prototype exists. Steering:
[prototype-first](../../steering/prototype-first.md). The gate decides whether the
[design](design.md) may go on to tasks, and which way the snapshot travels.

## What is built for the gate

- A page template with the view library named in the design (Preact with htm, one file, fetched
  for the prototype and not committed: `https://unpkg.com/htm@3.1.1/preact/standalone.umd.js`,
  13,296 bytes, SHA-256 `62C4FFEA…41D8AC51`).
- A Rust program that replays a recorded run as if it were happening: the receipt of the fault run
  on the renderer of `drift-rdp` (R164, 268 planted faults, three copies of which one dropped out).
  It writes the page, rewrites the data file, can serve an event stream, and writes the page with
  the data inside at the end. It also writes pages for fixed situations.
- A start script that opens the pages in a headless browser (Microsoft Edge), reads marks the page
  puts on itself (its state, how many snapshots it read, failures), and takes pictures.

The replay has one known shortfall: the recorded receipt does not say in which function a fault
lies, so the tree shows files while the run goes on and functions only at its end.

## Expectations

| # | Question | Expectation | Rejected when |
| --- | --- | --- | --- |
| P1 | Does a page opened from disk follow a data file that changes? | In one session, without a reload, the page reads at least 5 different snapshots while the tool rewrites the file, and ends in the state "ended" | fewer than 5, or another end state |
| P2 | Does putting a new data file in place disturb a page that reads it? | Over at least 250 rewrites with the page reading every 50 ms: the writer never fails for good (it may retry), the page misses at most 2% of its reads, and no failure of the page's code is recorded | a write is lost for good, more than 2% missed, or a failure |
| P3 | Is the report one file that needs nothing else? | The page with the data inside, moved alone to an empty directory, shows the state "ended" and the run's counts; it names no address on another machine; it is at most 120 KiB | another state, a missing count, an address, or larger |
| P4 | Does the page show the state its input calls for? | Six fixed inputs give "waiting for data", "running", "quiet", "stopped", "ended" and "cannot read" | any other state for any input |
| P5 | Does it break? | For an input with most fields missing, one with 2,000 functions whose names are 120 characters long, a void run without tests, and damaged data, at widths of 1,280 and 400: no failure of the page's code, and the page as a whole does not scroll sideways | a failure, or sideways scrolling of the page |
| P6 | Can the page subscribe to the tool? | With the standard library alone the program serves an event stream on `127.0.0.1`; a page subscribed to it shows at least 5 different snapshots and ends in "ended"; a request without the secret is refused | the stream cannot be served without a dependency, the page does not follow it, or a request without the secret is answered |
| P7 | How does it look? | Not judged by me. Pictures at 1,280 wide, light and dark, are shown to the owner | |

For P6 I do not know whether the headless browser can hold a stream open under the clock it uses
for such runs. If it cannot, the stream is checked with a plain client, and the record says that
the browser's side was not seen.

## What the gate does not show

- The page inside `lesscode-core`: the prototype is a program beside it.
- A real run: the replay plays a receipt back faster than it happened.
- Other browsers than Edge.
- Anyone using it.

## Results

Not run yet.
