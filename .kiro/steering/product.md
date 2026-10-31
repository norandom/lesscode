# Lesscode / Deconvolute

Build a local-first system that measures and reduces convolution in AI-generated codebases and lets developers set measurable coding rules for agents to prevent regressions. A Codex skill is an optional workflow interface, not the product's complete solution.

The user clarified on 2026-10-31 that prevention and retrospective improvement both matter. Agent rules must use observable criteria and executable checks, with configurable thresholds and change budgets. The Anthropic code-simplifier agent is a side reference for policy wording, not a replacement for the measured evaluator or its quality gates.

The priority is preserved behavior, security and public APIs, followed by reduced complexity, duplication and unnecessary code. Prefer deletion, inlining, reuse and native features before introducing abstractions. GoF patterns require demonstrated benefit.

The core workflow must require no SciTools installation, commercial analyzer, license key or hosted analysis account. Analysis should run independently of Codex; LLM usage costs remain separate. Code style does not reliably establish AI authorship.

Status: efficiency-baseline is complete: all 33 executable tasks are independently reviewed and the final feature gate returned GO. Integrated CLI, adverse paths and genuine native Windows/Linux/macOS validation passed. See its validation.md for evidence. Supplied-snapshot pilots validate the evaluator; skill effectiveness is unmeasured and later scope. Agent-quality-rules has approved requirements, design and an independently reviewed task plan: 25 executable tasks in seven groups cover all 65 criteria. Implementation is ready to start; no rule-engine implementation is complete yet. It precedes the product deconvolute-skill. Python is the first evaluation slice; English is the documentation language.
