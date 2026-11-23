# Prototype gate: the run page

Registered 2026-11-23, 15:10 Europe/Berlin (the time of commit `45e4bac`; I had first written 15:20
without reading the clock), before the prototype exists. Steering:
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

Run 2026-11-23, 15:16 to 15:25 Europe/Berlin (from the gate's own records), five times. Recorded 2026-11-23, 15:26.
Prototype in `docs/experiments/run-dashboard/` (`page.html`, `main.rs`, `gate.ps1`); outputs in
`D:\BuildArtifacts\lesscode\lesscode-metrics\run-page-proto\gate-1` to `gate-5`, each with `gate.json`
and pictures. Browser: Microsoft Edge 158.0.4370.33, headless. The page and the replay program
built and ran at the first attempt; between the first and the second run I changed two things in
the page, neither of which touches what is measured: a mark that says what failed, and the sign
for a function with some unnoticed faults.

| # | Expectation | Measured | Holds |
| --- | --- | --- | --- |
| P1 | The page follows a data file that changes: at least 5 different snapshots in one session, ending in "ended" | 158 to 160 different snapshots in each of five sessions; "ended" each time | yes |
| P2 | Putting a new data file in place does not disturb the page: no write lost, at most 2% of reads missed, no failure of the page's code | 276 rewrites per run, none lost, none retried; no read missed; **one failure in two of the five runs** (1 of about 160 reads), reported by the browser only as "Script error" | **no** |
| P3 | The report is one file that needs nothing else | moved alone to an empty directory: "ended", with 268 planted, 72 noticed, 193 unnoticed, 3 not built; 58,738 bytes; the only address in it is the name of the SVG format, which is not fetched | yes |
| P4 | Six fixed inputs give the six states | all six, at both sizes | yes |
| P5 | It does not break on sparse, large, void and damaged input at 1,280 and 400 wide | no failure and no sideways scrolling in any of them; but the browser gives 1,256 and 492, not 400 | at the widths the browser gave; 400 was not seen |
| P6 | The page can subscribe to the tool | an event stream served with the standard library alone; 196 to 198 different snapshots per session, "ended" each time, **no failure in five runs**; requests without the secret or with a wrong one get 404; when the tool goes away before the end the page says "stopped" within 2.5 seconds | yes |
| P7 | Pictures for the owner | six per run under `pictures` | shown |

### What P2 means

In two of five runs one read of the data file gave the browser something it could not run. The
page went on, kept what it had, and ended with the right counts; but its code recorded a failure,
which the row forbids. I did not capture the damaged content: a browser hides the details of a
failure in a script loaded from disk. The writer put the file in place by renaming a finished file
over the old one and never had to retry, so the likeliest reading is that replacing a file is not
one step for a reader on this system. That is a guess.

So reading a file again is not sound for the live view as built here. The event stream had no
failure in five runs, and it tells the page at once when the tool is gone, where the file needs up
to a minute of silence to conclude the same.

### What I see in the pictures

The layout holds, with the toolbar, the bar, the tree and the tabs readable in light and dark. Not
right yet: the status line says "Running for" after the run has ended; the fixed page for a void
run still shows its stages as done, which is a fault of my fixture; the tree has no way to fold a
single file.

### What the gate did not see

A width of 400; any browser but Edge; a real run; anyone using it. For P6 the browser's side was
seen, over the browser's debugging port.

## Assessment

Proceed, with the event stream as the way the snapshot travels during a run and the page with the
data inside as the report. The owner decides; see the design's decisions.
