# Resolved blockers: effects port

This file is the post-write log of blockers and load-bearing
questions that surfaced during implementation of the effects
port and how they were resolved. Each entry is dated and stays
append-only; entries are added when blockers resolve, never
edited or removed.

The file complements [decisions.md](decisions.md) (frozen
design rationale) and [plan.md](plan.md) (active phasing and
status). Use this file when you want context on "why does X
work this way?" or "what alternatives were considered for Y?".

For per-step deviations from the original plan (smaller-grain
implementation differences that didn't require a paused
investigation), see [deviations.md](deviations.md).

## Resolved (2026-04-28): Phase 2 step 9 scope is under-specified

Phase 2 step 9's plan-text originally read in full:

> 9. Coyoneda-wrapping smart constructors (`lift_f` analogues for each effect type).

Two plausible interpretations of "smart constructors for each
effect type" existed, and they differed substantially in scope:
the **generic combinator** interpretation (one helper that
takes any effect value plus a `Member` witness, lifts it
through Coyoneda, injects into the row, wraps in `Node::First`,
and `send`s) and the **per-effect helpers** interpretation
(concrete `State<S>` / `Reader<E>` / `Except<E>` / `Writer<W>` /
`Choose` types plus `ask`, `get`, `put`, `modify`, `tell`,
`throw`).

Reading the rest of the plan, the generic-combinator interpretation
was the intended one: Phase 3 step 4 explicitly schedules
_"Standard first-order effect types and their smart
constructors: `State<S>`, `Reader<E>`, `Except<E>`, `Writer<W>`,
`Choose`"_ as a separate Phase 3 deliverable, so doing per-effect
work in Phase 2 step 9 would duplicate it.
[decisions.md](decisions.md) section 6 likewise describes
per-effect smart constructors as **thin wrappers over** the
`inj + liftF`/`send` infrastructure, implying the row-aware
lift combinator is prerequisite infrastructure that ships first
(which is what step 9 lands).

Three sub-questions remained open under the generic-combinator
interpretation:

- **Free function vs per-wrapper inherent method.** The
  established Phase 2 pattern (steps 5, 7a-c) puts user-facing
  Run-program operations on the wrappers as inherent methods
  (`Run::pure`, `RcRun::bind`, etc.), but the combinator's key
  argument is the effect value, not `self`, so a free function
  would also be natural.
- **Exact signature.** The `Member` bounds, the `Coyoneda`
  decode closure (does the user supply it?), the alignment across
  the six wrappers (whose bounds differ: Erased Rc family wants
  `A: 'static`, Explicit family wants `A: 'a`, ArcRunExplicit
  wants `A: 'a + Send + Sync`), and whether `Idx` is turbofished
  or inferred.
- **HRTB-poisoning under `ArcFree`.** Per the prior 2026-04-27
  resolution, constructing a `Node`-projection literal inside
  an HRTB-bearing scope (which `ArcFree`'s struct propagates
  into every `ArcRun`-method context) fails GAT normalization.
  `ArcRun`'s row-aware lift combinator would need to thread
  the same workaround.

- **Naming: `lift` vs `lift_f`.** The combinator does the full
  chain (`Coyoneda::lift` + Member inject + `Node::First` +
  `*Run::send`); functionally it is the direct analog of
  PureScript Run's
  [`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)
  (signature `Proxy sym -> f a -> Run r a`, body
  `Run <<< liftF <<< inj p`), not of
  [`Free.liftF`](https://github.com/purescript/purescript-free/blob/main/src/Control/Monad/Free.purs).
  PureScript explicitly distinguishes the two: `Free.liftF`
  is the Free-only operation; `Run.lift` is the row-aware
  Run-level operation that consumes a row label and runs the
  full inject + liftF chain. fp-library already mirrors the
  Free side as
  [`Free::lift_f`](../../../fp-library/src/types/free.rs)
  (snake_case translation of `liftF`); the Run-level operation
  takes the bare name `lift`. Phase 3's per-effect smart
  constructors (`ask = lift ReaderBrand Reader::Ask`, etc.)
  read consistently with PureScript's
  `liftEffect = lift (Proxy :: "effect")` pattern under this
  naming.

### Resolution

**Generic combinator interpretation, named `lift` (matching
PureScript Run's
[`lift`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs)).
Inherent associated function on each of the six Run wrappers,
mirroring `*Run::send`'s shape. Take the raw effect (an
`EBrand::Of<'a, A>` value) and do the full chain (`Coyoneda::lift`
-> row inject -> `Node::First` -> `*Run::send`) inside the body.
Type-infer `Idx` at call sites where the row is unambiguous;
turbofish only when duplicate effect types make `Idx` ambiguous.
Try the simple inline body first; fall back to a free
`lift_node<R, S, EBrand, Idx, A>(effect)` helper for `ArcRun::lift`
if HRTB-poisoning recurs.**

The signature for `Run` is:

```rust
impl<R: Kind, S: Kind, A: 'static> Run<R, S, A> {
    pub fn lift<EBrand, Idx>(
        effect: Apply!(<EBrand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>),
    ) -> Self
    where
        Apply!(<R as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>):
            Member<Coyoneda<'static, EBrand, A>, Idx>,
        EBrand: Kind_cdc7cd43dac7585f + 'static,
    {
        let coyo: Coyoneda<'static, EBrand, A> = Coyoneda::lift(effect);
        let layer = <Apply!(<R as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'static, A>)
            as Member<Coyoneda<'static, EBrand, A>, Idx>>::inject(coyo);
        Self::send(Node::First(layer))
    }
}
```

The `Kind!()` macro can't appear in trait-bound position (per
its doc-comment limitation: invalid in supertrait bounds, type
aliases, and trait aliases on stable Rust); the bound on
`EBrand` uses the generated hash name `Kind_cdc7cd43dac7585f`
directly. The hash is deterministic from the signature
`type Of<'a, T: 'a>: 'a;` and is in scope via fp-library's
existing `kinds::*` re-export.

Per-wrapper deltas (the body shape is identical; only the bounds
change):

| Wrapper          | `'a`         | Extra `A` bound            | Extra row/node bound                                          |
| :--------------- | :----------- | :------------------------- | :------------------------------------------------------------ |
| `Run`            | `'static`    | `A: 'static`               | (none)                                                        |
| `RcRun`          | `'static`    | `A: 'static`               | the `Apply<...>: Clone` bound `RcRun::send` carries           |
| `ArcRun`         | `'static`    | `A: Send + Sync + 'static` | `NodeBrand<R, S>: Functor` plus the `Apply<...>: Clone` bound |
| `RunExplicit`    | `'a` (param) | `A: 'a`                    | (none)                                                        |
| `RcRunExplicit`  | `'a` (param) | `A: 'a`                    | (none)                                                        |
| `ArcRunExplicit` | `'a` (param) | `A: 'a + Send + Sync`      | (none)                                                        |

The Coyoneda decode closure is implicit: `Coyoneda::lift` defaults
to the trivial decode, which is what every smart-constructor case
wants. Users who want a non-trivial decode construct `Coyoneda`
themselves and use `*Run::send` directly.

### Why not a single polymorphic free function

The six wrappers use six different inner constructors with six
different bound shapes (`'static` vs `'a`, `Clone` on the Apply
node-projection, `Send + Sync` for Arc, etc.) and there is no
common trait abstracting "construct from a node-projection".
Inventing one to make `lift` polymorphic would be more code
than just writing six near-identical inherent methods, which is
the same trade-off `*Run::send` already settled on its 2026-04-27
resolution.

### Why raw effect input (not pre-lifted Coyoneda)

Matches PureScript Run's
[`lift :: Row.Cons sym f r1 r2 => Proxy sym -> f a -> Run r2 a`](https://github.com/natefaubion/purescript-run/blob/main/src/Run.purs),
which takes the raw effect (`f a`) and does the inject + liftF
chain internally. Phase 3's smart constructors then become
one-liners
(`pub fn ask<R, S, Idx>() -> Run<R, S, Env> { Run::lift::<ReaderBrand, _>(Reader::Ask) }`),
mirroring PureScript's
`liftEffect = lift (Proxy :: "effect")` pattern. The Coyoneda
detail stays an implementation concern of the helper rather than a
user-visible step. Users who want to construct a non-trivial
Coyoneda decode bypass `lift` and call `Coyoneda::new` plus
`*Run::send` directly.

### Why "try inline; fall back if needed" for the HRTB workaround

The 2026-04-27 GAT-normalization issue specifically hit
`Apply!(<NodeBrand<R, S> as Kind>::Of<'static, A>)` _normalization_
inside `ArcFree`'s HRTB-bearing scope. The `lift` body builds the
Node-projection by _literal construction_ (`Node::First(Member::inject(coyo))`);
no `Apply!` normalization is required on the result type, only on
the `effect` parameter (which is fine, since it's already
pre-resolved at the function boundary). Plausibly clean for
`ArcRun::lift`. If it does fail, the workaround is mechanical:
factor `Node::First(<_ as Member<_, Idx>>::inject(Coyoneda::lift(effect)))`
into a free helper outside the HRTB scope and have
`ArcRun::lift` call `Self::send(lift_node::<R, S, EBrand, Idx, A>(effect))`.
Pre-baking the `lift_node` helper in all six wrappers
prophylactically would be wasted code if the simple form works
for everything but `ArcRun`.

## Resolved (2026-04-28 implementation expansion): step 9 SendFunctor cascade prerequisites for Arc family

While implementing the original 2026-04-28 resolution above,
[`Run::lift`](../../../fp-library/src/types/effects/run.rs)
landed cleanly at commit `34b6a97`. Extending the same body to
`RunExplicit`, `RcRun`, `RcRunExplicit` worked. But `ArcRun::lift`
and `ArcRunExplicit::lift` hit a structural conflict the original
resolution didn't anticipate.

### Problem

`ArcRun`'s struct-level HRTB
(`Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync`)
forces every variant of the row's projection to be `Send + Sync`.
The bare
[`Coyoneda`](../../../fp-library/src/types/coyoneda.rs) stores
its accumulated continuation in `Box<dyn FnOnce>` (no
`Send + Sync`), so `Coyoneda<'_, EBrand, A>` is not
`Send + Sync` and `ArcRun` rejects `CoyonedaBrand`-headed rows.

The Send-aware companion
[`ArcCoyoneda`](../../../fp-library/src/types/arc_coyoneda.rs)
exists and is `Send + Sync`. But
[`ArcCoyonedaBrand`](../../../fp-library/src/brands.rs)
deliberately doesn't implement
[`Functor`](../../../fp-library/src/classes/functor.rs) (it
only implements [`SendFunctor`](../../../fp-library/src/classes/send_functor.rs)
and [`Foldable`](../../../fp-library/src/classes/foldable.rs)),
because the
[`Functor::map`](../../../fp-library/src/classes/functor.rs)
trait method's signature lacks `Send + Sync` bounds on its
closure parameter; closures stored in Arc-wrapped layers must
be `Send + Sync`. This is a deliberate fp-library design
choice: the Send-aware parallel trait family
([`SendFunctor`](../../../fp-library/src/classes/send_functor.rs),
[`SendPointed`](../../../fp-library/src/classes/send_pointed.rs),
[`SendSemimonad`](../../../fp-library/src/classes/send_semimonad.rs),
[`SendApplicative`](../../../fp-library/src/classes/send_applicative.rs),
etc., plus the
[`SendRef`](../../../fp-library/src/classes/send_ref_functor.rs)
prefix tree) exists to handle Arc-substrate brands; plain
`Functor` deliberately does not impose Send bounds for the
common-case non-thread-crossing brands.

`ArcRun`'s existing `peel` / `send` / `bind` / `map`
implementations route through `<NodeBrand<R, S> as Functor>::map`
on the row brand. `NodeBrand: Functor` cascades to `R: Functor`,
which `ArcCoyonedaBrand` cannot satisfy. So the universal
`Run.lift` shape (Coyoneda lift -> row inject -> `Node::First` ->
`*Run::send`) cannot work for the Arc family without a Send-aware
substrate path.

The substrate
[`ArcFree`](../../../fp-library/src/types/arc_free.rs) compounds
the issue: its internal machinery (`lift_f`, `wrap`, `bind`,
`evaluate`, `fold_free`, `hoist_free`) all bound `F: Functor` and
call `F::map` directly. Switching the Run wrappers to the
Send-aware tree requires switching the substrate too.

### Resolution

**Expand step 9 with a `SendFunctor` cascade as prerequisite
sub-steps before the universal `lift` work. Replace
`F: Functor` bounds with `F: SendFunctor` on the Arc-substrate
machinery (`ArcFree`, `ArcFreeExplicit`); land the missing
`SendFunctor` impls on the row-cascade brands; expand the
brand-level type-class surface on `ArcFreeExplicitBrand` and
`ArcRunExplicitBrand` to absorb newly-reachable Send-aware
impls; then complete `*Run::lift` for all six wrappers under
the now-supported cascade.** Implement `SendRefFunctor` on
`ArcRunExplicitBrand` via inherent-method delegation (calling
the wrapper's `ref_map` / `ref_bind` / `ref_pure` directly,
bypassing the brand-level cascade via the clone-trick).

The expanded sub-step structure lives in plan.md step 9 (9a
through 9i); each lands as a separate commit. The `Run::lift`
implementation already shipped at commit `34b6a97` stays as the
reference design; sub-step 9h fills in the remaining five
wrappers.

### Why "replace Functor with SendFunctor" instead of adding sibling methods

Two paths considered:

- **Replace `F: Functor` with `F: SendFunctor`** on `ArcFree` /
  `ArcFreeExplicit`'s methods (signatures change, internal
  `F::map` calls become `F::send_map`). Breaking change for any
  pre-existing caller passing a non-Send `Functor`-only row
  brand. Cleaner long-term: one method per operation; semantic
  alignment between the substrate's thread-safety bounds and
  the trait surface.
- **Add Send-aware sibling methods** (`ArcFree::send_lift_f`
  alongside `ArcFree::lift_f`). Backwards-compatible but doubles
  the API surface; users have to pick the right method. The cost
  compounds across `ArcFreeExplicit`'s siblings.

Replacement chosen because `ArcFree`'s struct-level Send+Sync
HRTB already restricts concrete callers to row brands that
satisfy `Send + Sync`; adding `SendFunctor` impls to the row-
cascade brands (sub-step 9a) keeps existing concrete callers
working without method-surface duplication.

### Why `SendRefFunctor` via inherent-method delegation

The 2026-04-27
"[brand-level type-class coverage gap on the Explicit Run brands](#resolved-2026-04-27-brand-level-type-class-coverage-gap-on-the-explicit-run-brands)"
resolution documented `SendRef`-family hierarchy as unreachable
through brand-level delegation: `ArcFreeExplicitBrand` can't
implement `SendRefFunctor` because the auto-derive of
`Send + Sync` on the closure return type requires a per-`A`
HRTB on the `Kind` projection that stable Rust's trait method
signatures cannot carry.

The unreachability is at the substrate-brand level. The
`ArcRunExplicit` _wrapper_ has inherent
[`ref_map`](../../../fp-library/src/types/effects/arc_run_explicit.rs)
/ `ref_bind` / `ref_pure` methods that work via the clone-trick
(`self.clone().send_map(move |a| f(&a))`); the `O(1)`
`Arc::clone` makes this cheap, and the per-`A` HRTB doesn't
appear at the wrapper-method signature because the closure
constraints are checked against the inherent method's bound
list rather than against the brand-level trait method's. So
`ArcRunExplicitBrand: SendRefFunctor` is reachable if the impl
delegates to the wrapper's inherent `ref_map`, sidestepping
`ArcFreeExplicitBrand` entirely.

This is a different delegation strategy than what step 4b's
resolution considered (substrate-brand delegation). The
inherent-method delegation pattern produces a working
brand-level `SendRefFunctor` impl with the same observable
behavior at the cost of an `O(1)` clone per call. The clone is
acceptable: brand-level dispatch is the path the user opted into
when they wrote `<ArcRunExplicitBrand as SendRefFunctor>::send_ref_map`,
and the alternative is no brand-level coverage at all.

### Why not defer the SendFunctor cascade to a later phase

Three plausible structures considered:

- **Defer to Phase 1.5 follow-up.** The SendFunctor cascade on
  the row-brand types is genuinely substrate-level
  infrastructure, and Phase 1's WrapDrop migration set a
  precedent for landing prerequisite trait-cascade work as a
  follow-up between phases. But Phase 1 has long completed; a
  retroactive "Phase 1.5" is structurally awkward and signals
  bigger drift than the work warrants.
- **Defer to Phase 3.** Phase 3 step 4 lands per-effect smart
  constructors that build on `*Run::lift`. Deferring the
  cascade would push `ArcRun::ask` / `ArcRun::get` etc. behind
  a structural prerequisite, breaking Phase 3's promise of
  thin one-liners over `*Run::lift`.
- **Expand step 9's scope.** Most coherent: the cascade is
  required by step 9's universal-`lift` promise; landing it as
  step 9 sub-steps keeps the prerequisite-and-payoff together,
  visible in one place, and verifiable as a unit. The smaller
  sub-step granularity ensures each is independently
  reviewable.

The third option chosen.

### Reference: scope inventory at start of expansion

Confirmed by code inspection at the time the blocker surfaced:

- `ArcCoyonedaBrand`: has [`SendFunctor`](../../../fp-library/src/types/arc_coyoneda.rs);
  needs [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs).
- `IdentityBrand`: has `Functor` and `WrapDrop`; needs
  `SendFunctor` (mechanical; `Identity<A>` has no closures, so
  the closure `Send + Sync` requirement is vacuous).
- `NodeBrand`, `CoproductBrand<H, T>`, `CNilBrand`: have
  `Functor` and `WrapDrop`; need `SendFunctor` (recursive
  cascade for the inductive cases; uninhabited base case for
  `CNilBrand`).
- `ArcFree`: bounds `lift_f` / `wrap` / `bind` / `evaluate` /
  `fold_free` / `hoist_free` on `F: Functor`; calls `F::map` at
  three sites; switch to `F: SendFunctor` and `F::send_map`.
- `ArcFreeExplicit`: same shape as `ArcFree`; same migration.
- `ArcRun`: methods route through `<NodeBrand<R, S> as Functor>::map`;
  switch to `<NodeBrand<R, S> as SendFunctor>::send_map` after
  the cascade lands.
- `ArcRunExplicit`: same as `ArcRun`.
- `ArcFreeExplicitBrand`: brand-level coverage limited to
  `SendPointed` per step 4b; expand to `SendFunctor` and
  cascade dependents under the Send-aware machinery.
- `ArcRunExplicitBrand`: same expansion path; plus the
  `SendRefFunctor`-via-inherent-method-delegation impl.

## Resolved (2026-04-27): `*Run::send` takes a `Node`-projection value to sidestep GAT-normalization poisoning under `ArcFree`'s HRTB

Step 5's `send` method on each of the six Run wrappers takes the
[`NodeBrand<R, S>`](../../../fp-library/src/brands.rs)
`Of<'_, A>` projection (already-constructed) rather than the
first-order row variant (constructed internally via
`Node::First(layer)`). This deviates from the natural shape that
mirrors PureScript Run's `send`, but is required because of a
stable-Rust GAT-normalization limit that surfaces in `ArcRun`'s
impl-block context.

### Problem

While implementing `ArcRun::send` with the natural shape (take
the row variant `R::Of<'static, A>`, construct
`Node::First(layer)` internally, pass to `ArcFree::lift_f`),
the compiler refused to unify `Node<'static, R, S, A>` (the
literal value) with
`<NodeBrand<R, S> as Kind_cdc7cd43dac7585f>::Of<'static, A>`
(the projection that `ArcFree::lift_f` expects), even though
`impl_kind!` declares them equal:

```
expected associated type `<NodeBrand<R, S> as kinds::Kind_cdc7cd43dac7585f>::Of<'static, A>`
                  found enum `node::inner::Node<'static, R, S, A>`
```

The same construction succeeds for `Run::send` (over
[`Free`](../../../fp-library/src/types/free.rs)) and
`RcRun::send` (over
[`RcFree`](../../../fp-library/src/types/rc_free.rs)). The
difference is that
[`ArcFree`](../../../fp-library/src/types/arc_free.rs)'s struct
carries a per-`A`-instantiation HRTB
`F: Kind<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>`
(needed so the compiler can auto-derive `Send + Sync` on
`ArcFree<F, A>` when `F`'s `Of` projection is `Send + Sync`).
This HRTB propagates to `ArcRun`'s impl block, and inside that
block stable Rust's normalizer refuses to fire for any other
instantiation of the same `Of` projection.

### Investigation

Eleven experiments at
[`fp-library/tests/arc_run_normalization_probe.rs`](../../../fp-library/tests/arc_run_normalization_probe.rs)
(see history; trimmed in the final commit to the four passing
patterns) isolated the trigger:

- The HRTB itself, not the `ArcFree` field, is the trigger
  (PhantomData-only struct + HRTB still fails).
- The trigger is not impl-block-specific: a free function
  carrying the HRTB also fails.
- The trigger poisons cross-substrate calls: a `RcFree::lift_f`
  call from inside an `ArcFree`-HRTB-bearing impl also fails.
- Workarounds tried that all fail: explicit `Apply!()`-typed
  local; turbofish `Node::<'static, R, S, A>::First(layer)`;
  using `<...as Kind_cdc7cd43dac7585f>::Of` directly bypassing
  `Apply!`; routing through `Functor::map(identity, Node::First(layer))`
  (whose input is also at the projection); restructuring the
  impl block to use direct `R: ... + 'static, S: ... + 'static`
  bounds plus the HRTB.
- The workaround that succeeds: pass an already-projection-typed
  value into the HRTB-scope function, never construct a Node
  literal there. The caller (typically code without HRTB in
  scope, e.g., test code, smart-constructor macro output) builds
  `Node::First(layer)` and passes the result.

The probe file at
[`fp-library/tests/arc_run_normalization_probe.rs`](../../../fp-library/tests/arc_run_normalization_probe.rs)
is the trimmed regression-test version that documents the four
patterns confirmed to work despite the limit.

### Resolution

`*Run::send` on all six wrappers takes the
`Node`-projection value as a parameter, uniform signature:

```rust
pub fn send(
    node: Apply!(<NodeBrand<R, S> as Kind!(...)>::Of<'_, A>),
) -> Self;
```

Smart constructors (Phase 2 step 9) will emit
`Node::First(<R as Member<...>>::inject(coyo))` in their bodies
and pass the result to `send`. User test code does the same.

### Why not work around at a different layer

- **Re-architect `ArcFree` to remove the struct-level HRTB**:
  out of scope for step 5 (would require a Phase 1 follow-up
  commit). The HRTB is load-bearing for `Send + Sync` auto-derive
  on `ArcFree`, which dozens of other code paths depend on.
- **Provide `unsafe impl Send` / `unsafe impl Sync` for
  `ArcRun`** with bounds that don't include the HRTB: the unsafe
  impl's `where` clause would still need to express the
  Send/Sync condition somehow, and any expression of "the
  projection at this instantiation is Send + Sync" is itself an
  HRTB-shaped constraint that re-triggers the issue.
- **Accept the asymmetry between `Run`/`RcRun` (take row
  variant) and `ArcRun` (take Node projection)**: the symmetric
  approach was chosen for design consistency (the two patterns
  diverging across the six wrappers would surface as confusion
  in users of step 7's macros and step 9's smart constructors).

## Resolved (2026-04-27): brand-level type-class coverage gap on the Explicit Run brands

The plan's Phase 2 step 4 specification named a full
`Functor / Pointed / Semimonad / Monad` hierarchy plus a
`RefFunctor / RefPointed / RefSemimonad / RefMonad` hierarchy
for `RunExplicitBrand`, with analogous coverage for
`RcRunExplicitBrand` and `ArcRunExplicitBrand`. Step 4b
landed the achievable subset: `Functor / Pointed / Semimonad`
plus the by-reference equivalents for `RunExplicitBrand`,
`Pointed` plus by-reference equivalents for
`RcRunExplicitBrand`, and `SendPointed` only for
`ArcRunExplicitBrand`.
[`Monad`](../../../fp-library/src/classes/monad.rs) /
[`RefMonad`](../../../fp-library/src/classes/ref_monad.rs) /
[`SendMonad`](../../../fp-library/src/classes/send_monad.rs) and
the [`SendRef`](../../../fp-library/src/classes/send_ref_functor.rs)-family
hierarchy are not reachable through brand-level delegation;
inherent `bind` and `map` methods on `RcRunExplicit` and
`ArcRunExplicit` (mirroring
[`RcFreeExplicit`](../../../fp-library/src/types/rc_free_explicit.rs)'s
inherent surface) cover the by-value monadic surface for
concrete-type call sites.

### Problem

Three independent gaps share the same root cause: stable Rust's
trait method signatures cannot carry per-`A` bounds (no HRTB
over types), and the `*FreeExplicitBrand`s the Run-Explicit
brands delegate to deliberately do not implement the missing
classes for the same reason.

1. **`Monad` blanket impl requires `Applicative`.** The
   project's [`Monad`](../../../fp-library/src/classes/monad.rs)
   trait at line 214 is
   `pub trait Monad: Applicative + Semimonad {}` with a blanket
   `impl<Brand> Monad for Brand where Brand: Applicative + Semimonad {}`
   at line 218. Same shape for
   [`RefMonad`](../../../fp-library/src/classes/ref_monad.rs)
   over `RefApplicative + RefSemimonad`. So a brand cannot be
   `Monad` without first being `Applicative`.
   [`FreeExplicitBrand`](../../../fp-library/src/brands.rs)
   deliberately does not implement
   [`Applicative`](../../../fp-library/src/classes/applicative.rs)
   (its [`Lift`](../../../fp-library/src/classes/lift.rs)
   supertrait's natural definition pattern
   `lift2 = bind(fa, |a| map(fb, |b| f(a, b)))` requires `fb` to
   be reusable across closure invocations, and
   [`FreeExplicit`](../../../fp-library/src/types/free_explicit.rs)
   is not `Clone` per [`free_explicit.rs`](../../../fp-library/src/types/free_explicit.rs)
   lines 369-388). The Run wrapper brands inherit this gap
   through delegation.
2. **`SendRef` hierarchy unreachable on `ArcRunExplicitBrand`.**
   The [`ArcFreeExplicit`](../../../fp-library/src/types/arc_free_explicit.rs)
   substrate auto-derives `Send + Sync` only when its struct
   carries a per-`A` `Kind` HRTB
   (`Of<'a, ArcFreeExplicit<'a, F, A>>: Send + Sync`).
   That bound's `'a` and `A` are the trait method's per-method
   generics; stable Rust does not support `for<'a, T>` HRTB at
   the impl-block level. So
   [`ArcFreeExplicitBrand`](../../../fp-library/src/brands.rs)
   does not implement
   [`SendRefFunctor`](../../../fp-library/src/classes/send_ref_functor.rs)
   /
   [`SendRefPointed`](../../../fp-library/src/classes/send_ref_pointed.rs)
   /
   [`SendRefSemimonad`](../../../fp-library/src/classes/send_ref_semimonad.rs)
   (see [`arc_free_explicit.rs`](../../../fp-library/src/types/arc_free_explicit.rs)
   lines 730-745). `ArcRunExplicitBrand`'s would-be Send-Ref
   delegation has no target.
3. **Ref hierarchy is bounded by `R: RefFunctor`.** The Ref
   impls on `RunExplicitBrand` and `RcRunExplicitBrand` delegate
   to the corresponding `*FreeExplicitBrand`'s Ref impls, which
   carry `F: WrapDrop + Functor + RefFunctor + 'static`.
   For `Run`, `F = NodeBrand<R, S>`; the cascade requires
   `R: RefFunctor` and `S: RefFunctor`. Step 4b adds
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
   impls on `CNilBrand`, `CoproductBrand<H, T>`, and
   `NodeBrand<R, S>`, but
   [`CoyonedaBrand`](../../../fp-library/src/brands.rs) does not
   implement
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs).
   Canonical Run rows (`CoproductBrand<CoyonedaBrand<E_i>, ...>`)
   do not satisfy the cascade. The Ref impls are present at the
   brand level but reachable only for synthetic rows whose
   brands carry their own `RefFunctor` impls (e.g.,
   `CoproductBrand<IdentityBrand, CNilBrand>`).

### Resolution

Ship the achievable subset; document gaps as deviations. Future
work that needs the missing coverage either reaches for the
inherent methods on the concrete Run wrapper types or, for
`Coyoneda`-wrapped effect rows, adds `RefFunctor` to
[`CoyonedaBrand`](../../../fp-library/src/types/coyoneda.rs)
(scope-creep beyond step 4b; tracked separately).

### Why not work around

- **Restructuring `Monad`'s supertrait chain:** would require
  editing [`monad.rs`](../../../fp-library/src/classes/monad.rs)
  and similar; out of scope for the effects port and would break
  every existing brand impl.
- **Adding `Applicative` impls with `Clone` bounds at the trait
  signature level:** stable Rust's
  [`Applicative::lift2`](../../../fp-library/src/classes/lift.rs)
  signature can't be augmented; per-method `where` clauses on
  trait impls are restricted to what the trait allows.
- **Adding the SendRef hierarchy directly on
  `ArcRunExplicitBrand`** (bypassing
  `ArcFreeExplicitBrand`): would have the same per-`A` HRTB
  obstacle the underlying brand has.

## Resolved (2026-04-27): row-brand `RefFunctor` and `Extract` cascade impls land in step 4b

Phase 2 step 4a left
[`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs),
[`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs),
and
[`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs)
with [`Functor`](../../../fp-library/src/classes/functor.rs)
and [`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs)
impls only. Step 4b added
[`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
and [`Extract`](../../../fp-library/src/classes/extract.rs)
cascade impls on each of the three brands, plus a
[`Clone`] impl for the
[`Node`](../../../fp-library/src/types/effects/node.rs) enum.

### Problem

Three trait gaps surfaced as step 4b's Explicit family was
landed:

1. **`RefFunctor` needed for Ref-hierarchy delegation.**
   `RunExplicitBrand`'s
   [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
   impl delegates to
   [`FreeExplicitBrand`](../../../fp-library/src/brands.rs)'s,
   which carries `F: WrapDrop + Functor + RefFunctor + 'static`.
   For `Run`, `F = NodeBrand<R, S>`; the cascade requires
   `R: RefFunctor` and `S: RefFunctor`, so the row brand chain
   must support it.
2. **`Extract` needed for `evaluate()` on canonical Run
   programs.**
   [`FreeExplicit::evaluate`](../../../fp-library/src/types/free_explicit.rs)
   requires `F: Extract`. For `Run`, `F = NodeBrand<R, S>`; the
   cascade requires `R: Extract` and `S: Extract`.
   [`IdentityBrand`](../../../fp-library/src/types/identity.rs)
   has [`Extract`](../../../fp-library/src/classes/extract.rs);
   the row chain (Coproduct / CNil / Node) did not.
   Without it, brand-level test programs and doctests over
   synthetic rows could not assert evaluation results.
3. **`Clone` needed by Rc/Arc Free's evaluate fallback.**
   [`RcFreeExplicit::evaluate`](../../../fp-library/src/types/rc_free_explicit.rs)
   and
   [`ArcFreeExplicit::evaluate`](../../../fp-library/src/types/arc_free_explicit.rs)
   carry the per-`A` bound
   `Apply!(<F as Kind!(...)>::Of<'a, *FreeExplicit<'a, F, A>>): Clone`.
   For `F = NodeBrand<R, S>`, this expands to
   `Node<'a, R, S, *FreeExplicit<'a, NodeBrand<R, S>, A>>: Clone`.
   `Node` did not implement
   [`Clone`].

### Resolution

Land mechanical cascade impls on the row brands following the
same shape as the existing
[`Functor`](../../../fp-library/src/classes/functor.rs) /
[`WrapDrop`](../../../fp-library/src/classes/wrap_drop.rs)
impls:

- [`CNilBrand`](../../../fp-library/src/types/effects/variant_f.rs):
  uninhabited base case for both
  [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs)
  and
  [`Extract`](../../../fp-library/src/classes/extract.rs).
- [`CoproductBrand<H, T>`](../../../fp-library/src/types/effects/variant_f.rs):
  dispatches by `Inl` / `Inr` recursing into the active brand;
  bounded `H: RefFunctor + 'static, T: RefFunctor + 'static`
  for [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs);
  same shape with [`Extract`](../../../fp-library/src/classes/extract.rs)
  for the Extract impl.
- [`NodeBrand<R, S>`](../../../fp-library/src/types/effects/node.rs):
  dispatches by `First` / `Scoped`; bounded
  `R: RefFunctor + 'static, S: RefFunctor + 'static` for
  [`RefFunctor`](../../../fp-library/src/classes/ref_functor.rs);
  same shape for [`Extract`](../../../fp-library/src/classes/extract.rs).
- [`Node<'a, R, S, A>`](../../../fp-library/src/types/effects/node.rs):
  manual [`Clone`] impl bounded on `Apply!(<R as Kind!(...)>::Of<'a, A>): Clone`
  and the `S` projection; clones the active variant's payload.

`SendRefFunctor` cascade is _not_ added because
[`ArcRunExplicitBrand`](../../../fp-library/src/brands.rs)
cannot have a SendRef hierarchy in the first place (see the
adjacent resolution about brand-level coverage gaps).

## Resolved (2026-04-27): re-export pattern for the effects subsystem types follows the optics A+B hybrid

Step 4b adopts the
[`optics`](../../../fp-library/src/types/optics.rs) precedent:
selective top-level re-exports of headline types in
[`crate::types::*`](../../../fp-library/src/types.rs), plus
comprehensive subsystem-scoped re-exports at
[`crate::types::effects::*`](../../../fp-library/src/types/effects.rs).

### Problem

Phase 2 step 4 left re-exports undecided. Three options were
considered:

- **A. Top-level only** (`crate::types::*`): matches the rest
  of the [`types/`](../../../fp-library/src/types/) directory;
  ergonomic; but ~12 names land in the top-level block and the
  effects subsystem stops being visually distinguished.
- **B. Subsystem-scoped only** (`crate::types::effects::*`):
  preserves the top-level namespace shape; matches what
  [`optics`](../../../fp-library/src/types/optics.rs) does for
  non-headline types; but deviates from the Free family's
  surface.
- **C. No re-exports**: zero maintenance, but friction at
  every import site and matches no existing pattern.

The existing
[`optics`](../../../fp-library/src/types/optics.rs) precedent
is neither pure A nor pure B: it re-exports every submodule
symbol via
`pub use submodule::*` at
[`crate::types::optics::*`](../../../fp-library/src/types/optics.rs)
(comprehensive, B), AND surfaces only the three headline types
[`Composed`](../../../fp-library/src/types/optics.rs),
[`Lens`](../../../fp-library/src/types/optics.rs),
[`LensPrime`](../../../fp-library/src/types/optics.rs) at the
top-level (selective, A).

### Resolution

Adopt the optics precedent literally: the six Run wrapper
headline types
(`Run`, `RcRun`, `ArcRun`, `RunExplicit`, `RcRunExplicit`,
`ArcRunExplicit`) are headline-class and ship at the top level
([`crate::types::*`](../../../fp-library/src/types.rs)) because
they are the user-facing types most callers will import; the
brands and row machinery (`Node`, `VariantF`,
`*RunExplicitBrand`) are subsystem-scoped and ship at
[`crate::types::effects::*`](../../../fp-library/src/types/effects.rs)
only. Brand types stay in
[`crate::brands::*`](../../../fp-library/src/brands.rs) per the
existing precedent for all brand types in the library.

## Resolved (2026-04-27): introduce `WrapDrop` trait for Free's struct-level Drop concern

A new trait `WrapDrop` lands at the struct level of the Free
family, replacing `Extract` for `Drop`'s iterative-dismantling
purposes while preserving `Extract` as a separate trait for
`evaluate` / `fold_free` / etc. Migration ships as two Phase 1
follow-up commits before Phase 2 step 4 resumes; the actual
step-by-step migration spec lives in
[plan.md](plan.md)'s "Phase 1 follow-up: WrapDrop migration"
section.

### Problem

Phase 2 step 4 (the six concrete `Run` types) commits to
`Run<R, S, A> = Free<NodeBrand<R, S>, A>` per
[decisions.md](decisions.md) section 5.2 and [plan.md](plan.md)'s
"Will change" table entry for
[`fp-library/src/types/effects.rs`](../../../fp-library/src/types/effects.rs).
This requires `Free<NodeBrand<R, S>, A>` to compile for typical
effect rows. It does not, because of a transitively-poisoning
trait bound:

1. [`Free<F, A>`](../../../fp-library/src/types/free.rs) (and
   the other five Free variants) declares its struct with
   `where F: Extract + Functor + 'static`. The `Extract` bound
   is enforced at the type-declaration site, not just on
   inherent methods, so a `Free<NodeBrand<R, S>, A>` instance
   fails to compile when `NodeBrand<R, S>` does not implement
   `Extract`.
2. [`Free::drop`](../../../fp-library/src/types/free.rs) calls
   `<F as Extract>::extract(fa)` to walk deep `Wrap` chains
   iteratively. This is what keeps a 100 000-deep `Wrap` chain
   from stack-overflowing during cleanup; the `Extract` bound
   is load-bearing for the existing `Drop` strategy, which is
   why the bound is on the struct rather than on individual
   methods (Rust requires `Drop` impl bounds to match struct
   bounds exactly).
3. To satisfy `NodeBrand<R, S>: Extract` for typical Run usage,
   the bound recurses into the row brands. For the first-order
   row, `R = CoproductBrand<CoyonedaBrand<E1>, CoproductBrand<...>>`,
   and the recursive bound bottoms out at
   `CoyonedaBrand<E>: Extract`.
4. `CoyonedaBrand<E>::extract` would need to recover an `A` from
   `Coyoneda<E, A>`. The natural implementation lowers the
   Coyoneda (`coyo.lower()` returns `E::Of<A>`, requires
   `E: Functor`) and then calls `<E as Extract>::extract(...)`.
   So the bound transitively requires `E: Extract` for every
   effect type in the row.
5. Effect types (`Reader<E>`, `State<S>`, `Choose`, `Except<E>`,
   `Writer<W>`, etc.) are pure data with no canonical
   "evaluate" semantics: they need a handler to interpret. So
   `Reader<E>: Extract` (and the same for every other effect)
   cannot hold without baking arbitrary semantics into each
   effect type.

The bound is correct for the Free family's general use cases
(`Free<IdentityBrand>` evaluates by unwrapping; `Free<ThunkBrand>`
evaluates by running the thunk). It is over-conservative for the
effects-as-data use case Run needs.

### Investigation: Wrap-depth probe

A probe at
[`fp-library/tests/run_wrap_depth_probe.rs`](../../../fp-library/tests/run_wrap_depth_probe.rs)
(commit `09d676b`) measures `Wrap`-arm depth in Run-shaped
programs over `Free<ThunkBrand, _>` (using `ThunkBrand` because
`Free<IdentityBrand, _>` is layout-cyclic per the Phase 1 step 8
deviation, but the structural behaviour the probe measures is
brand-independent). The probe distinguishes two metrics:

- **Evaluation depth:** how many `Wrap` layers materialise when
  `to_view` applies pending continuations and follows the
  resulting `Wrap` chain via `Extract`. This is what an
  interpreter sees when walking the program.
- **Structural depth:** how many `Wrap` layers exist in the
  original view BEFORE `to_view` applies any continuation.
  This is what `Drop` traverses, because `Drop` dismantles
  the view and continuations in place without applying the
  closures.

Seven tests and their findings:

| Pattern                                                                | Evaluation depth | Structural depth                |
| ---------------------------------------------------------------------- | ---------------- | ------------------------------- |
| `Free::pure(0)`                                                        | 0                | 0                               |
| `pure(0).bind(\|x\| pure(x+1))` chained 1000 times                     | 0                | 0                               |
| `lift_f(eff)` alone                                                    | 1                | 1                               |
| `lift_f(eff).bind(\|x\| pure(x+1))` chained 1000 times                 | 1                | 1                               |
| `pure(0).bind(\|x\| lift_f(eff))` chained 100 times                    | 100              | 0                               |
| `lift_f(eff).bind(\|x\| pure(x+1))` chained 100 000 times, then `drop` | n/a              | succeeds without stack overflow |
| Explicit `Free::wrap(...)` chained 100 times                           | 100              | 100                             |

Bottom-line finding: Run-typical programs (built via `lift_f`
plus a flat `bind` chain) have structural `Wrap` depth at most
1, regardless of bind-chain length. The depth that grows with
sequencing lives in the `CatList` of continuations, which the
existing iterative `Drop` already dismantles without calling
`Extract`. The 100 000-bind drop test passes without stack
overflow even though `Drop` only walks one `Wrap` layer (the
original `lift_f`'s `Wrap`) recursively.

The artificial 100-deep `Free::wrap` chain pattern (last row) is
the case that motivated the existing `Extract`-based iterative
`Drop`. Run-typical usage does not produce this pattern; users
inject effects via `lift_f` (one `Wrap` per call) and chain via
`bind` (no new `Wrap`s). The probe also covers
`nested_lift_f_via_bind_materializes_wraps_at_evaluation_time`,
showing that `bind` closures returning `lift_f` build their
`Wrap`s at _evaluation_ time, not construction time, so they
live in the `CatList` rather than the structural `Wrap` chain.

### Resolution: introduce the `WrapDrop` trait

A new trait `WrapDrop` separates the structural-cleanup question
(what `Drop` needs) from the semantic-interpretation question
(what `Extract` answers). `Extract` continues to mean "given
`F::Of<X>`, give me the `X`" and is used by `evaluate`,
`fold_free`, `resume`, etc. `WrapDrop` instead asks "given
`F::Of<X>`, can you yield the inner `X` without running user
code?", returning `Option<X>`.

#### Trait definition

```rust
pub trait WrapDrop: Kind {
    /// Drop-time decomposition. `Some(x)` means F materially
    /// stores X and the caller can iterate on it. `None` means
    /// F doesn't store X (or storing is closure-captured), so
    /// the caller should let `fa` drop normally.
    fn drop<'a, X: 'a>(fa: Self::Of<'a, X>) -> Option<X>;
}
```

#### Naming rationale

The trait's name reflects that it is the operation `Free`'s
`Wrap` variant performs at drop time. The method name `drop`
does not clash with `std::ops::Drop::drop` because they are
different traits with different receiver shapes
(`std::ops::Drop::drop(&mut self)` is a method;
`WrapDrop::drop(fa: F::Of<'_, X>)` is an associated function).
Call sites use fully-qualified syntax:
`<F as WrapDrop>::drop(fa)`.

#### Free's Drop dispatch

Free's `Drop` impl is rewritten to dispatch on the `Option`:

```rust
match F::drop(layer) {
    Some(inner) => worklist.push(inner.view); // existing iterative path
    None => { /* layer already dropped recursively by the match arm */ }
}
```

#### Per-F policy choices

- **F materially stores the inner X** (e.g., `IdentityBrand`):
  `WrapDrop::drop` returns `Some(<F as Extract>::extract(fa))`,
  preserving the existing iterative path.
- **F's storage runs user code to materialise X but the
  existing test suite relies on iterative dismantling** (e.g.,
  `ThunkBrand`): `WrapDrop::drop` returns
  `Some(<F as Extract>::extract(fa))`. This preserves
  side-effect-on-Drop semantics and the Phase 1
  `deep_drop_does_not_overflow` test. The alternative (return
  `None` to skip closures) was rejected because the closure's
  captures hold inner Frees that would drop recursively for
  100k-deep chains.
- **F does not materially store X at all** (e.g.,
  `CoyonedaBrand<E>`, `CoproductBrand<H, T>`, `CNilBrand`,
  `NodeBrand<R, S>`): `WrapDrop::drop` returns `None`. Drop
  falls through to recursive drop on `fa`; the probe validates
  this is sound for Run-typical patterns because the `F::Of<X>`
  storage doesn't materially recurse on inner Frees (Coyoneda's
  closure would construct a Free if called, but doesn't store
  one; the Coproduct's variants hold Coyonedas which have the
  same property).

#### Documented limitation

Artificial deep `wrap(...)` chains over F's whose
`WrapDrop::drop` returns `None` (e.g., a hand-built 100k-deep
`wrap(Coyoneda(...))` chain) overflow the stack on `Drop`.
Run-typical usage does not generate this pattern, and no
existing test exercises it. The trait's docs warn future
F-authors of the constraint.

### Alternatives considered and rejected

Four resolution paths were evaluated; the chosen path is the
`WrapDrop` introduction described above. The other three are
recorded for design-history transparency:

- **Build a parallel `RunFree`-like substrate without the
  `Extract` bound.** Define six new types in `types/effects/`
  paralleling the six existing Free variants, with relaxed
  bounds and recursive `Wrap` drop. Same insight as the chosen
  path but isolated to Run; Phase 1's Free family would stay
  untouched. Probe-validated as sound for Run usage. Rejected
  because it duplicates the entire substrate (CatList for
  Erased, naive recursive enum for Explicit, custom `Drop`)
  for one architectural concern. `WrapDrop` achieves the same
  expressivity with a single new trait and mechanical-but-
  unified migration.
- **Make Run a newtype struct that internally holds something
  other than a raw `Free<NodeBrand, A>`** (e.g., a
  `Box<dyn ...>` trait object, a custom enum, or a Free over a
  placeholder brand that does implement Extract trivially while
  effect data lives elsewhere). Rejected because it diverges
  from the plan's literal "Run is a Free" model
  ([decisions.md](decisions.md) section 5.2,
  [README of `purescript-run`](https://github.com/natefaubion/purescript-run))
  and the other paths achieve the goal without redesigning
  the relationship.
- **Implement `Extract` for `CoyonedaBrand<E>` /
  `CoproductBrand<H, T>` / `NodeBrand<R, S>` with panic
  semantics** (extract panics with a clear "handler required"
  message; Drop falls back to recursive drop when extract
  panics). Rejected as a footgun: programs that drop unhandled
  Run values panic in legitimate scenarios (program panics in
  user code mid-evaluation, deliberate program discarding,
  test fixtures asserting on Run structure without running it).

## Resolved (2026-04-26): brand-level dispatch for the multi-shot Explicit Free family lands on the by-reference hierarchy

`RcFreeExplicit::bind` requires `A: Clone` (because shared inner
state must clone to recover an owned `A`), and stable Rust does
not admit per-method `where A: Clone` on a `Functor::map` impl.
This is the same constraint that
[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)
documents under "Unexpressible Bounds in Trait Method Signatures"
for `RcCoyoneda`/`ArcCoyoneda` and addresses under "Memoized Types
Cannot Implement `Functor`" via the by-reference hierarchy
(`RefFunctor`, `RefSemimonad`, `RefMonad` and `SendRef*`
parallels) that `Lazy` already uses. The decision is to follow
`Lazy`'s precedent.

### Brand-level coverage

- `FreeExplicitBrand`: full by-value (`Functor` / `Pointed` /
  `Semimonad` / `Monad`) + full Ref hierarchy.
- `RcFreeExplicitBrand`: `Pointed` on the by-value side; full
  Ref hierarchy (`RefFunctor` / `RefSemimonad` / `RefMonad`,
  plus `RefPointed` and the supporting Ref traits per
  [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md)).
- `ArcFreeExplicitBrand`: `SendPointed` on the by-value side
  (added by step 6 alongside `SendFunctor` etc.); full SendRef
  hierarchy (`SendRefFunctor` / `SendRefSemimonad` /
  `SendRefMonad`, plus the supporting `SendRef*` traits).

### Inherent-method fallback

The remaining by-value operations (`bind`, `map`, etc.) on
`RcFreeExplicit` / `ArcFreeExplicit` ship as inherent methods
with their natural `Clone` bounds, mirroring the
`RcCoyoneda`/`ArcCoyoneda` precedent.

### Alternatives considered and rejected

- Modifying the existing by-value hierarchy to add `Clone`
  bounds taxes the entire ecosystem (`Option`, `Vec`,
  `Identity`, etc.) for one wrapper's storage strategy.
- Adding a parallel `CloneFunctor` / `CloneSemimonad` /
  `CloneMonad` family duplicates the Ref hierarchy's dispatch
  story and adds a third orthogonal trait-and-dispatch axis
  (closure shape, send-ness, Clone-ness). The Ref path is the
  documented library convention and exists today; revisit
  `CloneFunctor` only if Phase 5+ user feedback indicates
  Ref-only brand UX is insufficient for the multi-shot Explicit
  family.

### Plan-level consequences

The decision is reflected in Phase 1 step 7, Phase 2 step 4, the
Motivation section's multi-shot example, and the "Will change"
table's `*RunExplicitBrand` row. Step 7 also schedules an update
to
[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)'s
"Unexpressible Bounds" classification table to add rows for the
three Explicit Free variants once their impls land.

## Resolved earlier: Erased / Explicit dispatch split for the Free family

The earlier `RcFreeBrand` / `ArcFreeBrand` blocker is resolved by
adopting the Erased/Explicit dispatch split documented in
[decisions.md](decisions.md) section 4.4: the Erased family
(`Free`, `RcFree`, `ArcFree`) is inherent-method only and is not
Brand-dispatched, while the Explicit family (`FreeExplicit`,
`RcFreeExplicit`, `ArcFreeExplicit`) carries the full Brand
hierarchy. Phase 1 grows by three steps to add the two new
Explicit Rc/Arc siblings and the `SendFunctor` trait family;
Phase 2 grows the Run surface to six concrete types (one per Free
variant) plus an `into_explicit` / `from_explicit` conversion
API. See plan.md's resequenced phasing.

## Design-phase blockers (resolved in decisions.md)

All blockers from the design phase are resolved in
[decisions.md](decisions.md):

- Section 4 (six DECISIONs): row encoding, Functor dictionary,
  stack-safety, six-variant Free family with Erased/Explicit
  dispatch split, scoped-effect representation (heftia dual row),
  natural transformations as values.
- Section 9 (nine pre-implementation decisions): target audience,
  partial interpretation, async, IO/Effect story, higher-order
  effects, performance, lifetime constraints, macro
  infrastructure, testing strategy.
