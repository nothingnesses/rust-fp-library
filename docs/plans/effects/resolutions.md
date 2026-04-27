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
[`fp-library/src/types/run.rs`](../../../fp-library/src/types/run.rs).
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
  `Extract` bound.** Define six new types in `types/run/`
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
