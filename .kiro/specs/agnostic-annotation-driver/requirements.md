# Requirements Document

## Project Description (Input)

Who has the problem: a maintainer or agent who wants to apply Lesscode's
mutation and state-frame analysis to code that is not one of the project's
experiment specimens, in any of the target languages.

Current situation: the mutant generators replace fixed substrings taken from
the specimens and the analyzers are hand-written per language, with none for
Python in the released command
([R55](../../../docs/research/2026-11-10/01-released-binary-adopter-dogfood.md)).
On ordinary code the mutation command produces nothing, and each new language
or codebase needs new hand-written rules.

What should change: a driver that is agnostic to the code and the language.
A model annotates a scope under a versioned annotation guide; the native core
validates each annotation against the syntax tree and applies generic
operators at the validated sites; real tests classify the result. Mutation is
the first consumer of annotations and analysis the second. The annotation
guide is the artifact that later bounded and verified prompt optimisation
(`MIPROv2`, `GEPA` if needed) works on, which is how the second priority,
hardening against inference drift, attaches to this work.

See the [brief](brief.md) and the [prototype gate](prototype-gate.md). Per
[prototype-first](../../steering/prototype-first.md), requirements and a
provisional design follow the gate's preparation; production tasks need a
`proceed` result.

## Requirements
<!-- Will be generated in /kiro-spec-requirements phase -->
