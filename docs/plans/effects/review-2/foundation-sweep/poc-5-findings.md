# POC-5 Findings: elaborate a result-shape-changing higher-order effect

Tier D, foundation sweep. Charter question: can a result-shape-changing higher-order effect (`listen`, whose operation result `(RAction, W)` differs from its action result `RAction`, the exact case that forced today's boundary/carrier split) live in the unified row as a single cell and be elaborated by a plain interpret pass, with no boundary frames, no result-polymorphic protocol traits, and no scoped row?

Result: PASS (elaboration arm; the weave arm was not needed). This is the make-or-break for gate G2.

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_elaborate_listen.rs`. A public-API integration test over the real `Free` substrate with a uniformly `Coyoneda`-wrapped `Writer` + `Listen` row. Run via `just filtered test`. Two tests pass.

## Target property (restated from the Writer `listen` suite, per setup step S5)

From the boundary-split suite (`run_writer_listen_boundary_split.rs`): `listen(tell("first") >> tell("second") >> pure(40))`, then `(value, observed) -> (value + 2, observed)`, yields `(42, "firstsecond")`, and the two tells still propagate to the outer Writer. The operation result `(i32, String)` differs from the action result `i32`: that difference is exactly what the dual-row scoped cell encodes with two result parameters (for example `BoxWriterListen<'a, P, String, i32, ListenResult>` carries both the action result `i32` and the continuation result `ListenResult`).

## What was built

- A first-order `Writer` effect (`Tell` over a `String` log), `Coyoneda`-wrapped, with a hand-written `Functor`.
- A higher-order `Listen` effect as an in-row cell: `ListenBrand<RAction>` with `Of<'a, Next> = ListenCell<'a, RAction, Next>`, where `ListenCell` stores the action sub-program `Free<Row, RAction>` and a continuation `k: (RAction, String) -> Next`. The action result `RAction` is a brand type parameter (the adopted Approach (c) from the charter), instantiated at `i32` for this case. A hand-written `Functor` post-composes onto the hole `k`; because `RAction` is a concrete type there, this is object-safe.
- The interpret pass `run`: `Writer` reads and prepends each tell to the accumulated log; `Listen` is elaborated by running the action sub-program, observing its accumulated log, feeding `(value, observed)` into the continuation `k`, and re-emitting the action's log into the outer total. This is heftia's `runListen` semantics expressed over the unified row.

Two tests pass: the result-shape-changing case yields `(42, "firstsecond")` with the tells propagated (total log "firstsecond"), and a second test shows `observed` is scoped to the action while a surrounding tell still reaches the global log.

## Findings

1. A result-shape-changing higher-order effect is elaborated over the unified row by a plain interpret pass, reproducing the boundary-split semantics, with no boundary frames, no result-polymorphic protocol traits, and no scoped row. The cell holds the action and a continuation typed to the changed result shape `(RAction, W)`; the interpret pass recovers that continuation by `Coyoneda::lower` and applies it after running the action.
2. The result-shape change is carried by the cell's continuation type, not by a separate carrier mechanism. When `listen(action).bind(f)` is built, `f` accumulates into the `Coyoneda` map over the listen cell; `lower` (which needs `ListenBrand: Functor`) pushes it into the stored continuation `k: (RAction, W) -> Free<Row, Next>`. So after `resume().lower()`, `k` is the post-listen continuation, already typed to consume `(RAction, W)`. The dual row spends the boundary-frame subsystem and the result-polymorphic protocol traits to thread this shape change; here it falls out of the cell's own continuation type.
3. The action result is a brand type parameter, not existentially hidden, and this is the adopted Approach (c). Hiding `RAction` is blocked in stable Rust: the unified row `Coyoneda`-wraps every cell, so the interpret pass must call `Coyoneda::lower`, which is bounded `where F: Functor`, and `Functor::map` must post-compose onto `k: (RAction, W) -> Next`; if `RAction` were erased behind a trait object that post-composition would need a generic method on the trait object, which is not object-safe. Carrying `RAction` as a brand parameter is no worse than the current design (which carries it too, in `WriterListenBrand<P, W, A>`) and does not reintroduce the expensive carrier-split machinery, because elaboration is a plain interpret pass over the one row.

## Limitations and scope

- This is the single-Writer `listen` case, which is the make-or-break result-shape-changing case (action result differs from operation result). The NonDet+Writer `listen` interaction (branch-local versus global accumulation) is a separate Axis-2/R5 concern and belongs to the integration tier (POC-9), not to this feasibility probe.
- As in POC-4, the elaboration runs as part of the interpret pass (the interpreter elaborates `listen` when it peels the cell) rather than as a standalone reusable `runListen` interpret pass over a `Listen` node handled in arbitrary order. For this case the two are semantically equivalent (the rewrite is the same either way); the production design would package elaboration as an order-independent interpret pass. POC-5 proves the semantic feasibility, which is the gate question; the packaging is mechanical.
- The action carries only `Writer` here. Nested `Listen` inside the action is a generalisation not needed for the make-or-break and is not exercised.
- Harness note: the `just test` recipe keys its output cache on `git ls-files`, so an untracked spike test file replays stale cached output; the POC file must be `git add`ed for edits to take effect.

## Bearing on gate G2

POC-5 is the make-or-break for gate G2, and it passes by elaboration. Together with POC-4 (same-result `Catch`), both higher-order cases the sweep targets reproduce the heftia semantics by elaboration over the unified row with no boundary frames. The boundary-frame subsystem and the result-polymorphic protocol traits are therefore deletable, and FS-1 (unified row + elaboration) is the selected higher-order mechanism. Weave (FS-2) is not needed as the primary mechanism and remains available only for the later exponential-effect round. The G2 decision is recorded in the charter's Decision gates section.
