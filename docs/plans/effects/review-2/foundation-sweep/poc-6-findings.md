# POC-6 Findings: row widening (`expand`) over the unified row, including a higher-order cell

Tier D, foundation sweep. Charter question: can `expand` widen a program containing a higher-order `listen` cell (which carries a sub-program) from one row into a larger row, preserving semantics, over the public `Free` stepping surface, with no boundary frames and no raw-step path?

Result: PASS, via Approach (B-nominal) (a row-parameterised cell over nominal recursive rows). The fallback (C) (trait-object `hfmap` cell) was not needed.

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_expand_listen.rs`. A public-API integration test over the real `Free` substrate, `Coyoneda`, and `CoproductBrand`/`CNilBrand`. Run via `just filtered test`. One test passes.

## Target property (restated from the Writer `listen` suite, per setup step S5)

Widening `listen(tell("first") >> tell("second") >> pure(40))`, then `(value, observed) -> (value + 2, observed)`, from the source row `R1` into the strictly larger row `R2` preserves the result `(42, "firstsecond")` and the propagated log "firstsecond" (the boundary-split suite's listen value). The point is that widening must rebuild the `listen` cell and its stored sub-program for the new row without changing what the program computes.

## What was built

- A first-order `Writer` effect (row-agnostic) and a row-parameterised higher-order `Listen` effect: `ListenBrand<R, RAction>` with `Of<'a, Next> = ListenCell<'a, R, RAction, Next>`, where `ListenCell` stores `action: Free<R, RAction>` (so the row `R` appears in the cell type) and `k: (RAction, String) -> Next`. The cell's hand-written `Functor` post-composes onto `k`; `R` and `RAction` are concrete, so it is object-safe.
- Two nominal rows. `R1 = { Writer, Listen<R1> }` and the strictly larger `R2 = { Writer, Listen<R2>, Other }`, each a marker struct whose `Kind`, `WrapDrop`, and `Functor` impls delegate to a coproduct alias that names the row itself in the `ListenBrand<Self, _>` position. Encoding the row nominally (rather than as a type alias) is what makes the self-reference legal.
- `expand::<R1, R2>`: a hand-written traversal over the public `resume`/`lift_f`/`bind`/`pure` surface. First-order layers are re-injected by manual `Inl`/`Inr` construction into the wider coproduct; the `listen` layer is rebuilt by recursively widening both its action sub-program and its continuation.
- An interpreter for `R2` (the same elaborate-`listen`-by-running-the-action pass as POC-5).

One test passes: the widened program evaluates to `(42, "firstsecond")` with the propagated log "firstsecond".

## Findings

1. `expand` over the unified row, including a higher-order cell that carries a sub-program, works over the public `Free` stepping surface with no boundary frames and no raw-step path. This directly confirms setup step S4's hypothesis: the W1 spike needed a private raw-step path only because default `Run`'s boundary frames had to be lowered before branch selection; with the boundary frames deleted (gate G2), `expand` is expressible over public `resume`/`lift_f`/`bind`. W1's other rejection reason (the per-`A` `CoproductEmbedder` evidence the fixed `NaturalTransformation::transform` cannot carry) is sidestepped by manual `Inl`/`Inr` injection, which is generic over the hole and needs no embedding trait bound.
2. The carrier-parameterised brand (Approach (B)) is viable in stable Rust once the row is a nominal `Kind`. The earlier worry that `type Row = Coproduct<..., ListenBrand<Row, _>, ...>` is a fatal type-alias cycle was correct only for a type alias: a nominal marker struct whose associated `Of` refers to `Self` in the `ListenBrand<Self, _>` position compiles, exactly as a recursive struct does, because `Free<Self, _>` is nominal and boxed. The cell stays fully type-safe (no erasure, no object-safety hazard), and the action result remains a brand type parameter as in POC-5.
3. The feared trait-resolution cycle did not occur. The chain `R1: WrapDrop` (delegated to the coproduct) needs `ListenBrand<R1, i32>: Kind`, whose `impl_kind` bound needs `R1: WrapDrop`. Because the `WrapDrop`/`Functor`/`Kind` impls for the nominal row are unconditional (no `where`-clause obligations), the solver treats `R1: WrapDrop` as an established fact and the re-derivation succeeds rather than looping. No GAT-normalization wall arose either; the delegating associated type `type Of = <Coproduct as Kind>::Of` resolved cleanly.

## Limitations and scope

- `expand` is a hand-written traversal rather than `Free::hoist_free` with a defunctionalised `NaturalTransformation`. The hand-written form is in the same spirit (public stepping, manual injection) and is clearer for the higher-order layer, whose action lives in a stored field rather than the continuation hole; `hoist_free` would need the natural transformation to recurse into that non-hole field itself. The production design can package `expand` either way; POC-6 proves the semantic feasibility, which is the gate question.
- The widening is monomorphic in the two concrete rows `R1` and `R2`. A generic `expand` over arbitrary source and target rows would use the brand-keyed membership machinery (POC-2) to choose injection positions; POC-6 fixes the rows to isolate the carrier-polymorphism question.
- The `Other` effect exists only to make `R2` strictly larger and is not used by the program; its interpreter arm is unreachable.
- The shared deep-`Wrap`-nesting drop limitation recorded in POC-3 applies here too (Coyoneda cells), and is not exercised by this queue-deep program.

## Bearing on Tier D and the design

POC-6 completes Tier D. With POC-4 (same-result `Catch`), POC-5 (result-shape-changing `listen`), and POC-6 (`expand` over the unified row including a higher-order cell) all passing, the unified row supports the full higher-order story FS-1 needs: elaboration of both same-result and result-shape-changing effects, and row widening, all over the public substrate surface with no boundary frames. The carrier-parameterised, row-parameterised cell over a nominal recursive row is the confirmed encoding for higher-order cells that carry sub-programs.
