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

### Phase 4: Other Types

Consider which other types could implement the by-ref hierarchy:

- **Vec, Option, Result**: Could implement `RefFunctor` (iterate by
  reference) but the by-value path is almost always preferred. Low
  priority.
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

5. **Skip RefFoldable and RefTraversable initially.** `Lazy` already
   implements `Foldable` in the by-value hierarchy. Adding by-ref variants
   doesn't unlock new generic programming capabilities the way `RefMonad`
   does. Revisit if a concrete use case emerges.

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
6. **Add `CloneableFn<Ref>` impl for `FnBrand<P>`**. Implement
   using `coerce_ref_fn`. Apply same `ClosureMode` parameterization
   to `SendCloneableFn` and split its `new` into `SendLiftFn`.
7. **RefSemiapplicative**: Define using `CloneableFn<Ref>`. Implement
   for `LazyBrand<RcLazyConfig>`.
8. **Blanket traits**: `RefApplicative = RefPointed + RefSemiapplicative`,
   `RefMonad = RefApplicative + RefSemimonad`.
9. **SendRef variants**: `SendRefPointed`, `SendRefLift`,
   `SendRefSemimonad`, `SendRefSemiapplicative` with `ArcLazy`
   implementations. Follows the same pattern as existing
   `SendRefFunctor` (adds `Send + Sync` bounds on closures and elements).
   Uses `SendCloneableFn<Ref>` for `SendRefSemiapplicative`.
10. **SendRef blanket traits**: `SendRefApplicative`, `SendRefMonad`.
11. **Ref parity traits**: Add by-ref equivalents for all convenience
    traits and free functions that exist on the non-ref hierarchy:
    - `RefApplyFirst`, `RefApplySecond` (or default methods on
      `RefApplicative`). Require `Clone` on the kept value since the
      closure receives references.
    - `ref_monad_if`, `ref_monad_unless` free functions.
    - Any other convenience methods added to non-ref traits in the
      future should also get ref equivalents.
      The same applies to SendRef variants if/when they are implemented:
      `SendRefApplyFirst`, `SendRefApplySecond`, `send_ref_monad_if`,
      `send_ref_monad_unless`, etc.
12. **Rename traits**: Extract `Callable<Mode>` base trait from the
    shared `Deref` bound. Rename `CloneableFn` to `CloneableCallable`.
    Rename `Function` to `Arrow`. Rename `SendCloneableFn` to
    `SendCloneableCallable`. Update all references across the
    codebase. This is a mechanical rename done after the structural
    changes are verified.
13. **Documentation and tests**: Property tests for type class
    laws, doc examples, update limitations.md.
14. **m_do! integration**: Add `ref` qualifier to `m_do!` so it
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

### Planned trait renames (after verification)

Once the ClosureMode approach is verified, the function wrapper
traits will be renamed to better reflect their roles:

| Pre-rename name   | Post-rename name | Role                                             |
| ----------------- | ---------------- | ------------------------------------------------ |
| `Callable<Mode>`  | `Function<Mode>` | Base: wraps a closure, callable via Deref        |
| `CloneableFn`     | `CloneFn`        | Base + Clone. Used by Semiapplicative.           |
| `LiftFn`          | `LiftFn`         | Construction for Val mode.                       |
| `Function`        | `Arrow`          | Base (Val only) + Category + Strong. For optics. |
| `SendCloneableFn` | `SendCloneFn`    | Send variant of CloneFn.                         |
| `SendLiftFn`      | `SendLiftFn`     | Send variant of LiftFn.                          |

Post-rename hierarchy:

- `Function<Mode>` is the base callable wrapper (reclaims the name
  for the more general concept).
- `Arrow: Function<Val> + Category + Strong` is the composable
  specialization (aligns with Haskell's `Arrow` type class:
  `arr` + `Category` + `first`/`second`).
- `CloneFn<Mode>: Function<Mode>` adds `Clone`.
- `LiftFn: CloneFn<Val>` adds construction.

The renames are deferred to Step 10 (after structural changes are
verified) because they are mechanical and independent of the
ClosureMode design. If the ClosureMode approach fails, the rename
scope may differ.

## Open Questions

None at this time. The `ClosureMode` unsized GAT + `Deref` interaction
was verified in Step 5 and works on stable Rust.

## Completed Changes

- `RefFunctor` and `SendRefFunctor` closures changed from `FnOnce` to `Fn`.
- `FunctorDispatch` Ref impl updated to match.
- `RefPointed` trait and free function `ref_pure` added.
- `RefLift` trait and free function `ref_lift2` added.
- `RefSemimonad` trait and free function `ref_bind` added.
- All three implemented for `LazyBrand<RcLazyConfig>` with doc examples.
- All Lazy/TryLazy trait impls updated for `Fn` closure signatures.
- `ClosureMode` trait added with `Val`/`Ref` impls.
- `CloneableFn` parameterized with `Mode: ClosureMode = Val`.
- `Function` supertrait removed from `CloneableFn`.
- `CloneableFn::new` split into `LiftFn: CloneableFn<Val>`.
- Free function renamed from `cloneable_fn_new` to `lift_fn_new`.
- `coerce_ref_fn` added to `UnsizedCoercible` with `RcBrand`/`ArcBrand` impls.
- Many explicit class imports converted to wildcards.
- All downstream bounds updated (`CloneableFn` -> `LiftFn` where `new` is called).

## References

- [haskell_bits](https://github.com/clintonmead/haskell_bits):
  Demonstrates the `MapExt` marker-type dispatch pattern for unifying
  by-value (`LinearFunctor`) and by-ref (`Functor`) mapping into a single
  `map` free function. Also has dual `Applicative`/`LinearApplicative` and
  `Monad`/`LinearMonad` hierarchies showing the pattern extended to the
  full monadic stack. Key insight: dispatch uses `Val`/`Ref` phantom types
  resolved by trait resolution on owned `T` vs `&T` arguments.
- Dispatch implementation: `fp-library/src/classes/functor_dispatch.rs`
- RefFunctor trait: `fp-library/src/classes/ref_functor.rs`
- SendRefFunctor trait: `fp-library/src/classes/send_ref_functor.rs`
- Limitations doc: `fp-library/docs/limitations-and-workarounds.md` (section 5)
