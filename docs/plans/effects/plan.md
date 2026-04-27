# Plan: Port purescript-run to fp-library

**Status:** Phase 1 complete (all 9 steps); Phase 1 follow-up
both commits (`WrapDrop` migration plus `Functor` -> `Kind`
relaxation) landed; Phase 2 in progress (steps 1, 2, 3, 4a, 4b,
5, and 6 of 10 complete).

## Current progress

Phase 1 complete (steps 1-9). Phase 1 follow-up commits 1 and 2
complete. Phase 2 steps 1, 2, 3, 4a, 4b, 5, and 6 complete.

**Phase 2 step 6 (Erased -> Explicit conversion methods on the
six Run variants).** Each of the three Erased Run wrapper types
gains an `into_explicit(self)` inherent method that walks the
underlying Free chain via `peel` and rebuilds it through the
paired Explicit substrate's `wrap`:
[`Run::into_explicit`](../../../fp-library/src/types/effects/run.rs)
returns `RunExplicit<'static, R, S, A>`,
[`RcRun::into_explicit`](../../../fp-library/src/types/effects/rc_run.rs)
returns `RcRunExplicit<'static, R, S, A>`, and
[`ArcRun::into_explicit`](../../../fp-library/src/types/effects/arc_run.rs)
returns `ArcRunExplicit<'static, R, S, A>`. Each of the three
Explicit Run wrapper types gains a paired
`from_erased(run)` constructor in a separate `'static`-scoped
impl block ([`RunExplicit::from_erased`](../../../fp-library/src/types/effects/run_explicit.rs),
[`RcRunExplicit::from_erased`](../../../fp-library/src/types/effects/rc_run_explicit.rs),
[`ArcRunExplicit::from_erased`](../../../fp-library/src/types/effects/arc_run_explicit.rs))
that delegates to the corresponding `into_explicit` for a single
source of truth. Recursion is O(N) in the chain depth (one stack
frame per suspended `Wrap` layer); per the Wrap-depth probe at
[`fp-library/tests/run_wrap_depth_probe.rs`](../../../fp-library/tests/run_wrap_depth_probe.rs),
Run-typical patterns have structural depth at most 1, so the
recursion is essentially constant in practice.

The conversion preserves the underlying substrate's properties:
`Run -> RunExplicit` is single-shot via `Box`-in-`Wrap`
indirection; `RcRun -> RcRunExplicit` keeps multi-shot via
`Rc<dyn Fn>` continuations on both sides; `ArcRun -> ArcRunExplicit`
keeps `Send + Sync` and multi-shot via `Arc<dyn Fn + Send + Sync>`
continuations. The closure passed to `<NodeBrand<R, S> as Functor>::map`
inside each `into_explicit` body composes cleanly under the
HRTB-bearing impl-block scope on `ArcRun` because projection-typed
values come from `peel`'s return and from `Functor::map`'s output
(never constructed inline as `Node::First(...)` literals); this
matches the workaround established in step 5.

Twelve new tests cover the conversions: a pure-value round-trip
and a suspended-layer preservation per direction per pair, all
passing under `just verify`.

**Phase 2 step 5 (`pure` / `peel` / `send` core operations on
six Run variants).** Each of the six Run wrapper types (`Run`,
`RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
`ArcRunExplicit`) gains three inherent methods: `pure(a)`
delegates to the underlying Free variant's `pure`; `peel(self)`
returns `Result<A, NodeBrand<R, S>::Of<'_, Run<R, S, A>>>`,
exposing the next-step continuation as a [`Node`](../../../fp-library/src/types/effects/node.rs)
layer; `send(node)` lifts a pre-constructed `Node`-projection
value into the program. The signatures are uniform across all
six wrappers.

Step 5 also adds
[`FreeExplicit::to_view`](../../../fp-library/src/types/free_explicit.rs)
as a small precursor (FreeExplicit's `view` field is private; a
public `to_view` is needed for `RunExplicit::peel` to expose the
underlying view-shape).

The `send` signature deviates from the plan-text expectation
("takes the row-variant layer") to "takes the
`Node`-projection value already constructed". This is a
workaround for a stable-Rust GAT-normalization limitation
discovered while implementing `ArcRun::send`: the HRTB on
`ArcFree`'s struct (`Of<'static, ArcFree<...>>: Send + Sync`)
poisons normalization for `<NodeBrand<R, S> as Kind>::Of<...>`
in any scope mentioning the HRTB. Construction inside a
`Node::First(layer)` literal there fails to unify with the
projection. Passing the `Node`-projection value as a parameter
sidesteps this by ensuring the value is already in projection
form when it crosses the HRTB boundary. The probe at
[`fp-library/tests/arc_run_normalization_probe.rs`](../../../fp-library/tests/arc_run_normalization_probe.rs)
documents the limit and the workaround as a regression test.
See [resolutions.md](resolutions.md) for the full investigation.

For `pure` and `peel`, no workaround is needed: `pure` doesn't
construct projection-typed values, and `peel`'s `map_err`
through `Functor::map` receives the projection-typed value from
`*Free::resume` (Erased) or `*FreeExplicit::to_view` (Explicit)
and returns the projection-typed value, with no Node literal in
between.

**Phase 2 step 4b (Explicit family: three Explicit Run wrapper
types, three `RunExplicit` brands, brand-level type-class impls,
row-brand `RefFunctor` and `Extract` cascade impls, `Node`
`Clone` impl, A+B hybrid re-export pattern).** Three new
wrapper types land at
[`fp-library/src/types/effects/run_explicit.rs`](../../../fp-library/src/types/effects/run_explicit.rs),
[`fp-library/src/types/effects/rc_run_explicit.rs`](../../../fp-library/src/types/effects/rc_run_explicit.rs),
and
[`fp-library/src/types/effects/arc_run_explicit.rs`](../../../fp-library/src/types/effects/arc_run_explicit.rs).
Each is a thin tuple-struct wrapper over its respective
[`FreeExplicit`](../../../fp-library/src/types/free_explicit.rs)
/
[`RcFreeExplicit`](../../../fp-library/src/types/rc_free_explicit.rs)
/
[`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)
substrate over `NodeBrand<R, S>`, with
`from_*_free_explicit` / `into_*_free_explicit` zero-cost
conversion sugar. `RcRunExplicit` and `ArcRunExplicit` carry
inherent `bind` and `map` methods (the brand-level
[`Semimonad`](../../../fp-library/src/classes/semimonad.rs) /
[`Functor`](../../../fp-library/src/classes/functor.rs) cannot
be implemented because the underlying `bind` requires per-`A`
`Clone` bounds the trait method signatures cannot carry).

Three new brands land at
[`fp-library/src/brands.rs`](../../../fp-library/src/brands.rs):
`RunExplicitBrand<R, S>`, `RcRunExplicitBrand<R, S>`,
`ArcRunExplicitBrand<R, S>`, each with an `impl_kind!`
projection to its wrapper type. Brand-level coverage delegates
to the corresponding `*FreeExplicitBrand`'s impls:
`RunExplicitBrand` gets
[`Functor`](../../../fp-library/src/classes/functor.rs),
[`Pointed`](../../../fp-library/src/classes/pointed.rs),
[`Semimonad`](../../../fp-library/src/classes/semimonad.rs),
[`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs),
[`RefPointed`](../../../fp-library/src/classes/ref_pointed.rs),
[`RefSemimonad`](../../../fp-library/src/classes/ref_semimonad.rs);
`RcRunExplicitBrand` gets
[`Pointed`](../../../fp-library/src/classes/pointed.rs) plus the
Ref hierarchy;
`ArcRunExplicitBrand` gets
[`SendPointed`](../../../fp-library/src/classes/send_pointed.rs)
only.
[`Monad`](../../../fp-library/src/classes/monad.rs) /
[`RefMonad`](../../../fp-library/src/classes/ref_monad.rs) /
[`SendMonad`](../../../fp-library/src/classes/send_monad.rs) and
the [`SendRef`](../../../fp-library/src/classes/send_ref_functor.rs)-family
hierarchy are not reachable through brand-level delegation; see
`deviations.md` for the full rationale and
[resolutions.md](resolutions.md) for design history.

Row-brand cascade impls were extended:
[`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs),
[`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs),
and
[`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs)
each got
[`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
(needed by the
`RunExplicitBrand` Ref-hierarchy delegation) and
[`Extract`](../../../fp-library/src/classes/extract.rs) impls
(so canonical Run-shaped programs whose row brands implement
[`Extract`](../../../fp-library/src/classes/extract.rs) can have
`evaluate()` called).
[`Node<'a, R, S, A>`](../../../fp-library/src/types/effects/node.rs)
got a [`Clone`] impl (needed by
[`RcFreeExplicit::evaluate`](../../../fp-library/src/types/rc_free_explicit.rs)
/
[`ArcFreeExplicit::evaluate`](../../../fp-library/src/types/arc_free_explicit.rs)
when their outer refcounts are not unique).

A+B hybrid re-export pattern landed: the six Run wrapper
headline types
(`Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
`ArcRunExplicit`) are re-exported at the top level
([`crate::types::*`](../../../fp-library/src/types.rs)) for
ergonomic imports matching the rest of the
[`types/`](../../../fp-library/src/types/) directory; the same
six plus `Node` and `VariantF` are re-exported at the
subsystem-scoped
[`crate::types::effects::*`](../../../fp-library/src/types/effects.rs)
for callers preferring the namespaced form. This mirrors the
[`optics`](../../../fp-library/src/types/optics.rs)
precedent (selective top-level + comprehensive subsystem-scoped).

**Phase 2 step 4a (foundation: row-brand `WrapDrop` impls,
`Node` / `NodeBrand` machinery, three Erased Run wrapper
types).** Step 4 in the plan text is structurally large enough
that landing it as a single commit would be too disruptive for
review and risk leaving the working tree mid-step on context
exhaustion; the step is split into 4a (foundation) and 4b
(Explicit family + brand-level type-class hierarchy).

The directory previously named [`fp-library/src/types/run/`](../../../fp-library/src/types/effects/)
is renamed to
[`fp-library/src/types/effects/`](../../../fp-library/src/types/effects/),
and the parent module file from `types/run.rs` to
[`types/effects.rs`](../../../fp-library/src/types/effects.rs).
The submodules already covered the entire effects subsystem
(coproduct, variant_f, member, plus the new node/run/rc_run/arc_run);
the new name reflects that. The rename is also a deviation
recorded in `deviations.md`.

`WrapDrop` impls landed for the three row brands the canonical
Run shape uses:
[`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs)
(the uninhabited base case),
[`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs)
(dispatches by `Inl`/`Inr`, recurses into the active brand), and
[`CoyonedaBrand<F>`](../../../fp-library/src/types/coyoneda.rs)
(returns `None` per the Wrap-depth probe finding from the
`WrapDrop` resolution).

[`Node<'a, R, S, A>`](../../../fp-library/src/types/effects/node.rs)
ships at `fp-library/src/types/effects/node.rs` as a tagged
dispatch enum: `First(R::Of<'a, A>)` for first-order effect
layers and `Scoped(S::Of<'a, A>)` for scoped-effect layers (Phase
4 populates the scoped row). The brand
[`NodeBrand<R, S>`](../../../fp-library/src/brands.rs) is added
to `brands.rs` and registers a `Kind` projection
(`Of<'a, A> = Node<'a, R, S, A>`), a recursive `Functor` impl
dispatching by variant, and a recursive `WrapDrop` impl
dispatching by variant. Three unit tests cover constructing a
`First` layer, mapping over it, and dropping it through the
`WrapDrop` chain.

Three Erased Run wrapper types land as thin wrappers over their
respective Free-family substrates:
[`Run<R, S, A>`](../../../fp-library/src/types/effects/run.rs)
over [`Free<NodeBrand<R, S>, A>`](../../../fp-library/src/types/free.rs);
[`RcRun<R, S, A>`](../../../fp-library/src/types/effects/rc_run.rs)
over [`RcFree<NodeBrand<R, S>, A>`](../../../fp-library/src/types/rc_free.rs)
(plus `Clone` impl);
[`ArcRun<R, S, A>`](../../../fp-library/src/types/effects/arc_run.rs)
over [`ArcFree<NodeBrand<R, S>, A>`](../../../fp-library/src/types/arc_free.rs)
(plus `Clone` impl, `Send + Sync` witness via the
`Kind_*<Of<'static, ArcFree<..., ArcTypeErasedValue>>: Send + Sync>`
associated-type bound). Each wrapper ships
`from_*_free` / `into_*_free` zero-cost conversion sugar; the
user-facing operations (`pure`, `peel`, `send`, `bind`, `map`,
`lift_f`, `evaluate`, `handle`, etc.) land in Phase 2 step 5.

**Phase 1 follow-up commit 2 (`Functor` -> `Kind` relaxation on
the struct).** Across all six Free variants, the struct,
`*View`, `*Step`, `Inner`, `Continuation`, `Clone`, `Drop`, and
`impl_kind!` declarations relax from
`F: WrapDrop + Functor + 'static` (or `'a`) to
`F: WrapDrop + 'static` (or `'a`). The `Kind` GAT requirement
needed by the `Suspend` variant (`F::Of<'_, Free<F, _>>`) is
inherited from `WrapDrop`'s `Kind` supertrait, so no extra
bound is required at the data-type sites.

For the Erased family ([`Free`](../../../fp-library/src/types/free.rs),
[`RcFree`](../../../fp-library/src/types/rc_free.rs),
[`ArcFree`](../../../fp-library/src/types/arc_free.rs)), the
inherent construction methods (`pure`, `bind`, `map`,
`cast_phantom`, `erase_type`) do not call `F::map`; their impl
blocks relax to `F: WrapDrop + 'static`. Methods that do call
`F::map` (`wrap`, `lift_f`, `to_view`, `resume`, `peel_ref`,
`hoist_free`, plus `evaluate` and `lower_ref` transitively via
`to_view`) get `where F: Functor` as a per-method bound. The
brand-level type-class impls keep `F: WrapDrop + Functor + ...`
because they delegate to inherent methods that need `Functor`.

For the Explicit family ([`FreeExplicit`](../../../fp-library/src/types/free_explicit.rs),
[`RcFreeExplicit`](../../../fp-library/src/types/rc_free_explicit.rs),
[`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)),
the inherent `bind` (via the recursive `bind_boxed` worker)
walks the spine via `F::map`, so `bind`, `map`, `wrap`, and
`lift_f` all transitively require `F: Functor`. The inherent
impl blocks therefore stay at `F: WrapDrop + Functor + 'a`;
only the data-type / `Drop` declarations relax. The brand-level
impls similarly stay at `F: WrapDrop + Functor + ...`.

The Phase 1 stack-safety tests, including
`deep_drop_does_not_overflow` on both
`Free<ThunkBrand>` and `FreeExplicit<IdentityBrand>` plus the
deep-drop tests on the four `Rc` / `Arc` variants, all pass
post-relaxation. Phase 2 step 4
(`Run<R, S, A> = Free<NodeBrand<R, S>, A>`) is now unblocked:
the struct only requires `NodeBrand<R, S>: WrapDrop + 'static`
to instantiate; `Functor` is required only at use sites where
`NodeBrand<R, S>` will impl it (since `CoproductBrand<H, T>`
already does, per Phase 2 step 2).

**Phase 1 follow-up commit 1 (`WrapDrop` migration, struct/Drop
swap).** New trait
[`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs) lands at
`fp-library/src/classes/wrap_drop.rs`, with the signature
`fn drop<'a, X: 'a>(fa: Self::Of<'a, X>) -> Option<X>` and a
`Kind!(type Of<'a, A: 'a>: 'a;)` supertrait. `WrapDrop` impls land
on [`IdentityBrand`](../../../fp-library/src/types/identity.rs) and
[`ThunkBrand`](../../../fp-library/src/types/thunk.rs), each
delegating to their existing `Extract` impl by returning
`Some(<Self as Extract>::extract(fa))`.

All six Free variants
([`Free`](../../../fp-library/src/types/free.rs),
[`RcFree`](../../../fp-library/src/types/rc_free.rs),
[`ArcFree`](../../../fp-library/src/types/arc_free.rs),
[`FreeExplicit`](../../../fp-library/src/types/free_explicit.rs),
[`RcFreeExplicit`](../../../fp-library/src/types/rc_free_explicit.rs),
[`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs))
migrated their struct-, view-, step-, and `Drop`-declaration
bounds from `F: Extract + Functor + 'static` (or `'a`) to
`F: WrapDrop + Functor + 'static` (or `'a`); same for `Inner` /
`Continuation` / `Clone` impls and the brand-level type-class
impls (`FreeExplicitBrand` / `RcFreeExplicitBrand` /
`ArcFreeExplicitBrand`'s `Pointed` / `Functor` / `Semimonad` /
`SendPointed` / `RefFunctor` / `RefPointed` / `RefSemimonad`).
Methods that genuinely call `F::extract` (`evaluate` on all six
variants; `lower_ref` on the four `Rc` / `Arc` variants) keep
`where F: Extract` as a per-method bound. Each variant's `Drop`
body is rewritten to call `<F as WrapDrop>::drop(fa)` and
dispatch on the returned `Option`: `Some(inner)` follows the
existing iterative path; `None` lets the layer drop in place.
The `evaluate` impl block on `Free` keeps
`F: Extract + WrapDrop + Functor + 'static` so the methods
inherited from the `WrapDrop`-bounded blocks (`to_view` etc.)
stay in scope.

The Phase 1 stack-safety tests (including
`deep_drop_does_not_overflow` for both `Free<ThunkBrand>` and
`FreeExplicit<IdentityBrand>`, and the corresponding deep-drop
tests on the four `Rc` / `Arc` variants) all pass post-migration.

**Phase 2 step 3 (`Member<E, Idx>` trait).** [`Member<E, Idx>`](../../../fp-library/src/types/effects/member.rs)
lands at
[fp-library/src/types/effects/member.rs](../../../fp-library/src/types/effects/member.rs)
with `inject(E) -> Self` and `project(self) -> Result<E, Self::Remainder>`
methods. A blanket impl `impl<S, E, Idx, Rem> Member<E, Idx> for S
where S: CoprodInjector<E, Idx> + CoprodUninjector<E, Idx, Remainder = Rem>`
delegates to the frunk_core trait family re-exported by
[`coproduct.rs`](../../../fp-library/src/types/effects/coproduct.rs), so
every Coproduct value automatically gets `Member<E, Idx>` for whichever
`(E, Idx)` pairs frunk_core can prove. Six unit tests cover injection
at head and tail positions, projection success at head and tail,
projection-absent returning the remainder row, and round-trip through
the trait.

`Member` is single-effect only by design; row narrowing (subset /
sculpt) stays through `CoproductSubsetter` directly until Phase 3
handler code surfaces a single-bound need for multi-effect narrowing.
The trait is also agnostic to whether row variants are bare effect
types `E` or `Coyoneda<E, A>`-wrapped; the Coyoneda-wrapping policy
belongs to the smart constructors emitted by the `effects!` macro
(Phase 2 step 9), not to `Member`.

**Phase 2 step 2 (`VariantF<Effects>` and the row-`Functor`).**
[`CNilBrand`](../../../fp-library/src/brands.rs) and
[`CoproductBrand<H, T>`](../../../fp-library/src/brands.rs) are
promoted from the design probe into
[fp-library/src/brands.rs](../../../fp-library/src/brands.rs)
proper, with their `impl_kind!` registrations and recursive
[`Functor`](../../../fp-library/src/classes/functor.rs) impls
landing at
[fp-library/src/types/effects/variant_f.rs](../../../fp-library/src/types/effects/variant_f.rs).

`CoproductBrand<H, T>` resolves `Of<'a, A>` to
`Coproduct<H::Of<'a, A>, T::Of<'a, A>>`, recursing through the head
and tail brands. `CNilBrand` resolves to `CNil` (uninhabited; the
base case of the recursion). `Functor::map` on `CoproductBrand`
dispatches at runtime via `match` on the `Inl` / `Inr` variants;
`Functor::map` on `CNilBrand` is the uninhabited base case (its
input has no inhabitants, so the body is `match fa {}`).

A type alias `VariantF<H, T> = CoproductBrand<H, T>` is exposed
from [`variant_f`](../../../fp-library/src/types/effects/variant_f.rs)
so call sites can use the canonical name from
[decisions.md](decisions.md) section 5.1 without introducing a
separate brand. The Coyoneda-wrapping per row variant happens at
the `effects!` macro layer (Phase 2 step 8); step 2's `VariantF`
machinery does not require it because `CoyonedaBrand<E>` is
already a `Functor` for any `E`, so the recursive `Functor`
constraint composes through naturally.

Four unit tests in `variant_f.rs`'s `mod tests` exercise: Kind
resolution over an `OptionBrand` / `VecBrand` / `CNilBrand` row,
head-branch dispatch via `Inl`, tail-branch dispatch via `Inr` (a
two-deep recursion case), and the `VariantF` alias resolving to
`CoproductBrand`. The earlier
`fp-library/tests/coproduct_brand_probe.rs` design probe (three
tests) is removed; its content is fully covered by the new
production-level `mod tests` block.

**Phase 2 step 1 (`frunk_core` dependency + Brand-aware Coproduct
adapter).** `frunk_core = "0.4"` is added to
[fp-library/Cargo.toml](../../../fp-library/Cargo.toml) (resolves
to `frunk_core 0.4.4`, license MIT, already on the
[`deny.toml`](../../../deny.toml) allow-list, `just deny` passes).
The new `run/` submodule lands at
[fp-library/src/types/effects.rs](../../../fp-library/src/types/effects.rs)
with [coproduct.rs](../../../fp-library/src/types/effects/coproduct.rs)
as its first file, and `pub mod run;` is added to
[fp-library/src/types.rs](../../../fp-library/src/types.rs).

The adapter re-exports the frunk_core types and traits the rest of
Phase 2 / 3 / 4 will need (`Coproduct`, `CNil`, `CoprodInjector`,
`CoprodUninjector`, `CoproductSubsetter`, `CoproductEmbedder`,
`CoproductSelector`, `CoproductTaker`, plus `Here`, `There`,
`HCons`, `HNil`, `HList`). Two unit tests exercise inject /
uninject against a representative two-effect row.

A probe at `fp-library/tests/coproduct_brand_probe.rs` (since
removed once the design was promoted in step 2) validates the
Brand-level integration ahead of step 2: a generic
`CoproductBrand<H, T>` with `Of<'a, A> = Coproduct<H::Of<'a, A>, T::Of<'a, A>>`
and a recursive `Functor` impl on `CoproductBrand<H, T>` (with
`H: Functor + 'static, T: Functor + 'static`) compiles and
dispatches at runtime via `match` on `Inl` / `Inr`. `CNilBrand`
provides the base case via `match fa {}` on the uninhabited
`CNil`. Three probe tests cover Kind resolution, head-branch
dispatch, and tail-branch dispatch over an
`OptionBrand` / `VecBrand` / `CNilBrand` row.

Step 2 will promote `CoproductBrand` and `CNilBrand` from the
probe into [fp-library/src/brands.rs](../../../fp-library/src/brands.rs)
proper, alongside the new
`fp-library/src/types/effects/variant_f.rs` module that wraps the
Coproduct row in Coyoneda per effect.

**Step 1 (`FreeExplicit`).** `FreeExplicit<'a, F, A>` and
`FreeExplicitBrand<F>` are promoted from POC into production at
[fp-library/src/types/free_explicit.rs](../../../fp-library/src/types/free_explicit.rs)
and [fp-library/src/brands.rs](../../../fp-library/src/brands.rs).
The struct wraps its `Pure | Wrap` enum behind an
`Option<FreeExplicitView>` so the custom iterative `Drop` can take
the view via `Option::take` and walk a deep `Wrap` chain in a loop
via `Extract::extract`, mirroring
[`Free`](../../../fp-library/src/types/free.rs)'s strategy. The
struct-level bound is `F: Extract + Functor + 'a` per
[decisions.md](decisions.md) section 4.4. The POC test file imports
the production type and the previously-`#[ignore]`d
`q4_naive_drop_overflows` test is replaced by an actively-running
`q4_drop_deep_does_not_overflow` over a 100 000-deep chain.

**Step 2 (`RcFree`).** `RcFree<F, A>` lands at
[fp-library/src/types/rc_free.rs](../../../fp-library/src/types/rc_free.rs)
following the [`Free`](../../../fp-library/src/types/free.rs)
template. Continuation cells in the
[`CatList`](../../../fp-library/src/types/cat_list.rs) queue are
`Rc<dyn Fn>` (matching what
[`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)
resolves to) instead of `Box<dyn FnOnce>`, so multi-shot effects
like `Choose` can drive the same stored continuation more than
once. The whole substrate lives behind an outer `Rc<Inner>` so
[`Clone`](https://doc.rust-lang.org/std/clone/trait.Clone.html) is
unconditional and O(1) (refcount bump), matching the
[`RcCoyoneda`](../../../fp-library/src/types/rc_coyoneda.rs)
cloning pattern. The inner state's `Drop` impl iteratively
dismantles deep `Suspend` chains via `Extract::extract` (taking
ownership through `Rc::try_unwrap` when uniquely held), and
dropping a 100 000-deep chain is exercised by
`deep_drop_does_not_overflow` in the unit tests.

The full set of inherent methods covered is `pure`, `wrap`,
`lift_f`, `bind`, `map`, `to_view`, `resume`, `evaluate`,
`hoist_free`, plus the new non-consuming
`lower_ref(&self)` / `peel_ref(&self)` (clone-then-consume,
cheap because Clone is O(1)). 12 unit tests cover construction,
chaining, multi-shot via clone, and deep evaluate / Drop.

**Step 3 (`ArcFree`).** `ArcFree<F, A>` lands at
[fp-library/src/types/arc_free.rs](../../../fp-library/src/types/arc_free.rs)
following the
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
template. Same shape as `RcFree`, with three thread-safe
substitutions: `Arc<dyn Fn + Send + Sync>` for continuations
(constructed via
[`<ArcFnBrand as SendLiftFn>::new`](../../../fp-library/src/classes/send_clone_fn.rs)),
`Arc<dyn Any + Send + Sync>` for the type-erased value cell, and
the associated-type-bound trick
`Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>`
on every struct and impl that touches the inner data so the
compiler can auto-derive `Send + Sync` for concrete `F` (the
`F::Of<...>` field is otherwise opaque to the auto-trait
derivation). The whole substrate lives behind an outer
`Arc<Inner>` so cloning is O(1) (atomic refcount bump).
12 unit tests cover the same cases as `RcFree` plus
`cross_thread_via_spawn`, `cross_thread_clone_branches`, and
`is_send_and_sync` to actually exercise the thread-safety
contract.

**Step 4 (`RcFreeExplicit`).** `RcFreeExplicit<'a, F, A>` lands at
[fp-library/src/types/rc_free_explicit.rs](../../../fp-library/src/types/rc_free_explicit.rs)
extending [`FreeExplicit`](../../../fp-library/src/types/free_explicit.rs)'s
concrete recursive enum (no `dyn Any` erasure) with an outer
`Rc<RcFreeExplicitInner>` wrapper plus an `Rc<dyn Fn>`-shaped
continuation in the [`bind`](../../../fp-library/src/types/rc_free_explicit.rs)
worker constructed via
[`<RcFnBrand as LiftFn>::new`](../../../fp-library/src/types/fn_brand.rs)
so the unified function-pointer abstraction is on the construction
path. Because the wrapper is `Rc<Inner>`, the `Wrap` variant holds
`F::Of<'a, RcFreeExplicit<'a, F, A>>` directly (no extra `Box`
needed) and `Clone` is unconditionally O(1). The `RcFreeExplicitBrand<F>`
brand and its `Kind` registration land in
[fp-library/src/brands.rs](../../../fp-library/src/brands.rs)
mirroring `FreeExplicitBrand<F>`. The inner state's `Drop` impl
iteratively dismantles deep `Wrap` chains via `Extract::extract` +
`Rc::try_unwrap`, taking ownership through `try_unwrap` when
uniquely held and leaving shared chains for other holders to
dismantle when they release. The full set of inherent methods
covered is `pure`, `wrap`, `bind`, `evaluate`, `to_view`, plus the
non-consuming `lower_ref(&self)` / `peel_ref(&self)` (clone-then-consume,
cheap because Clone is O(1)). 10 unit tests cover construction,
chaining, multi-shot via clone, deep evaluate / Drop, and
non-`'static` payloads.

**Step 5 (`ArcFreeExplicit`).** `ArcFreeExplicit<'a, F, A>` lands at
[fp-library/src/types/arc_free_explicit.rs](../../../fp-library/src/types/arc_free_explicit.rs)
mirroring `RcFreeExplicit`'s structure with three thread-safe
substitutions: `Arc<...>` for the outer wrapper, `Arc<dyn Fn + Send + Sync>`
for the [`bind`](../../../fp-library/src/types/arc_free_explicit.rs)
worker continuation (constructed via
[`<ArcFnBrand as SendLiftFn>::new`](../../../fp-library/src/classes/send_clone_fn.rs)),
and `Send + Sync` bounds on the user closure passed to `bind`.
The `ArcFreeExplicitBrand<F>` brand and its `Kind` registration land
in [fp-library/src/brands.rs](../../../fp-library/src/brands.rs)
mirroring `RcFreeExplicitBrand<F>`. The inner state's `Drop` impl
iteratively dismantles deep `Wrap` chains via `Extract::extract` +
`Arc::try_unwrap`. The full set of inherent methods covered is
`pure`, `wrap`, `bind`, `evaluate`, `to_view`, plus the
non-consuming `lower_ref(&self)` / `peel_ref(&self)`. 12 unit tests
cover construction, chaining, multi-shot via clone, deep evaluate /
Drop, non-`'static` payloads, plus `cross_thread_via_spawn`,
`cross_thread_clone_branches`, and `is_send_and_sync` to exercise
the thread-safety contract.

**Step 6 (`SendFunctor` trait family).** The full by-value
parallel of the existing `send_ref_*` family lands across nine new
trait files at
[fp-library/src/classes/](../../../fp-library/src/classes/):
[send_pointed.rs](../../../fp-library/src/classes/send_pointed.rs),
[send_functor.rs](../../../fp-library/src/classes/send_functor.rs),
[send_lift.rs](../../../fp-library/src/classes/send_lift.rs),
[send_semiapplicative.rs](../../../fp-library/src/classes/send_semiapplicative.rs),
[send_apply_first.rs](../../../fp-library/src/classes/send_apply_first.rs),
[send_apply_second.rs](../../../fp-library/src/classes/send_apply_second.rs),
[send_applicative.rs](../../../fp-library/src/classes/send_applicative.rs),
[send_semimonad.rs](../../../fp-library/src/classes/send_semimonad.rs),
and [send_monad.rs](../../../fp-library/src/classes/send_monad.rs).

Trait shapes:

- `SendPointed::send_pure(a: A)` requires `A: Send + Sync`.
- `SendFunctor::send_map` and `SendSemimonad::send_bind` carry
  `Send + Sync` on both their `A`/`B` parameters and on the
  closure parameter (matching what
  [`<ArcFnBrand as SendLiftFn>::new`](../../../fp-library/src/classes/send_clone_fn.rs)
  requires).
- `SendLift::send_lift2` carries `Send + Sync` on the closure and
  `Clone + Send + Sync` on `A` / `B` so the closure can move
  copies of both arguments.
- `SendSemiapplicative::send_apply` carries `Clone + Send + Sync`
  on `A` and `Send + Sync` on `B`, taking the wrapped function
  via `<FnBrand as SendCloneFn>::Of<'a, A, B>` (parallel to
  `SendRefSemiapplicative` using `SendCloneFn<Ref>`).
- `SendApplyFirst` and `SendApplySecond` are blanket-implemented
  for any `Brand: SendLift`, paralleling
  [`SendRefApplyFirst`](../../../fp-library/src/classes/send_ref_apply_first.rs)
  / [`SendRefApplySecond`](../../../fp-library/src/classes/send_ref_apply_second.rs).
- `SendApplicative: SendPointed + SendSemiapplicative + SendApplyFirst + SendApplySecond`
  is a blanket impl, mirroring
  [`Applicative`](../../../fp-library/src/classes/applicative.rs)
  with thread-safety bounds layered on top.
- `SendMonad: SendApplicative + SendSemimonad` is a blanket impl,
  mirroring [`Monad`](../../../fp-library/src/classes/monad.rs).

The bonus integration with existing brands lands as follows:

- `ArcCoyonedaBrand`: implements `SendFunctor` (closure has
  `Send + Sync`, fits inside the existing `Arc<dyn Fn + Send + Sync>`
  layer storage). It does **not** implement `SendPointed`,
  `SendSemimonad`, `SendLift`, or `SendSemiapplicative` because
  all four go through
  [`ArcCoyoneda::lift`](../../../fp-library/src/types/arc_coyoneda.rs)
  which requires `F::Of<'a, A>: Clone + Send + Sync`, a per-`A`
  bound that cannot be expressed in the trait method signature
  (no HRTB-over-types). The module-level docs and the brand-impl
  block comment in
  [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)
  are updated to reflect the partial close.
- `OptionBrand`: implements `SendPointed`, `SendFunctor`,
  `SendLift`, `SendSemiapplicative`, and `SendSemimonad` directly
  via `Some(a)`, `fa.map(f)`, `fa.zip(fb).map(...)`, pattern
  matching, and `ma.and_then(f)` respectively. `SendApplicative`,
  `SendApplyFirst`, `SendApplySecond`, and `SendMonad` follow via
  the blanket impls. `Option`'s by-value path has no Clone or
  thread-affinity constraints, so the impls are mechanical.
  Adding these impls beyond the plan's named `ArcCoyonedaBrand`
  target is a small scope expansion documented in the deviations,
  needed because `#[document_examples]` requires real Rust code
  blocks and `ArcCoyonedaBrand` cannot supply them for the
  `SendPointed` / `SendSemimonad` / `SendLift` /
  `SendSemiapplicative` traits.

`send_pure`, `send_map`, `send_lift2`, `send_apply`,
`send_apply_first`, `send_apply_second`, and `send_bind` are
re-exported through
[fp-library/src/functions.rs](../../../fp-library/src/functions.rs)
alongside the existing `send_ref_*` free-function variants.

**Step 7 (brand-level trait hierarchies for the Explicit Free
family).** The realistic scope, after running the impls against
stable Rust:

- `FreeExplicitBrand`: `Functor`, `Pointed`, `Semimonad` on the
  by-value side; `RefFunctor`, `RefPointed`, `RefSemimonad` on
  the by-reference side. Brand impls live inside the inner mod
  alongside the type definitions; the by-reference impls share
  two private recursive helpers (`free_explicit_ref_map`,
  `free_explicit_ref_bind`) that box the user closure into
  `Rc<dyn Fn>` and walk the spine via `F::ref_map`.
- `RcFreeExplicitBrand`: `Pointed` on the by-value side
  (`bind`/`map` blocked by per-`A` Clone bounds the trait can't
  add); full `RefFunctor`/`RefPointed`/`RefSemimonad` on the
  by-reference side via `Rc::deref` + `F::ref_map`.
- `ArcFreeExplicitBrand`: `SendPointed` on the by-value side
  only. The `SendRef*` hierarchy is structurally blocked: the
  closure passed to `F::send_ref_map` returns
  `ArcFreeExplicit<F, B>`, which auto-derive of `Send + Sync`
  requires the `Kind<Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync>`
  bound dropped from the struct in step 5. Adding it back via
  the impl block would need `for<'a, T>` HRTB-over-types, which
  stable Rust does not provide. The block is documented in
  `arc_free_explicit.rs` for future revisits.

Three classes of by-value impl that step 7 originally intended
are also blocked:

- `Lift` / `Semiapplicative` / `Applicative` / `ApplyFirst` /
  `ApplySecond` / `Monad` for **all three** Explicit brands. The
  natural `lift2 = bind(fa, |a| map(fb, |b| f(a, b)))` pattern
  requires `fb` to survive multiple invocations of the bind
  closure; `FreeExplicit` is not `Clone`, and
  `Rc`/`ArcFreeExplicit::bind` carry per-`A` Clone bounds the
  trait can't add. Without `Lift`, `Applicative`'s blanket impl
  doesn't apply, which cascades to `Monad`.
- `RefLift` / `RefSemiapplicative` / `RefApplicative` /
  `RefMonad` for `FreeExplicit` and `RcFreeExplicit`. The
  natural `ref_lift2 = ref_bind(fa, |a: &A| ref_map(fb, |b: &B| f(a, b)))`
  pattern captures `&a` in the inner closure, but `ref_map`'s
  closure must satisfy `+ 'a` and the captured `&a`'s lifetime
  is shorter than `'a`. Cloning `a` would resolve it but the
  trait doesn't admit `A: Clone`.
- All `SendRef*` for `ArcFreeExplicit` (per the HRTB-over-types
  reason above).

The `fp-library/docs/limitations-and-workarounds.md` table is
extended with three by-value rows (`FreeExplicit`,
`RcFreeExplicit`, `ArcFreeExplicit`) and a parallel three-row
by-reference classification.

**Step 8 (per-variant Criterion benches).** Six per-variant bench
files plus a cross-variant comparison bench land at
[fp-library/benches/benchmarks/](../../../fp-library/benches/benchmarks/):
[free.rs](../../../fp-library/benches/benchmarks/free.rs),
[rc_free.rs](../../../fp-library/benches/benchmarks/rc_free.rs),
[arc_free.rs](../../../fp-library/benches/benchmarks/arc_free.rs),
[free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)
(extended from the POC bench),
[rc_free_explicit.rs](../../../fp-library/benches/benchmarks/rc_free_explicit.rs),
[arc_free_explicit.rs](../../../fp-library/benches/benchmarks/arc_free_explicit.rs),
and the cross-variant
[free_family_comparison.rs](../../../fp-library/benches/benchmarks/free_family_comparison.rs).

Each per-variant file covers three shapes: `bind-deep + evaluate` at
depths 10 / 100 / 1000 / 10000 (build a `Wrap` spine, then bind once
across it, then evaluate), `bind-wide + evaluate` at the same widths
(`pure(0).bind(...).bind(...)...` chained N times), and
`peel-and-handle (Pure)` (single-step view extraction). For variants
that expose a non-consuming `peel_ref(&self)`
(`RcFree`, `ArcFree`, `RcFreeExplicit`, `ArcFreeExplicit`), the
peel-and-handle group includes both the consuming `to_view()` and the
`peel_ref()` shapes; for `Free` and `FreeExplicit` only the consuming
form is available. The `evaluate only (reference)` shape (carried
over from the POC bench) stays in the per-variant files so the bind +
evaluate measurements have a paired baseline isolating the walk-once
cost.

The cross-variant bench groups all six variants under
`FreeFamily/bind-deep + evaluate` and
`FreeFamily/bind-wide + evaluate`, with one criterion subgroup per
shape so the output shows the variants side-by-side at each depth.
This documents the O(1) (Erased family: bind only snocs onto the
CatList) vs O(N) (Explicit family: bind walks the existing spine
inside the recursive worker) bind-cost asymmetry directly.

Brand choice: every variant uses `IdentityBrand` except `Free`,
which uses `ThunkBrand` because `Free<IdentityBrand, A>` is
layout-cyclic (the `Wrap` arm holds `F::Of<Free<F, TypeErasedValue>>`
where `TypeErasedValue = Box<dyn Any>`, and `Identity<T>` provides no
indirection inside that recursion). The Rc/Arc variants and the
Explicit family escape the cycle either via the outer `Rc<Inner>` /
`Arc<Inner>` wrapper or via the explicit `Box<...>` indirection in
`FreeExplicit`'s `Wrap` arm. `ThunkBrand` is the same brand the
existing `Free` unit tests use (`Thunk<A>` holds a boxed closure,
which provides the indirection).

**Step 9 (per-variant unit and `compile_fail` tests).** Closes the gap
in `FreeExplicit`'s test coverage and adds four new `compile_fail` UI
tests under [fp-library/tests/ui/](../../../fp-library/tests/ui/).

Unit tests added to
[free_explicit.rs](../../../fp-library/src/types/free_explicit.rs):
nine tests in the `mod tests` block covering construction
(`pure_evaluate`, `wrap_evaluate`), evaluation (`bind_chains`),
deep `Wrap` chains (`deep_evaluate_does_not_overflow`,
`deep_drop_does_not_overflow`), the non-`'static` payload property
(`non_static_payload`), and the brand-dispatched paths added in
step 7 (`brand_dispatched_pointed`, `brand_dispatched_functor`,
`brand_dispatched_semimonad` covering `RefPointed::ref_pure`,
`RefFunctor::ref_map`, `RefSemimonad::ref_bind`). The other five
variants already had source-level unit tests; their existing
coverage (multi-shot via clone for the multi-shot variants,
cross-thread + `is_send_and_sync` for the Arc variants, deep
eval/Drop for all, non-static for the Explicit family) was unchanged
and is sufficient for step 9's property-coverage requirements.

`compile_fail` UI tests added under
[fp-library/tests/ui/](../../../fp-library/tests/ui/):

- `free_not_clone.rs`: confirms `Free` is single-shot. Cloning a
  `Free` value fails because `Free` deliberately omits `Clone`;
  multi-shot clients must pick `RcFree` or `ArcFree`. Pairs the
  long-standing `free_requires_static.rs` test on the `'static`
  bound axis with the orthogonal single-shot axis.
- `erased_free_brands_do_not_exist.rs`: confirms the Erased family
  carries no Brand dispatch by importing a non-existent `FreeBrand`.
  Per decisions section 4.4, only the Explicit family has brands;
  typeclass-generic code over an Erased Free fails to resolve.
- `arc_free_explicit_bind_requires_send.rs`: confirms
  `ArcFreeExplicit::bind` rejects `!Send + !Sync` closures. Captures
  an `Rc<i32>` (which is `!Send` and `!Sync`) inside the bind
  closure; the `Arc<dyn Fn + Send + Sync>` storage shape rejects it.
  Multi-shot single-thread programs should use `RcFreeExplicit`.
- `rc_free_bind_requires_clone.rs`: confirms `RcFree::bind` requires
  `A: Clone`. The bound is needed because `RcFree`'s shared inner
  state recovers an owned `A` from a potentially-shared
  `Rc<dyn Any>` cell via `Rc::try_unwrap` with a fallback to
  `(*shared).clone()`.

The `.stderr` files were generated via
`TRYBUILD=overwrite cargo test --test compile_fail` (the
[trybuild](https://docs.rs/trybuild) bootstrap pattern, run via raw
cargo so the wip files do not persist under `fp-library/wip/` after
generation), then committed alongside their `.rs` counterparts.

All Phase 1 work is now landed; Phase 2 (Run substrate and
first-order effects) is the next phase per the resequenced phasing
below.

Other artefacts unchanged from pre-implementation:

- [poc-effect-row/](../../../poc-effect-row/) -- 25 tests across two
  suites validating the row-encoding hybrid (workaround 1 macro
  plus workaround 3 `CoproductSubsetter` fallback), the
  `tstr_crates` Phase 2 refinement, and static-via-Coyoneda
  Functor dispatch end-to-end. See
  [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
  for findings. Migrates into production during Phase 2.

## Open questions, issues and blockers

This section tracks **active** blockers only. Resolved blockers
are logged in [resolutions.md](resolutions.md) for design
history. Per-step deviations from the plan are logged in
[deviations.md](deviations.md) for code-review context.

### Active blockers

None at this time. The three blockers that surfaced
2026-04-27 while preparing Phase 2 step 4b have all been
resolved as part of the step 4b commit:

- Brand-level type-class coverage gap on the Explicit Run
  brands: shipped achievable subset, documented gaps; see
  [resolutions.md](resolutions.md#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands).
- Row-brand `RefFunctor` and `Extract` cascade impls land in
  step 4b: see
  [resolutions.md](resolutions.md#resolved-2026-04-27-row-brand-reffunctor-and-extract-cascade-impls-land-in-step-4b).
- Re-export pattern for the effects subsystem types follows
  the optics A+B hybrid: see
  [resolutions.md](resolutions.md#resolved-2026-04-27-re-export-pattern-for-the-effects-subsystem-types-follows-the-optics-ab-hybrid).

### Procedure for new blockers

If a load-bearing question surfaces during implementation:

1. Add an `### Active blocker (date): <summary>` subsection
   under `### Active blockers` above and pause work.
2. When the blocker resolves, move the entry verbatim (or with
   added resolution detail) to [resolutions.md](resolutions.md)
   as a new top-level entry, dated.
3. Replace the active-blocker subsection here with a one-line
   pointer if useful for cross-referencing, or remove it.

### Resolved blockers (summary)

For full investigation, alternatives, and rationale on each
resolved blocker, see [resolutions.md](resolutions.md). One-line
summaries:

- [Resolved (2026-04-27): `*Run::send` takes a `Node`-projection value to sidestep GAT-normalization poisoning under `ArcFree`'s HRTB](resolutions.md#resolved-2026-04-27-runsend-takes-a-node-projection-value-to-sidestep-gat-normalization-poisoning-under-arcfrees-hrtb)
  -- discovered while implementing `ArcRun::send`: the HRTB at
  `ArcFree`'s struct level poisons `<NodeBrand as Kind>::Of<...>`
  normalization in any scope mentioning it. Workaround: pass
  the `Node`-projection value as a parameter rather than
  constructing it inside the HRTB scope. Applied symmetrically
  to all six Run wrappers' `send` for API uniformity.
- [Resolved (2026-04-27): brand-level type-class coverage gap on the Explicit Run brands](resolutions.md#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)
  -- ship `Functor / Pointed / Semimonad` plus the by-reference
  equivalents for `RunExplicitBrand`; `Pointed` plus by-reference
  for `RcRunExplicitBrand`; `SendPointed` only for
  `ArcRunExplicitBrand`. `Monad` / `RefMonad` / `SendMonad` and
  the `SendRef`-family hierarchy are unreachable through
  brand-level delegation; inherent `bind` and `map` cover the
  by-value monadic surface.
- [Resolved (2026-04-27): row-brand `RefFunctor` and `Extract` cascade impls land in step 4b](resolutions.md#resolved-2026-04-27-row-brand-reffunctor-and-extract-cascade-impls-land-in-step-4b)
  -- add `RefFunctor` and `Extract` impls to `CNilBrand`,
  `CoproductBrand<H, T>`, and `NodeBrand<R, S>`, plus a `Clone`
  impl on `Node`. Required by the Run-Explicit brand
  Ref-hierarchy delegation and by `Rc`/`Arc`-Free's `evaluate`
  fallback.
- [Resolved (2026-04-27): re-export pattern for the effects subsystem types follows the optics A+B hybrid](resolutions.md#resolved-2026-04-27-re-export-pattern-for-the-effects-subsystem-types-follows-the-optics-ab-hybrid)
  -- six Run wrapper headline types at top-level
  (`crate::types::*`); same six plus `Node` and `VariantF` at
  subsystem-scoped (`crate::types::effects::*`). Mirrors the
  optics precedent.
- [Resolved (2026-04-27): introduce `WrapDrop` trait for Free's struct-level Drop concern](resolutions.md#resolved-2026-04-27-introduce-wrapdrop-trait-for-frees-struct-level-drop-concern)
  -- replace Free's struct-level `Extract` bound with a new
  `WrapDrop` trait that decouples Drop's structural cleanup
  from `Extract`'s semantic interpretation. Two-commit migration
  before Phase 2 step 4 resumes.
- [Resolved (2026-04-26): brand-level dispatch for the multi-shot Explicit Free family lands on the by-reference hierarchy](resolutions.md#resolved-2026-04-26-brand-level-dispatch-for-the-multi-shot-explicit-free-family-lands-on-the-by-reference-hierarchy)
  -- `RcFreeExplicitBrand` and `ArcFreeExplicitBrand` get
  `Pointed`/`SendPointed` on the by-value side and full Ref/SendRef
  hierarchies; remaining by-value operations ship as inherent
  methods.
- [Resolved earlier: Erased / Explicit dispatch split for the Free family](resolutions.md#resolved-earlier-erased--explicit-dispatch-split-for-the-free-family)
  -- Erased family (`Free`, `RcFree`, `ArcFree`) is
  inherent-method only; Explicit family (`FreeExplicit`,
  `RcFreeExplicit`, `ArcFreeExplicit`) is Brand-dispatched.
- [Design-phase blockers (resolved in decisions.md)](resolutions.md#design-phase-blockers-resolved-in-decisionsmd)
  -- pointer aggregating decisions.md sections 4 and 9.

<!-- The full problem statement, investigation, resolution,
migration plan, and alternatives for the WrapDrop blocker live
in resolutions.md per the link above. The phasing-side checklist
lives in "Phase 1 follow-up: WrapDrop migration" below. -->

<!-- old content removed; see resolutions.md -->

## Deviations

Per-step deviations from the original plan text (where the
shipped code or design diverged from what the step description
said) are logged in [deviations.md](deviations.md), grouped by
phase and step. New deviations are appended there as steps land.

## Implementation protocol

After completing each step within a phase:

1. Run verification: `just fmt`, `just check`, `just clippy`,
   `just deny`, `just doc`, `just test` (or `just verify` which
   runs all six in order).
2. If verification passes, update `Current progress`, `Open
questions, issues and blockers`, and `Deviations` sections at
   the top of this plan to reflect the current state.
3. Commit the step (including the plan updates).

---

Port `purescript-run`'s extensible algebraic effects to
`fp-library`, delivering Rust `Run` types that support
row-polymorphic first-order effects and heftia-style scoped
effects, with macro ergonomics for common cases and a six-variant
`Free` substrate covering single-shot, multi-shot, thread-safe,
non-`'static` payload, and Brand-dispatched-vs-inherent-method
combinations via the Erased/Explicit dispatch split.

## API stability stance

`fp-library` is pre-1.0. API-breaking changes are acceptable when
they lead to a better end state. This plan prioritises design
correctness and internal coherence over preserving compatibility
with any pre-existing user surface for `Run` (there is none yet;
this is an additive port).

## Motivation

PureScript's `purescript-run` ships an extensible algebraic-effect
system shaped around row polymorphism, partial interpretation, and
multi-shot continuations. fp-library has the building blocks
(`Free<F, A>`, `Coyoneda<F>`, the Brand-and-Kind HKT machinery, and
the `MonadRec` interpreter family) but no public `Run` type. This
plan delivers `Run` and the surrounding effect machinery, ported to
match PureScript's user-facing semantics where stable Rust permits
and explicitly diverging where it doesn't (e.g., `pure` takes a
brand turbofish; multi-shot effects require choosing `RcRun` or
`ArcRun` rather than the default `Run`; typeclass-generic dispatch
requires the corresponding Explicit Run variant).

User surface after this plan, fast-path inherent-method version:

```rust
// Declare a row of effects via the macro:
type AppEffects = effects![Reader<Env>, State<Counter>, Logger];

// Build a program with the run_do! macro (inherent-method-based,
// O(1) bind, no Brand dispatch):
fn run_program() -> Run<AppEffects, NoScoped, String> {
    run_do! {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        log(format!("config = {cfg:?}, counter = {n}"));
        pure(format!("got {n}"))
    }
}

// Compose handlers as a pipeline that narrows the row at each step:
let result: String = run_program()
    .handle(run_reader(env))
    .handle(run_state(0))
    .handle(run_logger())
    .extract();
```

For Brand-dispatched typeclass-generic code (or programs with
non-`'static` payloads), use the corresponding Explicit variant.
The single-shot single-thread variant `RunExplicit` (built on
`FreeExplicit`) keeps full by-value brand coverage and is the
ergonomic default:

```rust
fn run_program_explicit<'a>() -> RunExplicit<'a, AppEffects, NoScoped, String> {
    m_do!(RunExplicitBrand {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        pure(format!("got {n}"))
    })
}
```

The multi-shot variants `RcRunExplicit` / `ArcRunExplicit` get
brand dispatch via the by-reference hierarchy (`RefFunctor` /
`RefSemimonad` / `RefMonad` and their `SendRef*` parallels),
matching `Lazy`'s precedent for the same constraint. The existing
`m_do!` / `a_do!` macros support a `ref` qualifier
(`m_do!(ref Brand { ... })`) that routes through
`RefSemimonad::ref_bind`; closures take `&A`:

```rust
fn run_program_rc_explicit<'a>() -> RcRunExplicit<'a, AppEffects, NoScoped, String> {
    m_do!(ref RcRunExplicitBrand {
        cfg <- ask::<Env>();          // cfg: &Env
        n <- get::<Counter>();         // n: &Counter
        pure(format!("got {n}"))
    })
}
```

For inherent-method calls on multi-shot Explicit Run programs
(e.g., when `A: Clone` is satisfied and consuming continuations
are preferred), the by-value `bind` / `map` ship as inherent
methods on `RcRunExplicit` / `ArcRunExplicit` directly, with their
natural `Clone` bounds, mirroring the
[`RcCoyoneda`/`ArcCoyoneda` precedent](../../../fp-library/docs/limitations-and-workarounds.md).

Convert between Erased and Explicit on demand:
`run_program().into_explicit()` walks the structure once and
returns the corresponding Explicit Run of the same program,
suitable for handing into typeclass-generic consumers.

`runReader: Run<R + READER, S, A> -> Run<R, S, A>`-style row
narrowing matches PureScript Run (the scoped-effect row `S`
threads unchanged through first-order handlers and is narrowed
only by scoped-effect handlers); the macro layer plus
`CoproductSubsetter`-mediated permutation proofs handle the
ordering-mitigation problem (see
[decisions.md](decisions.md) section 4.1).

## Design

The design is recorded in full in
[decisions.md](decisions.md) sections 4 (six core DECISIONs) and
5 (draft architecture). Quick reference:

- **Row encoding (decisions §4.1):** Option 4 hybrid (frunk-style
  Peano-indexed `Coproduct<H, T>` plus `effects![...]` macro
  layer). Workaround 1 (macro lexical sort) is primary; workaround
  3 (`CoproductSubsetter` permutation proof) is fallback for
  hand-written rows.
- **Functor dictionary (decisions §4.2):** static option via
  `Coyoneda` per effect. Each row variant is `Coyoneda<E, A>`,
  which is a `Functor` for any `E` regardless of `E`'s own shape.
  `Coproduct<H, T>` implements `Functor` via recursive trait
  dispatch (`H: Functor + T: Functor`). The dynamic
  `DynFunctor` option is retained as a fallback only.
- **Stack safety (decisions §4.3):** ship both interpreter
  families, mirroring PureScript: `interpret`/`run`/`runAccum`
  (assume target stack-safe) and `interpretRec`/`runRec`/
  `runAccumRec` (require `MonadRec` on target).
- **Free family (decisions §4.4):** six variants in two rows.
  Erased family (`Free`, `RcFree`, `ArcFree`) is inherent-method
  only with O(1) bind via `dyn Any` erasure plus CatList; pins
  `A: 'static`. Explicit family (`FreeExplicit<'a, ...>`,
  `RcFreeExplicit<'a, ...>`, `ArcFreeExplicit<'a, ...>`) carries
  Brand dispatch with O(N) bind via concrete recursive enum;
  supports `A: 'a`. The Erased/Explicit split is the dispatch
  story: typeclass-generic code uses the Explicit row, fast-path
  code uses the Erased row, and `into_explicit()` converts between
  them when needed. The `ArcFreeExplicitBrand` `Functor` impl
  lands via the new `SendFunctor` trait family (Phase 1 step 6).
- **Scoped effects (decisions §4.5):** heftia-style dual-row
  architecture. `Run` carries a separate higher-order row of
  scoped-effect constructors (`Catch<'a, E>`, `Local<'a, E>`,
  `Bracket<'a, A, B>`, `Span<'a, Tag>`). Day-one `'a` parameter,
  fixed `Run<R, A>` continuation, coproduct-of-constructors
  extension shape. (A `Mask<'a, E>` constructor for duplicated-effect
  masking was considered and deferred to a future revision; see
  [decisions.md](decisions.md) section 4.5's "Deferred to a future
  revision" sub-decision for the four options preserved on the
  shelf.)
- **Natural transformations (decisions §4.6):** `handlers!{...}`
  macro DSL primary, builder pattern (`nt().on::<E>(handler)...`)
  as fallback.

`Run` core type:

```text
Run<Effects, ScopedEffects, A> = FreeFamily<Node<Effects, ScopedEffects>, A>

Node<Effects, ScopedEffects>   = First(VariantF<Effects>)
                               | Scoped(ScopedCoproduct<ScopedEffects>)
```

where `FreeFamily` is one of `Free` / `RcFree` / `ArcFree` /
`FreeExplicit`, and `Run<R, S, A>` has matching aliases (`RcRun`,
`ArcRun`, `RunExplicit`).

## Validated via POCs

| POC                                                                                                       | Findings                                                                                                                                                                                                                                                                                                                                                   |
| --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [poc-effect-row/tests/feasibility.rs](../../../poc-effect-row/tests/feasibility.rs)                       | 17 tests covering workaround 1 (lexical-sort macro) plus workaround 3 (`CoproductSubsetter` fallback), generic-effect handling, lifetime parameters, 5- and 7-effect rows for trait-inference scaling, plus `tstr_crates` Phase 2 refinement (3 tests showing content-addressed naming + `tstr::cmp` compile-time ordering). All pass on stable Rust 1.94. |
| [poc-effect-row/tests/coyoneda.rs](../../../poc-effect-row/tests/coyoneda.rs)                             | 8 tests validating static-via-Coyoneda end-to-end: `effects_coyo!` macro emits Coyoneda-wrapped Coproducts canonically; `Coyoneda<F, A>` is `Functor` for any `F`; `Coproduct<H, T>` implements `Functor` via recursive trait dispatch with no specialization or runtime dictionary; row canonicalises across input orderings under wrapping.              |
| [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)                   | 6 tests validating `FreeExplicit<'a, F, A>` integrates with the Brand-and-Kind machinery, supports non-`'static` payloads, supports two-effect Run-shaped composition. One `#[ignore]`d test documents that naive `Drop` overflows on deep chains; the iterative custom `Drop` ships in Phase 1.                                                           |
| [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs) | Criterion bench at depths 10 / 100 / 1000 / 10000 confirming `FreeExplicit`'s per-node cost is approximately 27ns in the linear regime. The Phase-1 baseline for measuring `RcFree` / `ArcFree` regressions.                                                                                                                                               |

The POC code (the `effects!` / `effects_coyo!` macros, the stub
Coyoneda) migrates into production during Phase 2 and Phase 3; the
POC repos remain as reference until then and are deleted once the
production tests cover the same surface.

## Key decisions

The full decision rationale is in [decisions.md](decisions.md).
Quick reference table:

| ID        | Decision                                                                                                                | Rationale (one-line)                                                                                                                                  |
| --------- | ----------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| 4.1       | Option 4 hybrid (macro + nested Coproduct) with corophage-style `'a` per effect                                         | Most production-credible reference (corophage) and best stable-Rust ergonomics                                                                        |
| 4.1       | Workaround 1 (macro canonicalisation) primary; workaround 3 (`CoproductSubsetter`) fallback                             | Macro pays the sort cost once at row construction; Subsetter handles hand-written rows                                                                |
| 4.1       | tstr_crates content-addressed naming as Phase 2 refinement                                                              | Stable type-level identity across import paths; the only credible stable-Rust improvement                                                             |
| 4.2       | Static option via `Coyoneda` per effect                                                                                 | Each row variant is trivially a Functor; section 5.2 commits to Coyoneda anyway                                                                       |
| 4.3       | Ship both `interpret` and `interpretRec` families                                                                       | Documentation parity with PureScript Run; few-percent runtime cost is small                                                                           |
| 4.4       | Six-variant Free: `Free`, `RcFree`, `ArcFree` (Erased) + `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit` (Explicit) | Erased family is inherent-method-only with O(1) bind; Explicit family is Brand-dispatched with O(N) bind; Erased/Explicit split is the dispatch story |
| 4.4       | `SendFunctor` / `SendPointed` / `SendSemimonad` / `SendMonad` trait family for `ArcFreeExplicitBrand`                   | By-value parallel of existing `SendRef*` family; closes the same gap that today prevents `ArcCoyonedaBrand` from implementing `Functor`               |
| 4.5       | Heftia dual-row for scoped effects                                                                                      | Cleanest higher-order effect encoding surveyed; preserves first-class programs                                                                        |
| 4.5       | `'a` lifetime parameter on every scoped-effect constructor from day one                                                 | Avoids breaking-change retrofit when `FreeExplicit` use cases want non-`'static` actions                                                              |
| 4.5       | Fixed `Run<R, A>` interpreter continuation (no associated type)                                                         | Matches every Haskell library surveyed; associated type deferred until use case forces it                                                             |
| 4.5       | Coproduct-of-constructors for user-defined scoped effects                                                               | Mirrors the first-order row's structure; preserves first-class-programs property                                                                      |
| 4.6       | `handlers!{...}` macro DSL primary; builder pattern fallback                                                            | Same shape as section 4.1's macro + mechanical-fallback hybrid                                                                                        |
| 9.3 / 9.4 | Sync interpreters in v1; async (and async IO) via `Future` as a `MonadRec` target in Phase 3                            | "User picks the target monad" -- single mechanism, no parallel `AsyncRun` family                                                                      |
| 9.8       | All effects-related macros live in `fp-macros`; split off a separate crate only if needed                               | One crate, one release cadence, one place to coordinate macro semantics                                                                               |
| 9.9       | TalkF + DinnerF integration test from `purescript-run` as the headline Phase 4 milestone                                | Real-world reference; validates the port behaves like `purescript-run` for a worked example                                                           |

## Integration surface

### Will change

| Component                                                                                         | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fp-library/src/types/free.rs`                                                                    | Existing `Free<F, A>` keeps its current shape; inherent-method only (no Brand). Minor adjustments if integration with `Run` requires.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `fp-library/src/types/free_explicit.rs`                                                           | **New module (Phase 1 step 1).** Promote `FreeExplicit<'a, F, A>` from POC, add iterative custom `Drop`, add full by-value `Functor` / `Pointed` / `Semimonad` / `Monad` impls plus full `RefFunctor` / `RefPointed` / `RefSemimonad` / `RefMonad` impls (Phase 1 step 7). The naive recursive enum has no Clone bound on bind, so both hierarchies land cleanly.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `fp-library/src/types/rc_free.rs`                                                                 | **New module (Phase 1 step 2).** `RcFree<F, A>` following the `Free` template with `FnBrand<RcBrand>`-shaped continuations (i.e., `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` via the unified [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) abstraction). Multi-shot effects (`Choose`, `Amb`). Inherent-method only; no `RcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `fp-library/src/types/arc_free.rs`                                                                | **New module (Phase 1 step 3).** `ArcFree<F, A>` following the `ArcCoyoneda` template with `FnBrand<ArcBrand>`-shaped continuations (i.e., `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` via [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) parameterised by [`ArcBrand`](../../../fp-library/src/brands.rs#L43)) and the `Send`/`Sync` Kind-trait pattern via `SendRefCountedPointer`. Inherent-method only; no `ArcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `fp-library/src/types/rc_free_explicit.rs`                                                        | **New module (Phase 1 step 4).** `RcFreeExplicit<'a, F, A>` extending `FreeExplicit`'s concrete recursive enum with an outer `Rc<RcFreeExplicitInner>` wrapper plus `Rc<dyn Fn>` continuations. O(N) bind, multi-shot, `A: 'a`, Brand-compatible (`RcFreeExplicitBrand<F>` registered in step 4). Custom iterative `Drop` via `Extract` + `Rc::try_unwrap`. Brand-level dispatch in step 7: `Pointed` only on by-value (`pure` has no Clone bound); full `RefFunctor` / `RefSemimonad` / `RefMonad` plus supporting Ref traits per [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). By-value `bind` / `map` ship as inherent methods with their natural `A: Clone` bounds.                                                                                                                                                                                                                                                                                           |
| `fp-library/src/types/arc_free_explicit.rs`                                                       | **New module (Phase 1 step 5).** `ArcFreeExplicit<'a, F, A>` extending `RcFreeExplicit`'s shape with `Arc<...>` wrapping and `Arc<dyn Fn + Send + Sync>` continuations. Same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick as `ArcFree`. Brand-compatible (`ArcFreeExplicitBrand<F>` registered in step 5). Brand-level dispatch in step 7: `SendPointed` (added by step 6) on by-value; full `SendRefFunctor` / `SendRefSemimonad` / `SendRefMonad` plus supporting `SendRef*` traits. By-value `bind` / `map` ship as inherent methods with `A: Clone + Send + Sync` bounds.                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/src/classes/send_functor.rs`, `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` | **New trait files (Phase 1 step 6).** By-value parallels of the existing `send_ref_*` family with `Send + Sync` bounds on the closure parameters and on values entering the structure (`SendPointed::pure(a: A)` requires `A: Send + Sync`). `SendPointed` lands as the brand-level `pure` for `ArcCoyonedaBrand` (closing the open gap module docs flag) and `ArcFreeExplicitBrand`. `SendFunctor` / `SendSemimonad` / `SendMonad` carry trait impls for `ArcCoyonedaBrand` (whose by-value path has no Clone bound). The multi-shot Explicit Free family does not implement `SendFunctor` / `SendSemimonad` / `SendMonad` at the brand level (Clone bound on bind makes them unexpressible) and instead routes brand-level dispatch through the existing `SendRef*` hierarchy in step 7.                                                                                                                                                                                                 |
| `fp-library/src/types/effects.rs`                                                                 | **New module (Phase 2 step 4).** Six concrete Run types: `Run<R, S, A>`, `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family, inherent-method only) and `RunExplicit<'a, R, S, A>` (Explicit, full by-value brand-dispatched), `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>` (Explicit, Pointed/SendPointed by-value plus full Ref/SendRef brand coverage). `Node<R, S>` enum dispatching first-order vs scoped layers. `into_explicit()` / `from_erased()` conversion API between paired Erased and Explicit Run variants.                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/src/types/effects/coproduct.rs`                                                       | **New submodule.** Brand-aware adapter layer over `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`: newtype wrappers, `impl` blocks bridging `frunk_core`'s Plucker / Sculptor / Embedder traits to the project's `Brand` system. Direct (non-newtyped) `Functor` impls on `frunk_core::Coproduct<H, T>` live here too (own-trait + foreign-type, orphan-permitted).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `fp-library/src/types/effects/variant_f.rs`                                                       | **New submodule.** `VariantF<Effects>` first-order coproduct with Coyoneda-wrapped variants and recursive `Functor` impl on `Coproduct<H, T>` (delegating to the adapter in `coproduct.rs`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `fp-library/src/types/effects/scoped.rs`                                                          | **New submodule.** `ScopedCoproduct<ScopedEffects>` higher-order coproduct, standard scoped constructors. `Catch<'a, E>` and `Span<'a, Tag>` ship Val-only. `Local` ships in Val and Ref flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket` ships in Val and Ref flavours (`Bracket<'a, A, B>` + `RefBracket<'a, P, A, B>`) per [decisions.md](decisions.md) section 4.5 sub-decisions. `Mask` is deferred to a future revision per the same section.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/src/dispatch/run/`                                                                    | **New submodule.** Closure-driven Val/Ref dispatch for `bracket` and `local` smart constructors, mirroring the existing layout described in [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). Files: `bracket.rs` (`BracketDispatch` trait + `Val` impl + `Ref<P>` impls per pointer brand + `bracket` inference wrapper + `explicit::bracket` brand-explicit wrapper); `local.rs` (`LocalDispatch` trait + `Val` and `Ref` impls + `local` inference wrapper + `explicit::local` wrapper). Re-exported from `fp-library/src/functions.rs` alongside `map`, `bind`, etc.                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-library/src/types/effects/handler.rs`                                                         | **New submodule.** Handler-pipeline machinery (`Run::handle`), natural-transformation type, `peel` / `send` / `extract`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/src/types/effects/interpreter.rs`                                                     | **New submodule.** `interpret` / `run` / `runAccum` (recursive) and `interpretRec` / `runRec` / `runAccumRec` (`MonadRec`-targeted) families.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-macros/src/effects/`                                                                          | **New module tree.** `effects!`, `effects_coyo!`, `handlers!`, `define_effect!`, `define_scoped_effect!`, `scoped_effects!`, and `run_do!` proc-macros. `run_do!` is the inherent-method-based monadic do-notation for the Erased Run family (`Run` / `RcRun` / `ArcRun`); the Explicit Run family uses the existing `m_do!` / `a_do!` over `RunExplicitBrand` (full by-value brand coverage), and the same macros with the `ref` qualifier (`m_do!(ref RcRunExplicitBrand { ... })`) over `RcRunExplicitBrand` / `ArcRunExplicitBrand` (Ref-hierarchy brand coverage; closures take `&A`). Migration from POC for the row-construction macros.                                                                                                                                                                                                                                                                                                                                            |
| `fp-library/src/brands.rs`                                                                        | Add brands for the Brand-dispatched (Explicit) types only: `FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>`, `RunExplicitBrand<R, S>`, `RcRunExplicitBrand<R, S>`, `ArcRunExplicitBrand<R, S>`. The Erased family (`Free`, `RcFree`, `ArcFree`, `Run`, `RcRun`, `ArcRun`) does NOT get brands; those types remain inherent-method only. `*FreeExplicitBrand<F>` are single-parameter `PhantomData<F>` structs mirroring [`CoyonedaBrand<F>`](../../../fp-library/src/brands.rs#L155); the three `*RunExplicitBrand<R, S>` variants are two-parameter `PhantomData<(R, S)>` structs mirroring [`CoyonedaExplicitBrand<F, B>`](../../../fp-library/src/brands.rs#L171). For all of them, `'static` bounds live on impls (so the row types `R`, `S` and the payload `'a`, `A` stay out of the brand identity and appear only in `Of<'a, A>` at instantiation, keeping brand types `'static`-clean while admitting non-`'static` payloads via the Explicit family). |
| `fp-library/tests/run_*.rs`                                                                       | **New test files.** Per-Free-variant unit tests for all six variants (Phase 1 step 9, including `compile_fail` cases for Brand-dispatched calls against Erased variants and missing `Send + Sync` on `ArcFreeExplicit::bind` closures), row-canonicalisation regression tests migrated from `poc-effect-row/` (Phase 2), `Run <-> RunExplicit` conversion tests (Phase 2 step 6), TalkF + DinnerF integration test (Phase 4).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-library/benches/benchmarks/run_*.rs`                                                          | **New bench files.** Per-Free-variant Criterion benches for all six variants (bind-deep, bind-wide, peel-and-handle) plus a cross-variant comparison documenting the O(1) vs O(N) bind-cost asymmetry between the Erased and Explicit families. Row-canonicalisation benches (macro vs Subsetter), handler-composition benches, and `Run <-> RunExplicit` conversion benches.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |

### Unchanged

- **`Free<F, A>` core** (`fp-library/src/types/free.rs`): the existing
  `Box<dyn FnOnce>`-based variant stays as-is. New variants are
  added beside it.
- **Coyoneda family** (`Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`,
  `CoyonedaExplicit`): used by `Run`'s first-order row but not
  modified.
- **Brand-and-Kind machinery** (`fp-macros` HKT macros, `brands.rs`,
  `kinds.rs`): used by `Run` but not modified beyond the new brand
  registrations above.
- **Optics subsystem** (`Lens`, `Prism`, `Iso`, `Traversal`,
  etc.): unrelated to `Run`.
- **Existing `MonadRec` impls** (`Option`, `Result`, `Thunk`,
  etc.): used as interpretation targets but not modified.
- **Pre-existing dispatch traits and `m_do!` / `a_do!`**: continue
  to work for `RunExplicit` (full by-value brand coverage) once
  the corresponding `RunExplicitBrand` impls from Phase 2 step 4
  ship. `RcRunExplicit` / `ArcRunExplicit` carry only `Pointed` /
  `SendPointed` on the by-value side (the Clone bound on bind
  makes `Functor` / `Semimonad` / `Monad` unexpressible at the
  brand level, per `RcCoyoneda` / `ArcCoyoneda` precedent
  documented in
  [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md));
  brand-dispatched typeclass-generic code over them uses the
  existing `m_do!` / `a_do!` macros with the `ref` qualifier
  (`m_do!(ref RcRunExplicitBrand { ... })`), routing through
  `RefFunctor` / `RefSemimonad` (and their `SendRef*` parallels
  for `ArcRunExplicitBrand`). The Erased Run family (`Run`,
  `RcRun`, `ArcRun`)
  uses the new `run_do!` macro, since those types are not
  Brand-dispatched.

## Out of scope

Permanently excluded from this plan. Revisit only if design
constraints change.

- **Multi-prompt delimited continuations** (Koka / MpEff style).
  Ruled out by [decisions.md](decisions.md) section 1.2; no Rust
  equivalent of GHC's `prompt#` / `control0#`.
- **Tag-based type-level sorting** (workaround 2 from
  decisions §4.1). Surveyed in
  [docs/plans/type-level-sorting/research/](../type-level-sorting/research/);
  the credible building blocks exist (`tstr_crates`) but the full
  sort engine on stable Rust requires the user to write it. The
  workaround-1 macro plus workaround-3 Subsetter hybrid is
  sufficient.
- **Evidence-passing dispatch** (EvEff style). Surveyed in
  [research/deep-dive-evidence-passing.md](research/deep-dive-evidence-passing.md);
  collapses to Option 1 (Peano) or Option 3 (TypeId) once removed
  from Haskell's closed-type-family setting.
- **Coroutine substrate without Free.** Surveyed in
  [research/deep-dive-coroutine-vs-free.md](research/deep-dive-coroutine-vs-free.md);
  loses 4 of 5 first-class-program properties section 4.4
  requires.
- **`mtl`-style trait-bound effect set** (Option 5 from
  decisions §4.1). Loses first-class programs.
- **Custom `Effect`-monad analogue.** Section 9.4 commits to
  `Thunk` (v1) and `Future` (Phase 3) as `MonadRec` targets;
  inventing a Rust-specific `Effect` monad is unnecessary.
- **`async fn`-shaped interpreters.** Section 9.3 commits to
  sync interpreters with async-via-target-monad. No parallel
  `AsyncRun` family.

## Implementation phasing

All five phases ship together as one feature release.

### Phase 1: Complete the Free family

Land the five missing Free variants and the `SendFunctor` trait
family. Phases 2-5 treat the choice of variant as a user-level
parameter, so completing the substrate first prevents later
refactor. The Erased family (`Free`, `RcFree`, `ArcFree`) is
inherent-method only; the Explicit family (`FreeExplicit`,
`RcFreeExplicit`, `ArcFreeExplicit`) carries Brand dispatch. See
[decisions.md](decisions.md) section 4.4 for rationale.

1. Promote `FreeExplicit<'a, F, A>` from POC to
   `fp-library/src/types/free_explicit.rs`. Add iterative custom
   `Drop` per [decisions.md](decisions.md) section 4.4 ("What to
   do about `Drop`"). Delete the POC's local copy in
   `fp-library/tests/free_explicit_poc.rs` and replace with a
   `use fp_library::types::FreeExplicit;` import. Move the bench at
   `fp-library/benches/benchmarks/free_explicit.rs` to use the
   imported type. Un-`#[ignore]` the deep-`Drop` test once the
   iterative `Drop` ships.
2. Implement `RcFree<F, A>` at `fp-library/src/types/rc_free.rs`
   following the `Free` template, with continuations expressed via
   [`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)
   (yielding `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` after `Kind`
   resolution) and the `RcCoyoneda` cloning pattern. Add
   `lower_ref(&self)` / `peel_ref(&self)` for non-consuming
   reinterpretation. The `FnBrand`-based shape is preferred over a
   raw `Rc<dyn Fn>` field so the new module participates in the
   library's unified function-pointer abstraction from day one
   (see [`fn_brand.rs`](../../../fp-library/src/types/fn_brand.rs)).
   Inherent-method only; no `RcFreeBrand` (the `'static` requirement
   from `Rc<dyn Any>` erasure is incompatible with `Kind`'s
   `Of<'a, A: 'a>: 'a` signature).
3. Implement `ArcFree<F, A>` at `fp-library/src/types/arc_free.rs`
   following the `ArcCoyoneda` template, with continuations
   expressed via
   [`FnBrand<ArcBrand>`](../../../fp-library/src/types/fn_brand.rs)
   parameterised by
   [`ArcBrand`](../../../fp-library/src/brands.rs#L43) (yielding
   `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` after
   `Kind` resolution via `SendRefCountedPointer`) and the
   `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick.
   Inherent-method only; no `ArcFreeBrand` (same `'static` reason).
4. Implement `RcFreeExplicit<'a, F, A>` at
   `fp-library/src/types/rc_free_explicit.rs` extending
   `FreeExplicit`'s concrete recursive enum with an outer
   `Rc<RcFreeExplicitInner>` wrapper plus
   [`FnBrand<RcBrand>`](../../../fp-library/src/types/fn_brand.rs)-shaped
   continuations. `A: 'a` (no `'static` requirement) because the
   structure has no `dyn Any` cell; O(N) bind via spine recursion
   through `F::map`. Add `lower_ref(&self)` / `peel_ref(&self)` and
   custom iterative `Drop` (the same `Extract`-driven dismantling
   pattern as `FreeExplicit`, with `Rc::try_unwrap` inside the
   loop). Brand-compatible: this is the multi-shot variant that
   carries Brand dispatch in Phase 1 step 7.
5. Implement `ArcFreeExplicit<'a, F, A>` at
   `fp-library/src/types/arc_free_explicit.rs` extending
   `RcFreeExplicit`'s shape with `Arc<...>` wrapping and
   `Arc<dyn Fn + Send + Sync>` continuations (constructed via
   [`<ArcFnBrand as SendLiftFn>::new`](../../../fp-library/src/classes/send_clone_fn.rs)).
   Same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick
   as `ArcFree`. `Send + Sync`-capable; Brand-compatible (with the
   `SendFunctor` family from step 6 supplying the missing
   trait-method bounds).
6. Add the `SendFunctor` trait family at
   `fp-library/src/classes/`: `send_functor.rs`,
   `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` (the
   by-value parallels of the existing `send_ref_*` files).
   `SendFunctor` / `SendSemimonad` / `SendMonad` take their
   closure parameter as `impl Fn(...) + Send + Sync`;
   `SendPointed::pure(a: A)` requires `A: Send + Sync`. Resolves
   the gap that today prevents `ArcCoyonedaBrand` from
   implementing `Functor` and gives `ArcFreeExplicitBrand` a
   brand-level `pure`. Add `SendFunctor` / `SendPointed` /
   `SendSemimonad` / `SendMonad` implementations for
   `ArcCoyonedaBrand` as a bonus integration, closing the open
   gap that
   [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)'s
   module docs flag (`ArcCoyoneda`'s by-value path has no Clone
   bound, so the full hierarchy lands).
7. Add by-value and by-reference trait hierarchies for the three
   Explicit Free brands. The brand structs (`FreeExplicitBrand<F>`,
   `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>`) and their
   `Kind` registrations land alongside the type definitions in
   steps 1, 4, and 5; this step implements the type-class traits
   on top of them. Brand-level coverage matches the resolved
   open question above:
   - `FreeExplicitBrand`: full by-value (`Functor` / `Pointed` /
     `Semimonad` / `Monad`) plus full by-reference (`RefFunctor`
     / `RefPointed` / `RefSemimonad` / `RefMonad` and supporting
     Ref traits per
     [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md)).
     The naive recursive enum has no Clone bound on bind, so
     both hierarchies land cleanly.
   - `RcFreeExplicitBrand`: `Pointed` only on the by-value side
     (`pure` has no Clone bound); full Ref hierarchy
     (`RefFunctor` / `RefSemimonad` / `RefMonad`, plus
     `RefPointed` and supporting Ref traits). The remaining
     by-value operations (`bind`, `map`, etc.) ship as inherent
     methods on `RcFreeExplicit` with their natural `A: Clone`
     bounds.
   - `ArcFreeExplicitBrand`: `SendPointed` (from step 6) on the
     by-value side; full `SendRef*` hierarchy (`SendRefFunctor`
     / `SendRefSemimonad` / `SendRefMonad` plus supporting
     `SendRef*` traits, which already exist in
     [`fp-library/src/classes/`](../../../fp-library/src/classes/)).
     The remaining by-value operations ship as inherent methods
     on `ArcFreeExplicit` with `A: Clone + Send + Sync` bounds.
   - The Erased family (`Free`, `RcFree`, `ArcFree`) does not get
     brands; those types remain inherent-method only.
   - Both hierarchies are required so `dispatch::map` /
     `dispatch::bind` route correctly over each Brand-dispatched
     Free variant once `Run` and the scoped-effect smart
     constructors land in Phase 2 / Phase 4. The Ref hierarchy
     is the dispatch path for typeclass-generic code over the
     multi-shot Explicit Run variants.
   - Update
     [`fp-library/docs/limitations-and-workarounds.md`](../../../fp-library/docs/limitations-and-workarounds.md)'s
     "Unexpressible Bounds in Trait Method Signatures"
     classification table to add rows for the three Explicit
     Free variants documenting the brand-level coverage above
     (matching the existing `RcCoyoneda` / `ArcCoyoneda` rows).
8. Per-variant Criterion benches for all six variants (bind-deep
   at depths 10 / 100 / 1000 / 10000, bind-wide, peel-and-handle),
   plus a cross-variant comparison bench documenting the O(1) vs
   O(N) bind-cost asymmetry. The existing
   [`free_explicit.rs`](../../../fp-library/benches/benchmarks/free_explicit.rs)
   POC bench has the `bind-deep` shape only; step 8 extends that
   shape with `bind-wide` (single bind closure mapping over a
   wide-but-shallow chain) and `peel-and-handle` (single-step
   `to_view` / `peel_ref` cost) and replicates the full set
   across all six variants.
9. Per-variant unit tests covering construction, evaluation, and
   the property each variant promises (single-shot vs.
   multi-shot, thread-safe, `'static` vs `'a`, Brand-dispatched
   vs inherent-method-only). The canonical interpretation method
   varies by variant: `Free::fold_free` for `Free` (the only
   variant with that inherent method); `evaluate` for
   `RcFree`/`ArcFree`/`FreeExplicit`/`RcFreeExplicit`/`ArcFreeExplicit`.
   Plus `compile_fail` UI tests under
   [`fp-library/tests/ui/`](../../../fp-library/tests/ui/)
   (registered via the existing
   [`fp-library/tests/compile_fail.rs`](../../../fp-library/tests/compile_fail.rs)
   `trybuild` harness) for the negative cases: multi-shot via
   `Free`, Brand-dispatched call against an Erased variant,
   missing `Send + Sync` on a closure passed to
   `ArcFreeExplicit::bind`, missing `Clone` on a closure passed
   to `RcFree::bind`, etc.

### Phase 1 follow-up: `WrapDrop` migration (resolves Phase 2 step 4 blocker)

These commits land between Phase 1 and Phase 2 step 4. They lift
the Free family's struct-level `Extract` bound so the Phase 2
step 4 architectural commitment
`Run<R, S, A> = Free<NodeBrand<R, S>, A>` can compile over
typical effect rows whose effect types do not implement
`Extract`. Full rationale, problem statement, probe results, and
per-F policy decisions live at
`## Open questions, issues and blockers -> ### Resolved (2026-04-27): introduce WrapDrop trait for Free's struct-level Drop concern`;
this section is the phasing-side checklist.

1. **Introduce the `WrapDrop` trait and migrate the Free family.**
   New trait at `fp-library/src/classes/wrap_drop.rs` with
   signature
   `fn drop<'a, X: 'a>(fa: Self::Of<'a, X>) -> Option<X>`.
   `WrapDrop` impls for the two existing
   `Extract`-implementing brands (`IdentityBrand`,
   `ThunkBrand`), each delegating to their existing
   `Extract` impl by returning
   `Some(<Self as Extract>::extract(fa))`. Replace
   `F: Extract + Functor + 'static` with
   `F: WrapDrop + Functor + 'static` on the struct, `FreeView`,
   `FreeStep`, and `Drop` declarations of all six Free
   variants (`Free`, `RcFree`, `ArcFree`, `FreeExplicit`,
   `RcFreeExplicit`, `ArcFreeExplicit`). Inventory: 71
   occurrences of `F: Extract` across the six variant source
   files, mechanically migrated. Methods that call
   `F::extract` semantically (`evaluate`, `resume`, etc.)
   keep `where F: Extract` on their impl blocks. Rewrite
   Free's `Drop` loop to call `F::drop` and switch on the
   returned `Option`: `Some(inner)` follows the existing
   iterative path; `None` lets the layer drop in place.
   Existing Phase 1 tests (including
   `deep_drop_does_not_overflow` for both `Free<ThunkBrand>`
   and `FreeExplicit<IdentityBrand>`) must all pass.
2. **Relax `Functor` bound to `Kind` on Free's struct.**
   Change the struct, `FreeView`, `FreeStep`, and `Drop`
   bounds from `F: WrapDrop + Functor + 'static` to
   `F: WrapDrop + 'static` (the `Kind` GAT requirement is
   inherited from `WrapDrop`'s `Kind` supertrait). Add
   `where F: Functor` to impl blocks that call `F::map`
   (`wrap`, `lift_f`, `to_view`, and methods that go through
   them transitively such as `evaluate`, `resume`,
   `fold_free`). Methods like `pure`, `bind`, `map` (the
   inherent method, not `Functor::map`) do not need the
   bound. Process: relax the struct bound, run
   `cargo check`, add `where F: Functor` to every impl block
   the compiler flags, repeat until clean. Same six Free
   variants. Same tests must pass.

### Phase 2: Run substrate and first-order effects

1. Add `frunk_core` as a direct dependency of `fp-library`
   (license check via `just deny`, MSRV verification, and
   workspace `Cargo.toml` registration). Introduce a thin
   Brand-aware adapter layer at `fp-library/src/types/effects/coproduct.rs`:
   newtype wrappers around `frunk_core::coproduct::{Coproduct, CNil}`
   plus `impl` blocks bridging `frunk_core`'s Plucker / Sculptor /
   Embedder traits to the project's `Brand` system. Direct `impl`s
   of fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   are permitted by the orphan rules; `Brand`-style impls require
   the newtype wrapper. See Implementation note 1 below.
2. `VariantF<Effects>` at `fp-library/src/types/effects/variant_f.rs`:
   Coyoneda-wrapped Coproduct row with recursive `Functor` impl
   on `Coproduct<H, T>` (where `H: Functor + T: Functor`) and base
   case on `CNil`. Migrate the trait-shape from
   [poc-effect-row/src/lib.rs](../../../poc-effect-row/src/lib.rs)
   under the production `Functor` trait.
3. `Member<E, Indices>` trait for first-order injection /
   projection, layered on top of `frunk_core::CoproductSubsetter`
   via the adapter from step 1.
4. Six `Run` types at `fp-library/src/types/effects.rs` (and
   sibling files), one per Free variant: `Run<R, S, A>`,
   `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family,
   inherent-method only) and `RunExplicit<'a, R, S, A>`,
   `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>`
   (Explicit family, Brand-dispatched). Each is a thin wrapper
   over its Free variant with a shared `Node<R, S>` enum
   dispatching first-order vs scoped layers.
   This step depends on the Phase 1 follow-up commits above
   (the `WrapDrop` migration); without them,
   `Free<NodeBrand<R, S>, A>` does not compile because effect
   types do not implement `Extract`. As part of this step,
   `WrapDrop` impls also land for the row brands that this
   step exercises:
   - `NodeBrand<R, S>`: dispatches by First/Scoped, delegating
     to `R::drop` and `S::drop` respectively. (New brand
     defined in this step.)
   - `CoproductBrand<H, T>` (already exists from Phase 2 step 2):
     dispatches by `Inl`/`Inr`, delegating to `H::drop` and
     `T::drop`.
   - `CNilBrand` (already exists from Phase 2 step 2): the
     uninhabited base case, `match fa {}`.
   - `CoyonedaBrand<E>` (already exists): returns `None`. The
     Coyoneda's stored function would construct a Free if
     called, but does not store one in its environment;
     recursive drop on the Coyoneda is sound for Run-typical
     patterns per the Wrap-depth probe findings recorded in
     the resolution above.
   - `RunExplicitBrand`: full by-value (`Functor` / `Pointed` /
     `Semimonad` / `Monad`) plus full by-reference (`RefFunctor`
     / `RefPointed` / `RefSemimonad` / `RefMonad` and supporting
     Ref traits) by delegating to `FreeExplicitBrand`'s impls
     from Phase 1 step 7.
   - `RcRunExplicitBrand`: `Pointed` only on the by-value side
     (delegating to `RcFreeExplicitBrand::pure`); full Ref
     hierarchy delegating to `RcFreeExplicitBrand`'s `Ref*`
     impls. By-value `bind` / `map` ship as inherent methods on
     `RcRunExplicit`, mirroring `RcFreeExplicit`'s inherent
     surface.
   - `ArcRunExplicitBrand`: `SendPointed` (added by step 6) on
     the by-value side; full `SendRef*` hierarchy delegating to
     `ArcFreeExplicitBrand`'s impls. By-value `bind` / `map`
     ship as inherent methods on `ArcRunExplicit` with
     `A: Clone + Send + Sync` bounds.
   - The three Erased Run types do NOT get brands. They expose
     identical inherent-method APIs (`pure`, `peel`, `send`,
     `bind`, `map`, `lift_f`, `handle`, `extract`, etc.) but
     `m_do!` / `a_do!` do not work over them; `run_do!` from
     step 7 below is the inherent-method-based macro analogue.
   - The hierarchies on the Explicit brands are scoped so
     `dispatch::map` / `dispatch::bind` and the matching
     do-notation macros (`m_do!` / `a_do!` for `RunExplicit`;
     `m_do!(ref ...)` / `a_do!(ref ...)` for `RcRunExplicit` /
     `ArcRunExplicit`) route correctly. Inherent by-value `bind`
     / `map` on the multi-shot Explicit Run variants cover the
     non-generic case where the user has `A: Clone` available.
5. `Run::pure`, `Run::peel`, `Run::send` core operations on
   each of the six Run variants, delegating to the underlying
   Free variant.
6. Conversion methods between paired Erased and Explicit Run
   variants: `Run::into_explicit() -> RunExplicit`,
   `RcRun::into_explicit() -> RcRunExplicit`,
   `ArcRun::into_explicit() -> ArcRunExplicit`, and the reverse
   `RunExplicit::from_erased(...)`, etc. Walks the Free
   structure once via `peel` / `to_view`, rebuilds in the other
   shape; O(N) in the chain depth. Preserves multi-shot /
   `Send + Sync` properties of the underlying substrate
   (`RcRun -> RcRunExplicit` keeps multi-shot via `Rc<dyn Fn>`
   continuations on both sides; `ArcRun -> ArcRunExplicit`
   keeps `Send + Sync`).
7. `run_do!` macro in `fp-macros/src/effects/run_do.rs`, the
   inherent-method-based monadic do-notation that desugars to
   chained `.bind(|x| ...)` calls. Required for the Erased Run
   family (`Run`, `RcRun`, `ArcRun`) since they are not
   Brand-dispatched and `m_do!` does not apply. `RunExplicit`
   uses the existing `m_do!` / `a_do!` over `RunExplicitBrand`
   (full by-value brand coverage). `RcRunExplicit` /
   `ArcRunExplicit` use the existing `m_do!` / `a_do!` macros
   with the `ref` qualifier (`m_do!(ref RcRunExplicitBrand { ... })`),
   dispatching through the `Ref*` / `SendRef*` hierarchies that
   step 4 wires up on `RcRunExplicitBrand` / `ArcRunExplicitBrand`.
   All forms accept the same surface syntax so users moving
   between families do not have to re-learn the do-notation; the
   only difference is whether their binding closures take `A`
   (by-value) or `&A` (by-reference, signalled by the `ref`
   qualifier).
8. `effects!` macro in `fp-macros/src/effects/effects.rs`,
   migrated from
   [poc-effect-row/macros/src/lib.rs](../../../poc-effect-row/macros/src/lib.rs).
   Lexical-sort by `quote!{}.to_string()`; emit Coyoneda-wrapped
   variants. The un-wrapped Coproduct form lives at
   `crate::__internal::raw_effects!` for fp-library-internal use
   (test fixtures, lower-level combinators) and is not part of
   the public surface; see [decisions.md](decisions.md) section 4.6.
   Factor the lexical-sort logic into a shared `proc-macro2`
   helper used by both `effects!` and `scoped_effects!` (Phase 4
   step 4) so sort-correctness fixes land in one place.
9. Coyoneda-wrapping smart constructors (`lift_f` analogues for
   each effect type).
10. Migrate the 25 row-canonicalisation tests from
    `poc-effect-row/tests/` into
    `fp-library/tests/run_row_canonicalisation.rs` as the
    regression baseline. Verify all pass under the production
    types (exercise both Erased and Explicit Run families).
    Delete the POC repository once the migration lands.

### Phase 3: First-order effect handlers, interpreters, natural transformations

1. `handlers!{...}` macro in
   `fp-macros/src/effects/handlers.rs` producing tuple-of-closures
   keyed on the row's type-level structure. Builder fallback
   (`nt().on::<E>(handler)...`) as the non-macro path
   ([decisions.md](decisions.md) section 4.6).
2. `interpret` / `run` / `runAccum` recursive-target interpreter
   family in `fp-library/src/types/effects/interpreter.rs`.
3. `interpretRec` / `runRec` / `runAccumRec` `MonadRec`-target
   interpreter family in the same module.
4. Standard first-order effect types and their smart
   constructors: `State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`,
   `Choose` (multi-shot, `RcRun`-only).
5. `define_effect!` macro at
   `fp-macros/src/effects/define_effect.rs` generating effect
   enum + smart constructors + label / brand registration.
6. `compile_fail` UI tests for negative cases (handler missing an
   effect, wrong type ascription, multi-shot via single-shot
   `Run`).

### Phase 4: Scoped effects (heftia dual row)

1. `ScopedCoproduct<ScopedEffects>` at
   `fp-library/src/types/effects/scoped.rs` with the dual-row
   integration into `Run<Effects, ScopedEffects, A>`.
2. Standard scoped-effect constructors. Per
   [decisions.md](decisions.md) section 4.5 sub-decisions, `Bracket`
   and `Local` ship in two parallel flavours each (Val and Ref) that
   mirror the library's existing Val/Ref dispatch pattern at
   [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md);
   `Catch` and `Span` ship Val-only (Ref flavours rejected per the
   sub-decision).
   - `Catch<'a, E>` for `Error.catch`, with `action: Run<R, S, A>`,
     `handler: Box<dyn FnOnce(E) -> Run<R, S, A>>`. Val only.
   - `Local<'a, E>` (Val flavour) for `Reader.local` with a
     consuming modify, holding `modify: Box<dyn FnOnce(E) -> E>`,
     `action: Run<R, S, A>`.
   - `RefLocal<'a, E>` (Ref flavour) for `Reader.local` with a
     borrowing modify, holding `modify: Box<dyn FnOnce(&E) -> E>`,
     `action: Run<R, S, A>`. Removes the `E: Clone` requirement
     that the Val flavour imposes when users want to derive a
     sub-scope env from the parent without owning it.
   - `Bracket<'a, A, B>` (Val flavour) for non-refcounted-substrate
     users (`Run` / `RunExplicit`), with `acquire: Run<R, S, A>`,
     `body: Box<dyn FnOnce(A) -> Run<R, S, (A, B)>>`,
     `release: Box<dyn FnOnce(A) -> Run<R, S, ()>>`. The body
     consumes `A`, threads it back to the interpreter via
     `(A, B)`, and the interpreter moves the returned `A` into
     `release`.
   - `RefBracket<'a, P, A, B>` (Ref flavour) for refcounted-substrate
     users (`RcRun`, `ArcRun`, `RcRunExplicit`, `ArcRunExplicit`),
     parameterised by
     [`P: RefCountedPointer`](../../../fp-library/src/classes/ref_counted_pointer.rs)
     ([`RcBrand`](../../../fp-library/src/brands.rs#L250) for
     `RcRun` / `RcRunExplicit`,
     [`ArcBrand`](../../../fp-library/src/brands.rs#L43) for
     `ArcRun` / `ArcRunExplicit`), with `acquire: Run<R, S, A>`,
     `body: Box<dyn FnOnce(P::Of<A>) -> Run<R, S, B>>`,
     `release: Box<dyn FnOnce(P::Of<A>) -> Run<R, S, ()>>`. Body
     and release both receive a pointer clone; the resource lives
     until the last clone drops, mirroring PureScript's
     GC-aliased `bracket` semantics
     ([`Aff.purs:308`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs#L308)).
   - `Span<'a, Tag>`, with `tag: Tag`, `action: Run<R, S, A>`.
     Val only (no closure to dispatch over).
3. Scoped-effect interpreter trait. Method per constructor;
   fixed `Run<R, A>` continuation
   ([decisions.md](decisions.md) section 4.5).
4. `scoped_effects!` macro and `define_scoped_effect!` macro,
   sharing the lexical-sort helper with Phase 2's `effects!` (one
   helper, two thin entry-point macros, distinct output shapes:
   Coyoneda-wrapped Coproduct vs `ScopedCoproduct`).
5. Smart constructors: `catch`, `span` (single-flavour
   wrappers); `bracket` and `local` (closure-driven dispatch over
   Val and Ref flavours, reusing the existing `Val` / `Ref`
   markers and dispatch machinery from
   [`fp-library/src/dispatch/`](../../../fp-library/src/dispatch/);
   `bracket`'s Ref impl additionally carries the pointer brand
   `P` so `Ref<RcBrand>` and `Ref<ArcBrand>` resolve to distinct
   `RefBracket` node types). Concretely:
   - `BracketDispatch<R, S, A, B, Marker>` trait with `Val` impl
     (closures of shape `FnOnce(A) -> Run<R, S, (A, B)>` plus
     `FnOnce(A) -> Run<R, S, ()>`) and `Ref<P>` impls for each
     `P: RefCountedPointer` (closures of shape
     `FnOnce(P::Of<A>) -> Run<R, S, B>` plus
     `FnOnce(P::Of<A>) -> Run<R, S, ()>`).
   - `LocalDispatch<R, S, E, A, Marker>` trait with `Val` impl
     (`FnOnce(E) -> E`) and `Ref` impl (`FnOnce(&E) -> E`).
   - The dispatch traits and their impls live at
     `fp-library/src/dispatch/run/bracket.rs` and
     `fp-library/src/dispatch/run/local.rs`, mirroring the
     existing layout described in
     [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md).
     No `mask` smart constructor in v1; the `Mask` constructor is
     deferred per [decisions.md](decisions.md) section 4.5
     sub-decisions.
6. Standard handlers (`run_reader`'s `local` clause,
   `run_except`'s `catch` clause, etc.) wired through the dual
   row.
7. Tests: scoped-effect unit tests covering each of the four
   standard constructors (`Catch`, `Local`, `Bracket`, `Span`)
   plus `compile_fail` cases. Reformulate relevant Phase 3 tests
   to use scoped operations where appropriate.

### Phase 5: Integration test, deferred items as needed

1. Port the canonical TalkF + DinnerF example from
   [`purescript-run/test/Examples.purs`](https://github.com/natefaubion/purescript-run/blob/master/test/Examples.purs#L13-L106)
   into
   `fp-library/tests/run_talkf_dinnerf_integration.rs`.
   Multi-effect program demonstrating Reader, State, Talk, and
   Dinner effects composed and handled in turn. Faithful port
   from PureScript's source.
2. Add row-canonicalisation Criterion benches (macro path vs
   `CoproductSubsetter` permutation-proof fallback path) and
   handler-composition benches per
   [decisions.md](decisions.md) section 9 item 6.
3. (Phase 3 deferred items, scheduled here so they're not lost):
   - Optional `tstr_crates` content-addressed-naming refinement
     for the macro layer
     ([decisions.md](decisions.md) section 4.1's Phase 2 note).
     Add only if real-world usage shows import-path-sensitive
     sorting causes confusion.
   - Compile-time index-table refinement (Koka-inspired). Add
     only if a benchmark shows Coproduct pattern-match dispatch
     is a measurable bottleneck.
4. Write `fp-library/docs/run.md` documenting the effects
   subsystem for users. Cross-link to
   [decisions.md](decisions.md) for design rationale.
5. **Documentation finalization.** Update the documents listed
   below so they reflect the production state of the effects
   subsystem once Phases 1-5 are complete.

   **Living step:** Each implementation phase, on completion,
   must review the bullets here and add any new public items,
   behavioural surprises, or constraints that surfaced during
   that phase's work, under the relevant document. The goal is
   that when this step finally runs, every documentation change
   it lists is accurate and nothing has been forgotten. Treat
   the per-document bullets as a checklist that grows over time;
   do not rely on memory or `git log` to reconstruct the change
   set at the end. If a phase finds that a planned doc update
   is no longer needed (e.g., a feature was deferred), strike
   it through with rationale rather than deleting it.

   Documents and what to add:
   - **[fp-library/docs/features.md](../../../fp-library/docs/features.md):**
     Add a "Free family" table parallel to the existing "Free
     functors" Coyoneda table (currently around lines 199-204),
     listing the six variants (`Free`, `RcFree`, `ArcFree`,
     `FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) with
     columns for Family (Erased / Explicit), Clone, Send,
     `'a`-payload, Bind cost (O(1) vs O(N)). Add a "Run
     subsystem" section listing the six concrete Run types,
     the dual-row structure, the Erased/Explicit dispatch
     split, and the `into_explicit` / `from_erased` API.
   - **[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md):**
     The "Unexpressible Bounds in Trait Method Signatures"
     classification table already has rows for the three
     Explicit Free variants (added by Phase 1 step 7). Append
     rows for any `*RunExplicitBrand` (Phase 2 step 4) or
     scoped-effect dispatch (Phase 4) impls that hit further
     HRTB-over-types or per-`A` Clone-bound walls.
   - **[fp-library/CHANGELOG.md](../../../fp-library/CHANGELOG.md):**
     Populate the `[Unreleased]` section under `Added` with the
     new public items: six-variant Free family (one promoted
     from POC, five new), `SendFunctor` trait family, six
     `Run` types, `Node` / `VariantF` / `ScopedCoproduct`,
     standard first-order effects (`State`, `Reader`, `Except`,
     `Writer`, `Choose`), standard scoped effects (`Catch`,
     `Local` / `RefLocal`, `Bracket` / `RefBracket`, `Span`),
     the macro family (`effects!`, `effects_coyo!`,
     `handlers!`, `define_effect!`, `define_scoped_effect!`,
     `scoped_effects!`, `run_do!`), the
     `interpret`/`interpretRec`/`run*` interpreter pair, and
     the natural-transformation builder. If any pre-existing
     public API changed shape during the port, record it under
     `Changed`. Match the categorization style established in
     0.17.x entries.
   - **[README.md](../../../README.md):** Add a brief
     "Effects" entry alongside the existing "Dispatch System"
     summary, pointing at `fp-library/docs/run.md` (created by
     step 4) for details.
   - **[docs/todo.md](../../../docs/todo.md):** Strike through
     or remove the "Algebraic effects/effect system" bullet
     (and its sub-bullets pointing at
     [plans/effects/effects.md](effects.md) and external Eff
     references); the work it tracks is now landed.
   - **[fp-library/docs/architecture.md](../../../fp-library/docs/architecture.md):**
     If the effects subsystem warrants top-level architectural
     description (parallel to existing "Free Functions" /
     "Dispatch" sections), add one summarising the
     six-variant Free substrate, the Erased/Explicit dispatch
     split, the dual-row Run shape, and the heftia-style
     scoped-effect encoding. Skip if `run.md` (step 4) already
     covers this depth and an architecture-level summary
     would duplicate.
   - **[fp-library/docs/dispatch.md](../../../fp-library/docs/dispatch.md):**
     If Phase 4's `BracketDispatch` / `LocalDispatch` Val/Ref
     dispatch introduces a pattern that doesn't follow the
     existing convention this doc describes, add a section
     covering the new shape. Skip if the new dispatch is a
     direct application of the existing pattern.

   Per-phase records (append as phases complete):
   - **Phase 1 (complete).** Six Free variants land
     (`FreeExplicit` promoted, `RcFree` / `ArcFree` /
     `RcFreeExplicit` / `ArcFreeExplicit` new), `SendFunctor`
     trait family lands across nine files, brand impls for the
     three Explicit Free brands land. The
     `limitations-and-workarounds.md` "Unexpressible Bounds"
     table gained six new rows (three by-value, three
     by-reference) for the Explicit Free family;
     `features.md` and `CHANGELOG.md` are not yet updated for
     these (waiting for this finalization step). The Phase 1
     step 8 finding that `Free<IdentityBrand, A>` is
     layout-cyclic should be mentioned in `run.md` (step 4)'s
     "When to use which" section because it constrains
     concrete-`F` choices.
   - **Phase 2 (in progress).** Phase 2 ships the `WrapDrop`
     trait at
     `fp-library/src/classes/wrap_drop.rs`
     as a Phase 1 retroactive refinement before step 4 resumes
     (see Open questions resolution above for the rationale).
     `WrapDrop` is a new public trait that needs to land in
     `features.md` (effects subsystem section) and the
     `CHANGELOG.md` `[Unreleased]` Added list. Free's struct
     bound migrates from
     `F: Extract + Functor + 'static` to
     `F: WrapDrop + 'static`; this is technically a breaking
     change to the bound but is purely-additive in practice
     because every existing F that implements `Extract` gains
     a paired `WrapDrop` impl.
     Append remaining findings here when Phase 2 completes:
     new public items in `Run`, `VariantF`, `Node`, the
     `effects!` macro, the `run_do!` macro, the conversion
     API, plus any unexpressible-bound rows that surface in
     the `*RunExplicitBrand` impls.
   - **Phase 3 (TBD).** Append findings here when complete:
     handler-pipeline machinery, interpreter family, standard
     first-order effects, `handlers!` and `define_effect!`
     macros, plus negative-case `compile_fail` UI tests.
   - **Phase 4 (TBD).** Append findings here when complete:
     scoped-effect coproduct, dual-row integration, the four
     standard scoped-effect constructors and their Val/Ref
     flavours where applicable, dispatch additions for
     `bracket` / `local`, plus any new dispatch.md /
     limitations-and-workarounds.md material.

### Phase 6+ (deferred, not in this plan)

These items arrive when concrete need surfaces. Each one names
the artifact, what it would deliver, why it is deferred, and a
revisit trigger; entries are ordered roughly from substrate
outward to user surface.

- **Cargo feature gating for the Free family.** Cargo feature
  gates that let downstream crates opt out of compiling
  individual Free variants if their compile cost becomes
  uncomfortable ([decisions.md](decisions.md) section 4.4
  "Open questions left after this decision"). _Why deferred:_
  the compile cost of shipping six variants plus the
  `SendFunctor` trait family is unverified; a real
  feature-gating design needs benchmark or downstream-feedback
  evidence to be motivated. _Trigger:_ benchmark or compile-time
  evidence that the unified compile cost is meaningfully painful
  for downstream crates.
- **`State::modify` Val/Ref split.** Add a `RefState<S>`
  first-order effect alongside `State<S>` whose `modify`
  operation takes `FnOnce(&S) -> S` instead of `FnOnce(S) -> S`,
  with a unified `modify(...)` smart constructor dispatching
  over closure shape via the same `Val` / `Ref` markers used by
  `Local` / `RefLocal` in Phase 4. _What this is for:_ users who
  want to derive a new state from the old without owning it,
  avoiding an `S: Clone` requirement. _Why deferred:_ Phase 3's
  standard first-order effect set ships Val-only to keep the
  surface small; the Run-brand by-ref hierarchy from Phase 2
  step 4 already supplies the trait routing this would build
  on. _Trigger:_ first user who hits the `S: Clone` wall on the
  Val flavour, or the first integration test that benefits from
  `&S` in `modify`.
- **`Writer::censor` Val/Ref split.** Add `censor` to the
  standard `Writer<W>` set (currently only `tell` ships in
  Phase 3 step 4), then ship a `RefWriter<W>` extension whose
  `censor` takes `FnOnce(&W) -> W` instead of
  `FnOnce(W) -> W`. _What this is for:_ deriving a transformed
  log without consuming the parent, the writer-log analogue of
  `State::modify`'s ergonomic story. _Why deferred:_ `censor`
  itself is not in v1's standard Writer set, and the Val/Ref
  split is a follow-up to adding it. _Trigger:_ a real
  log-censoring use case, plus the same `W: Clone` ergonomic
  wall.
- **`handlers!{...}` macro Val/Ref variants.** Extend the
  Phase 3 macro so each per-effect handler entry can be emitted
  as Val or Ref based on the user's closure type, reusing the
  same `Val` / `Ref` markers as the rest of the dispatch
  system. Each entry is conceptually a closure of shape
  `FnOnce(E::Operation) -> Run<R', S, A>`; the Ref variant
  takes `FnOnce(&E::Operation) -> Run<R', S, A>`. _What this is
  for:_ handlers that inspect operations without consuming them
  (e.g., a logging handler that records and then forwards),
  avoiding a `Clone` bound on operation payloads. _Why
  deferred:_ the macro is already non-trivial in v1; shipping it
  Val-only first and extending it once a real handler benefits
  is a smaller initial bite, and the extension is non-breaking.
  _Trigger:_ first handler (in the standard library or
  downstream) that needs inspect-without-consuming on an
  operation payload.
- **`generalBracket` and `BracketConditions`.** Port the
  more general bracket from PureScript Aff at
  [`Aff.purs:364-373`](https://github.com/purescript-contrib/purescript-aff/blob/master/src/Effect/Aff.purs#L364-L373):
  `generalBracket` accepts a `BracketConditions` record with
  separate `killed`, `failed`, and `completed` handlers, each
  receiving the resource. _What this is for:_ observing how a
  bracketed action terminated (success, failure, cancellation)
  and running different cleanup per outcome, instead of v1's
  single uniform `release`. _Why deferred:_ v1's interpreter is
  sync and has no cancellation event, so `killed` has no
  semantics until the async target monad lands; without async,
  `generalBracket` collapses to a more verbose `bracket` with
  two unused branches. The Ref-flavour `RefBracket<'a, P, A, B>`
  already shows that multiple closures can each receive
  `P::Of<A>` without contention, so the dispatch design extends
  cleanly when the time comes. _Trigger:_ the async target
  monad lands (next item), at which point cancellation becomes
  a real event handlers want to observe.
- **`MonadRec` impl for `Future` as an async target monad**
  ([decisions.md](decisions.md) section 9 items 3 + 4). _What
  this is for:_ asynchronous interpretation of `Run` programs
  via the same target-monad mechanism that already lets users
  pick `Identity` or `Thunk` for sync interpretation; no
  parallel `AsyncRun` family. _Why deferred:_ v1 ships sync
  interpreters and async users wrap calls in `spawn_blocking`
  or similar; adding `Future` as a `MonadRec` target requires
  designing around pinned futures, executor coupling, and
  multi-shot continuation friction, which is a separate body of
  work. _Trigger:_ first user request for async interpretation
  that cannot be satisfied by `spawn_blocking` around a sync
  interpreter call.
- **Split `fp-macros` into `fp-effects-macros`**
  ([decisions.md](decisions.md) section 9 item 8). _What this
  is for:_ a separate crate housing the effects-related
  proc-macros (`effects!`, `effects_coyo!`, `handlers!`,
  `define_effect!`, `define_scoped_effect!`,
  `scoped_effects!`) so their release cadence is independent of
  the HKT-system macros and do-notation macros that share
  `fp-macros` today. _Why deferred:_ v1 keeps everything in one
  crate to avoid multiplying release coordination and adding a
  parallel macro-resolution path. _Trigger:_ `fp-macros`
  compile time grows uncomfortably, or effects-related macro
  changes start blocking unrelated macro releases.

## Implementation notes

1. **`Coproduct` choice (Phase 2).** The POC depends on
   `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`,
   and the production port adopts the same dependency. Phase 2
   step 1 adds `frunk_core` to `fp-library`'s `Cargo.toml`,
   confirms the license is permitted by `just deny`, and
   introduces a thin Brand-aware adapter layer at
   `fp-library/src/types/effects/coproduct.rs` (newtypes plus `impl`
   blocks that bridge `frunk_core`'s Plucker / Sculptor / Embedder
   traits to the project's `Brand` system). Implementing
   fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   directly is permitted by the orphan rules (own-trait +
   foreign-type) and is the preferred shape for the recursive
   `Functor` dispatch; `Brand`-style impls on the foreign type
   require the newtype wrapper. If the adapter ever exceeds
   approximately 200 lines, that signals real impedance mismatch
   and a fork to an in-house reimplementation should be
   considered, but the default is to stay on `frunk_core`.
2. **POC-to-production migration (Phases 2 + 3).** The POC at
   `poc-effect-row/` is a separate Cargo workspace and does not
   integrate with fp-library's `Brand` system; the production
   types use `Brand` machinery throughout. Migration is mostly
   mechanical (swap the stub Coyoneda for fp-library's, swap the
   raw Coproduct types for branded equivalents) but expect
   surface-area changes around the macro output (the `effects!`
   macro must emit Brand-shaped types in production). Plan one
   step per POC test as a regression-safety strategy.
3. **`Drop` correctness (Phase 1).** `RcFree` and `ArcFree`
   inherit `Free`'s iterative `Drop` strategy via the underlying
   `CatList`; `FreeExplicit` requires its own iterative `Drop`
   per the POC findings. Test deep-`Drop` for all four variants
   in Phase 1 unit tests.
4. **Async-via-target-monad gating (Phase 5+).** The interpreter
   functions stay sync; an async target monad arrives in Phase 6+.
   Until then, async users wrap the interpreter call in
   `spawn_blocking` or similar.

## Success criteria

The plan is complete when all of the following hold:

- All six Run types are publicly exported from `fp-library`:
  `Run`, `RcRun`, `ArcRun` (Erased family, inherent-method only)
  and `RunExplicit`, `RcRunExplicit`, `ArcRunExplicit` (Explicit
  family, Brand-dispatched). Conversion methods
  (`into_explicit()` / `from_erased(...)`) link paired Erased
  and Explicit variants.
- The `effects!` macro accepts `effects![A, B, C]` over arbitrary
  effect types and produces a canonical row across input
  orderings; the same row composes with `CoproductSubsetter`
  permutation proofs for hand-written cases.
- `m_do!` and `a_do!` work over the three Explicit Run brands
  (`RunExplicitBrand`, `RcRunExplicitBrand`,
  `ArcRunExplicitBrand`) for first-order effect programs.
  `run_do!` provides the equivalent monadic do-notation for the
  three Erased Run types via inherent methods.
- Each of the six Free variants supports its promised property
  (single-shot vs. multi-shot, thread-safe, `'static` vs `'a`,
  Brand-dispatched vs inherent-method-only) with per-variant unit
  tests passing.
- The `SendFunctor` / `SendPointed` / `SendSemimonad` /
  `SendMonad` trait family ships and is used by
  `ArcFreeExplicitBrand` and `ArcRunExplicitBrand` for their
  by-value Brand impls. `ArcCoyonedaBrand` also gains
  `SendFunctor` (and downstream) impls, retroactively closing
  the gap that
  [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)'s
  module docs flag.
- `Reader`, `State`, `Except`, `Writer`, `Choose` ship as standard
  first-order effects with smart constructors.
- `Catch<'a, E>` and `Span<'a, Tag>` ship as Val-only
  scoped-effect constructors; `Local` ships in Val and Ref
  flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket`
  ships in Val and Ref flavours (`Bracket<'a, A, B>` +
  `RefBracket<'a, P, A, B>` parameterised over
  `P: RefCountedPointer`), each with scoped-handler
  interpreters. The `bracket` and `local` smart constructors
  use closure-driven Val/Ref dispatch (mirroring
  [`dispatch.md`](../../../fp-library/docs/dispatch.md)) so the
  user picks the flavour by closure type, not by turbofish.
  (`Mask` deferred per [decisions.md](decisions.md) section 4.5
  sub-decisions.)
- The by-value hierarchy (`Functor` / `Pointed` / `Semimonad` /
  `Monad`, with `SendFunctor` / etc. for the `Arc`-affected
  brand) and by-reference hierarchy
  (`RefFunctor` / `RefSemimonad` / `RefMonad`, etc.) are both
  implemented for every Brand-dispatched Free variant
  (`FreeExplicitBrand<F>`, `RcFreeExplicitBrand<F>`,
  `ArcFreeExplicitBrand<F>`) and every Brand-dispatched Run
  variant (`RunExplicitBrand`, `RcRunExplicitBrand`,
  `ArcRunExplicitBrand`); `dispatch::map` / `dispatch::bind`
  route correctly for both consuming and borrowing closures over
  these brands. The Erased family (`Free`, `RcFree`, `ArcFree`,
  `Run`, `RcRun`, `ArcRun`) does NOT participate in dispatch by
  design; users access those types via inherent methods or
  convert to the corresponding Explicit variant.
- The TalkF + DinnerF integration test passes.
- All 25 row-canonicalisation tests migrated from
  `poc-effect-row/` pass under the production types.
- Per-Free-variant Criterion benches show no regression beyond
  ~50% of the `FreeExplicit` POC baseline (~27ns / node in the
  linear regime).
- `just verify` passes (fmt, check, clippy, deny, doc, test).

## Reference material

- Design and decisions: [decisions.md](decisions.md).
- Effects research arc:
  [research/](research/) (13 codebase classifications,
  `_classification.md` synthesis, 3 Stage 2 deep dives).
- Type-level-sorting research arc:
  [../type-level-sorting/research/](../type-level-sorting/research/)
  (16 codebase classifications, `_classification.md` synthesis).
- POC validation:
  - [poc-effect-row/](../../../poc-effect-row/) -- row-encoding
    hybrid, `tstr_crates` refinement, static-via-Coyoneda.
  - [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
    -- POC findings document.
  - [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)
    -- `FreeExplicit` POC.
  - [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)
    -- `FreeExplicit` Criterion bench.
- PureScript Run reference:
  [`purescript-run`](https://github.com/natefaubion/purescript-run).
- Comparison table for the Rust port versus PureScript Run and
  Hasura's `eff` is in [decisions.md](decisions.md) section 10.
