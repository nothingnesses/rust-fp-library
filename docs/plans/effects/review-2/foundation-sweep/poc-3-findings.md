# POC-3 Findings: unified row over the real `Free` substrate

Tier A, foundation sweep. Charter question: can one row hold both a first-order and a higher-order effect (each a valid `Functor`/`WrapDrop` row cell), support an order-directed `peel` that tells the two kinds apart, and drop a deep chain without overflowing the stack?

Result: PASS (with one recorded shared limitation, below).

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_mixed_row.rs`. A public-API integration test over the real `Free` substrate, `Coyoneda`, and the real `CoproductBrand`/`CNilBrand` row, per setup steps S2 and S3. Run via `just filtered test`. Three active tests pass; one is ignored as documentation of a shared limitation.

## What was built

A unified row holding a first-order effect (`OutBrand`, an emit-and-continue effect) and a higher-order effect (`ScopeBrand`, which carries an action sub-program plus the continuation), each uniformly `Coyoneda`-wrapped:

```text
Row = CoproductBrand<CoyonedaBrand<OutBrand>, CoproductBrand<CoyonedaBrand<ScopeBrand>, CNilBrand>>
```

Because `CoyonedaBrand<F>` supplies `Functor` and `WrapDrop` for any `F: Kind`, and `CoproductBrand`/`CNilBrand` compose those, the row is a valid `Free<Row, A>` substrate and the custom effects need only an `impl_kind!` projection, no hand-written `Functor`/`WrapDrop`. This validates setup step S3's uniform-Coyoneda choice end to end on the real substrate.

Order-directed peel: after `Free::resume` yields the suspended row coproduct, a `ClassifyActive` walk reads each arm's order off the `Coyoneda` cell via a value-level `HasOrder` adapter (`impl<E: OrderedEffect + Kind> HasOrder for Coyoneda<E, A> { type Order = E::Order; }`) and returns a runtime `OrderTag`. The two tests confirm a peeled first-order effect classifies `First` and a peeled higher-order effect classifies `Higher`. This is the unified-row analog of the dual row's `Node::First`/`Node::Scoped` split, recovered at runtime from one row by reading the order marker.

Deep-drop: a deep bind-chain (the realistic, queue-deep program shape) of depth 100,000 drops without overflowing.

## Findings

1. The unified row composes with the real `Free` substrate with no friction beyond one expected bound. The only compile error was that `Coyoneda<'a, E, A>` requires `E: Kind`, so the `HasOrder` adapter needed `E: Kind` added; no GAT-normalization issue arose. Custom effects over the unified row are `impl_kind!`-only, which is a real ergonomic win and confirms the uniform-Coyoneda decision.
2. Order-directed peel works: one row, peeled once, the active effect classified first-order vs higher-order by its order marker read off the Coyoneda cell. This is the mechanism a unified-row interpreter would use to route to first-order handling versus the higher-order mechanism (Tier D), replacing the dual row's structural `Node` split.
3. Realistic deep programs drop safely. A 100,000-deep bind-chain (continuation-queue-deep, the shape real effect programs take) drops without overflow over the unified row, the same as the dual row.

## Recorded limitation (shared with the current design, not a regression)

A deep chain built by explicit `Free::wrap` nesting of `Coyoneda`-wrapped cells overflows on drop. The cause: `CoyonedaBrand::drop` returns `None` (it cannot generically extract the inner program without knowing the wrapped functor), so `Free`'s iterative `WrapDrop` drain cannot drain a `Coyoneda`-tipped `Wrap` spine, and the recursive `Drop` overflows at depth. This is a property of the existing `Coyoneda` row encoding, not of the unified row: the library's own deep-wrap-drop stack-safety test uses `ThunkBrand` (whose `drop` returns `Some`, so it drains), and the dual row's first-order rows are `Coyoneda`-wrapped too. Realistic effect programs are queue-deep, not `Wrap`-nesting-deep, so this does not arise in practice; it is kept as an ignored test documenting the shared limitation. If a future construction produces deep `Wrap`-nesting of `Coyoneda` cells, it would need the same treatment under either design (for example a `WrapDrop` for `CoyonedaBrand` that can surrender the inner program, which requires the wrapped functor to expose it).

## Limitations and scope

- POC-3 proves the row holds and distinguishes both kinds and drops safely; it does not interpret the higher-order effect. Elaborating or weaving `ScopeBrand` is Tier D (POC-4/POC-5).
- `ScopeBrand` carries its action and continuation in the same `A` (continuation) position; the result-shape-changing higher-order case (`listen`, where the action result differs from the operation result) is the harder Tier D case and is not exercised here.

## Gate G1 input

Tier A is complete and all four POCs pass: classification (POC-0), partition (POC-1), brand-keyed dispatch (POC-2), and the unified row over the real substrate with order-directed peel and drop safety (POC-3). The cheap shared machinery of the unified row is therefore proven feasible on the real encoding. The G1 decision is recorded in the charter's Decision gates section.
