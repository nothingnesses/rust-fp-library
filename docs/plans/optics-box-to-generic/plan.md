# Optics: Replace hard-coded `Box<dyn Fn>` with generic pointer abstraction

## Problem

Several optics types use `Box<dyn Fn(A) -> B + 'a>` directly in struct
fields and trait method signatures. This is inconsistent with the rest
of the optics system (Lens, Prism, Iso, Traversal, Fold, etc.), which
uses `FnBrand<PointerBrand>` to abstract over the pointer type.

The hard-coded `Box` means:

- **Not cloneable.** `Box<dyn Fn>` does not implement `Clone`, so these
  closures cannot be shared. The rest of the optics system uses
  `Rc<dyn Fn>` or `Arc<dyn Fn>` via `FnBrand<P>`, which are cloneable.
- **Not thread-safe.** Cannot use `Arc` for concurrent access.
- **Inconsistent API.** Setter's `new` constructor requires the caller
  to accept `Box<dyn Fn(A) -> B>` in their closure signature, leaking
  the pointer choice into the public API.

## Affected files

### Setter (struct field uses `Box<dyn Fn>`)

- `fp-library/src/types/optics/setter.rs`
  - Line 39: `over_fn` field contains `Box<dyn Fn(A) -> B + 'a>` inside
    the tuple passed to `FnBrand`.
  - Line 117: `new` constructor takes `impl Fn((S, Box<dyn Fn(A) -> B + 'a>)) -> T`.
  - Line 147: `over` method boxes the user closure with `Box::new(f)`.
  - Line 201: `Optic::evaluate` creates `Box::new(move |a| pab_clone(a))`.
  - Lines 276, 346: Same pattern in `SetterPrime`.

### Indexed optics (trait methods use `Box<dyn Fn>`)

These types store a generic `F: SomeFunc<...>` (no `Box` in the struct),
but the `SomeFunc` trait methods take `Box<dyn Fn>` parameters:

- `fp-library/src/types/optics/indexed_fold.rs`
  - Line 70: `IndexedFoldFunc::apply` takes `Box<dyn Fn(I, A) -> R + 'a>`.
  - Line 267: `Folded::apply` impl.

- `fp-library/src/types/optics/indexed_setter.rs`
  - Line 61: `IndexedSetterFunc::apply` takes `Box<dyn Fn(I, A) -> B + 'a>`.
  - Line 120: `Mapped::apply` impl.

- `fp-library/src/types/optics/indexed_traversal.rs`
  - Line 104: `Traversed::apply` impl takes `Box<dyn Fn(I, A) -> ... + 'a>`.

- `fp-library/src/classes/optics/indexed_traversal.rs`
  - Line 79: `IndexedTraversalFunc::apply` trait definition.

## Not affected

The following are already generic over `FnBrand<PointerBrand>`:

- Exchange, Market, Shop, Forget, Grating (profunctor types)
- Lens, LensPrime, Prism, PrismPrime, Iso, IsoPrime
- Traversal, TraversalPrime, Fold, FoldPrime
- AffineTraversal, AffineTraversalPrime
- Getter, GetterPrime, Review, ReviewPrime
- Grate, GratePrime
- IndexedLens, IndexedLensPrime

## Design

### Setter

The current Setter stores its function as:

```
FnBrand<P>::Of<'a, (S, Box<dyn Fn(A) -> B + 'a>), T>
```

This should become:

```
FnBrand<P>::Of<'a, (S, FnBrand<P>::Of<'a, A, B>), T>
```

The `over_fn` closure receives a cloneable `FnBrand<P>::Of<'a, A, B>`
instead of a `Box<dyn Fn(A) -> B>`. The `over` method wraps the user's
`impl Fn` into `FnBrand<P>` via `LiftFn::new` instead of `Box::new`.
The `Optic::evaluate` impl wraps `pab` (already an `FnBrand<Q>::Of`)
directly instead of re-boxing it.

This aligns Setter with how Lens stores its "put-back" function:

```
FnBrand<P>::Of<'a, S, (A, FnBrand<P>::Of<'a, B, T>)>
```

### Indexed optics

The `IndexedFoldFunc`, `IndexedSetterFunc`, and `IndexedTraversalFunc`
traits currently take `Box<dyn Fn(I, A) -> R>` in their `apply` methods.

These should take a generic closure parameter instead. Two approaches:

**Option A: `impl Fn` parameter.** Change `Box<dyn Fn(I, A) -> R + 'a>`
to `impl Fn(I, A) -> R + 'a`. This is the simplest change and avoids
allocation entirely, but `impl Fn` in trait methods requires the trait
to use `impl Trait` in trait position (RPITIT), which may affect object
safety.

**Option B: `FnBrand<P>::Of` parameter.** Add a `PointerBrand` parameter
to the traits and use `FnBrand<P>::Of<'a, (I, A), R>`. This is
consistent with the Setter approach but requires a two-argument encoding
since `FnBrand` wraps binary functions.

**Recommendation:** Option A where possible (simpler, zero-cost), falling
back to Option B if object safety or cloneability is needed. The indexed
func traits are not used as trait objects in the current codebase, so
Option A should work.

## Steps

1. **Setter / SetterPrime**: Replace `Box<dyn Fn(A) -> B + 'a>` with
   `<FnBrand<PointerBrand> as CloneFn>::Of<'a, A, B>` in the struct
   field, constructor, `over` method, and `Optic::evaluate` impl.
   Update doc examples.

2. **IndexedFoldFunc**: Change `apply` signature from
   `Box<dyn Fn(I, A) -> R + 'a>` to `impl Fn(I, A) -> R + 'a`.
   Update `Folded` impl accordingly.

3. **IndexedSetterFunc**: Same change as IndexedFoldFunc.
   Update `Mapped` impl accordingly.

4. **IndexedTraversalFunc**: Same change as IndexedFoldFunc.
   Update `Traversed` impl and the trait definition in
   `classes/optics/indexed_traversal.rs`.

5. **Verify**: Run `just verify`. Ensure all optics tests, doc tests,
   and compile-fail tests pass.

6. **Update AGENTS.md**: Remove the outdated claim about optics
   hard-coding `Rc`. The actual issue was `Box`, and after this
   refactoring it will be resolved.

## Risk

- **API breaking change.** Setter's `new` constructor signature changes:
  callers currently write `|(s, f): (S, Box<dyn Fn(A) -> B>)|` and will
  need to write `|(s, f): (S, Rc<dyn Fn(A) -> B>)|` (or the
  `CloneFn::Of` type). This is a minor version bump.
- **Indexed func traits** are public, so changing `Box` to `impl Fn` in
  their method signatures is also API-breaking if anyone implements
  them externally.
