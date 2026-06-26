# FS-1 Catalog-Port Fan-Out: Per-Effect Task Template

This is the concrete per-effect porting task each `agent-images` fan-out agent
receives (remediation item 4 step 7.3, executed by step 7.4). It codifies the
recipe proven by the solo `Fresh` (step 7.1) and `Input` (step 7.3) ports.

The integration mechanism is OQ-7C approach A (folded into plan step 7.3): the
agent authors one self-contained effect file and applies the shared-file
snippets in its own worktree to validate locally; the orchestrator then
re-applies those snippets to the canonical `fs1.rs` at the marked `FAN-OUT
ANCHOR` comments, in sequence (single writer, collision-free), and re-runs the
cumulative bucket-A suite. The agents' shared-file edits are throwaway (local
validation only); the orchestrator does not git-merge them.

## Inputs (given to the agent)

- The effect's dual-row source: `fp-library/src/types/effects/<effect>.rs` (the
  `define_effect!` invocation) and its handler in
  `named_helpers/<effect>.rs` (the interpretation logic to reproduce).
- The effect's bucket-A test and asserted result, from
  [parity-oracle-inventory.md](parity-oracle-inventory.md) (the semantics to
  preserve; the API calls are rewritten, the asserted results are not).
- The conventions: the worktree's `AGENTS.md` and the auto-loaded project
  memory (the FS-1 design and the slice's established patterns).
- An existing effect module as the pattern to copy: a first-order effect copies
  `fs1/input.rs`; a higher-order effect copies `fs1/catch.rs` or
  `fs1/censor.rs`.

## Required output (what the agent produces)

1. A new self-contained module `fp-library/src/types/effects/fs1/<effect>.rs`
   exposing:
   - the effect definition: the brand (`<Effect>Brand`), the functor payload
     (`<Effect>F`), the `impl_kind!`, the `impl Functor`, and the
     `impl OrderOf` (`FirstOrder` or `HigherOrder`);
   - the smart constructor(s), each building the `Coyoneda` cell and injecting
     it type-directed via a type-pinned `let node: Node<_> = Coproduct::inject(coyo);`
     (never naming a row position);
   - a `#[cfg(test)] mod tests` with the bucket-A parity test, importing
     constructors via the flat `crate::types::effects::fs1::{...}` path and
     building a `Handlers` from the `Fixture`.
2. The shared-file snippets it applied locally and hands back, each at its
   `FAN-OUT ANCHOR`:
   - `mod <effect>;` (effect-module anchor);
   - `<effect>::<ctor>` added to the `pub(crate) use self::{...}` re-export
     (rustfmt sorts it, so no anchor line is needed there);
   - `<effect>::{<Brand>, <F>}` added to the `use self::{...}` import (sorted);
   - a `Row` cell, wrapping the terminal `CNilBrand` as
     `CoproductBrand<CoyonedaBrand<<Brand>>, CNilBrand>` (Row-tail anchor);
   - a `run` dispatch arm: a brand-keyed `uninject` block that reads the
     effect's `Handlers` field (or, for a higher-order effect, recursively
     `run`s its sub-program, sharing or scoping the `Handlers` as the semantics
     require) and sets `program` (dispatch-arm anchor);
   - if the effect carries handler state: a `Handlers` field (`&'h ...`), a
     matching `Fixture` field, a `new()` default, an optional `with_<effect>`
     seeder, and a `handlers()` binding (Handlers-field and Fixture-builder
     anchors).

## Steps

1. Copy the closest existing effect module as the starting pattern.
2. Define the brand, payload, `impl_kind!`, `Functor`, `OrderOf`, and
   constructor; reproduce the dual-row handler's interpretation in the dispatch
   arm.
3. Apply the shared-file snippets at the `FAN-OUT ANCHOR` comments in the
   worktree's `fs1.rs`.
4. Validate locally (see below).
5. Hand back the module file and the snippets.

## Validation command

- `just test --features effects --lib <effect>` (the effect's bucket-A parity
  test green).
- `just clippy --features effects --all-targets` (clean; the project runs
  `-D warnings`).

## Dependency ordering and scope

Only capability-present bucket-A effects port here (the Box-store, single-shot,
`'static` slice):

- First-order, independent, port in parallel: `Input` (done), `KVStore`, and
  `Empty` if its short-circuit is single-shot.
- Higher-order, elaborated like `Catch`/`Censor`, port after the first-order
  constructors their parity tests compose with exist: `Listen`, `Local`,
  `Bracket`, `Except`, `Interpose`.
- Deferred to Phase D (they need a capability the slice lacks): `Choose`/NonDet
  (multi-shot stepping), `Span` (non-`'static` payload), `Coroutine` (item 16).
  An effect found during porting to need a Phase-D capability defers and
  surfaces.

## Orchestrator integration (OQ-7C approach A)

For each returned effect, in sequence (single writer, collision-free):

1. Copy the module file into `fs1/`.
2. Apply the snippets at the canonical `fs1.rs` anchors.
3. `just fmt`, then re-run the cumulative bucket-A suite
   (`just test --features effects --lib fs1`) and
   `just clippy --features effects --all-targets`.

When the capability-present bucket-A subset is green on FS-1, each ported
behaviour's dual-row code is unblocked for item 4 step 4's deletion.

## Worked example: Input (step 7.3)

`fs1/input.rs` is the reference first-order port. Its shared-file snippets were:

- `mod input;`;
- `input::input` (re-export) and `input::{InputBrand, InputF}` (import);
- the `Row` cell `CoproductBrand<CoyonedaBrand<InputBrand>, CNilBrand>`;
- the dispatch arm reading `handlers.input.borrow_mut().pop_front()`;
- the `Handlers` field `input: &'h RefCell<VecDeque<&'static str>>`;
- the `Fixture` field, its `VecDeque::new()` default, the
  `with_input(items)` seeder, and the `handlers()` binding.

Its bucket-A test asserts three `input()` calls over `["red", "blue"]` yield
`(Some("red"), Some("blue"), None)`, matching the dual-row sequence runner.
