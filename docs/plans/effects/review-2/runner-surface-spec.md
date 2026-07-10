# The public runner surface specification (items 8 and 14)

This document settles the public interpretation surface that items 8 and 14 converge on, per the OQ-8A decision (one joint design round, owned by item 8, before either item's implementation stage). It is grounded in three inputs: the `pub(crate)` `fs1` reference interpreter (the semantics oracle: a monolithic `run(program, &Handlers) -> Result<A, Abort>` with brand-keyed dispatch arms, elaboration by direct recursion, and handler scoping by struct-update over `Copy` cell references), the public hand-written dispatch-loop model the custom-effects guide teaches (the only public interpretation surface today, per the OQ-11E finding), and purescript-run's runner vocabulary (`Run.purs`: the `runAccum` family, `runPure`, and the per-effect narrowing runners such as `runState`, whose loop re-emits unmatched layers into the residual row).

## Scope and baseline

In scope: the shape of the public runner surface (signatures, semantics, naming), enough to unblock item 14's implementation stage (threaded runners, the NonDet zoo) and item 8's handler-surface implementation (the handler-list type and the `define_row!` extension). Out of scope, recorded as deferrals below: the multi-shot `Choose` fork mechanics, label variants (item 9), the payload-generalised public catalog (OQ-4I), and async (item 18).

The baseline substrate facts the surface builds on: a row is a nominal brand whose kind projection is a coproduct of `Coyoneda`-wrapped effect cells; `Free<Row, A>` steps by `resume()` into `Ok(value)` or `Err(layer)`; a layer dispatches by type-directed `uninject`, lowers by `Coyoneda::lower`, and re-embeds a remainder by `embed`; `Free::lift_f` constructs a program from a layer whose holes are results (as distinct from `Free::wrap`, whose holes are continuation programs); `bind` is O(1) via the continuation queue.

## The interpretation model: two tiers

The surface has two tiers with distinct jobs, mirroring purescript-run (which ships both the per-effect runners and the generic `run`/`runAccum` loops):

- **Tier 1, narrowing runners (the compositional primitives).** A runner eliminates one effect family from the row and returns the residual program: `Free<Row, A> -> Free<Narrow, B>`, where `Narrow` is the row without the eliminated cells and `B` reifies the effect's outcome at its own boundary (`(S, A)` for state-like accumulators, `Result<A, E>` for aborts). Runners stack in any order, and handler-ordering semantics (global-versus-branch-local state, throw-past-versus-caught) fall out of the stacking order; this is the tier item 14's threaded runners live in, and the tier that makes its "both handler orders" zoo cases expressible at all.
- **Tier 2, the one-pass handler list (the full-elimination convenience).** For the common case (eliminate every effect at once, one walk, no intermediate re-wrapping), a brand-keyed handler-list value drives a single loop, the generic form of the `fs1` reference interpreter. This is item 8's remaining scope: the handler-list type, its order-insensitive construction, and the `define_row!` extension that emits both per row.

Two consequences of the tier-1 model dissolve open design questions:

- **The monolithic abort channel disappears from the public surface.** The `fs1` `Abort` enum is an artifact of the all-at-once loop (one loop needs one error type). In the compositional model each aborting effect reifies at its own runner boundary, purescript-run's `runExcept :: Run (EXCEPT e + r) a -> Run r (Either e a)` shape, so the public surface needs no closed global abort enum and no public `Abort` type.
- **Interception needs no dedicated method (the item 13 criterion resolves compositionally).** The deleted `handle_with_either`'s capability, drive the whole program but surrender the first matched operation with its continuation intact, is expressed by narrowing every other effect away and calling `resume()` on the residual single-effect program: `Ok` is the completed value, `Err` is the surrendered operation with its continuation. No wrongly-named method returns, no driver-style rename is needed, and `run_except`'s narrowing shape becomes the general tier-1 shape rather than a special case.

## Tier 1: the accumulator core and the runner family

The generic core is `handle_accum`, the pure-Rust translation of purescript-run's `runAccumPure`, generic over the eliminated effect brand:

```rust,ignore
fn handle_accum<EBrand, Row, Narrow, S, A>(
	s: S,
	program: Free<Row, A>,
	step: impl Fn(S, Op<EBrand, Row, A>) -> (S, Free<Row, A>) + 'static,
) -> Free<Narrow, (S, A)>
```

where `Op<EBrand, Row, A>` abbreviates the lowered operation type, the effect brand's kind projection at `Free<Row, A>` (what `Coyoneda::lower` yields). The loop:

- `resume()` yields `Ok(value)`: return `Free::pure((s, value))`.
- `Err(layer)`, `uninject` matches `EBrand`: lower, apply `step(s, op)` to get the new accumulator and the next program, and iterate (native stack constant over matched runs).
- `uninject` misses: embed the remainder into `Narrow` at value type `Free<Row, A>` (the layer's holes become results, not continuations), lift it with `Free::lift_f`, and `bind` the recursive continuation: `lift_f(embedded).bind(move |rest| handle_accum(s, rest, step))`. The accumulator moves into the deferred continuation (sequential threading needs no `Clone` on `S`; only the multi-shot fork will), and the deferral makes the walk constant-stack per step, the same structural-deferral property the item 2 suites pinned for widening and rewriting walks.

The per-effect runner family is instances of this core (plus non-accumulator variants of the same loop shape), one runner per effect family, where a family is an effect's first-order operations together with the higher-order operations scoped to it:

- `handle_state(s0, program) -> Free<Narrow, (S, A)>`, with `eval`/`exec` projections derived.
- `handle_writer(program) -> Free<Narrow, (W, A)>` and a `fold_writer` generalisation; the `Writer` family includes `Listen` and `Censor`, elaborated inside the runner.
- `handle_reader(env, program) -> Free<Narrow, A>`; the `Reader` family includes `Local`.
- `handle_throw(program) -> Free<Narrow, Result<A, ()>>` with the `Catch` elaboration in the same family; `handle_except::<E>` is the typed sibling.
- `handle_choose(program) -> Free<Narrow, F<A>>` (the `Alternative`-collecting runner) is designed here and implemented at the multi-shot stage (deferrals below).

Elaboration in tier 1 is recursive self-application: a higher-order cell's sub-program is run through the same runner (`Catch` runs its action through the `Throw`/`Catch` runner; on the reified abort it runs the recovery), composing in the `Free<Narrow, _>` monad, so the elaboration recursion contract (native stack grows with nesting depth, not program length) carries over from the reference interpreter unchanged.

Ordering semantics are pinned by one reference case: applying `handle_state` outermost (last) reproduces the reference interpreter's bucket-A semantics (a write before a caught throw survives, the shared-cell "global across branches" declaration in the effects guide); applying it innermost scopes the accumulator per enclosing elaboration. Item 14's zoo cases pin both orders against heftia's answers.

Terminal extraction: once every cell is eliminated the residual is `Free<EmptyRow, A>`, and `extract(program) -> A` (purescript-run's `extract`) closes the pipeline; its loop's `uninject` remainder is uninhabited, so the match is total with no wildcard.

## Tier 2: the handler list

The one-pass loop is `handle(program, handlers) -> R`, where `handlers` is a per-row struct that `define_row!` emits: one field per cell holding that effect's arm (for first-order cells a resumptive closure over the lowered operation; for higher-order cells an elaboration arm that receives the sub-programs and a re-entry handle), plus the row's order-insensitive constructor (named-field struct construction is already order-insensitive in Rust; the macro's job is emitting the struct, its lifetimes over borrowed handler state, and the loop that dispatches to it). The `fs1` interpreter's `Handlers` struct (`Copy` over cell references, scoping by struct-update) is the existence proof and the emission template. The exact emission grammar is item 8's handler-surface implementation step, designed against this tier's pinned shape; it is deliberately not frozen here because the tier-1 POC does not exercise it and freezing untested emission detail is what the define-effect spec's staged validation avoided.

## Naming resolutions (items 12 and 13)

- The interpretation vocabulary is `handle` (item 12's adopted decision: one vocabulary, no alias pairs): `handle` for the tier-2 loop, `handle_accum` for the tier-1 core, `handle_<effect>` for the runner family, `extract` for the terminal step. No `run`/`run_rec` names ship; the purescript-run correspondence table in the effects guide maps `runState`-family names to `handle_<effect>` when the runners land.
- Item 13's criterion (interception distinct from narrowing, under an honest name) is satisfied structurally: interception is narrowing plus `resume()`, so no interception method exists to misname. If a convenience wrapper is ever added, it names the surrender (`resume_residual`-style), not `handle`.

## Recorded deferrals

- **The multi-shot `Choose` fork.** `Choose` (`#[multi_shot] fn alt() -> bool`, the define-effect spec's non-emitting instance) requires re-callable continuations, so `handle_choose` and the accumulator fork (`S: Clone`, forked per branch) implement at the multi-shot stores in item 14's implementation stage, which also carries the cell-storage parameterisation that multi-shot rows need. The tier-1 signatures above are store-agnostic by design; the POC validates them at the Box store.
- **Label variants** (item 9): the `handle_<effect>_at`-style tagged forms follow the label-brand mechanism; nothing here blocks them.
- **The payload-generalised public catalog** (OQ-4I): rides the runner family's implementation; the catalog's runners are the `handle_<effect>` instances above with payloads generalised from the slice's pinned `i32`/`String`.
- **Store parameterisation of the runner family**: the family lands Box-first; the multi-shot instantiations follow item 14's stage.

## Validation: the proof of concept

The risky half is tier 1's generic bounds: a single generic `handle_accum` must name, in a `where` clause over four type parameters, the row's resume machinery, the type-directed `uninject` at the eliminated brand, the remainder's `embed` into the narrow row at a program-valued type, and `lift_f`/`bind` on the residual free monad; the item 2 suites proved these operations on concrete rows, not in generic position. The POC therefore is: on `feat/effects-fs1`, a test file defining a small row and narrow row with the public macros, a generic `handle_accum` written once against the public surface, and two distinct instantiations (a state-threading step and a log-folding step) plus a depth case on the order of 100k steps proving the deferred unmatched-path walk is constant-stack. Pass criteria: the generic function compiles with both instantiations, the threading observations match a hand-threaded oracle, and the depth case passes under `just verify` and the effects-off build. Fallback per principle 3: if the generic bounds prove unwritable in stable Rust, the runner family is emitted concrete-per-row by `define_row!` (the row macro already owns per-row emission, and the define-effect spec's constructor fallback set the precedent), preserving the public surface at the cost of monomorphic runner homes; if that also fails, surface the impasse.

Outcome: passed, with no fallback needed (commit `805a32ab` on `feat/effects-fs1`, `fp-library/tests/effects_narrowing_runner.rs`, under the full `just verify` and effects-off gates). The generic `handle_accum` compiles once against the public surface; both instantiations pass (state threading over the counter brand into a tick-only residual row, and log folding over the tick brand into a counter-only residual row), the threading oracle matches program order, and both 100k depth cases pass (matched-iterative and unmatched-deferred). Findings that sharpen the signatures above: the bounds are plain `kinds::LifetimeUnaryKind` projections (the public rename of the macro-generated kind trait), so no type-level macros appear in the generic signature; the two coproduct index parameters (`UninjectIndex`, `EmbedIndices`) are inferred at every call site under the all-or-underscore turbofish idiom; sequential threading needs no `Clone` on the accumulator or the step function (both move into the deferred continuation), so `S: Clone` is a requirement of the multi-shot fork only; and `bind` in generic position needs its receiver's store pinned by an explicit annotation (the per-store inherent `bind` impls are otherwise ambiguous under E0034), the generic-position sibling of the one-unified-definition constructor rule.
