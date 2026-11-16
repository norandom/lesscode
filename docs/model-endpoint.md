# Model endpoint

Most of lesscode never calls a model. The commands that do (`regenerate run`
in the core, and the guide optimiser under `tools/`) go through a second
binary, `lesscode-model`. It reads one endpoint file, sends the request, waits
when the endpoint asks it to, and tells you what it is doing. `lesscode-core`
holds no HTTP client and no model name.

To follow this page you need an API key for an OpenAI-compatible endpoint or
for Anthropic's Messages API, or a local endpoint that speaks one of the two.
Sections 1 to 3 are enough to configure an endpoint and check it.

## 1. Install `lesscode-model`

Releases after 1.2.0 carry a second archive per platform,
`lesscode-model-<target>`. The install scripts fetch it when you set
`LESSCODE_WITH_MODEL=1`, and put it beside `lesscode-core`.

Linux and macOS:

```sh
curl -fsSL https://github.com/norandom/lesscode/releases/latest/download/install.sh | LESSCODE_WITH_MODEL=1 sh
```

Windows (PowerShell):

```powershell
$env:LESSCODE_WITH_MODEL = '1'
irm https://github.com/norandom/lesscode/releases/latest/download/install.ps1 | iex
```

From a checkout, build it with Rust 1.98.1:

```sh
cargo +1.98.1 build --release --locked --manifest-path model/Cargo.toml
```

Then:

```sh
lesscode-model --version
```

`lesscode-core` looks for the binary in three places, in this order: the path
in `LESSCODE_MODEL_BIN`, the directory of `lesscode-core` itself, and the
directories of `PATH`. If `LESSCODE_MODEL_BIN` is set, only that path is tried.

## 2. Write the endpoint file

Copy [`.env.sample`](../.env.sample) to a file outside any repository, for
example `~/.lesscode/model.env`, and fill in four entries:

```ini
LESSCODE_MODEL_API_STYLE=openai
LESSCODE_MODEL_BASE_URL=https://api.mistral.ai/v1
LESSCODE_MODEL_NAME=mistral-large-4
LESSCODE_MODEL_API_KEY=replace-with-your-key
```

| Entry | Meaning |
| --- | --- |
| `LESSCODE_MODEL_API_STYLE` | `openai` for chat completions, `anthropic` for the Messages API |
| `LESSCODE_MODEL_BASE_URL` | where the endpoint is; for `openai` the address that ends in `/v1` |
| `LESSCODE_MODEL_NAME` | the model, as the endpoint names it |
| `LESSCODE_MODEL_API_KEY` | your key |
| `LESSCODE_MODEL_REASONING_EFFORT` | optional; the reasoning level, sent only when you set it |

Every command finds the file the same way: `--endpoint-env FILE` if you give
it, else the path in `LESSCODE_MODEL_ENV`, else `.lesscode/model.env` in your
home directory. `LESSCODE_MODEL_NAME` and `LESSCODE_MODEL_REASONING_EFFORT`
can also be set in the environment for one run, and then win over the file.

The key stays in that file. No command prints it or writes it to a receipt or
a record, and the core hands `lesscode-model` the path of the file, never the
key.

## 3. Check the endpoint

```sh
lesscode-model check --endpoint-env ~/.lesscode/model.env
```

The command sends one small fixed task, runs the core's own checks on the
answer, and asks again with the reasons if the answer is rejected (at most
two repairs, so at most three requests). It writes nothing into the directory
you run it in. A usable endpoint looks like this:

```text
model zai-glm-5-3 at 127.0.0.1:8765: 1 calls answered in 1.7 s, 638 tokens in, 185 tokens out, no wait
this endpoint can be used
```

| Exit code | Meaning |
| --- | --- |
| 0 | the endpoint can be used |
| 2 | the endpoint file is missing or incomplete, or the command line is wrong; nothing was sent |
| 3 | the endpoint failed: not reachable, refused the request, or did not answer in time |
| 4 | every answer was rejected by the check; the report lists the reasons per call |

If an answer needed a repair, the report also says why the earlier answers
were rejected, one line per call.

## 4. What happens while it waits

An endpoint that is busy refuses a request and asks you to come back later.
`lesscode-model` waits and sends the request again. Before each wait it says
so on standard error:

```text
[model] api.mistral.ai asks to come back later (attempt 2); waiting about 20 s, 10 s waited so far
```

The wait is what the endpoint states. If it states none, the first wait is
10 seconds, and each further wait is at least twice the one before. One call
sends one request at a time.

Two limits end a call:

| Limit | Default | Change it with |
| --- | --- | --- |
| refused attempts and the pauses between them, in one call | 600 s | `--max-wait SECONDS` (`call`, `check`) |
| one request waiting for its answer | 3,600 s | `--answer-limit SECONDS` (`call`) |

The time an endpoint holds a request before it refuses counts against the
first limit. When a limit is reached the call fails with exit code 3 and says
how long it waited and how often it tried.

A slow answer does not end the call before that second limit. When a request has been running longer than
the answers before it (60 s for the first), you get a line every 30 seconds:

```text
[model] still waiting for the answer of api.mistral.ai: 90 s so far
```

## 5. Is the model good enough?

`check` says whether the endpoint works. `measure` says whether the model
behind it is good enough for the small steps lesscode asks of it:

```sh
lesscode-model measure --endpoint-env ~/.lesscode/model.env
```

It runs 20 trials over a built-in set of four small tasks. Each trial is one
request, checked by the core's parsing and static checks, with at most two
repairs. That is 20 to 60 requests, so run it when you change the model, not
before every use. The report:

```text
task set: built-in
check: the core's parsing and static checks on a copy (parse_result, prepare_candidate, evaluate_tier1_static)
model: zai-glm-5-3 at 127.0.0.1:8765
trials: 5 (quick), 0 not run
first reply passed: 4 of 5 (80 %)
passed within 2 repairs: 5 of 5 (100 %)
per accepted answer: 1.2 calls, 433 output tokens, 2.0 s
verdict: good enough so far
```

The three verdicts of a full run:

| Verdict | Rule | What to do |
| --- | --- | --- |
| good enough direct | at least 15 of 20 first replies passed | use the model as it is |
| good enough with the repair loop | fewer than that, and at least 18 of 20 passed within two repairs | use it; expect more than one request per answer |
| not good enough as is | fewer than 18 of 20 passed even with repairs | choose another model, or put an optimiser around the instruction |

Read the verdict as a floor. On the built-in set four quite different models
all came out "good enough direct"
([R110](research/2026-11-14/03-model-seam-indicator-on-real-models.md)), so
the set cannot rank capable models against each other. The cost per accepted
answer is in the report and does not enter the verdict. A verdict holds only
for the task set and the check named in the report.

`--quick` runs 5 trials and gives a coarser answer: "good enough so far" when
all passed, "not good enough" when at most two passed, "undecided" between.
If more than a fifth of the trials could not run because the endpoint failed,
there is no verdict.

| Option | Meaning |
| --- | --- |
| `--trials N` | the number of trials (default 20) |
| `--quick` | 5 trials and the coarser verdicts |
| `--task-set DIR` | your own tasks in place of the built-in set |
| `--out FILE` | also write the report as JSON to this file |

Exit codes: 0 for a verdict that says good enough, 4 for one that does not
(or "undecided"), 3 for no verdict, 2 for a command line or endpoint file
that cannot be used.

## 6. `regenerate run` and the second binary

`lesscode-core regenerate run` starts `lesscode-model` for its one model call.
The notices above appear on your terminal while it runs. The receipt of the
turn records what was sent: the host, the model, the output limit, and the
reasoning level and temperature only if they were sent.

- `--model-env FILE` names the endpoint file, as `--endpoint-env` does above.
- `--temperature` is sent when you give it. Without it no temperature is sent.
- `--effort` and `--thinking-display` are still accepted and are not sent. The
  reasoning level comes from the endpoint file.

If `lesscode-model` is not installed, the command ends with exit code 3 and
says where it looked (the line starts with the label you gave the turn, here
`p1`):

```text
p1 failed: the model commands need lesscode-model, which was not found: LESSCODE_MODEL_BIN is not set, and lesscode-model is neither beside this program in /home/you/.local/bin nor in a directory of the search path
```

Every other command of `lesscode-core` works without it.

## For callers: one request, one answer

`lesscode-model call` is the form programs use. It reads one JSON request on
standard input and writes one JSON answer on standard output; notices and the
reason of a failure go to standard error.

```sh
echo '{"protocol": 1, "purpose": "example", "system": "Answer in one word.", "user": "Capital of France?", "max_output_tokens": 200}' \
  | lesscode-model call --endpoint-env ~/.lesscode/model.env
```

The answer is `{"protocol": 1, "ok": true, "reply": {...}}` with the text, the
stop reason, the token counts, the seconds, the waits and the settings that
were sent, or `{"protocol": 1, "ok": false, "error": {"kind": ..., "reason": ...}}`.
Exit codes: 0 answered, 3 the call failed, 2 the request or the command line
could not be used. `--record PATH` appends one line per call to a file; the
line holds no key and no prompt text.
