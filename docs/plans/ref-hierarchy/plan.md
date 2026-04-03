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
pub trait MapDispatch<Brand, F, A, B, Ownership> {
    fn dispatch_map(f: F, container: Self) -> Brand::Of<B>;
}
```

**Four impls:**

1. `impl MapDispatch<..., Val> for Of<A>` where `Brand: Functor`
   -> calls `Functor::map`
2. `impl MapDispatch<..., Ref> for &Of<A>` where `Brand: RefFunctor`
   -> calls `RefFunctor::ref_map`

(The closure `Fn(A) -> B` vs `Fn(&A) -> B` distinction could be a second
axis of dispatch, but this adds complexity. Initially, by-value dispatch
uses `Fn(A) -> B` and by-ref dispatch uses `Fn(&A) -> B`. The closure
type is determined by the container ownership.)

**Unified free function:**

```rust
pub fn map<Brand, F, A, B, Ownership>(
    f: F,
    container: impl MapDispatch<Brand, F, A, B, Ownership>,
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

## Open Question

**Does the dispatch actually compile?** The `MapExt` pattern from
haskell_bits relies on Rust's trait resolution picking the right impl
based on owned vs reference arguments. This needs a proof-of-concept
to verify it works with the fp-library's GAT-based Kind system and
macro-generated trait names. Potential issues:

- Type inference may fail to select the right impl.
- Coherence rules may reject overlapping impls.
- The `Apply!` macro expansion may not work inside dispatch trait bounds.

This is the first implementation step. If dispatch doesn't compile,
the by-ref traits are still valuable on their own with separate free
functions (`ref_map`, `ref_bind`, etc.).

## Implementation Order

1. **Proof of concept**: Implement `MapDispatch` with `Functor`
   and `RefFunctor` routing. Verify compilation and type inference.
2. **RefPointed, RefLift, RefSemiapplicative, RefSemimonad**:
   Add the by-ref traits with Lazy implementations.
3. **Blanket traits**: `RefApplicative`, `RefMonad`.
4. **Unified dispatch for bind, apply, lift2**: Extend the
   dispatch pattern.
5. **SendRef variants**: Mirror for `ArcLazy`.
6. **Documentation and tests**: Property tests for type class
   laws, doc examples, update limitations.md.
7. **m_do! integration**: Make do-notation work with dispatch.

## References

- [haskell_bits](https://github.com/clintonmead/haskell_bits):
  Demonstrates the `MapExt` marker-type dispatch pattern for unifying
  by-value (`LinearFunctor`) and by-ref (`Functor`) mapping into a single
  `map` free function. Also has dual `Applicative`/`LinearApplicative` and
  `Monad`/`LinearMonad` hierarchies showing the pattern extended to the
  full monadic stack. Key insight: dispatch uses `Val`/`Ref` phantom types
  resolved by trait resolution on owned `T` vs `&T` arguments.
- Current RefFunctor: `fp-library/src/classes/ref_functor.rs`
- Current SendRefFunctor: `fp-library/src/classes/send_ref_functor.rs`
- Limitations doc: `fp-library/docs/limitations-and-workarounds.md` (section 5)
