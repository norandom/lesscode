# Agentic SDLC and Spec-Driven Development

Kiro-style Spec-Driven Development on an agentic SDLC

## Project Memory
Project memory keeps persistent guidance (steering, specs notes, component docs) so Codex honors your standards each run. Treat it as the long-lived source of truth for patterns, conventions, and decisions.

- Task-group handovers live in `.kiro/task_N_handover.md` (owner rule, 2026-11-07): written when a task group finishes, before asking the user to clear context. Start the next task group from the latest handover file.

- Use `.kiro/steering/` for project-wide policies: architecture principles, naming schemes, security constraints, tech stack decisions, api standards, etc.
- Use local `AGENTS.md` files for feature or library context (e.g. `src/lib/payments/AGENTS.md`): describe domain assumptions, API contracts, or testing conventions specific to that folder. Codex auto-loads these when working in the matching path.
- Specs notes stay with each spec (under `.kiro/specs/`) to guide specification-level workflows.

## Project Context

### Research continuity

- For research-bearing design, task generation and implementation, load [.kiro/steering/prototype-first.md](.kiro/steering/prototype-first.md). A provisional design must pass a bounded real-code prototype gate before production task generation or execution; `-y` does not waive evidence. Prototype work itself is allowed before production tasks to avoid a bootstrap deadlock.

- Before continuing experimental work, read [docs/research/README.md](docs/research/README.md) and the latest applicable dated entry, then the linked Kiro spec.
- Record experiments, failed attempts and material hypothesis revisions under `docs/research/YYYY-MM-DD/NN-topic.md`, using [the entry template](docs/research/ENTRY-TEMPLATE.md) and COBESY's falsifiable framing.
- Update the research index and reciprocal spec links in the same change. Distinguish writing and execution dates; label retrospectives. Preserve evidence and earlier interpretations through dated amendments.
- Expand through evidence gates. Setup, extraction, model fidelity, behavioral checks, compression and adoption are distinct outcomes. Do not silently approve spec phases or weaken checks to obtain progress.

### Paths
- Steering: `.kiro/steering/`
- Specs: `.kiro/specs/`

### Steering vs Specification

**Steering** (`.kiro/steering/`) - Guide AI with project-wide rules and context
**Specs** (`.kiro/specs/`) - Formalize development process for individual features

### Active Specifications
- Check `.kiro/specs/` for active specifications
- Use `$kiro-spec-status [feature-name]` to check progress

## Development Guidelines
- Think in English, generate responses in English. All Markdown content written to project files (e.g., requirements.md, design.md, tasks.md, research.md, validation reports) MUST be written in the target language configured for this specification (see spec.json.language).

## Minimal Workflow
- Phase 0 (optional): `$kiro-steering`, `$kiro-steering-custom`
- Discovery: `$kiro-discovery "idea"` — determines action path, writes brief.md + roadmap.md for multi-spec projects
- Phase 1 (Specification):
  - Single spec: `$kiro-spec-quick {feature} [--auto]` or step by step:
    - `$kiro-spec-init "description"`
    - `$kiro-spec-requirements {feature}`
    - `$kiro-validate-gap {feature}` (optional: for existing codebase)
    - `$kiro-spec-design {feature} [-y]`
    - `$kiro-validate-design {feature}` (optional: design review)
    - `$kiro-spec-tasks {feature} [-y]`
  - Multi-spec: `$kiro-spec-batch` — creates all specs from roadmap.md in parallel by dependency wave
- Phase 2 (Implementation): `$kiro-impl {feature} [tasks] [--review required|inline|off]`
  - Without task numbers: autonomous mode (subagent per task + independent review + final validation)
  - With task numbers: manual mode (selected tasks in main context, still reviewer-gated before completion)
  - Owner default (2026-11-07): implementation runs in the foreground (manual mode with explicit task numbers); autonomous subagent-per-task mode only on explicit owner request
  - `--review off` skips task-local review; use it intentionally and keep `$kiro-validate-impl {feature}` as the final quality gate
  - `$kiro-validate-impl {feature}` (standalone re-validation)
- Progress check: `$kiro-spec-status {feature}` (use anytime)

## Skills Structure
Skills are located in `.agents/skills/kiro-*/SKILL.md`
- Each skill is a directory with a `SKILL.md` file
- Use `/skills` to inspect currently available skills
- Invoke a skill directly with `$kiro-<skill-name>`
- `kiro-review` — task-local adversarial review protocol used by reviewer subagents
- `kiro-debug` — root-cause-first debug protocol used by debugger subagents
- `kiro-verify-completion` — fresh-evidence gate before success or completion claims
- Use skills explicitly requested by the user and skills relevant to the task's domain, including design, accessibility, and UX.
- Select skills from their descriptions or metadata first, then read only the selected skills and the references needed for the task.
- Follow explicit host and project rules and retain required workflow checks. Do not skip relevant skills just because the task is small.

## Subagents

Owner rule, 2026-11-07: implementation tasks stay in the foreground (main
conversation context). Do not dispatch implementer, reviewer or debugger
subagents unless the user explicitly requests subagent execution; autonomous
subagent-per-task mode is not the default.

Current Codex releases enable subagents by default. Use the available tools when the user, project rules, or this skill's workflow calls for delegation; no experimental feature flag is required. An administrator or user can disable subagents by setting `enabled = false` under `[agents]` in Codex configuration.

Use a fresh context for each independent implementer or reviewer, passing the task-relevant inputs explicitly. If delegation is unavailable, follow the skill's inline fallback and identify the review as inline. Skill discovery alone does not prove subagent execution or independent review.

## Development Rules
- 3-phase approval workflow: Requirements → Design → Tasks → Implementation
- Human review required each phase; use `-y` only for intentional fast-track
- Keep steering current and verify alignment with `$kiro-spec-status`
- Follow the user's instructions precisely, and within that scope act autonomously: gather the necessary context and complete the requested work end-to-end in this run, asking questions only when essential information is missing or the instructions are critically ambiguous.
- Owner rules, 2026-11-07: (1) implementation tasks run in the foreground — no subagents unless the user states otherwise; (2) on Windows, run commands through PowerShell, never cmd.exe; (3) Rust tests run with cargo-nextest and Python tests with pytest-xdist, per [docs/testing.md](docs/testing.md); (4) one commit per completed task on main, and commit outstanding changes before starting an implementation run or the next task; (5) native CI and cross-platform verification run through the repository's existing Dagger configuration (`.dagger/`, `dagger.json`, `.github/workflows/native-core.yml` per [docs/ci-validation.md](docs/ci-validation.md)); WSL is a Dagger CI/build environment only — never execute development-task code in WSL; (6) once a task group is finished, write `.kiro/task_N_handover.md` (incrementing N; what was done, evidence pointers, current state, next steps) and ask the user to clear the conversation context before the next task group.

## Steering Configuration
- For spec and implementation work, load the core steering files below from `.kiro/steering/`. Reuse current context rather than rereading unchanged files.
- Load additional steering only when required by project rules or relevant to the task.
- Default files: `product.md`, `tech.md`, `structure.md`
- Custom files are supported (managed via `$kiro-steering-custom`)
