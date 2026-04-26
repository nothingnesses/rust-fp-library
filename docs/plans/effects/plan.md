# Plan: Port purescript-run to fp-library

**Status:** Phase 1 in progress (steps 1, 2, 3, and 4 of 9 complete).

## Current progress

Phase 1 steps 1, 2, 3, and 4 complete.

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

Remaining Phase 1 work: step 5 (`ArcFreeExplicit`), step 6
(`SendFunctor` trait family), step 7 (brand registrations +
by-value and by-reference trait hierarchies for all three Explicit
brands), step 8 (per-variant Criterion benches), step 9
(per-variant unit and `compile_fail` tests).

Other artefacts unchanged from pre-implementation:

- [poc-effect-row/](../../../poc-effect-row/) — 25 tests across two
  suites validating the row-encoding hybrid (workaround 1 macro
  plus workaround 3 `CoproductSubsetter` fallback), the
  `tstr_crates` Phase 2 refinement, and static-via-Coyoneda
  Functor dispatch end-to-end. See
  [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
  for findings. Migrates into production during Phase 2.

## Open questions, issues and blockers

**Open question for Phase 1 step 7 and Phase 2 step 4:
brand-level dispatch for the multi-shot Explicit Free family
lands on the by-reference hierarchy, not the by-value
hierarchy.** `RcFreeExplicit::bind` (and any derived `map`)
consumes `self` and routes through `into_inner_owned`, which
falls back to cloning the inner state when the outer `Rc` is
shared. The inner state's `Pure(A)` arm holds `A` directly, so
that fallback path requires `A: Clone`. The `Functor` trait's
`map<'a, A: 'a, B: 'a>` signature is fixed and stable Rust does
not admit per-method `where A: Clone` on the impl. The same logic
applies to `Semimonad::bind` and downstream. `Pointed::pure` is
unaffected because `pure(a)` does not need `A: Clone`.

The library already has the answer to this kind of constraint:
the **by-reference trait hierarchy** documented in
[fp-library/docs/limitations-and-workarounds.md](../../../fp-library/docs/limitations-and-workarounds.md)
under "Memoized Types Cannot Implement `Functor`". `RefFunctor`,
`RefPointed`, `RefSemimonad`, `RefMonad` (and the `SendRef*`
variants) take their closure as `Fn(&A) -> B` and the container
by reference, so the implementor never needs to own `A`. `Lazy`
is brand-dispatched via the Ref hierarchy alone for exactly this
reason. The unified `map` free function dispatches on closure
shape: `Fn(A) -> B` routes to `Functor`, `Fn(&A) -> B` routes to
`RefFunctor`. Step 7 already scopes both hierarchies for
`RcFreeExplicitBrand` and `FreeExplicitBrand` (and step 6 already
scopes `SendFunctor` plus by extension `SendRefFunctor` for
`ArcFreeExplicitBrand`).

For `RcFreeExplicit`, `RefFunctor::ref_map` is implementable: in
the `Pure(a)` arm, apply `f(&a)` directly with no Clone bound; in
the `Wrap(fa)` arm, recurse via `F::ref_map` (requires
`F: RefFunctor`). Same shape extends to `RefSemimonad::ref_bind`,
`RefMonad`, etc. This makes the brand-level coverage:

| Type              | by-value Functor | by-value Pointed  | by-value Semimonad | Ref Functor | Ref Pointed | Ref Semimonad                |
| :---------------- | :--------------- | :---------------- | :----------------- | :---------- | :---------- | :--------------------------- |
| `FreeExplicit`    | Yes              | Yes               | Yes                | Yes         | Yes         | Yes                          |
| `RcFreeExplicit`  | No               | Yes               | No                 | Yes         | Yes         | Yes                          |
| `ArcFreeExplicit` | No               | via `SendPointed` | No                 | Yes         | Yes         | Yes (via `SendRefSemimonad`) |

The user-facing implication: `m_do!` and typeclass-generic
dispatch over `RcRunExplicit` / `ArcRunExplicit` work via the Ref
hierarchy. Users writing `m_do!` over those Run variants pass
`Fn(&A) -> ...` continuations rather than `Fn(A) -> ...`. The
single-threaded single-shot variant `RunExplicit` (built on
`FreeExplicit`) keeps full by-value brand coverage and is the
ergonomic "default" for typeclass-generic code; the multi-shot
variants get brand dispatch via Ref. This matches `Lazy`'s
precedent exactly. The plan's Motivation section needs a small
clarification on this, and Phase 2 step 4 should explicitly state
that `m_do!` continuations on `RcRunExplicit` / `ArcRunExplicit`
take `&A`.

Resolutions in priority order:

1. **Stick with Ref hierarchy + inherent by-value methods**
   (recommended). Step 7 implements `RefFunctor` /
   `RefSemimonad` / `RefMonad` for `RcFreeExplicitBrand`, plus
   `SendRefFunctor` / `SendRefSemimonad` / `SendRefMonad` for
   `ArcFreeExplicitBrand`, plus `Pointed` (and `SendPointed`)
   on the by-value side because `pure` needs no Clone bound.
   `RcFreeExplicit::bind` / `ArcFreeExplicit::bind` ship as
   inherent methods (with their natural Clone bounds) for direct
   non-generic use, mirroring the
   [`RcCoyoneda`/`ArcCoyoneda` precedent](../../../fp-library/docs/limitations-and-workarounds.md).
   Update plan Motivation to show `m_do!` over multi-shot
   Explicit Run variants taking `Fn(&A) -> ...` continuations.
2. Restructure `RcFreeExplicit`/`ArcFreeExplicit` to a
   Coyoneda-hybrid `Pure` cell (e.g.,
   `Pure<X>(Rc<X>, Rc<dyn Fn(X) -> A>)`) so by-value `map` adds
   a function layer without consuming `A`. Adds one `Rc` per
   `pure` and an existential type parameter; bind still has the
   spine-recursion cost. Restores by-value `Functor` brand
   coverage (matching `RcCoyoneda` precedent) but does not solve
   by-value `Semimonad` (bind still needs ownership of the
   intermediate value). Higher-effort and only partially helps.
3. Add a `CloneFunctor` / `CloneSemimonad` / `CloneMonad`
   parallel trait family with `A: Clone` on the closure
   parameter, paralleling `SendFunctor`. Discussed below in
   "Hierarchy modification options"; not recommended without
   user feedback indicating Ref-hierarchy UX is insufficient.

### Hierarchy modification options

The user asked whether to consider (a) modifying the existing
by-value hierarchy to add Clone bounds or (b) adding a parallel
by-value hierarchy with Clone bounds.

**(a) Modify the existing by-value hierarchy.** Don't do this.
`Functor` / `Pointed` / `Semimonad` / `Monad` are foundational
abstractions implemented by every container type in the library
(`Option`, `Result`, `Vec`, `Identity`, `Free`, `Coyoneda`, etc.).
Adding `A: Clone` (or any extra trait bound on the element) to
the trait method signatures would force every existing
implementor to carry the bound, restricting them to `Clone`
elements without benefit. It would also break the abstraction:
the categorical Functor laws don't require Clone, so encoding
Clone in the trait signature mis-models what a Functor is. This
is the same argument that keeps `Send + Sync` out of `Functor`
(handled via parallel `SendFunctor` instead).

**(b) Add a parallel `CloneFunctor` / `CloneSemimonad` /
`CloneMonad` family.** Plausible but speculative. Mirrors the
existing `SendFunctor` pattern (planned for `ArcFreeExplicit` in
step 6) and the `RefFunctor` family. Each trait would carry
`A: Clone` (and possibly `B: Clone`) on the closure parameter,
admitting impls for types whose internal storage forces clone-on-
shared-access. `RcFreeExplicit` would implement `CloneFunctor`
/ `CloneSemimonad` / `CloneMonad`; `ArcFreeExplicit` would
implement `SendCloneFunctor` / etc. Generic code that wants the
multi-shot Explicit family would write `F: CloneFunctor` instead
of `F: Functor`.

The arguments against doing this now:

- The Ref hierarchy already exists and already covers the same
  use cases that motivated this question (brand dispatch on a
  type that can't consume `A` cheaply).
- `Send + Sync` is fundamentally a different kind of constraint
  from `Clone`. `Send + Sync` weakens what the implementor can
  store internally (no `Rc`, no `Cell`, etc.); `Clone` is a
  demand on the closure's input. From the user's perspective,
  calling a `CloneFunctor::map` looks identical to calling a
  regular `Functor::map`; the only difference is which trait
  bound the user wrote in their generic code. That's a less
  natural axis to split on than Send-vs-not-Send.
- The dispatch surface (`m_do!`, the unified `map` free
  function) currently splits on closure shape (`Fn(A)` vs
  `Fn(&A)`) and on send-ness (regular vs `Send` variants).
  Adding a Clone axis means three orthogonal splits, which the
  macro/dispatch wiring would have to learn. Real complexity
  cost.
- "Don't add features beyond what the task requires." The Ref
  hierarchy resolves the immediate problem; `CloneFunctor`
  should be added only if real-world usage shows the Ref-only
  brand path is too cumbersome for users of the multi-shot
  Explicit family. That's a Phase 5+ refinement question.

If we **were** to add `CloneFunctor` later, the smallest version
would parallel `SendFunctor` exactly: an independent trait,
implemented by exactly the brands that need it, and a unified
dispatch arm in the `map` / `bind` free functions that prefers
the Clone-bounded impl when both are available. That's a
mechanical addition that doesn't require re-design.

Step 4's commit lands the `RcFreeExplicit` type and `Kind`
registration as planned. The question affects step 7's trait
hierarchy scope (which Ref impls land), step 6's design
(whether `SendFunctor` for `ArcFreeExplicitBrand` is paired with
`SendRefFunctor` from the start), and the plan's Motivation /
Phase 2 wording (Ref-shaped continuations on `m_do!` over
multi-shot Run variants). It does not block step 5
(`ArcFreeExplicit`, which is structural).

The earlier `RcFreeBrand`/`ArcFreeBrand` blocker
is resolved by adopting the **Erased/Explicit dispatch split**
documented in [decisions.md](decisions.md) section 4.4: the
Erased family (`Free`, `RcFree`, `ArcFree`) is inherent-method
only and is not Brand-dispatched, while the Explicit family
(`FreeExplicit`, `RcFreeExplicit`, `ArcFreeExplicit`) carries
the full Brand hierarchy. Phase 1 grows by three steps to add
the two new Explicit Rc/Arc siblings and the `SendFunctor`
trait family; Phase 2 grows the Run surface to six concrete
types (one per Free variant) plus an `into_explicit` /
`from_explicit` conversion API. See the resequenced phasing
below.

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

If a load-bearing question surfaces during implementation, record
it here and pause until it's resolved.

## Deviations

- **Phase 1 step 1: removed `OptionBrand`-using POC tests.** Adding
  the `F: Extract + Functor + 'a` bound to `FreeExplicit` (required
  by the iterative `Drop` impl per
  [decisions.md](decisions.md) section 4.4) means
  `OptionBrand` can no longer back a `FreeExplicit`, since `None`
  has no value to surrender and `OptionBrand` therefore cannot
  lawfully implement `Extract`. The POC's `q5_two_effect_run`
  short-circuit test and the `evaluate_option` helper were dropped;
  the same Run-shaped semantics are reachable via handler
  interpretation in Phase 3+. This is exactly the caveat the
  decision predicts ("this forces every effect functor used with
  `FreeExplicit` to implement `Extract`"), but the plan step text
  said to "replace the local definition with an import" without
  explicitly listing test removals, so it is recorded here.
- **Phase 1 step 1: introduced `FreeExplicitView` enum.** The POC's
  `FreeExplicit` was a two-variant enum directly. The production
  type wraps the variants in `view: Option<FreeExplicitView>` so
  the custom `Drop` impl can move the view out via `Option::take`
  without producing a sentinel `A` value. `FreeExplicitView` is
  `pub` and re-exported alongside `FreeExplicit` to keep the
  variants visible for users who want to pattern-match. No external
  test or bench needed to change shape; the POC tests only used
  `pure`, `wrap`, `bind`, and `evaluate` (no direct match on the
  variants).
- **Phase 1 step 2: `RcFree` uses `Rc<dyn Any>` (not `Box<dyn Any>`)
  for the type-erased value cell.** Decision 4.4's table summarises
  `RcFree`'s erasure as "`Box<dyn Any>` + CatList" while also
  committing to "Cloneable: Yes, O(1)". `Box<dyn Any>` is not
  `Clone`, so the literal table reading conflicts with the Clone
  commitment. The minimal resolution is to swap the Box-erased
  cell for an `Rc<dyn Any>`, which keeps the `dyn Any` erasure
  shape but lets the inner state participate in Clone. Recovering
  an owned `A` from the cell uses `Rc::try_unwrap` and falls back
  to `(*shared).clone()` when the cell is shared, which constrains
  the public methods that perform the final downcast (`to_view`,
  `resume`, `evaluate`, `lower_ref`, `peel_ref`, `hoist_free`) to
  require `A: Clone`. This matches the multi-shot semantics: a
  handler that wants to evaluate the same program more than once
  needs the result type to be reproducible.
- **Phase 1 step 2: `RcFree<F, A>` is `Rc<RcFreeInner<F, A>>`
  (outer `Rc` wrapping).** Step 2's text says "follow the `Free`
  template" without specifying outer-Rc-wrapping, but the unconditional
  O(1) Clone commitment plus the `Suspend` arm holding
  `F::Of<RcFree<F, RcTypeErasedValue>>` produce a recursive Clone
  bound that only resolves cleanly when `RcFree: Clone` is
  unconditional. Outer-Rc-wrapping (the
  [`RcCoyoneda`](../../../fp-library/src/types/rc_coyoneda.rs)
  pattern) makes Clone trivially `Rc::clone(&self.inner)`. State-
  extending operations (`bind`, `map`, `wrap`, `lift_f`, `cast_phantom`)
  use `Rc::try_unwrap` to move out when uniquely owned and clone
  the inner state otherwise.
- **Phase 1 step 2: `RcContinuation` is a newtype, not the bare
  `<RcFnBrand as CloneFn>::Of` projection.** Step 2's text says
  "expressed via `FnBrand<RcBrand>`". Using the macro-mediated GAT
  projection directly as a type alias does not parse (the type
  parameter `F` does not surface through the `Apply!` expansion).
  The production type uses a thin newtype `RcContinuation<F>(Rc<dyn Fn(...)>)`
  with the same in-memory shape as `<RcFnBrand as CloneFn>::Of`,
  and constructs values via `<RcFnBrand as LiftFn>::new(...)` so
  the library's unified function-pointer abstraction is still on
  the construction path. The newtype's `Clone` impl bumps the
  underlying `Rc`'s refcount.
- **Phase 1 step 3: `ArcFree` carries the same trio of
  Deviations as `RcFree`** (the type-erased value uses
  `Arc<dyn Any + Send + Sync>` for `Clone`/`Send`/`Sync`
  participation, the substrate is wrapped in outer `Arc<Inner>`,
  and `ArcContinuation<F>` is a newtype wrapping
  `Arc<dyn Fn(...) + Send + Sync>` constructed via
  `<ArcFnBrand as SendLiftFn>::new`). All three deviations carry
  forward unchanged from step 2's analysis with `Rc` substituted
  for `Arc`.
- **Phase 1 step 3: associated-type-bound trick is propagated to
  every struct and impl.** Decision 4.4 names the trick
  (`Kind<Of<'a, A>: Send + Sync>`) but does not prescribe scope.
  In production, `Send + Sync` auto-derivation on `ArcFreeInner`
  via the `F::Of<...>` field requires the bound at the struct
  definition. To keep all uses of the inner data type-checkable,
  the same `Kind_cdc7cd43dac7585f<Of<'static, ArcFree<F, ArcTypeErasedValue>>: Send + Sync>`
  bound is added to `ArcContinuation<F>`, `ArcFreeView<F>`,
  `ArcFreeStep<F, A>`, `ArcFreeInner<F, A>`, `ArcFree<F, A>`, and
  every `impl` block that mentions any of them. This is verbose
  but mechanical; `ArcCoyoneda`'s template uses the same trick at
  fewer sites because its trait-object internal representation
  hides the `F::Of` from auto-derivation.
- **Phase 1 step 4: `RcFreeExplicitBrand<F>` struct and `impl_kind!`
  registration land in step 4, not step 7.** Step 4's text says
  "Brand-compatible: this is the multi-shot variant that carries
  Brand dispatch in Phase 1 step 7", which on a strict reading could
  mean step 7 introduces both the brand struct and the trait impls.
  Step 1 set the precedent of pairing the brand struct + `impl_kind!`
  with the type definition (`FreeExplicitBrand<F>` was added in step 1
  even though its `Functor`/`Pointed`/`Semimonad`/`Monad` impls are
  scheduled for step 7). Step 4 follows the same precedent: the
  brand and `Kind` registration ship now, the trait hierarchies
  ship in step 7. This keeps step 7's scope to "trait impls" only.
- **Phase 1 step 4: `Wrap` variant holds `RcFreeExplicit` directly,
  not `Box<RcFreeExplicit>`.** `FreeExplicit`'s `Wrap` variant uses
  `F::Of<'a, Box<FreeExplicit<'a, F, A>>>` because the outer struct
  is unboxed and a recursive type needs indirection to be sized.
  `RcFreeExplicit`'s outer wrapper is `Rc<RcFreeExplicitInner>`,
  which already provides the indirection, so the `Wrap` arm holds
  `F::Of<'a, RcFreeExplicit<'a, F, A>>` directly. Skipping the `Box`
  layer avoids one extra heap hop per node and keeps the `F::extract`
  call site free of a `*extracted` deref.
- **Phase 1 step 4: `to_view(self)` is exposed as a public
  consuming method.** Step 4's text only names `lower_ref(&self)`
  and `peel_ref(&self)`. `peel_ref` is naturally implemented as
  `self.clone().to_view()`, which requires a consuming `to_view`
  on the underlying type (the `view` field is private). Exposing
  `to_view` publicly keeps the implementation symmetric with
  `RcFree::to_view` and avoids burying the consuming version as a
  private helper. `FreeExplicit` does not have `to_view` because
  it does not have `peel_ref` either.
- **Phase 1 step 4: inherent-method API is intentionally narrower
  than `RcFree`'s.** `RcFree` exposes `pure`, `wrap`, `lift_f`,
  `bind`, `map`, `to_view`, `resume`, `evaluate`, `hoist_free`,
  plus `lower_ref` / `peel_ref`. `RcFreeExplicit` exposes only
  `pure`, `wrap`, `bind`, `evaluate`, `to_view`, `lower_ref`,
  `peel_ref`. The omitted methods (`lift_f`, `map`, `resume`,
  `hoist_free`) belong on the Brand-dispatched API surface that
  step 7 builds via `Functor` / `Pointed` / `Semimonad` / `Monad`,
  so adding them as inherent methods here would duplicate that
  surface. `RcFree` has them inherently because the Erased family
  has no Brand dispatch at all (decisions section 4.4); the
  Explicit family routes the same operations through the trait
  hierarchy.

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
non-`'static` payloads), use the corresponding Explicit variant:

```rust
fn run_program_explicit<'a>() -> RunExplicit<'a, AppEffects, NoScoped, String> {
    m_do!(RunExplicitBrand {
        cfg <- ask::<Env>();
        n <- get::<Counter>();
        pure(format!("got {n}"))
    })
}
```

Convert between the two on demand: `run_program().into_explicit()`
walks the structure once and returns an `RunExplicit` of the same
program, suitable for handing into typeclass-generic consumers.

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
| 9.3 / 9.4 | Sync interpreters in v1; async (and async IO) via `Future` as a `MonadRec` target in Phase 3                            | "User picks the target monad" — single mechanism, no parallel `AsyncRun` family                                                                       |
| 9.8       | All effects-related macros live in `fp-macros`; split off a separate crate only if needed                               | One crate, one release cadence, one place to coordinate macro semantics                                                                               |
| 9.9       | TalkF + DinnerF integration test from `purescript-run` as the headline Phase 4 milestone                                | Real-world reference; validates the port behaves like `purescript-run` for a worked example                                                           |

## Integration surface

### Will change

| Component                                                                                         | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| ------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `fp-library/src/types/free.rs`                                                                    | Existing `Free<F, A>` keeps its current shape; inherent-method only (no Brand). Minor adjustments if integration with `Run` requires.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| `fp-library/src/types/free_explicit.rs`                                                           | **New module (Phase 1 step 1).** Promote `FreeExplicit<'a, F, A>` from POC, add iterative custom `Drop`, add `Functor` / `Pointed` / `Semimonad` / `Monad` impls (Phase 1 step 7).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `fp-library/src/types/rc_free.rs`                                                                 | **New module (Phase 1 step 2).** `RcFree<F, A>` following the `Free` template with `FnBrand<RcBrand>`-shaped continuations (i.e., `Rc<dyn 'a + Fn(B) -> RcFree<F, A>>` via the unified [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) abstraction). Multi-shot effects (`Choose`, `Amb`). Inherent-method only; no `RcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `fp-library/src/types/arc_free.rs`                                                                | **New module (Phase 1 step 3).** `ArcFree<F, A>` following the `ArcCoyoneda` template with `FnBrand<ArcBrand>`-shaped continuations (i.e., `Arc<dyn 'a + Fn(B) -> ArcFree<F, A> + Send + Sync>` via [`FnBrand`](../../../fp-library/src/types/fn_brand.rs) parameterised by [`ArcBrand`](../../../fp-library/src/brands.rs#L43)) and the `Send`/`Sync` Kind-trait pattern via `SendRefCountedPointer`. Inherent-method only; no `ArcFreeBrand`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| `fp-library/src/types/rc_free_explicit.rs`                                                        | **New module (Phase 1 step 4).** `RcFreeExplicit<'a, F, A>` extending `FreeExplicit`'s concrete recursive enum with an outer `Rc<RcFreeExplicitInner>` wrapper plus `Rc<dyn Fn>` continuations. O(N) bind, multi-shot, `A: 'a`, Brand-compatible (registered as `RcFreeExplicitBrand<F>` in step 7). Custom iterative `Drop` via `Extract` + `Rc::try_unwrap`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `fp-library/src/types/arc_free_explicit.rs`                                                       | **New module (Phase 1 step 5).** `ArcFreeExplicit<'a, F, A>` extending `RcFreeExplicit`'s shape with `Arc<...>` wrapping and `Arc<dyn Fn + Send + Sync>` continuations. Same `Kind<Of<'a, A>: Send + Sync>` associated-type-bound trick as `ArcFree`. Brand-compatible (`ArcFreeExplicitBrand<F>`) via the new `SendFunctor` family from step 6.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| `fp-library/src/classes/send_functor.rs`, `send_pointed.rs`, `send_semimonad.rs`, `send_monad.rs` | **New trait files (Phase 1 step 6).** By-value parallels of the existing `send_ref_*` family with `Send + Sync` bounds on the closure parameters. Required by `ArcFreeExplicitBrand`'s `Functor`/`Pointed`/`Semimonad`/`Monad` impls; also supplies the missing trait impls for `ArcCoyonedaBrand` that the existing module docs flag.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| `fp-library/src/types/run.rs`                                                                     | **New module (Phase 2 step 4).** Six concrete Run types: `Run<R, S, A>`, `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family, inherent-method only) and `RunExplicit<'a, R, S, A>`, `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>` (Explicit family, Brand-dispatched). `Node<R, S>` enum dispatching first-order vs scoped layers. `into_explicit()` / `from_erased()` conversion API between paired Erased and Explicit Run variants.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `fp-library/src/types/run/coproduct.rs`                                                           | **New submodule.** Brand-aware adapter layer over `frunk_core::coproduct::{Coproduct, CNil, CoproductSubsetter}`: newtype wrappers, `impl` blocks bridging `frunk_core`'s Plucker / Sculptor / Embedder traits to the project's `Brand` system. Direct (non-newtyped) `Functor` impls on `frunk_core::Coproduct<H, T>` live here too (own-trait + foreign-type, orphan-permitted).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| `fp-library/src/types/run/variant_f.rs`                                                           | **New submodule.** `VariantF<Effects>` first-order coproduct with Coyoneda-wrapped variants and recursive `Functor` impl on `Coproduct<H, T>` (delegating to the adapter in `coproduct.rs`).                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `fp-library/src/types/run/scoped.rs`                                                              | **New submodule.** `ScopedCoproduct<ScopedEffects>` higher-order coproduct, standard scoped constructors. `Catch<'a, E>` and `Span<'a, Tag>` ship Val-only. `Local` ships in Val and Ref flavours (`Local<'a, E>` + `RefLocal<'a, E>`); `Bracket` ships in Val and Ref flavours (`Bracket<'a, A, B>` + `RefBracket<'a, P, A, B>`) per [decisions.md](decisions.md) section 4.5 sub-decisions. `Mask` is deferred to a future revision per the same section.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `fp-library/src/dispatch/run/`                                                                    | **New submodule.** Closure-driven Val/Ref dispatch for `bracket` and `local` smart constructors, mirroring the existing layout described in [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md). Files: `bracket.rs` (`BracketDispatch` trait + `Val` impl + `Ref<P>` impls per pointer brand + `bracket` inference wrapper + `explicit::bracket` brand-explicit wrapper); `local.rs` (`LocalDispatch` trait + `Val` and `Ref` impls + `local` inference wrapper + `explicit::local` wrapper). Re-exported from `fp-library/src/functions.rs` alongside `map`, `bind`, etc.                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-library/src/types/run/handler.rs`                                                             | **New submodule.** Handler-pipeline machinery (`Run::handle`), natural-transformation type, `peel` / `send` / `extract`.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `fp-library/src/types/run/interpreter.rs`                                                         | **New submodule.** `interpret` / `run` / `runAccum` (recursive) and `interpretRec` / `runRec` / `runAccumRec` (`MonadRec`-targeted) families.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |
| `fp-macros/src/effects/`                                                                          | **New module tree.** `effects!`, `effects_coyo!`, `handlers!`, `define_effect!`, `define_scoped_effect!`, `scoped_effects!`, and `run_do!` proc-macros. `run_do!` is the inherent-method-based monadic do-notation for the Erased Run family (`Run` / `RcRun` / `ArcRun`); the Explicit Run family uses the existing `m_do!` / `a_do!` over the corresponding `*RunExplicitBrand`. Migration from POC for the row-construction macros.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
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
  to work for the Explicit Run family (`RunExplicit`,
  `RcRunExplicit`, `ArcRunExplicit`) once the corresponding
  `*RunExplicitBrand` impls from Phase 2 step 4 ship. The Erased
  Run family (`Run`, `RcRun`, `ArcRun`) uses the new `run_do!`
  macro instead, since those types are not Brand-dispatched.

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
   by-value parallels of the existing `send_ref_*` files). Each
   takes its closure parameter as `impl Fn(...) + Send + Sync`,
   resolving the gap that today prevents `ArcCoyonedaBrand` from
   implementing `Functor` and that `ArcFreeExplicitBrand` would
   otherwise inherit. Add `SendFunctor` (and downstream)
   implementations for `ArcCoyonedaBrand` as a bonus integration,
   closing the open gap that
   [arc_coyoneda.rs](../../../fp-library/src/types/arc_coyoneda.rs)'s
   module docs flag.
7. Add brand registrations and trait hierarchies for the three
   Explicit Free brands (`FreeExplicitBrand<F>`,
   `RcFreeExplicitBrand<F>`, `ArcFreeExplicitBrand<F>`):
   - The by-value hierarchy (`Functor` / `Pointed` / `Semimonad` /
     `Monad`) plus the by-reference hierarchy (`RefFunctor` /
     `RefSemimonad` / `RefMonad`, etc., per
     [`fp-library/docs/dispatch.md`](../../../fp-library/docs/dispatch.md))
     for `FreeExplicitBrand` and `RcFreeExplicitBrand` via the
     existing trait family.
   - `SendFunctor` / `SendPointed` / `SendSemimonad` /
     `SendMonad` impls for `ArcFreeExplicitBrand` via the new
     trait family from step 6.
   - The Erased family (`Free`, `RcFree`, `ArcFree`) does not get
     brands; those types remain inherent-method only.
   - Both hierarchies are required so `dispatch::map` /
     `dispatch::bind` route correctly over each Brand-dispatched
     Free variant once `Run` and the scoped-effect smart
     constructors land in Phase 2 / Phase 4.
8. Per-variant Criterion benches for all six variants (bind-deep
   at depths 10 / 100 / 1000 / 10000, bind-wide, peel-and-handle),
   plus a cross-variant comparison bench documenting the O(1) vs
   O(N) bind-cost asymmetry. Match the `FreeExplicit` POC bench
   shape.
9. Per-variant unit tests covering construction, evaluation,
   `fold_free` interpretation, and the property each variant
   promises (single-shot vs. multi-shot, thread-safe,
   `'static` vs `'a`, Brand-dispatched vs inherent-method-only).
   Plus `compile_fail` UI tests for the negative cases:
   multi-shot via `Free`, Brand-dispatched call against an
   Erased variant, missing `Send + Sync` on a closure passed to
   `ArcFreeExplicit::bind`, etc.

### Phase 2: Run substrate and first-order effects

1. Add `frunk_core` as a direct dependency of `fp-library`
   (license check via `just deny`, MSRV verification, and
   workspace `Cargo.toml` registration). Introduce a thin
   Brand-aware adapter layer at `fp-library/src/types/run/coproduct.rs`:
   newtype wrappers around `frunk_core::coproduct::{Coproduct, CNil}`
   plus `impl` blocks bridging `frunk_core`'s Plucker / Sculptor /
   Embedder traits to the project's `Brand` system. Direct `impl`s
   of fp-library's own `Functor` for `frunk_core::Coproduct<H, T>`
   are permitted by the orphan rules; `Brand`-style impls require
   the newtype wrapper. See Implementation note 1 below.
2. `VariantF<Effects>` at `fp-library/src/types/run/variant_f.rs`:
   Coyoneda-wrapped Coproduct row with recursive `Functor` impl
   on `Coproduct<H, T>` (where `H: Functor + T: Functor`) and base
   case on `CNil`. Migrate the trait-shape from
   [poc-effect-row/src/lib.rs](../../../poc-effect-row/src/lib.rs)
   under the production `Functor` trait.
3. `Member<E, Indices>` trait for first-order injection /
   projection, layered on top of `frunk_core::CoproductSubsetter`
   via the adapter from step 1.
4. Six `Run` types at `fp-library/src/types/run.rs` (and
   sibling files), one per Free variant: `Run<R, S, A>`,
   `RcRun<R, S, A>`, `ArcRun<R, S, A>` (Erased family,
   inherent-method only) and `RunExplicit<'a, R, S, A>`,
   `RcRunExplicit<'a, R, S, A>`, `ArcRunExplicit<'a, R, S, A>`
   (Explicit family, Brand-dispatched). Each is a thin wrapper
   over its Free variant with a shared `Node<R, S>` enum
   dispatching first-order vs scoped layers.
   - For the three Explicit Run brands (`RunExplicitBrand`,
     `RcRunExplicitBrand`, `ArcRunExplicitBrand`), add the
     by-value hierarchy (`Functor` / `Pointed` / `Semimonad` /
     `Monad`) plus the by-reference hierarchy (`RefFunctor` /
     `RefSemimonad` / `RefMonad`, etc.) by delegating to the
     underlying `*FreeExplicitBrand` impls from Phase 1 step 7.
     `ArcRunExplicitBrand` additionally delegates through the
     `SendFunctor` family.
   - The three Erased Run types do NOT get brands. They expose
     identical inherent-method APIs (`pure`, `peel`, `send`,
     `bind`, `map`, `lift_f`, `handle`, `extract`, etc.) but
     `m_do!` / `a_do!` do not work over them; `run_do!` from
     step 7 below is the inherent-method-based macro analogue.
   - Both hierarchies on the Explicit brands are required so
     `m_do!` / `a_do!` and `dispatch::map` / `dispatch::bind`
     work over Brand-dispatched Run programs regardless of
     whether the user's closures consume or borrow.
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
   Brand-dispatched and `m_do!` does not apply. The Explicit
   Run family continues to use `m_do!` over the corresponding
   `*RunExplicitBrand`. Both expansions accept the same
   surface syntax so users moving between the two families do
   not have to re-learn the do-notation form.
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
   family in `fp-library/src/types/run/interpreter.rs`.
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
   `fp-library/src/types/run/scoped.rs` with the dual-row
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
   `fp-library/src/types/run/coproduct.rs` (newtypes plus `impl`
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
  - [poc-effect-row/](../../../poc-effect-row/) — row-encoding
    hybrid, `tstr_crates` refinement, static-via-Coyoneda.
  - [poc-effect-row-canonicalisation.md](poc-effect-row-canonicalisation.md)
    — POC findings document.
  - [fp-library/tests/free_explicit_poc.rs](../../../fp-library/tests/free_explicit_poc.rs)
    — `FreeExplicit` POC.
  - [fp-library/benches/benchmarks/free_explicit.rs](../../../fp-library/benches/benchmarks/free_explicit.rs)
    — `FreeExplicit` Criterion bench.
- PureScript Run reference:
  [`purescript-run`](https://github.com/natefaubion/purescript-run).
- Comparison table for the Rust port versus PureScript Run and
  Hasura's `eff` is in [decisions.md](decisions.md) section 10.
