# Plan: By-Reference Trait Hierarchy with Unified Dispatch

## Motivation

Currently, `Lazy` (`RcLazy`, `ArcLazy`) can only implement `RefFunctor` and
`Foldable`. It cannot participate in generic code that uses `Applicative`,
`Monad`, or other higher-level type classes because those require by-value
consumption of the container. This limits the composability of memoized types.

Additionally, the call site requires users to choose between `map` and
`ref_map` explicitly. A unified `map` function that dispatches based on
argument ownership (inspired by `haskell_bits`) would improve ergonomics
without changing the underlying trait definitions.

## Goals

1. Enable `Lazy` to implement the full Functor -> Applicative -> Monad chain
   via by-reference trait variants.
2. Provide unified free functions (`map`, `bind`, `apply`, etc.) that
   dispatch to the correct trait based on whether the container is owned
   or borrowed.
3. Preserve zero-cost abstractions: no hidden clones, no heap allocation
   in dispatch.
4. Keep the existing by-value traits unchanged; the new by-ref traits are
   independent (not supertraits/subtraits of the existing ones).

## Design

### Phase 1: By-Ref Trait Variants

Add new traits that mirror the by-value hierarchy but take containers and
elements by reference. These follow the existing `RefFunctor` pattern.

| By-value trait    | New by-ref trait     | Method signature change                                                                                                                                                                                                                       |
| ----------------- | -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Functor`         | `RefFunctor`         | Already exists. `ref_map(f: FnOnce(&A) -> B, fa)`                                                                                                                                                                                             |
| `Pointed`         | `RefPointed`         | `ref_pure(a: &A) -> Of<A>` where `A: Clone`. Needed because by-ref generic code only has `&A`, and constructing `Of<A>` from `&A` requires cloning. The `Clone` bound is on the trait method, not the trait itself, making the cost explicit. |
| `Lift`            | `RefLift`            | `ref_lift2(f: impl Fn(&A, &B) -> C, fa, fb) -> Of<C>`. No `Clone` bound needed; the closure receives references and produces an owned `C`. What the closure does with `&A` and `&B` (including whether to clone) is user-controlled.          |
| `Semiapplicative` | `RefSemiapplicative` | `ref_apply(fab: Of<Rc<dyn Fn(&A) -> B>>, fa) -> Of<B>`. No `Clone` bound; the wrapped function receives `&A`.                                                                                                                                 |
| `Semimonad`       | `RefSemimonad`       | `ref_bind(ma, f: impl Fn(&A) -> Of<B>) -> Of<B>`. No `Clone` bound; the closure receives `&A` and decides what to do with it.                                                                                                                 |

**Clone bound design principle:** Only `RefPointed` requires `Clone`, because
it is the only operation that must produce an owned value from a reference
without a user-supplied closure to mediate. All other by-ref traits pass `&A`
to a user closure, and the user controls whether cloning happens through what
the closure body does. This keeps implicit cloning out of the trait system.

Blanket traits follow naturally:

| By-value blanket | New by-ref blanket | Supertraits                       |
| ---------------- | ------------------ | --------------------------------- |
| `Applicative`    | `RefApplicative`   | `RefPointed + RefSemiapplicative` |
| `Monad`          | `RefMonad`         | `RefApplicative + RefSemimonad`   |

**SendRef variants** mirror the same pattern with `Send + Sync` bounds on
closures and elements, following the existing `SendRefFunctor` precedent:

- `SendRefFunctor` (already exists)
- `SendRefLift`
- `SendRefSemiapplicative`
- `SendRefSemimonad`
- `SendRefApplicative` (blanket)
- `SendRefMonad` (blanket)

### Phase 2: Unified Dispatch

Add a marker-type dispatch pattern (inspired by `haskell_bits`'s `MapExt`)
to let a single free function route to the correct trait.

**Marker types:**

```rust
pub struct Val;  // container passed by value
pub struct Ref;  // container passed by reference
```

**Dispatch trait (for map):**

```rust
pub trait FunctorDispatch<Brand, F, A, B, Ownership> {
    fn dispatch_map(f: F, container: Self) -> Brand::Of<B>;
}
```

**Four impls:**

1. `impl FunctorDispatch<..., Val> for Of<A>` where `Brand: Functor`
   -> calls `Functor::map`
2. `impl FunctorDispatch<..., Ref> for &Of<A>` where `Brand: RefFunctor`
   -> calls `RefFunctor::ref_map`

(The closure `Fn(A) -> B` vs `Fn(&A) -> B` distinction could be a second
axis of dispatch, but this adds complexity. Initially, by-value dispatch
uses `Fn(A) -> B` and by-ref dispatch uses `Fn(&A) -> B`. The closure
type is determined by the container ownership.)

**Unified free function:**

```rust
pub fn map<Brand, F, A, B, Ownership>(
    f: F,
    container: impl FunctorDispatch<Brand, F, A, B, Ownership>,
) -> Brand::Of<B> {
    container.dispatch_map(f)
}
```

Repeat this pattern for `bind`, `apply`, `lift2`.

### Phase 3: Lazy Implementations

Implement the new by-ref traits for `LazyBrand<C>` (both `RcLazyConfig`
and `ArcLazyConfig`):

- **RefFunctor**: Already implemented.
- **RefPointed**: `ref_pure(a: &A) -> Lazy<A>` where `A: Clone` -> `Lazy::new({ let v = a.clone(); move || v })`.
- **RefLift**: `ref_lift2(f, la, lb)` -> `Lazy::new(move || f(la.evaluate(), lb.evaluate()))`.
- **RefSemiapplicative**: `ref_apply(lf, la)` -> evaluate both, apply.
- **RefSemimonad**: `ref_bind(la, f)` -> evaluate `la`, call `f(&a)`.
- **RefApplicative**: Blanket from RefPointed + RefSemiapplicative.
- **RefMonad**: Blanket from RefApplicative + RefSemimonad.

For `ArcLazyBrand`:

- Implement `SendRefFunctor` (already done).
- Implement `SendRefLift`, `SendRefSemiapplicative`, `SendRefSemimonad`.
- Blanket `SendRefApplicative`, `SendRefMonad`.

### Phase 4: Collection Types

Implement Ref trait variants for collection types (`Vec`, `Option`,
`Result`, `CatList`, `Identity`). Unlike memoized types (Lazy) where
Ref traits replace by-value traits, collection types implement both:
by-value consumes the container, by-ref borrows it.

**Ref traits for collections:**

- `RefFunctor`: `ref_map(f: Fn(&A) -> B, fa) -> F<B>` (iterate by
  reference, produce new container).
- `RefFoldable`: `ref_fold_map(f: Fn(&A) -> M, fa) -> M` (fold by
  reference without consuming).
- `RefFunctorWithIndex`, `RefFoldableWithIndex`: indexed variants.
- `RefFilterable`, `RefFilterableWithIndex`: filter by reference.
- `RefWitherable`: effectful filter by reference.

**Par-Ref traits for collections:**
Once collection types implement Ref traits, parallel by-ref variants
become meaningful:

- `ParRefFunctor`: `par_ref_map(f: Fn(&A) -> B + Send + Sync, fa)`
  Parallel map over borrowed elements (rayon `par_iter().map()`).
- `ParRefFoldable`: `par_ref_fold_map` (parallel fold by reference).
- `ParRefFunctorWithIndex`, `ParRefFoldableWithIndex`: indexed.
- `ParRefFilterable`, `ParRefFilterableWithIndex`: parallel filter
  by reference.

These are deferred until collection Ref impls exist.

**Other types:**

- **Coyoneda variants**: `RcCoyoneda` and `ArcCoyoneda` already use
  `lower_ref` (by-reference lowering). They could implement `RefFunctor`
  to map over the lowered result by reference. Medium priority.
- **Identity**: Trivial to implement both paths.

## Design Decisions

1. **By-ref and by-value traits are independent (no sub/supertrait
   relationship).** A supertrait relationship would force types to implement
   both, preventing `Lazy` from implementing only by-ref. The unified
   dispatch layer handles ergonomics without the type system constraint.

2. **Use `Fn` (not `FnOnce`) for all by-ref trait closures.** Change
   existing `RefFunctor` from `FnOnce` to `Fn`. This is a breaking change,
   but necessary: types like `Vec` that call the closure per element need
   `Fn`. `Lazy` works with `Fn` too (calls it once). Closures that move
   out of captures (`FnOnce` but not `Fn`) will no longer compile with
   `ref_map`; these are rare and can be restructured.

3. **The dispatch replaces the existing free functions.** The unified `map`
   dispatches to `Functor::map` for owned arguments, which is identical to
   the current behavior. Same for `bind`, `apply`, `lift2`. If type
   inference issues emerge during the proof of concept, add annotations
   rather than keeping two sets of functions.

4. **`m_do!` gets a `ref` qualifier.** `m_do!(ref LazyBrand { ... })`
   generates `ref_bind` calls. `m_do!(VecBrand { ... })` generates `bind`
   calls as before. One macro, explicit ownership mode at the block level.

5. **Add `RefFoldable`, skip `RefTraversable` initially.** `Lazy`'s
   by-value `Foldable` impl is semantically dishonest: it takes `self`
   but internally borrows via `evaluate()`. `RefFoldable` fixes this by
   honestly taking `&self`, matching the `RefFunctor` pattern.
   `RefTraversable` is deferred (complex interaction with the applicative
   used for the output context). Revisit if a concrete use case emerges.

## Proof of Concept Results

The dispatch proof of concept is in `fp-library/src/classes/functor_dispatch.rs`.
All concerns from the original open question have been resolved:

- **Type inference works.** The compiler correctly infers the `Val` or `Ref`
  marker from the closure's argument type. `|x: i32| x * 2` resolves to
  `Val` (Functor::map); `|x: &i32| *x * 2` resolves to `Ref`
  (RefFunctor::ref_map). No explicit marker annotation needed at call sites.
- **No coherence issues.** `FunctorDispatch<..., Val>` and `FunctorDispatch<..., Ref>`
  impls coexist without conflict.
- **The `Apply!` macro is not needed.** The dispatch trait and unified
  function use the raw `Kind_cdc7cd43dac7585f` trait name with its `Of`
  associated type directly. This avoids the `Kind!(...)` / `Apply!(...)`
  macro nesting that is only required in positions where the `#[kind(...)]`
  attribute macro processes trait definitions.

### Key finding

Unlike `haskell_bits`, which dispatches on container ownership (owned `T`
vs borrowed `&T`), this library's dispatch is on **closure argument type**
(`Fn(A) -> B` vs `Fn(&A) -> B`). Both paths take the container by value.
This works because the compiler resolves the `Marker` type parameter from
the `Fn` impl of the closure, which is unambiguous: a closure taking `i32`
can only satisfy `FunctorDispatch<..., Val>`, and a closure taking `&i32` can
only satisfy `FunctorDispatch<..., Ref>`.

### Rejected alternative: Mode-parameterized Functor

A single `Functor<Mode>` trait (with `Mode` selecting owned vs borrowed
element access) was investigated and rejected for three reasons:

1. **Lifetime provenance.** `Mode::Apply<'a, A> = &'a A` requires the
   reference to borrow from something that outlives the function body.
   Types that are consumed by `map` (Vec, Option) cannot produce such
   borrows. Only types with internal shared storage (Lazy) can.
2. **Circular inference.** The compiler cannot infer `Mode` from the
   closure type because the closure type depends on `Mode`.
3. **Hierarchy contamination.** Every downstream trait (~30+) would need
   to carry the `Mode` parameter or pin it, adding dead weight.

## Implementation Order

1. ~~**Proof of concept**: Implement `FunctorDispatch` with `Functor`
   and `RefFunctor` routing. Verify compilation and type inference.~~ Done.
2. ~~**Promote dispatch to production**: Replace the separate `map` and
   `ref_map` free functions with a unified `map` that dispatches via
   `FunctorDispatch`. Update all call sites (`map::<Brand, _, _>` ->
   `map::<Brand, _, _, _>`), the `a_do!` macro, benchmark imports, and
   doc examples. The `Marker` type parameter is hidden behind `impl` and
   inferred by the compiler; callers never specify it.~~ Done.
   Implementation: `fp-library/src/classes/functor_dispatch.rs`.
3. ~~**Change RefFunctor/SendRefFunctor from FnOnce to Fn.**~~ Done.
   Breaking change: closures that move out of captures (FnOnce but not
   Fn) no longer compile with ref_map. This enables types like Vec to
   implement RefFunctor in the future.
4. ~~**RefPointed, RefLift, RefSemimonad**: Add the by-ref traits with
   RcLazy implementations.~~ Done. ArcLazy impls require SendRef variants
   due to `Send + Sync` bounds on `ArcLazy::new`.
   Implementation: `fp-library/src/classes/ref_pointed.rs`,
   `fp-library/src/classes/ref_lift.rs`,
   `fp-library/src/classes/ref_semimonad.rs`.
5. ~~**Add `ClosureMode` trait and parameterize `CloneableFn`**~~: Done.
   Added `ClosureMode` with `Val`/`Ref` impls. Parameterized
   `CloneableFn<Mode: ClosureMode = Val>`. Removed `Function`
   supertrait. Split `new` into `LiftFn: CloneableFn<Val>`. Renamed
   free function from `cloneable_fn_new` to `lift_fn_new`. Added
   `coerce_ref_fn` to `UnsizedCoercible` for by-ref construction.
   Converted many explicit class imports to wildcards.
   Implementation: `ClosureMode` in `functor_dispatch.rs`,
   `LiftFn` in `cloneable_fn.rs`.
6. ~~**Add `CloneableFn<Ref>` impl for `FnBrand<P>`**~~. Done.
   Added `CloneableFn<Ref>` and `SendCloneableFn<Ref>` impls for
   `FnBrand<P>`. Parameterized `SendCloneableFn` with `ClosureMode`,
   split `send_cloneable_fn_new` into `SendLiftFn::new`. Added
   `SendTarget` GAT to `ClosureMode`.
7. ~~**RefSemiapplicative**~~: Done. Defined using `CloneableFn<Ref>`.
   Implemented for `LazyBrand<RcLazyConfig>`.
8. ~~**Blanket traits**~~: Done. `RefApplicative` and `RefMonad` with
   monad law examples. Updated `RefApplicative` to include
   `RefApplyFirst + RefApplySecond` supertraits (matching `Applicative`).
9. ~~**SendRef variants**~~: Done. `SendRefPointed`, `SendRefLift`,
   `SendRefSemimonad`, `SendRefSemiapplicative` with `ArcLazy` impls.
10. ~~**SendRef blanket traits**~~: Done. `SendRefApplicative` (including
    `SendRefApplyFirst + SendRefApplySecond` supertraits),
    `SendRefMonad` with monad law examples.
11. ~~**Ref parity traits**~~: Done. Added `RefApplyFirst`,
    `RefApplySecond`, `SendRefApplyFirst`, `SendRefApplySecond` with
    blanket impls from `RefLift`/`SendRefLift`. Added `ref_if_m`,
    `ref_unless_m` free functions.
12. **Rename traits**: Rename `CloneableFn` to `CloneFn`,
    `SendCloneableFn` to `SendCloneFn`, `Function` to `Arrow`.
    Make `SendCloneFn` independent (remove `CloneFn` supertrait),
    rename `SendOf` to `Of`. Rename `Function::new` to
    `Arrow::arrow`, free function `fn_new` to `arrow`. Rename
    files to match. See "Planned trait renames" section for full
    table.
13. ~~**Core dispatch unification**~~: Done. `BindDispatch`,
    `Lift2Dispatch`, `Lift3Dispatch`, `Lift4Dispatch`,
    `Lift5Dispatch` added alongside existing `FunctorDispatch`.
    Unified `bind` replaces separate `bind` and `ref_bind`. Unified
    `lift2`-`lift5` replace separate `liftN` and `ref_liftN`. Updated
    `m_do!` and `a_do!` macros for new generic parameter counts
    (uniform `n + 2` underscores for all liftN).
    All call sites updated.

14. ~~**Restructure dispatch module**~~: Done. Replaced
    `functor_dispatch.rs` with `dispatch.rs` (shared types, tests,
    brand inference POC) and `dispatch/` directory:
    - `dispatch/functor.rs` (FunctorDispatch + map)
    - `dispatch/semimonad.rs` (BindDispatch + bind)
    - `dispatch/lift.rs` (Lift2-5Dispatch + lift2-5)
      Explicit re-exports added to `functions.rs` since the macro
      scanner does not traverse sub-directories.

15. ~~**Remove `dispatch/apply_first.rs`**~~: Done. Deleted the
    incorrectly created dispatch file. `apply_first` and
    `apply_second` take two containers and no closure, so there is
    no argument for the compiler to infer `Val`/`Ref` from. These
    remain as separate `apply_first` / `ref_apply_first` free
    functions.

16. ~~**Add `Ref*` foldable and indexed traits, remove `Lazy`'s
    by-value impls**~~: Done. Added:
    - `RefFoldable` (ref_fold_map, ref_fold_right, ref_fold_left)
    - `SendRefFoldable`
    - `RefFoldableWithIndex: RefFoldable + WithIndex`
    - `SendRefFoldableWithIndex: SendRefFoldable + WithIndex`
    - `RefFunctorWithIndex: RefFunctor + WithIndex` (RcLazy only)
    - `SendRefFunctorWithIndex: SendRefFunctor + WithIndex` (ArcLazy)
      Removed `Lazy`'s by-value `Foldable` and `FoldableWithIndex`
      impls (breaking change). Kept `WithIndex` impl (`Index = ()`).
      Same migration for `TryLazy`.

    **Deferred Ref variants** (no memoized type currently needs them):
    - `RefFilterableWithIndex` (needs `RefFilterable` + `RefCompactable`)
    - `RefBifunctor`, `RefBifoldable`, `RefBitraversable` and their
      WithIndex variants (no memoized bifunctor type exists).

17. ~~**Add missing free functions for WithIndex traits**~~: Done.
    Added free functions for:
    - `FunctorWithIndex`: `map_with_index`
    - `FoldableWithIndex`: `fold_map_with_index`
    - `TraversableWithIndex`: `traverse_with_index`
    - `RefFoldableWithIndex`: `ref_fold_map_with_index`
    - `RefFunctorWithIndex`: `ref_map_with_index`
    - `SendRefFoldableWithIndex`: `send_ref_fold_map_with_index`
    - `SendRefFunctorWithIndex`: `send_ref_map_with_index`

18. ~~**Add ref semimonad helpers**~~: Done. Added ref variants:
    - `ref_bind_flipped` in `ref_semimonad.rs`
    - `ref_compose_kleisli` in `ref_semimonad.rs`
    - `ref_compose_kleisli_flipped` in `ref_semimonad.rs`
    - `ref_join` in `ref_semimonad.rs` (not dispatchable, separate)

19. ~~**Dispatch foldable operations**~~: Done. Added
    `dispatch/foldable.rs` unifying:
    - `fold_right` / `ref_fold_right`
    - `fold_left` / `ref_fold_left`
    - `fold_map` / `ref_fold_map`
      Both paths take `FnBrand` parameter. Added `FnBrand` parameter
      and default impl to `RefFoldable::ref_fold_map`, making all
      three methods mutually derivable (matching `Foldable`'s design).

20. ~~**Dispatch semimonad helpers**~~: Done. Extended
    `dispatch/semimonad.rs` to unify:
    - `bind_flipped` / `ref_bind_flipped`
    - `compose_kleisli` / `ref_compose_kleisli`
    - `compose_kleisli_flipped` / `ref_compose_kleisli_flipped`

21. ~~**Add RefFilterable hierarchy**~~: Done. Added:
    - `RefFilterable: RefFunctor + Compactable` (not `RefCompactable`,
      since compact/separate are structural and mode-independent)
    - `RefTraversable: RefFunctor + RefFoldable`
    - `RefWitherable: RefFilterable + RefTraversable`
    - `RefFilterableWithIndex: RefFilterable + RefFunctorWithIndex`
    - `RefTraversableWithIndex: RefTraversable + RefFunctorWithIndex`
    - Doc examples marked `ignore` until collection impls.
    - SendRef variants deferred (no thread-safe memoized type needs
      filtering/traversal).
    - Design principle: traits whose methods take only containers
      (no closures) don't need Ref variants. Compound traits use the
      non-Ref version of structural supertraits and the Ref version
      of element-accessing supertraits.

22. ~~**Ref trait impls for collection types**~~: Done.
    Vec, Option, CatList, Identity all implement:
    - RefFunctor, RefFoldable, RefTraversable (+ WithIndex variants)
    - RefFilterable, RefFilterableWithIndex, RefWitherable (Vec/Option/CatList)
    - RefPointed, RefLift, RefSemiapplicative, RefSemimonad
    - RefApplicative, RefMonad (blanket, auto-derived)
    - Identity: WithIndex, FunctorWithIndex, FoldableWithIndex,
      TraversableWithIndex and their Ref variants added.

23. ~~**Foldable mutual derivation parity**~~: Done. Added `FnBrand`
    parameter and mutual derivation to all foldable trait variants:
    - `FoldableWithIndex`: `FnBrand`, `fold_right_with_index`,
      `fold_left_with_index`. `Clone` bound on `WithIndex::Index`.
      `FnBrand` on `IndexedFoldFunc` (defaulted to `RcFnBrand`).
    - `RefFoldableWithIndex`: `FnBrand`, `ref_fold_right_with_index`,
      `ref_fold_left_with_index` via `Endofunction`.
    - `SendRefFoldable`: `FnBrand`, `send_ref_fold_right`,
      `send_ref_fold_left` via `SendEndofunction`. `M: Send + Sync`.
    - `SendRefFoldableWithIndex`: `FnBrand`,
      `send_ref_fold_right_with_index`,
      `send_ref_fold_left_with_index` via `SendEndofunction`.
      `Self::Index: Send + Sync`.
    - Added `SendEndofunction` type (Arc-based Endofunction) with
      `Semigroup`/`Monoid` impls and property tests.
    - `ParFoldable`/`ParFoldableWithIndex`: skipped (parallel folds
      are reduction-only; fold_right/fold_left are sequential).

    **Not dispatchable** (no closure or container to infer from):
    - `pure`, `ref_pure`, `send_ref_pure`
    - `apply_first`, `ref_apply_first`, `send_ref_apply_first`
    - `apply_second`, `ref_apply_second`, `send_ref_apply_second`
    - `apply`, `ref_apply`, `send_ref_apply`
    - `if_m`, `ref_if_m` (takes containers, not closures)
    - `unless_m`, `when_m`, `when`, `unless`
    - `join`, `ref_join`
    - `compact`, `separate`
      These remain as separate free functions.

24. **Par-Ref traits**: Add parallel by-reference trait variants
    for collection types:
    - `ParRefFunctor` (par_ref_map)
    - `ParRefFoldable` (par_ref_fold_map)
    - `ParRefFunctorWithIndex` (par_ref_map_with_index)
    - `ParRefFoldableWithIndex` (par_ref_fold_map_with_index)
    - `ParRefFilterable`, `ParRefFilterableWithIndex`
      Implement for Vec, CatList. Requires rayon feature.

25. **Documentation and tests**: Property tests for type class
    laws, doc examples, update limitations.md.
26. **m_do! integration**: Add `ref` qualifier to `m_do!` so it
    generates `ref_bind` calls for by-ref monadic code.

## Design Decision: CloneableFn with Mode Parameter

The by-value `Semiapplicative::apply` takes
`Of<CloneableFn::Of<'a, A, B>>`, where `CloneableFn` wraps functions
in `Rc<dyn Fn(A) -> B>` (or `Arc` via `SendCloneableFn`). The by-ref
variant needs closures that take `&A` instead of `A`.

### Obstacles to parameterizing CloneableFn

Two properties of the current `CloneableFn` trait make naive
parameterization infeasible:

1. **`Deref` bound on the associated type.** `CloneableFn` has
   `type Of<'a, A, B>: Clone + Deref<Target = dyn Fn(A) -> B>`.
   For `Mode = Ref`, the target would need to be `dyn Fn(&A) -> B`.
   A fixed `Deref` bound in the trait definition cannot produce
   different targets per mode.

2. **`Function: Category + Strong` supertrait.** `CloneableFn: Function`,
   and `Function` also has `Deref<Target = dyn Fn(A) -> B>` on its
   `Of` type. `Category` and `Strong` define composition and pairing
   operations that only apply to `Fn(A) -> B`, not `Fn(&A) -> B`.
   A by-ref function `Fn(&A) -> B` is not composable in the `Category`
   sense (the output `B` is owned, so you cannot chain
   `Fn(&A) -> B` with `Fn(&B) -> C` generically).

   The `Function` supertrait exists because `CloneableFn` "is-a"
   `Function` (it adds `Clone`). But `Semiapplicative::apply` only
   needs to _call_ and _clone_ the wrapped function, not compose it.
   The `Category`/`Strong` capabilities are unused by `apply`.

### Chosen approach: ClosureMode trait (Approach A1)

Define a `ClosureMode` trait that parameterizes the closure target:

```rust
trait ClosureMode {
    type Target<'a, A: 'a, B: 'a>: ?Sized + 'a;
}

impl ClosureMode for Val {
    type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(A) -> B;
}

impl ClosureMode for Ref {
    type Target<'a, A: 'a, B: 'a> = dyn 'a + Fn(&A) -> B;
}
```

Then `CloneableFn` becomes:

```rust
trait CloneableFn<Mode: ClosureMode = Val> {
    type PointerBrand: RefCountedPointer;
    type Of<'a, A: 'a, B: 'a>: 'a + Clone
        + Deref<Target = Mode::Target<'a, A, B>>;
}
```

The `Deref` bound is preserved but parameterized by `Mode`. Existing
code using `CloneableFn` (without `Mode`) defaults to `Val` and
sees `Deref<Target = dyn Fn(A) -> B>` as before. Code using
`CloneableFn<Ref>` sees `Deref<Target = dyn Fn(&A) -> B>`.

The `new` method cannot remain on `CloneableFn<Mode>` because its
parameter type depends on the mode (`Fn(A) -> B` for Val vs
`Fn(&A) -> B` for Ref), and a trait method has one fixed signature.
This is the same situation as `pure`, which can't use brand
inference because there's no container argument to infer from.

The solution is to split construction into a separate trait:

```rust
/// Construction of Val-mode wrapped functions.
trait LiftFn: CloneableFn<Val> {
    fn new<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(A) -> B,
    ) -> <Self as CloneableFn>::Of<'a, A, B>;
}
```

Generic code that needs to _construct_ wrapped functions (e.g.,
`Foldable::fold_right`, some `Semiapplicative` impls) uses
`FnBrand: LiftFn` instead of `FnBrand: CloneableFn`.
Code that only needs the _type_ (e.g., `RefSemiapplicative`) uses
`FnBrand: CloneableFn<Ref>`.

The same pattern applies to the Send variants:
`SendLiftFn: SendCloneableFn<Val>`.

Similarly for `SendCloneableFn<Mode: ClosureMode = Val>` and
`ArcFnBrand`.

**Breaking changes:**

- `CloneableFn` no longer implies `Function`. Code that uses
  `Brand: CloneableFn` to access `Function`/`Category`/`Strong`
  methods adds `+ Function` explicitly.
- `CloneableFn::new` moves to `LiftFn`. Call sites that
  construct wrapped functions change their bound from
  `FnBrand: CloneableFn` to `FnBrand: LiftFn`.
- `CloneableFn` gains a defaulted type parameter (`Mode`).

**What this unblocks:**

- `RefSemiapplicative` can be defined using `CloneableFn<Ref>`.
- `RefApplicative = RefPointed + RefSemiapplicative` follows the
  standard hierarchy.
- `RefMonad = RefApplicative + RefSemimonad` is the correct shape.
- No new function wrapper traits needed.

### Planned trait renames

The `Callable<Mode>` base trait extraction was investigated and
rejected. Approach A (associated type bounds on supertrait GATs,
`Callable<Mode, Of<'_, _, _>: Clone>`) is syntactically unsupported
on stable Rust. Approach B (re-declare `Of` on the subtrait) creates
non-interoperable types, no better than separate traits. Without a
way to share a single `Of` across the hierarchy, the extraction
doesn't achieve its goal. The traits remain independent with their
own `Of` types, matching the current design.

| Pre-rename name         | Post-rename name    | Role                                        |
| ----------------------- | ------------------- | ------------------------------------------- |
| `CloneableFn<Mode>`     | `CloneFn<Mode>`     | Clone + Deref. Used by Semiapplicative.     |
| `LiftFn`                | `LiftFn`            | Construction for Val mode (no change).      |
| `Function`              | `Arrow`             | Deref + Category + Strong. For optics.      |
| `Function::new`         | `Arrow::arrow`      | Lifts a pure function into an arrow.        |
| free fn `fn_new`        | free fn `arrow`     | Free function delegating to `Arrow::arrow`. |
| `SendCloneableFn<Mode>` | `SendCloneFn<Mode>` | Send variant of CloneFn.                    |
| `SendLiftFn`            | `SendLiftFn`        | Send variant of LiftFn (no change).         |

Additional structural changes during the rename:

- **`SendCloneFn` becomes independent.** Remove `CloneFn` supertrait
  from `SendCloneFn`. The supertrait was logically sound (Send
  capabilities are a superset) but practically unused, and it forces
  `FnBrand<P>` to implement both traits even when only the Send
  variant is needed.
- **Rename `SendOf` to `Of` on `SendCloneFn`.** With the supertrait
  removed, there is no name conflict. Both `CloneFn` and
  `SendCloneFn` have their own `Of`, disambiguated by trait name.
- **Rename files.** `cloneable_fn.rs` -> `clone_fn.rs`,
  `send_cloneable_fn.rs` -> `send_clone_fn.rs`,
  `function.rs` -> `arrow.rs`.
- **Update `generate_function_re_exports!`** in `functions.rs` to
  reflect new module and function names.

Post-rename hierarchy:

- `CloneFn<Mode>`: cloneable callable wrapper (Clone + Deref).
- `LiftFn: CloneFn<Val>`: adds Val-mode construction.
- `Arrow`: composable callable wrapper (Deref + Category + Strong).
  Aligns with Haskell's `Arrow` type class (`arr` + `Category` +
  `first`/`second`).
- `SendCloneFn<Mode>`: independent Send variant of CloneFn
  (Clone + Send + Sync + Deref).
- `SendLiftFn: SendCloneFn<Val>`: adds Val-mode Send construction.
- `Arrow` and `CloneFn` are independent traits, both implemented by
  `FnBrand<P>` with the same concrete type but separate `Of` types.

## Open Questions

None at this time.

## Completed Changes

- `RefFunctor` and `SendRefFunctor` closures changed from `FnOnce` to `Fn`.
- `FunctorDispatch` Ref impl updated to match.
- `RefPointed` trait and free function `ref_pure` added.
- `RefLift` trait and free function `ref_lift2` added.
- `RefSemimonad` trait and free function `ref_bind` added.
- All three implemented for `LazyBrand<RcLazyConfig>` with doc examples.
- All Lazy/TryLazy trait impls updated for `Fn` closure signatures.
- `ClosureMode` trait added with `Val`/`Ref` impls, including `SendTarget` GAT.
- `CloneableFn` parameterized with `Mode: ClosureMode = Val`.
- `Function` supertrait removed from `CloneableFn`.
- `CloneableFn::new` split into `LiftFn: CloneableFn<Val>`.
- Free function renamed from `cloneable_fn_new` to `lift_fn_new`.
- Many explicit class imports converted to wildcards.
- All downstream bounds updated (`CloneableFn` -> `LiftFn` where `new` is called).
- `SendCloneableFn` parameterized with `Mode: ClosureMode = Val`.
- `SendCloneableFn::send_cloneable_fn_new` split into `SendLiftFn::new`.
- Free function renamed from `send_cloneable_fn_new` to `send_lift_fn_new`.
- `CloneableFn<Ref>` and `SendCloneableFn<Ref>` impls added for `FnBrand<P>`.
- `RefSemiapplicative` trait added using `CloneableFn<Ref>`, implemented for RcLazy.
- `RefApplicative` blanket trait (RefPointed + RefSemiapplicative + RefApplyFirst + RefApplySecond).
- `RefMonad` blanket trait (RefApplicative + RefSemimonad) with monad law examples.
- `SendRefPointed`, `SendRefLift`, `SendRefSemimonad`, `SendRefSemiapplicative` added with ArcLazy impls.
- `SendRefApplicative` blanket trait (including SendRefApplyFirst + SendRefApplySecond).
- `SendRefMonad` blanket trait with monad law examples.
- `RefApplyFirst`, `RefApplySecond` with blanket impls from `RefLift`.
- `SendRefApplyFirst`, `SendRefApplySecond` with blanket impls from `SendRefLift`.
- `ref_if_m`, `ref_unless_m` free functions added to `ref_monad`.
- Traits renamed: `CloneableFn` -> `CloneFn`, `SendCloneableFn` -> `SendCloneFn`, `Function` -> `Arrow`.
- `Arrow::new` renamed to `Arrow::arrow`. Free function `fn_new` removed (profunctor::arrow exists).
- `SendCloneFn` made independent (removed `CloneFn` supertrait). `SendOf` renamed to `Of`.
- Files renamed: `cloneable_fn.rs` -> `clone_fn.rs`, `send_cloneable_fn.rs` -> `send_clone_fn.rs`, `function.rs` -> `arrow.rs`.
- `BindDispatch` added: unified `bind` replaces separate `bind` and `ref_bind` free functions.
- `Lift2Dispatch`, `Lift3Dispatch`, `Lift4Dispatch`, `Lift5Dispatch` added: unified `liftN` replaces separate `liftN` and `ref_liftN` free functions.
- `m_do!` and `a_do!` macros updated for new generic parameter counts (uniform `n + 2` for all liftN).
- All call sites updated for dispatched turbofish generic counts.
- Dispatch module restructured: `functor_dispatch.rs` replaced with `dispatch.rs` + `dispatch/` directory (`dispatch/functor.rs`, `dispatch/semimonad.rs`, `dispatch/lift.rs`).
- Incorrect `dispatch/apply_first.rs` removed (no closure parameter to infer from).
- `RefFoldable` trait added with `ref_fold_map`, `ref_fold_right`, `ref_fold_left`. Default impls for right/left use `Endofunction` with `A: Clone`.
- `RefFoldableWithIndex: RefFoldable + WithIndex` trait added.
- `RefFunctorWithIndex: RefFunctor + WithIndex` trait added (RcLazy impl).
- `SendRefFoldable`, `SendRefFoldableWithIndex`, `SendRefFunctorWithIndex` added with ArcLazy impls.
- Lazy's by-value `Foldable` and `FoldableWithIndex` impls removed (breaking change). Replaced by `RefFoldable` and `RefFoldableWithIndex`. `WithIndex` impl kept (`Index = ()`).
- TryLazy's by-value `Foldable` and `FoldableWithIndex` impls replaced similarly.
- All Lazy/TryLazy foldable tests updated to use `ref_fold_*` free functions.
- Free functions added for all WithIndex traits (map_with_index, fold_map_with_index, traverse_with_index, and Ref/SendRef variants).
- `ref_bind_flipped`, `ref_compose_kleisli`, `ref_compose_kleisli_flipped`, `ref_join` added to `ref_semimonad.rs`.
- `dispatch/foldable.rs` added: unifies `fold_right`/`ref_fold_right`, `fold_left`/`ref_fold_left`, `fold_map`/`ref_fold_map`.
- `RefFoldable::ref_fold_map` given `FnBrand` parameter and default impl (derives from `ref_fold_right`), making all three RefFoldable methods mutually derivable.
- `#[document_module]` macro fixed to emit module items even when inner documentation attributes fail, preventing cascading import errors.
- `RefFilterable`, `RefTraversable`, `RefWitherable`, `RefFilterableWithIndex`, `RefTraversableWithIndex` traits added.
- Design principle documented: structural traits (compact/separate) don't need Ref variants; compound traits use non-Ref structural supertraits + Ref element-accessing supertraits.
- Vec, Option, CatList: RefFilterable, RefFilterableWithIndex, RefTraversable, RefTraversableWithIndex, RefWitherable impls added.
- Vec, Option, CatList, Identity: RefPointed, RefLift, RefSemiapplicative, RefSemimonad impls added (RefApplicative/RefMonad auto-derived via blankets).
- Identity: WithIndex (Index = ()), FunctorWithIndex, FoldableWithIndex, TraversableWithIndex and their Ref variants added.
- Test cache improved: content hashing via `git ls-files` + `md5sum`, SIGPIPE handling, `just clean` recipe added.
- `FoldableWithIndex::fold_map_with_index` given `FnBrand` parameter and default impl. Added `fold_right_with_index`/`fold_left_with_index` with mutual derivation via `Endofunction`.
- `WithIndex::Index` given `Clone` bound to support endofunction composition.
- `IndexedFoldFunc` trait given `FnBrand` parameter (defaulted to `RcFnBrand`).
- All `FoldableWithIndex` impls, call sites, and doc examples updated for new `FnBrand` parameter.
- `ref_apply` doc examples fixed: use `Rc::new` directly instead of `coerce_fn` to avoid HRTB lifetime mismatch.

## References

- [haskell_bits](https://github.com/clintonmead/haskell_bits):
  Demonstrates the `MapExt` marker-type dispatch pattern for unifying
  by-value (`LinearFunctor`) and by-ref (`Functor`) mapping into a single
  `map` free function. Also has dual `Applicative`/`LinearApplicative` and
  `Monad`/`LinearMonad` hierarchies showing the pattern extended to the
  full monadic stack. Key insight: dispatch uses `Val`/`Ref` phantom types
  resolved by trait resolution on owned `T` vs `&T` arguments.
- Dispatch implementation: `fp-library/src/classes/dispatch.rs` and `fp-library/src/classes/dispatch/`
- RefFunctor trait: `fp-library/src/classes/ref_functor.rs`
- SendRefFunctor trait: `fp-library/src/classes/send_ref_functor.rs`
- Limitations doc: `fp-library/docs/limitations-and-workarounds.md` (section 5)
