# POC-8 Findings: substrate unification via one `ClosureStorage` associated stored-closure type

Tier E, foundation sweep (the merged POC-7 + POC-8). Charter question: can one `ClosureStorage` class with an associated stored-closure type unify the per-`Store` closure storage so that one `Run<Store: ClosureStorage, A>` substrate carries the per-`Store` `Clone`/`Send + Sync` bounds, collapsing the six wrappers? Tested on the `Fn`-only layer first (it avoids the `FnOnce`-versus-`Fn` consumption mismatch).

Result: PASS on the `Fn`-only layer, with one recorded limitation (construction stays per-`Store`) and the `FnOnce` reconciliation left as the documented second layer.

POC code (on the `spike/foundation-sweep` branch): `fp-library/tests/poc_fs_closure_storage.rs`. Three tests pass. Run via `just filtered test`. Per S2 this is a bespoke pointer-parameterised substrate (a finding in itself: the public `Free` is `Box`-spine only, so a `Store`-parameterised substrate cannot be a public-`Free` integration test).

## What was built

- `ClosureStorage`: one class with a GAT `Stored<'a, I, O>` (the per-`Store` stored closure, `Box<dyn Fn>` / `Rc<dyn Fn>` / `Arc<dyn Fn + Send + Sync>`) and a `call` method. The `Stored` projection and `call` unify cleanly across Box/Rc/Arc; `call` has a uniform signature and only its body and the associated type vary per `Store`.
- `Run<Store: ClosureStorage, A>`: one substrate type over that associated type, with an `Ask` effect whose continuation is stored via `Store`.
- `handle`: one generic interpreter function over `Store`.
- A conditional `Clone` impl: one impl block with a `where for<'a> Store::Stored<'a, ...>: Clone` clause, so `Run<RcBrand, _>` and `Run<ArcBrand, _>` are `Clone` and `Run<BoxBrand, _>` is not. This is the per-`Store` `Clone` asymmetry expressed on a single impl.

Three tests pass: `Ask` runs end-to-end for Box/Rc/Arc through the one generic `handle`; the Arc instantiation is statically asserted `Send + Sync`; and the Rc substrate clones (via the conditional impl) and both copies run.

## Findings

1. The substrate type, its interpreter, and its `Clone` instance unify over one `Store` parameter. `ClosureStorage`'s associated `Stored` type absorbs the per-`Store` variation (pointer, and the Arc `Send + Sync` bound on the `dyn`), so `Run<Store, A>`, `handle`, and the conditional `Clone` are each written once and instantiate for Box/Rc/Arc. This is the core of Axis 4: the associated stored-closure type carries what a uniform field could not (one field cannot be `dyn Fn` for Box/Rc and `dyn Fn + Send + Sync` for Arc, and forcing `Send + Sync` on all would defeat Box), and the per-`Store` `Clone`/`Send + Sync` bounds live on the substrate's inherent impl via `where`-clauses on `Store`'s associated type, which a fixed trait method signature could not carry. So the six erased wrappers' type, interpreter, and `Clone` machinery collapse to one parameterised substrate.

2. Construction of a stored closure stays per-`Store`. Building a `Store::Stored<...>` from a closure requires the per-`Store` input bound (Arc requires the closure `Send + Sync`), and a single generic constructor signature cannot carry that, the same reason the three `ToDyn*` classes exist separately. A helper trait with per-`Store` impls (the Arc impl adding `Send + Sync`) can carry the bound, but the associated-type projection `Store::Stored` is not injective, so the compiler cannot infer `Store` from the constructed type; generic construction needs an explicit `Store`. The POC uses direct per-`Store` construction (`Box::new`/`Rc::new`/`Arc::new` with a typed binding that drives the unsize coercion to `dyn` and enforces the per-`Store` bound through the `Ask` field's `dyn` type). So the smart constructors and per-effect injection helpers stay per-`Store` (or are generated), while the substrate type/interpreter/`Clone` unify.

3. The `Fn`-only unification restricts Box to `Fn` continuations. Under one `Store`-parameterised substrate the `Fn`-only layer stores every continuation as `Fn` (reusable), so the Box instantiation loses the `Box<dyn FnOnce>` one-shot continuation the current `Free` spine uses. Reconciling that (letting Box keep `FnOnce` while Rc/Arc use `Fn`) is the second layer and the documented load-bearing risk: it would need the `ClosureStorage` associated type to also vary the closure's `Fn`-versus-`FnOnce` kind, or the prefix-scheme fallback. This POC did not build the `FnOnce` layer; the `Fn`-only collapse is what answers gate G3's "can the matrix collapse."

## Limitations and scope

- The minimal effect is `Ask` (its continuation is in the substrate, stored via `Store`); `pure` and `handle` are exercised, `bind`/`map` are not. `bind`/`map` compose stored closures, so they construct new stored closures and follow the per-`Store` construction pattern of finding 2 (they would be per-`Store` or generated, not a new obstacle).
- The `FnOnce` reconciliation (layer two) is not built; recorded as the remaining stretch with the prefix-scheme fallback per the project principles.
- Targets the erased single-wrapper path (S2); the explicit variants are out of the minimal slice.

## Bearing on gate G3

POC-8 shows substrate unification is feasible: the substrate type, interpreter, and `Clone` collapse from six erased wrappers to one `Run<Store: ClosureStorage, A>` over a `ClosureStorage` associated stored-closure type, with the per-`Store` `Clone`/`Send + Sync` bounds carried on inherent `where`-clauses. The recorded limitation is that closure construction stays per-`Store` (smart constructors and injection helpers do not collapse, matching the existing per-effect-brand-sibling generation), and the `FnOnce` one-shot optimization for Box is a second-layer stretch. The G3 decision is recorded in the charter's Decision gates section.
