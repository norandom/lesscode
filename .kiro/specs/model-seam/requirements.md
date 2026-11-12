# Requirements: the model seam

Generated 2026-11-12 (Europe/Berlin) from the [brief](brief.md). Approved by the owner on 2026-11-12, 18:21.
Evidence: [R102](../../../docs/research/2026-11-12/09-product-path-two-models.md),
[R103](../../../docs/research/2026-11-12/10-model-seam-library-prototype.md),
[R99](../../../docs/research/2026-11-12/06-repair-gate-result.md).

## Points decided with the approval

These were proposals when the requirements were generated. The owner approved them with the document.

1. Reading of "estimated wait time": the tool waits and tells the user how long it expects to wait
   (Requirement 3). Not: a forecast of how long a whole run will take.
2. Total wait for one call before giving up: ten minutes unless the user sets another limit (3.6).
3. Repairs after a rejected answer: at most two, as measured in R99 (6.3).
4. The check command sends one small fixed task and costs one to three calls (5.1, 5.6).

## Introduction

An adopter who wants a model to do work behind lesscode can today use only the model the tool was built
against. On 2026-11-12 six live turns against three vendors failed for four different reasons before any
answer was used, and a failed turn said nothing on the terminal. The language packs are not the problem:
they are published once with the tool and are its knowledge base. What is missing is the place where the
tool talks to a model.

This feature provides that place, the model seam. An adopter describes one endpoint in one file, and every
endpoint is treated the same way: a vendor's service, another vendor's service, or a service on the
adopter's own machine. When a call has to wait, the tool waits and says for how long it expects to. When a
call fails, the tool says why. Before relying on a model, an adopter can check it once. What a model
returns is used only after the existing checks, because the same request does not give the same answer
twice.

Terms: an *endpoint* is one model service together with the model to use on it. The *endpoint file*
describes it. A *call* is one request to the endpoint and its reply. A *wait* is time spent before an
endpoint accepts or answers a call. The *fence* is the set of existing checks (admission, control cases,
the step flow's verdicts) that decide whether a model's answer is used. A *repair* is a further call that
shows the model its rejected answer and the fence's reasons. The *check command* is what an adopter runs
to find out whether a configured endpoint can be used.

## Boundary Context

- **In scope**: describing an endpoint; one call that behaves the same on every endpoint; waiting with a
  stated estimate; reasons for failures on the terminal; the check command; the repair loop around a
  caller's own check; the existing model-driven commands using the seam; a record of every call.
- **Out of scope**: what the fence checks; the language packs and how they are written; which model is
  better or cheaper; prices; what a model is asked to do with a codebase; showing a model's answer as it
  is produced; a model calling tools.
- **Adjacent expectations**: the step flow and the driver make no model call and are not changed. The
  commands that already send prompts to a model keep their receipts. A later spec decides what a model is
  asked to propose; the onboarding work uses the check command.

## Requirements

### Requirement 1: Describing an endpoint
**Objective:** As an adopter, I want to describe my model service in one file, so that no code and no
command line has to know about it.

#### Acceptance Criteria
1. The model seam shall read an endpoint from one endpoint file that states the request style, the address,
   the model and the key.
2. Where the endpoint file states a reasoning level, the model seam shall send that level with every call.
3. Where a model or a reasoning level is set for one run outside the endpoint file, the model seam shall
   use that value in place of the file's.
4. If the endpoint file is missing or lacks a required entry, then the model seam shall name the file and
   the entry and make no call.
5. If the endpoint file states a request style the model seam does not know, then the model seam shall
   name the styles it knows and make no call.
6. The model seam shall accept an endpoint on the adopter's own machine in the same way as any other.
7. The model seam shall never write the key to the terminal, to a record or into an error message.
8. The model seam shall take no address, model or key from anywhere but the endpoint file and the settings
   of the run.

### Requirement 2: One call on every endpoint
**Objective:** As an adopter, I want the same request to work on whichever endpoint I configure, so that no
endpoint is the reference and the others the exception.

#### Acceptance Criteria
1. The model seam shall offer one call that takes an instruction, a request text and an output limit and
   returns the reply's text.
2. The model seam shall send only settings that the endpoint file, the run or the caller states.
3. When an endpoint delivers a reply's text in several parts, the model seam shall return the text parts
   joined and leave out the model's reasoning.
4. When a reply arrives, the model seam shall report why the model stopped and how many tokens went in and
   out.
5. When a reply was cut off at the output limit, the model seam shall report it as cut off.
6. When a caller asks for a reply in a stated structure, the model seam shall return a reply that has that
   structure or report that it has not.
7. The model seam shall complete the same call on each of these first cases: GLM 5.3 and Mistral Large on
   Mistral's service, `gpt-6.1-sol` on OpenAI's service, one model on Anthropic's service, and two models
   on the owner's local endpoint.
8. The model seam shall contain no rule that names one of these models.

### Requirement 3: Waiting with an estimate
**Objective:** As a user, I want to be told how long a call is expected to wait, so that I know the tool is
working and can decide whether to stay.

#### Acceptance Criteria
1. If an endpoint refuses a call because of a rate limit, then the model seam shall wait and send the call
   again.
2. When the model seam starts a wait, it shall tell the user on the terminal that it waits, for which
   endpoint and for how long it expects to.
3. Where the endpoint states how long to wait, the model seam shall base its estimate on that statement.
4. Where the endpoint states nothing, the model seam shall base its estimate on the waits it has seen in
   this run.
5. While a call has been running longer than the last estimate given, the model seam shall tell the user
   at intervals that it is still waiting and for how long so far.
6. If the total wait for one call exceeds ten minutes or the limit the user set, then the model
   seam shall stop, report the wait and its limit, and fail the call.
7. When a call succeeds after a wait, the model seam shall record how long it waited and how often it
   asked.
8. The model seam shall not start a second call to an endpoint to shorten a wait.

### Requirement 4: Saying why a call failed
**Objective:** As a user, I want a failed call to state its reason where I see it, so that I do not have to
search a file to learn what happened.

#### Acceptance Criteria
1. If a call fails, then the model seam shall write the reason to the terminal.
2. If an endpoint refuses a call with a message, then the model seam shall include the endpoint's message,
   shortened, in the reason.
3. If no reply arrives, then the model seam shall say whether the endpoint could not be reached, refused
   the call or did not answer in time.
4. If a call fails, then the command that made it shall end with an exit code that is not zero.
5. The model seam shall distinguish in its report a failure of the endpoint from an answer the fence
   rejected.

### Requirement 5: Checking a configured model
**Objective:** As an adopter, I want to check my endpoint once before I rely on it, so that I learn in a
minute whether this model can be used.

#### Acceptance Criteria
1. When the user runs the check command, the model seam shall send one small fixed task to the configured
   endpoint.
2. When the reply to the task passes the fence, the check command shall report that the endpoint can be
   used and end with exit code zero.
3. If the endpoint refuses or does not answer, then the check command shall report what was refused, in
   the endpoint's words, and end with an exit code that is not zero.
4. If the reply does not pass the fence after the repairs allowed, then the check command shall report
   which check rejected it and end with an exit code that is not zero.
5. The check command shall report the model that answered, the time taken, the tokens used and any wait.
6. The check command shall make at most three calls.
7. The check command shall change no file of the project it is run in.

### Requirement 6: Repairing a rejected answer
**Objective:** As a user, I want a rejected answer to get a second and third chance with the reasons, so
that a cheaper model reaches a usable answer without anything unchecked getting through.

#### Acceptance Criteria
1. When a caller's check rejects a reply, the model seam shall make a fresh call that shows the request,
   the rejected reply and the check's reasons.
2. When no reply arrived or the reply was cut off, the model seam shall send the request again as it was.
3. The model seam shall make at most two repairs for one request, or the number the caller
   states.
4. When a reply passes the caller's check, the model seam shall stop and return that reply.
5. If no reply passes after the repairs allowed, then the model seam shall return the last reply marked as
   rejected, with the reasons of every attempt.
6. The model seam shall record for every attempt whether it passed, why not, and its tokens.
7. The model seam shall never return a reply as accepted that the caller's check did not pass.

### Requirement 7: The fence stays, and the core stays free of models
**Objective:** As the owner, I want the tool's core to work without any model and every model's answer to
pass the checks, so that the result does not depend on which model was used.

#### Acceptance Criteria
1. The step flow and the driver shall make no call to a model.
2. The step flow and the driver shall work on a machine where the model seam is not installed.
3. The model seam shall hand a model's answer to the caller's check before the answer changes any file.
4. If a caller supplies no check, then the model seam shall refuse to run the repair loop.
5. The model seam shall change nothing in what admission, the control cases or the step flow's verdicts
   decide.

### Requirement 8: The existing model-driven commands
**Objective:** As a user of the commands that already send prompts to a model, I want them to work on my
endpoint, so that I do not need a second configuration or a second tool.

#### Acceptance Criteria
1. The regenerate command shall make its calls through the model seam.
2. The research tooling for guides shall make its calls through the model seam.
3. When the regenerate command completes a turn, its receipt shall state the endpoint's host, the model
   and the settings that were sent.
4. The regenerate command's receipt shall state no setting that was not sent.
5. If a turn of the regenerate command fails, then the command shall write the reason to the terminal as
   well as to the receipt.
6. The model seam shall leave receipts written before this feature readable and verifiable.

### Requirement 9: A record of every call
**Objective:** As the owner, I want every call recorded, so that cost and behaviour of a model can be
compared afterwards without calling it again.

#### Acceptance Criteria
1. When a call ends, the model seam shall record the model, the purpose the caller named, the tokens in
   and out, the time taken, the waits and whether it succeeded.
2. Where the endpoint reports reasoning tokens or cached input tokens, the model seam shall record them.
3. The model seam shall record no key and no part of one.
4. The model seam shall write its records where the caller says, and nowhere else.
