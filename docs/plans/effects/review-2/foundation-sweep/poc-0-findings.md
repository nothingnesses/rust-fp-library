# POC-0 Findings: order classification over a unified row

Tier A, foundation sweep. Charter question: can each effect brand carry an order marker, and can a recursive `AllFirstOrder` constraint over the library's real `CoproductBrand` row reject a row containing a higher-order effect with a readable error?

Result: PASS.

POC code (kept on the `spike/foundation-sweep` branch, not on `feat/effects`): `fp-library/tests/poc_fs_order_marker.rs` and the compile-fail companion `fp-library/tests/ui/poc_fs_order_marker_hoe.rs` (plus its golden `.stderr`). Built on stable Rust with `--features effects`, using the real `CoproductBrand`/`CNilBrand` row primitives and no `Run`/`Node`/handler machinery, per setup step S2.

## What was built

The Rust analog of heftia's classification layer, three pieces:

- `OrderedEffect { type Order; }` with order markers `FirstOrderMark` / `HigherOrderMark`: the analog of the `OrderOf e` type family. Every effect brand declares its order.
- `FirstOrderEffect`: a directly-implemented marker (the analog of heftia's `FirstOrder` class, which heftia also keeps separate from `OrderOf`). First-order effect brands implement it; higher-order ones do not.
- `AllFirstOrder`: a recursive trait over the real `CoproductBrand<H, T>` chain terminating in `CNilBrand` (`impl AllFirstOrder for CNilBrand`; `impl<H: FirstOrderEffect, T: AllFirstOrder> AllFirstOrder for CoproductBrand<H, T>`). The analog of heftia's `FOEs es` constraint.

Three positive tests confirm the empty row and a pure first-order row (`State`-like + `Reader`-like effects) satisfy `AllFirstOrder`, and that the first-order effects are classified first-order. One trybuild compile-fail test confirms a row containing a `Catch`-like higher-order effect is rejected.

## Key finding: marker trait, not associated-type blanket, for diagnostics

The first attempt derived `FirstOrderEffect` via a blanket impl keyed on the associated type, `impl<E: OrderedEffect<Order = FirstOrderMark>> FirstOrderEffect for E {}`. That classifies correctly and rejects higher-order rows, but the rejection surfaces as an `E0271` associated-type mismatch (`<CatchBrand as OrderedEffect>::Order == FirstOrderMark`), which `#[diagnostic::on_unimplemented]` cannot customise (that attribute only applies to `E0277` "trait not implemented"). The error was still readable and did name `CatchBrand`, but the crafted message did not appear.

Making `FirstOrderEffect` a directly-implemented marker (first-order brands `impl` it; higher-order brands do not) routes the failure through `E0277`, so the attribute fires. The resulting top-line error is:

```text
error[E0277]: the effect `CatchBrand` is not a first-order effect
  --> tests/ui/poc_fs_order_marker_hoe.rs:47:28
   |
47 |     require_all_first_order::<HoRow>();
   |                               ^^^^^ this effect is higher-order (or unclassified) but appears in a first-order-only row
   |
   = note: `AllFirstOrder` requires every effect in the row to be first-order; a higher-order effect must be elaborated or woven away before first-order interpretation
```

This is also the heftia-faithful shape: `data-effects` keeps `FirstOrder` (a marker class, derived per effect) separate from the `OrderOf` type family, for reasons that turn out to include exactly this diagnostic behaviour. The redundancy between `OrderedEffect::Order` and the `FirstOrderEffect` marker is not a per-effect authoring cost, because a `define_effect!`-style macro would emit both from one effect spec.

## Decisions this settles for later POCs and the production design

1. The unified row carries order as a directly-implemented `FirstOrderEffect` marker plus an `OrderedEffect::Order` companion associated type, mirroring heftia's `FirstOrder` + `OrderOf` split. The marker is what `AllFirstOrder` (the `FOEs` analog) and any first-order-only interpreter bound builds on; the associated type is available for the partition (POC-1) and any order-directed dispatch.
2. The classification composes with the library's real `CoproductBrand`/`CNilBrand` encoding with no friction (no GAT-normalization issue arises here, because `AllFirstOrder` matches the row type structurally and does not project the brands' `Kind`).
3. Effect definitions in the eventual `define_effect!` macro must emit the `FirstOrderEffect` marker for first-order effects (and withhold it for higher-order ones); the order associated type alone is insufficient for good diagnostics.

## Limitations and scope

- The prototype rows use bare effect brands, not `CoyonedaBrand`-wrapped ones. Whether the row wraps effects in Coyoneda is fixed to uniform-Coyoneda for the feasibility tier by setup step S3 and does not affect the order classification (the marker and `AllFirstOrder` are independent of the Kind projection). POC-3 exercises the Coyoneda-wrapped row.
- This POC proves classification and rejection only. It does not exercise dispatch (POC-2) or any interpreter; "reject a higher-order row from first-order-only handling" is the property demonstrated, not "interpret a mixed row".
- Per setup step S2, the POC is throwaway and exempt from the full `just verify` gate; it compiles and its assertions and trybuild case pass.
