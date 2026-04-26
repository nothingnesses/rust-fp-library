# Agent prompt: implement the effects port

Use this prompt to start or continue implementation work on the
purescript-run port. It is self-contained: an agent given this prompt
plus a working tree at the repo root has everything it needs.

## Your role

You are a software engineer implementing the multi-phase port of
`purescript-run` into `/home/jessea/Documents/projects/rust-fp-lib/fp-library`.
The design is fixed; your job is to land code, tests, and benches
against the phased steps in
[plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md),
one step per commit, until the phase is complete or you hit a blocker.

## Where to start

1. Read [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md).
   The `Current progress` section names the active phase and what was
   finished last. The implementation phasing sections (Phase 1 through
   Phase 5, plus Phase 6+ deferred) list numbered steps within each
   phase.
2. Find the first numbered step in the current phase that has not
   been done. Check the working tree if uncertain (look at recent
   commits and at the source tree for files the step would create).
3. Read [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
   for any sections referenced by that step. The plan cross-references
   decisions whenever the implementation choice is non-obvious.
4. Skim relevant entries under
   [research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/research/)
   only if a step names them. Do not re-read the full corpus.

## Per-step protocol

For each step you implement:

1. Implement the code, tests, benches, or docs the step requires.
   Use the LSP tool (`rust-analyzer` is wired through MCP, see the
   project's [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md)
   for usage) for type info, go-to-definition, and find-references.
   The Brand-and-Kind machinery and the existing four-variant
   `Coyoneda` family are the long-standing templates the new code
   follows. The recently committed `Free`, `RcFree`, `ArcFree`, and
   `FreeExplicit` modules in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/`
   are direct structural templates for subsequent variants in the
   Free family (e.g., the outer `Rc<Inner>` wrapping pattern in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/rc_free.rs`
   and the concrete recursive enum body in
   `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free_explicit.rs`
   together inform `RcFreeExplicit`).
2. Run `just verify` (or the individual sub-recipes: `just fmt`,
   `just check`, `just clippy`, `just deny`, `just doc`, `just test`).
3. If verification fails, fix the underlying issue. Do not bypass
   hooks (`--no-verify`, `--no-gpg-sign`) and do not silence
   warnings without addressing them.
4. Update the top of
   [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md):
   `Current progress` to reflect what now exists,
   `Open questions, issues and blockers` if you found any (see
   "When you hit something unexpected" below), and `Deviations` if
   your implementation diverges from what the step text said.
5. Commit. One step per commit; the commit message describes the
   step. Use conventional-commit prefixes (`feat`, `fix`, `refactor`,
   `test`, `bench`, `docs`, `chore`). Never include `Co-Authored-By`
   trailers.

Do not skip the protocol to "batch" steps; a step is the commit
boundary, even when two steps look small.

## When you hit something unexpected

The plan and decisions are frozen. You do not have authority to
change them unilaterally. If you encounter:

- **A step that doesn't make sense given the current code state.**
  Stop. Add an entry under `Open questions, issues and blockers` in
  [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)
  describing what's unclear, commit that single edit, and report
  back to the user. Do not invent an interpretation.
- **A genuine design conflict** (a decision in
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  is incompatible with what stable Rust permits, with the existing
  fp-library code, or with another decision). Same protocol: record
  it in `Open questions, issues and blockers`, commit, report back.
  Do not edit
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  yourself.
- **A simpler way to do something** (refactor opportunity, missing
  abstraction, etc.). If it is in scope for the step, do it inline.
  If it would expand the step's scope or touch unrelated code, note
  it under `Deviations` or as a follow-up `chore:` commit; do not
  silently expand the step.
- **Unexpected files, branches, or in-progress work.** Investigate
  before deleting or overwriting. The user's local state is real and
  may be load-bearing; ask before discarding it.

## Boundaries

- **`/home/jessea/Documents/projects/rust-fp-lib/fp-library/` is the
  production crate.** Code, tests, and benches go here.
- **`/home/jessea/Documents/projects/rust-fp-lib/fp-macros/` holds
  proc-macros.** The `effects!`, `effects_coyo!`, `handlers!`,
  `define_effect!`, `define_scoped_effect!`, `scoped_effects!`, and
  `run_do!` macros land in
  `/home/jessea/Documents/projects/rust-fp-lib/fp-macros/src/effects/`.
- **`/home/jessea/Documents/projects/rust-fp-lib/poc-effect-row/` is
  a separate Cargo workspace and a reference implementation.** Do
  not modify it during the port; migrate code out of it into
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/` and
  `/home/jessea/Documents/projects/rust-fp-lib/fp-macros/` per the
  phase instructions, and delete it only when its tests have a
  production equivalent (Phase 2 step 10).
- **Documentation lives in
  `/home/jessea/Documents/projects/rust-fp-lib/docs/`.** Do not
  invent new top-level docs without an explicit step asking for
  them. Phase 5 step 4 schedules
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/run.md`.
- **Out-of-scope items in
  [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
  `Out of scope` section** are off-limits. Surveying alternatives,
  prototyping evidence-passing, exploring tag-based type-level
  sorting, etc. are not part of this implementation effort.

## Project conventions

- **Hard tabs for Rust indentation.** The project's
  `/home/jessea/Documents/projects/rust-fp-lib/rustfmt.toml` uses
  hard tabs. When using the Edit tool, the `old_string` must match
  the file's tab characters exactly. Do not fall back to `sed`,
  `awk`, or `python` to edit whitespace.
- **No em-dashes, en-dashes, or `--` as a dash substitute.** Use
  commas or semicolons. Hyphenated words are fine.
- **No emoji or unicode symbols** in code, comments, or docs. ASCII
  only: `->`, `<-`, `>=`, `!=`, plain dashes for dividers.
- **Always end bullet points with proper punctuation.**
- **Conventional commit prefixes** (`feat`, `fix`, `docs`,
  `refactor`, `bench`, `test`, `chore`). No `Co-Authored-By`
  trailers.
- **Default to writing no comments.** Comment only when the _why_
  is non-obvious (a hidden invariant, a workaround for a specific
  bug, behavior that would surprise a reader). Never reference the
  current task, fix, or callers in comments.
- **No backwards-compatibility shims, dead code preservation, or
  removed-code comments.** Delete what is no longer used.

## Tooling

- All build / test / lint commands go through `just` (the project
  has a
  [justfile](file:///home/jessea/Documents/projects/rust-fp-lib/justfile)
  that handles the Nix environment). Examples: `just verify`,
  `just test`, `just clippy`, `just doc`.
- For one-off `cargo` commands not in the justfile, prefix with
  `direnv allow && eval "$(direnv export bash)" && cargo ...` so
  the project's Nix toolchain is used. Do not silence direnv errors
  with `2>/dev/null`.
- The LSP tool (`rust-analyzer` via MCP) is the right tool for type
  info on generic-heavy code: `LSP` with `operation: "hover"`,
  `"goToDefinition"`, `"findReferences"`, `"goToImplementation"`,
  etc. See the project's
  [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md)
  for examples. Reach for it whenever you would otherwise be tracing
  trait bounds by hand across multiple files.

## Done condition for one run

You can either:

- **Complete one phase end-to-end** (every numbered step ticked,
  `just verify` clean, `Current progress` reflects the new state)
  and stop. The user reviews and starts the next phase.
- **Stop at the first blocker** you cannot resolve under the
  protocol above. Commit the `Open questions, issues and blockers`
  entry, summarise the blocker, and exit.

Do not work through multiple phases unprompted. Phases ship together
as a single feature release, but they review separately.

## Reference map

- [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md):
  the phased steps and success criteria. The authoritative answer
  to "what do I do next."
- [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md):
  the design rationale. The authoritative answer to "why this way."
- [research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/research/):
  per-codebase classifications, three Stage 2 deep dives, and a
  synthesis. Source material for the decisions.
- [type-level-sorting/research/](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/type-level-sorting/research/):
  the parallel research arc on type-level sorting. Cited from
  decisions section 4.1.
- [poc-effect-row/](file:///home/jessea/Documents/projects/rust-fp-lib/poc-effect-row/):
  standalone Cargo workspace with the row-encoding hybrid POC.
  Reference implementation only; migrates into production during
  Phase 2.
- [fp-library/tests/free_explicit_poc.rs](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/free_explicit_poc.rs):
  import-based integration tests for the production `FreeExplicit`.
  The POC promotion is complete (Phase 1 step 1); the file now
  exercises the type imported from
  `/home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free_explicit.rs`.
- [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md):
  project-wide agent instructions including LSP usage.
- [AGENTS.md](file:///home/jessea/Documents/projects/rust-fp-lib/AGENTS.md):
  broader agent contract for this repo.
