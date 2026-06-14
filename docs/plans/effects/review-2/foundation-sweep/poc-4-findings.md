# POC-4 Findings: elaborate a same-result higher-order effect

Tier D, foundation sweep. Charter question: can a same-result higher-order effect (`Catch`, whose action result equals its operation result) be elaborated into first-order `Throw` plus interpose, reproducing the boundary-frame semantics?

Result: PASS (elaboration arm; the weave arm was not needed for this same-result case).

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_elaborate_catch.rs`. A public-API integration test over the real `Free` substrate with a uniformly `Coyoneda`-wrapped `State` + `Throw` row. Run via `just filtered test`. Two tests pass.

## Target property (restated from the heftia suite, per setup step S5)

From `run_heftia_semantics.rs` (`state_write_before_caught_throw_survives_handler_orders`): the program `put(true) >> throw`, protected by `catch(_, recover = pure(()))`, then `get`, yields value `true` and final state `true`. The state write performed before a caught throw survives the catch (catch is not transactional over state). Both handler orderings give `(true, true)`.

## What was built

- A first-order `State` effect (`Get`/`Put` over a `bool` cell) and a first-order `Throw` effect (unit error, phantom result), each `Coyoneda`-wrapped in the row. `State` and `Throw` carry hand-written `Functor` impls (needed so the interpreter can `Coyoneda::lower` to recover the operation).
- `elaborate_catch(action, recover)`: the elaboration. It walks the action program; a `State` layer is rebuilt with each continuation recursively elaborated; a `Throw` layer is replaced by `recover`. This is heftia's `runCatch` semantics, `action & interposeWith (\Throw _ -> recover)`, implemented as a peel/rebuild interpose over `Throw`.
- `catch(action, recover)` is defined as exactly this elaboration.
- A plain peel-loop interpreter for the first-order `State` + `Throw` row (state in a cell; `Throw` aborts).

Two tests pass: the heftia state-write-survives-catch case yields `(true, true)`, and an uncaught throw aborts with the prior write intact.

## Findings

1. Elaboration of a same-result higher-order effect into first-order `Throw` plus interpose reproduces the boundary-frame semantics, on the real `Free` substrate, with no boundary frames and no result-polymorphic protocol traits. The interpose is a direct peel/rebuild over the action: replace `Throw` with `recover`, pass other effects through.
2. State writes survive the catch automatically. Because elaboration only rewrites `Throw` nodes and leaves `State` nodes in place (running against the same outer state cell), the write performed before the throw is already committed when the throw is caught. This is the algebraic semantics the dual row achieves with boundary frames, obtained here for free by the structure of the interpose.
3. The mechanical requirement is `Coyoneda::lower` (hence a `Functor` impl per effect). An interpreter or elaboration that inspects an operation must lower the Coyoneda cell, which needs the effect's `Functor`. This is the per-effect `Functor` that a `define_effect!` macro would generate; it is not extra authoring cost in the end state.

## Limitations and scope

- The elaboration is applied at construction (`catch` builds the interposed program directly) rather than as an interpret pass over a `Catch` node in the row. For this same-result case the two are semantically equivalent (the interpose is the same rewrite either way); the production design would run elaboration as an interpret pass (heftia's `runCatch`) so `Catch` can sit in the row and be handled in any order. POC-4 proves the semantic equivalence, which is the feasibility question; the interpret-pass packaging is mechanical.
- This is the easy higher-order case: action result equals operation result. The load-bearing case is POC-5 (`listen`/`censor`, where the action result differs from the operation result), which is what forced today's boundary/carrier split and which drives gate G2.
- `recover` is held as `Rc<dyn Fn() -> Free<Row, R>>` so it can be substituted at every `Throw` site (structurally there may be several). This matches the requirement that a scoped recovery be reusable across the action's throw sites.

## Bearing on gate G2

POC-4 establishes that the elaboration mechanism works and reproduces the algebraic semantics for the same-result higher-order case. It does not by itself decide G2: that turns on POC-5, whether a result-shape-changing higher-order effect can be elaborated (or, failing that, woven) without boundary frames. POC-4 is the necessary precondition (elaboration works at all) and a positive signal, but POC-5 is the make-or-break.
