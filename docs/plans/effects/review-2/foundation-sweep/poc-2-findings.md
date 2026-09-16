# POC-2 Findings: brand-keyed handler dispatch

Tier A, foundation sweep. Charter question: can handlers be dispatched by effect brand via type-level search over a coproduct, so handler-list order is irrelevant? This is the Axis 3 / remediation-item-8 alternative to today's positional cons-list dispatch, which forces the row and handler macros to sort by a syntactic key (the spelling footgun).

Result: PASS (all three sub-criteria: order-independence, no turbofish, readable missing-handler error naming the brand).

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_brand_dispatch.rs` and the compile-fail companion `fp-library/tests/ui/poc_fs_brand_dispatch_missing.rs` (plus its golden). Built on stable Rust with `--features effects`, over the real `frunk_core` coproduct value type, per setup step S2.

## What was built

- Effect operation values carry their brand via `HasBrand { type Brand; }`.
- Handlers are brand-tagged (`Handler<EBrand, F>`), held in a heterogeneous cons-list in the user's writing order.
- `HandleByBrand<EBrand, Op, Out, Idx>` searches the handler list for the handler whose brand is `EBrand`, with the position `Idx` inferred via the `Here`/`There` index trick (the head-match and recurse impls are non-overlapping because their index parameter differs structurally).
- `Dispatch` walks the row coproduct; at each active arm it selects the handler by the operation's brand (`HeadOp::Brand`) rather than by position.

Two positive tests confirm the same row value (`Get(10)`) gives the same result under two opposite handler orders, and that every variant routes to its brand's handler with no turbofish. One trybuild compile-fail test confirms a row whose effect has no handler is rejected.

## Findings

1. Brand-keyed dispatch is feasible and makes handler-list order irrelevant, which is the property that removes item 8's spelling/sort footgun: a handler list need not be sorted to match the row, so the row and handler macros would no longer need a shared structural sort key.
2. The frunk Sculptor pattern is required for the row walk. A single free selector-index parameter on the `Dispatch` impl is rejected as unconstrained (E0207, "the type parameter `Idx` is not constrained"). The fix is to thread a type-level list of per-arm indices (here a nested tuple `(HeadIdx, TailIdx)` terminated by `()`) as a `Dispatch` trait parameter, so every index is constrained by the trait reference. The whole index structure is inferred at the call site; no turbofish is needed.
3. The missing-handler error is readable and names the brand. `#[diagnostic::on_unimplemented]` on `HandleByBrand` fires (the failure is a genuine unsatisfied-trait-bound, E0277, because the recursion bottoms out at `HandlersNil` which implements `HandleByBrand` for no brand). The top-line error is:

```text
error[E0277]: no handler for the effect `LogBrand` in the handler list
   |  let _out: i32 = log.dispatch(&handlers);
   |                      ^^^^^^^^^ the row contains `LogBrand`, but the handler list has no entry for it
   = note: brand-keyed dispatch searches the handler list by effect brand; add a `Handler::<LogBrand, _>` entry (its position does not matter)
```

This is the same `on_unimplemented`-on-a-marker technique POC-0 used, applied to the handler list. It is a strict improvement over the current positional dispatch's "not implemented for `HandlersNil`" error, which requires the documented reading guide.

## Decisions this settles

1. The unified-row design adopts brand-keyed dispatch: handlers are selected by effect brand, not by position. This unifies remediation items 8 (the footgun, eliminated) and 11 (tagged effects: a tag is a wrapper that changes the brand/label used as the search key), matching heftia's label-keyed `:>` membership.
2. The dispatch machinery uses the frunk Sculptor index-list pattern for the row walk; this is the same family of technique as the marker-keyed helpers in POC-0 and POC-1, so the unified row's type-level machinery rests on one stable-Rust idiom (marker/index-keyed traits with inferred positions).
3. Handler-list construction no longer needs the macro-side structural sort. The row macro may keep a sort for canonical display, but handler correctness no longer depends on the two agreeing, which is the durable elimination of the footgun that remediation item 8's note called for.

## Limitations and scope

- The POC dispatches first-order operations to closures returning a common `Out`. It does not address higher-order (scoped) dispatch, which is POC-3 (mixed-row peel) and the Tier D higher-order mechanism; brand-keyed selection of a scoped handler composes with the same machinery but is not exercised here.
- Operations here carry plain payloads; in the real design the row arms are `Coyoneda`-wrapped per setup step S3, and the op handed to a handler is the lowered Coyoneda. That wrapping is orthogonal to the brand-keyed selection demonstrated here.
- Inference quality was checked on a three-effect row (rubric item 3 asked for more than the two-effect minimum); it resolves without turbofish. Behaviour on very wide rows and the readability of inference-failure errors at scale remain to be observed during the integration tier.

## Bearing on the gates

For gate G1: brand-keyed dispatch holds, so the unified row does not inherit (and in fact cures) the positional-sort footgun, removing one of the risks that would have argued against pursuing FS-1/FS-2. Combined with POC-0 (classification) and POC-1 (partition), the cheap shared machinery of the unified row is proven; POC-3 (mixed-row peel and drop) is the remaining Tier A step before G1.
