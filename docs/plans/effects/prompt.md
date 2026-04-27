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

## Current resume point

**Phase 1 complete; Phase 1 follow-up commits both landed
(`WrapDrop` migration plus the `Functor` -> `Kind` relaxation);
Phase 2 steps 1, 2, 3, 4a, 4b, and 5 complete; Phase 2 step 6
is the immediate next work.**

Recent commit milestones:

- `c3712f6` (Phase 2 step 4a, foundation): row-brand `WrapDrop`
  impls; `Node` / `NodeBrand` machinery (Kind, Functor, WrapDrop);
  three Erased Run wrappers (`Run`, `RcRun`, `ArcRun`). Renamed
  the effects subsystem directory from
  `fp-library/src/types/run/` to
  [`fp-library/src/types/effects/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/effects/)
  (parent module file from `types/run.rs` to `types/effects.rs`).
- `289d3c6` (Phase 2 step 4b): three Explicit Run wrappers
  (`RunExplicit`, `RcRunExplicit`, `ArcRunExplicit`); three
  brands (`RunExplicitBrand`, `RcRunExplicitBrand`,
  `ArcRunExplicitBrand`); brand-level type-class impls
  delegating to
  [`FreeExplicitBrand`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/src/types/free_explicit.rs);
  inherent `bind` / `map` on `RcRunExplicit` and
  `ArcRunExplicit`; row-brand `RefFunctor` and `Extract` cascade
  on `CNilBrand`, `CoproductBrand`, `NodeBrand`; `Clone` impl
  on `Node`; A+B hybrid re-export pattern (top-level + scoped).
- `4950c50` (Phase 2 step 5): three inherent methods (`pure`,
  `peel`, `send`) on each of the six Run wrapper types.

Step 6 implements **conversion methods between paired Erased and
Explicit Run variants**: `Run::into_explicit() -> RunExplicit`
and `RunExplicit::from_erased(Run) -> RunExplicit`, plus the
analogous `RcRun <-> RcRunExplicit` and
`ArcRun <-> ArcRunExplicit` pairs. Each walks the underlying
Free structure once via `peel` and rebuilds in the other shape;
O(N) in the chain depth. Per
[plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
Phase 2 step 6 entry, conversions preserve multi-shot /
`Send + Sync` properties of the underlying substrate.

**There are no active blockers** as of this resume point. Six
resolutions in
[resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md)
document the design-shaping decisions made in steps 4b and 5
(brand-level coverage gaps on the Explicit Run brands; row-brand
cascade expansion; re-export pattern; GAT-normalization
HRTB-poisoning workaround). Read those if step 6 surfaces a
question that plan.md or decisions.md alone cannot answer.

**Pre-existing items still open from earlier steps** (not blocking
step 6, but likely to surface in step 7+):

- `CoyonedaBrand: RefFunctor` is missing. Blocks step 7's
  `m_do!(ref RcRunExplicit)` over canonical Coyoneda-wrapped
  rows.
- `ArcRunExplicit`'s `SendRef*` hierarchy is permanently
  unreachable on stable Rust. Blocks step 7's
  `m_do!(ref ArcRunExplicit)` form entirely.
- `RcCoyonedaBrand` / `ArcCoyonedaBrand` lack `WrapDrop`.

If any of these surface during step 6, surface them as new
active-blocker entries in plan.md and pause.

**HRTB-poisoning is a structural pattern to expect.** Step 5
discovered (and the new gotcha below documents) that
`ArcFree`'s struct-level HRTB on the `Kind` projection poisons
GAT normalization in any scope mentioning it. Step 6's
conversions recurse through `peel` results and rebuild via
`send`; both engage GAT normalization. If `ArcRun::into_explicit()`
or `ArcRunExplicit::from_erased(ArcRun)` fails to compile at a
recursion site, apply the same workaround: produce
projection-typed values outside the HRTB scope, never construct
`Node::First(...)` literals inside it. The probe at
[`fp-library/tests/arc_run_normalization_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/arc_run_normalization_probe.rs)
is a quick way to validate workaround variants.

If you encounter unexpected behavior during step 6, plan.md's
`Active blockers` section is the place to record load-bearing
questions; entries should cite concrete file paths and line
numbers so the next implementor (or you in a future session) can
verify claims without conversational context.

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
5. If your step touches type-class impls, brand-level dispatch, or
   `Send + Sync` auto-derive, also skim
   [fp-library/docs/limitations-and-workarounds.md](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md)'s
   "Unexpressible Bounds in Trait Method Signatures" table. Phase
   1 step 7 added rows for the Explicit Free family that record
   where stable Rust's lack of `for<T>` HRTB caps brand coverage.
   The pattern (Pointed at the brand level; `bind`/`map`
   inherent-only; Ref hierarchy as the by-reference dispatch
   path) is the precedent any new wrapper type with shared
   internal state will end up following. Saves rediscovering the
   constraint mid-implementation.
6. Check plan.md's `Active blockers` subsection for any open
   items. If non-empty, read those before writing code; their
   resolution is part of your work. (Currently empty as of the
   most recent commit; surface new blockers there if you
   encounter them.)

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
4. Update the docs that capture state and history:
   - [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)'s
     `Current progress` section to reflect what now exists.
   - [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
     (append-only) for any per-step deviation from the original
     plan text. Group entries by phase and step, matching the
     existing structure.
   - If you encounter a blocker, add an entry to plan.md's
     `Open questions, issues and blockers -> Active blockers`
     subsection (see "When you hit something unexpected" below).
     Once the blocker resolves, move the entry to
     [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md)
     as a new top-level entry, dated; replace the active-blocker
     subsection in plan.md with a one-line summary plus an
     anchor link to resolutions.md.
5. Commit. One step per commit; the commit message describes the
   step. Use conventional-commit prefixes (`feat`, `fix`, `refactor`,
   `test`, `bench`, `docs`, `chore`). Never include `Co-Authored-By`
   trailers.

Do not skip the protocol to "batch" steps; a step is the commit
boundary, even when two steps look small.

**Splitting an oversized step is permitted but exceptional.** If
a numbered step in plan.md is large enough that landing it as
one commit would risk leaving the working tree mid-step on
context exhaustion (rough rule of thumb: ~1500+ new lines, 7+
new files, or multiple new public types with mixed concerns),
you may split it into sub-commits (e.g., 4a foundation, 4b
follow-on) under the following conditions:

1. Surface the scope to the user before starting. Explain what's
   bundled and offer the split as an option; do not split
   unilaterally.
2. The split must be coherent: each sub-commit must compile and
   pass `just verify` independently, and each must be
   independently reviewable.
3. Record the split in
   [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
   under the step's heading, explaining the scope rationale and
   what each sub-commit lands. Phase 2 step 4's split into 4a
   (foundation) and 4b (Explicit family) is the existing
   precedent.

The default remains "one step per commit"; splitting is for
genuinely outsized steps, not for convenience.

## When you hit something unexpected

The plan and decisions are frozen. You do not have authority to
change them unilaterally. If you encounter:

- **A step that doesn't make sense given the current code state.**
  Stop. Add an entry under
  `Open questions, issues and blockers -> Active blockers` in
  [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md)
  describing what's unclear, commit that single edit, and report
  back to the user. Do not invent an interpretation.
- **A genuine design conflict** (a decision in
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  is incompatible with what stable Rust permits, with the existing
  fp-library code, or with another decision). Same protocol: record
  it under
  `Open questions, issues and blockers -> Active blockers` in
  plan.md, commit, report back. Do not edit
  [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md)
  yourself.
- **A simpler way to do something** (refactor opportunity, missing
  abstraction, etc.). If it is in scope for the step, do it inline.
  If it would expand the step's scope or touch unrelated code, note
  it under
  [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md)
  or as a follow-up `chore:` commit; do not silently expand the step.
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

## Common gotchas from prior steps

These bit prior steps repeatedly. Internalising them up front saves
debug cycles.

- **Stage new files before `just verify`.** Untracked files do not
  invalidate the test-output cache, so a green verify on untracked
  code is not trustworthy. After creating new files, run `git add`
  before `just verify`. If verify reformats existing files (via
  `treefmt`), `git status` will show `MM` on the staged file; re-stage
  with `git add` before retrying the commit.
- **`#[document_examples]` requires a real Rust code block.** It
  rejects `\`\`\`ignore`, `\`\`\`text`, and other non-Rust fences.
If no working example exists for a method whose impl depends on
scaffolding from a later step, options are: (a) add a working
example that uses an existing brand which already supports the
trait (e.g., `OptionBrand`for the`Send\*`family); (b) provide
a small one-off impl alongside so the example compiles; (c)
remove the macro and use plain`# Examples`markdown, but the
resulting deprecation warning is escalated by`-D warnings`in`just clippy`, so this only works after careful suppression.
- **Inherent-method bounds do not propagate into trait impl
  bodies.** When implementing a brand-level type-class trait by
  delegating to an inherent method (e.g.,
  `RcFreeExplicitBrand::bind` -> `RcFreeExplicit::bind`), the
  inherent method's `where A: Clone, F::Of<...>: Clone` bounds are
  not in scope inside the trait method body. Stable Rust does not
  let you add per-method `where` bounds beyond what the trait
  declares (no HRTB-over-types). When this hits, the right move
  is usually documenting the brand-level coverage gap (see the
  [`limitations-and-workarounds.md`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/docs/limitations-and-workarounds.md)
  precedent) and routing through the Ref hierarchy where possible,
  not fighting the constraint.
- **`Free<IdentityBrand, A>` is layout-cyclic.** `Free`'s `Wrap`
  arm holds `F::Of<Free<F, TypeErasedValue>>` where
  `TypeErasedValue = Box<dyn Any>`. For `IdentityBrand`,
  `Identity<T>` is `T` with no indirection, so layout recursion
  has no termination and rustc rejects with
  `error[E0391]: cycle detected when computing layout`. Tests
  and benches that wrap Free over an identity-shaped functor
  must use `ThunkBrand` instead (`Thunk<A>` holds a boxed
  closure, providing the indirection). The Rc/Arc Erased family
  escapes via outer `Rc<Inner>` / `Arc<Inner>` wrapping; the
  Explicit family escapes via `Box<...>` in `FreeExplicit`'s
  `Wrap` arm or the same outer wrapping for the Rc/Arc Explicit
  variants. See deviations.md's Phase 1 step 8 entry.
  **For Run-shaped programs**: `Run<R, S, A>` (over `Free`) hits
  this cycle when `R` has a no-indirection head (e.g.,
  `IdentityBrand`); use `CoyonedaBrand`-headed rows for `Run`'s
  doctests/tests. `RcRun` / `ArcRun` / all three Explicit
  variants escape via their respective outer-pointer or
  `Box`-in-Wrap indirection, so `IdentityBrand`-headed rows
  work for them.
- **HRTB on a GAT projection poisons normalization in scope.**
  Discovered while implementing `ArcRun::send` in Phase 2 step 5. When a struct, impl block, or function's where-clause
  carries an HRTB on a generic associated type at a specific
  instantiation (e.g.,
  `NodeBrand<R, S>: Kind<Of<'static, ArcFree<...>>: Send + Sync>`,
  which `ArcFree`'s struct propagates to every `ArcRun`
  impl-block context), rustc refuses to normalize that GAT at
  _other_ instantiations in the same scope. So a literal
  `Node::First(layer)` cannot be unified with
  `<NodeBrand<R, S> as Kind>::Of<'_, A>` even though
  `impl_kind!` declares them equal. The trigger is the HRTB
  itself, not the substrate: PhantomData-only structs with the
  HRTB hit it; free functions carrying the HRTB hit it;
  cross-substrate calls (e.g., `RcFree::lift_f` from inside an
  `ArcFree`-HRTB-bearing impl) hit it. **Workaround**: receive
  projection-typed values as parameters; never construct
  projection-typed values inside an HRTB-bearing scope. The
  caller (typically test code, smart-constructor macro output,
  or top-level concrete-type code with no HRTB in scope) builds
  the projection literal and passes it in. The probe file
  [`fp-library/tests/arc_run_normalization_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/arc_run_normalization_probe.rs)
  documents four passing patterns and is the regression-test
  home for this limit. This is the design driver for
  `*Run::send` taking the `Node`-projection value (rather than
  the row-variant layer) symmetrically across all six Run
  wrappers.
- **The Wrap-depth probe at
  [`fp-library/tests/run_wrap_depth_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs)
  is a regression test guarding the `WrapDrop` resolution.** It
  measures structural Wrap depth across Run-shaped Free
  programs and documents that Run-typical patterns have
  structural depth at most 1, which is the property the
  `WrapDrop::None` policy relies on for soundness (effect-row
  brands like `CoyonedaBrand` / `CoproductBrand` / `NodeBrand`
  all return `None` from `WrapDrop::drop` because they do not
  materially store the inner Free; `Drop` then falls through to
  recursive drop on the layer, which is sound only as long as
  the structural Wrap depth stays bounded). If a future Phase
  2-4 change appears to invalidate the probe (e.g., new patterns
  emit deeper structural Wrap chains), pause and re-evaluate
  before shipping; the probe finding is load-bearing for the
  no-`Extract`-bound semantics.
- **`Kind!(...)` macro invocations inside `Apply!(...)` do not
  require `use fp_macros::Kind;` to be in scope.** `Apply!` is a
  procedural macro that parses the inner `Kind!(...)` syntax
  itself; the inner macro never gets invoked as a real macro,
  so rustc's unused-import analysis flags the import as dead.
  Some older test files used to carry an
  `#[expect(unused_imports)] use fp_macros::Kind;` shim to
  suppress the warning; that shim is no longer needed and was
  removed during Phase 2 step 4a (commits `9adabd5` and
  `c3712f6`). When writing a test file that uses
  `Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)`
  patterns, do not import `Kind` from `fp_macros`. If you
  separately call `<F as Kind!(...)>::Of<...>` outside an
  `Apply!` (rare; the explicit form bypasses `Apply!`), then
  the import is needed.

## Bench and compile_fail test infrastructure

Many phases add benches or `compile_fail` UI tests. The
infrastructure pointers below apply across phases; reach for
them whenever a step asks for benchmarking or negative-case
testing.

- **Criterion benches** go in
  [`fp-library/benches/benchmarks/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/).
  Existing per-variant Free benches
  ([`free.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/free.rs),
  [`free_explicit.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks/free_explicit.rs),
  etc.) are the baseline shape. Wire new bench files into the
  `criterion_group!` registration in
  [`fp-library/benches/benchmarks.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/benches/benchmarks.rs).
- **`compile_fail` UI tests** go in
  [`fp-library/tests/ui/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/ui/).
  The
  [`fp-library/tests/compile_fail.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/compile_fail.rs)
  driver registers them via `trybuild::TestCases::new().compile_fail("tests/ui/*.rs")`,
  and `trybuild = "1.0"` is already in
  `fp-library/Cargo.toml`. Each negative case is one `.rs` file
  plus a sibling `.stderr` capturing expected error output;
  `.stderr` files are auto-generated on first run via
  `TRYBUILD=overwrite cargo test --test compile_fail` (use raw
  `cargo`, not `just test`, when bootstrapping `.stderr` files
  so the wip files do not persist under `fp-library/wip/`).
- **Probe / investigation tests** can also live in
  [`fp-library/tests/`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/).
  Existing examples include
  [`run_wrap_depth_probe.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs)
  (regression-guards a property load-bearing for the WrapDrop
  resolution) and
  [`free_explicit_poc.rs`](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/free_explicit_poc.rs)
  (integration-tests `FreeExplicit` against the questions the
  POC originally asked). Use the same shape when a step's work
  benefits from a self-documenting investigation as a test.

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
- **Complete a focused follow-up commit set** (e.g., the Phase 1
  follow-up `WrapDrop` migration's two commits, or the
  Phase 2 step 4a/4b split's two commits) and stop. The user
  reviews before proceeding to the next phase step that the
  follow-up unblocks.
- **Stop at the first blocker** you cannot resolve under the
  protocol above. Commit the active-blocker entry under
  plan.md's
  `Open questions, issues and blockers -> Active blockers`,
  summarise the blocker, and exit.

Do not work through multiple phases unprompted. Phases ship together
as a single feature release, but they review separately.

## Reference map

The four-corner doc taxonomy:

- [plan.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/plan.md):
  the active working spec. Phased steps, current progress, active
  blockers, success criteria. The authoritative answer to "what do
  I do next."
- [decisions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/decisions.md):
  frozen design rationale. The authoritative answer to "why this
  way." Do not edit.
- [resolutions.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/resolutions.md):
  append-only post-write log of resolved blockers. Holds full
  problem statements, investigations, alternatives considered,
  and rationale for each load-bearing question that paused
  implementation. Read this when plan.md's `Active blockers`
  section points at it for context, or when "why does X work this
  way?" cannot be answered from decisions.md alone.
- [deviations.md](file:///home/jessea/Documents/projects/rust-fp-lib/docs/plans/effects/deviations.md):
  append-only post-write log of per-step implementation choices
  that diverged from the plan text. Grouped by phase and step.
  Read this when "the code doesn't match the step description"
  needs explanation; append a new entry when your own work
  diverges.

Other reference material:

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
- [fp-library/tests/run_wrap_depth_probe.rs](file:///home/jessea/Documents/projects/rust-fp-lib/fp-library/tests/run_wrap_depth_probe.rs):
  regression test for the property the WrapDrop resolution relies
  on (Run-typical structural Wrap depth at most 1). Background
  investigation, see resolutions.md's "Resolved (2026-04-27): introduce WrapDrop trait..."
  entry.
- [CLAUDE.md](file:///home/jessea/Documents/projects/rust-fp-lib/CLAUDE.md):
  project-wide agent instructions including LSP usage.
- [AGENTS.md](file:///home/jessea/Documents/projects/rust-fp-lib/AGENTS.md):
  broader agent contract for this repo.
