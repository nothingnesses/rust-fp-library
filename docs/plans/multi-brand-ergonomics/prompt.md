You are implementing the multi-brand ergonomics plan for the fp-library Rust crate.

## Context

The plan is at:
/home/jessea/Documents/projects/rust-fp-lib/docs/plans/multi-brand-ergonomics/plan.md

Read it in full before doing anything. It contains:

- An implementation protocol (stage, verify, commit per step).
- Current progress, open questions, and deviations sections to update per step.
- Decisions A-V with rationales.
- Phases 0-5 with numbered steps.
- A test matrix at analysis/07-test-matrix.md listing 58 test cases.
- 9 validated POCs in fp-library/tests/slot\_\*.rs demonstrating the adopted patterns.

The project's build/test/lint commands are in CLAUDE.md. Key points:

- Always use `just <recipe>` (never raw `cargo`).
- `just verify` runs fmt, check, clippy, deny, doc, test in order.
- `just test` caches output keyed on staged file content hashes. Stage changes before running tests.
- The project uses hard tabs for indentation (see rustfmt.toml).

## Your task

Implement the plan starting from the current progress (read the "Current progress" section to see where to begin). Work through each phase's steps in order.

For each step:

1. Read the step's requirements from the plan.
2. Read the relevant source files before modifying them.
3. Implement the step.
4. Stage all changes with `git add`.
5. Run `just verify`. If it fails, fix the issue and re-run.
6. Update the plan's "Current progress", "Open questions, issues and blockers", and "Deviations" sections.
7. Commit with a conventional-commit-prefixed message describing the step. Do not include Co-Authored-By trailers.

If you encounter something the plan did not anticipate:

- Record it in the "Deviations" section with a brief explanation.
- If it blocks progress, record it in "Open questions, issues and blockers" and stop.
- If it is a minor adjustment, proceed and document the deviation.

## Key patterns to follow

The adopted Slot design (to be named `Slot` temporarily, renamed to `InferableBrand` in phase 4) is demonstrated in:

- fp-library/tests/slot_marker_via_slot_poc.rs (the adopted design for map)
- fp-library/tests/slot_bind_poc.rs (bind pattern)
- fp-library/tests/slot_arity2_poc.rs (bimap/arity-2 pattern)
- fp-library/tests/slot_apply_poc.rs (apply with dual Slot bounds)
- fp-library/tests/slot_generic_fixed_param_poc.rs (generic fixed parameters)

Use these as reference for the trait shape, impl patterns, and inference wrapper signatures.

## Constraints

- Do not use emoji or unicode symbols in code, comments, or documentation.
- Do not add Co-Authored-By trailers to commits.
- Use conventional commit prefixes (feat, fix, refactor, test, docs, chore).
- POC test file documentation must be self-contained. Do not reference external plan documents, review finding IDs, or file paths that may not exist in the future.
- When using the Edit tool on files with hard tabs, match the indentation exactly from the Read output.
- Never run `cargo` directly. Always use `just <recipe>`.
- Stage files before running `just test` or `just verify` to ensure cache invalidation.
