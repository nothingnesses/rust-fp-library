# Analysis: CloneFn, Arrow, and ClosureMode

This document analyzes the refactoring of CloneableFn -> CloneFn,
SendCloneableFn -> SendCloneFn, Function -> Arrow, and the introduction
of the ClosureMode dispatch trait. It covers design correctness,
consistency, and potential improvements.

## 1. ClosureMode Trait Design

### Current Design

```rust
trait ClosureMode {
    type Target<'a, A: 'a, B: 'a>: ?Sized + 'a;
    type SendTarget<'a, A: 'a, B: 'a>: ?Sized + 'a;
}
```

Two impls map `Val -> dyn Fn(A) -> B` and `Ref -> dyn Fn(&A) -> B`,
with `SendTarget` adding `+ Send + Sync`.

### Assessment: Mostly Sound, One Redundancy

The dual GATs (`Target` and `SendTarget`) exist because `CloneFn` needs
`dyn Fn(A) -> B` while `SendCloneFn` needs `dyn Fn(A) -> B + Send + Sync`.
This is the right approach given Rust's trait system; there is no way to
conditionally add auto-trait bounds to a GAT.

However, the `SendTarget` GAT is only used in one place:
`SendCloneFn::Of`'s `Deref` bound. An alternative design would have
`SendCloneFn` define its own `Deref` target inline rather than routing
through `ClosureMode::SendTarget`. This would remove `SendTarget` from
`ClosureMode`, making the trait simpler and more cohesive.

**Trade-off:** The current design keeps `ClosureMode` as the single
source of truth for all mode-dependent type mappings. If a third
dimension emerged (e.g., `FnMut`), having both `Target` and `SendTarget`
on the same trait keeps changes centralized. This is a reasonable
design decision given the current scope, but worth revisiting if
`ClosureMode` never gains additional impls beyond `Val` and `Ref`.

### Potential Unification

One could imagine a `ClosureMode` that is itself parameterized by
a threading marker:

```rust
trait ClosureMode {
    type Target<'a, A: 'a, B: 'a, Thread: ThreadMode>: ?Sized + 'a;
}
```

This would unify `Target` and `SendTarget` into one GAT. However,
Rust does not support conditionally adding auto-trait bounds via
type parameters (you cannot write `dyn Fn(A) -> B + Thread::Bounds`),
so this is not feasible today. The current two-GAT approach is the
pragmatic choice.

## 2. LiftFn Separation

### Current Design

`LiftFn: CloneFn<Val>` provides `fn new(f: impl Fn(A) -> B)`. It
is separated from `CloneFn` because the closure parameter type
depends on the mode (`Fn(A) -> B` for Val vs `Fn(&A) -> B` for Ref),
and a trait method has one fixed signature.

### Assessment: Correct and Necessary

The separation is well-motivated. A `CloneFn<Mode>` trait cannot have
a `new` method because the method's parameter type depends on `Mode`,
and Rust trait methods cannot have mode-dependent signatures without
specialization or GAT-level dispatch.

### Issue: No LiftRefFn Trait

The `LiftFn` trait covers Val-mode construction. For Ref-mode, the
documentation in `clone_fn.rs` mentions `coerce_ref_fn` as the
construction mechanism, but this function does not exist in the
codebase. The `coerce_ref_fn` was mentioned in plan step 5 as having
been added to `UnsizedCoercible`, but the `UnsizedCoercible` trait
only has `coerce_fn` (which takes `impl Fn(A) -> B`, not
`impl Fn(&A) -> B`).

In practice, Ref-mode `CloneFn<Ref>` values are constructed manually
using raw pointer construction:

```rust
Rc::new(|x: &i32| *x * 2) as Rc<dyn Fn(&i32) -> i32>
```

This works but has several drawbacks:

1. **No generic construction.** There is no way to generically
   construct a `CloneFn<Ref>::Of<'a, A, B>` from an
   `impl Fn(&A) -> B`. The caller must know the concrete pointer
   type (Rc vs Arc) and perform the cast manually.

2. **Inconsistency with Val mode.** Val mode has `LiftFn::new`
   which hides the pointer type behind the brand. Ref mode exposes
   the pointer type at every call site.

3. **Stale documentation.** The `LiftFn` doc comment references
   `coerce_ref_fn` as if it exists, but it does not.

**Recommendation:** Either add a `LiftRefFn: CloneFn<Ref>` trait
with `fn new_ref(f: impl Fn(&A) -> B)`, or add `coerce_ref_fn` to
`UnsizedCoercible` as the plan originally stated. The trait approach
is more consistent with the existing pattern.

## 3. SendCloneFn Independence

### Current Design

`SendCloneFn<Mode>` is fully independent of `CloneFn<Mode>`. It has
its own `Of` associated type (renamed from `SendOf`). Both traits
are implemented separately for `FnBrand<P>`.

### Assessment: Trade-offs

**Advantages:**

- Types can implement `SendCloneFn` without implementing `CloneFn`.
  This is correct for types where the non-Send variant is meaningless
  (though no such type currently exists in the codebase).
- Simpler trait bounds in generic code that only needs Send capabilities.

**Disadvantages:**

- **Implementation duplication.** `FnBrand<P>` must implement both
  `CloneFn` and `CloneFn<Ref>` (2 impls) AND `SendCloneFn` and
  `SendCloneFn<Ref>` (2 more impls), for a total of 4 impls that
  are structurally very similar. With a supertrait relationship,
  `SendCloneFn` could have inherited the `Of` type from `CloneFn`
  and only added `Send + Sync` bounds.

- **No relationship between the types.** Given `Brand: SendCloneFn`,
  you cannot obtain a `CloneFn::Of` value from a `SendCloneFn::Of`
  value, even though `Arc<dyn Fn(A) -> B + Send + Sync>` trivially
  dereferences to the same trait object as `Rc<dyn Fn(A) -> B>`.
  This means generic code that accepts `CloneFn` cannot be called
  with a `SendCloneFn::Of` value without additional bounds.

- **Parallel trait hierarchies.** `Semiapplicative` uses `CloneFn`,
  `SendRefSemiapplicative` uses `SendCloneFn<Ref>`. There is no
  blanket that derives one from the other, so implementations must
  be written twice.

**Current mitigation:** In practice, the only implementor is
`FnBrand<P>`, and `P: SendUnsizedCoercible` implies
`P: UnsizedCoercible`, so both impls are always available. The
independence costs nothing in the current codebase.

**Risk:** If a second `CloneFn` implementor appears that is Send-capable,
the independence forces writing four impls instead of two.

## 4. Arrow Naming

### Current Design

The `Function` trait was renamed to `Arrow` with supertraits
`Category + Strong`. It provides:

- `type Of<'a, A, B>: Deref<Target = dyn Fn(A) -> B>`
- `fn arrow(f: impl Fn(A) -> B) -> Self::Of<A, B>`

### Assessment: Partially Appropriate

In Haskell, `Arrow` is defined as:

```haskell
class Category a => Arrow a where
    arr :: (b -> c) -> a b c
    first :: a b c -> a (b, d) (c, d)
    second :: a b c -> a (d, b) (d, c)
    (***) :: a b c -> a b' c' -> a (b, b') (c, c')
    (&&&) :: a b c -> a b c' -> a b (c, c')
```

The library's `Arrow` trait has `Category + Strong` supertraits,
where `Strong` provides `first`. This is a reasonable mapping:
`arr` corresponds to `Arrow::arrow`, `first` comes from `Strong`,
and `second` is derivable from `Strong + Profunctor`.

However, the library's `Arrow` also inherits from `Profunctor`
(via `Strong: Profunctor`), which means it has `dimap`. In Haskell,
`Arrow` does not require `Profunctor`, though every `Arrow` gives
rise to a `Profunctor` instance. This is a minor conceptual
enrichment, not a problem.

**Missing operations:** Haskell's `Arrow` includes `(***)` and
`(&&&)` which combine two arrows in parallel. The library does not
provide these, though they could be derived from `first`, `second`,
and composition. This is a feature gap, not a design flaw.

**Name appropriateness:** The name `Arrow` is appropriate given the
trait's capabilities. It provides `arr` (lifting pure functions into
arrows), composition (from `Category`), and `first`/`second` (from
`Strong`). The documentation correctly notes the alignment with
Haskell's `Arrow` type class.

### Observation: Arrow vs CloneFn Overlap

`Arrow::Of` and `CloneFn<Val>::Of` resolve to the same concrete type
for `FnBrand<P>`:

```rust
// Arrow::Of for FnBrand<P>
type Of<'a, A, B> = Apply!(<Self as Kind!(...)>::Of<'a, A, B>);
// = P::CloneableOf<'a, dyn 'a + Fn(A) -> B>

// CloneFn<Val>::Of for FnBrand<P>
type Of<'a, A, B> = Apply!(<Self as Kind!(...)>::Of<'a, A, B>);
// = P::CloneableOf<'a, dyn 'a + Fn(A) -> B>
```

Both resolve to the same `Apply!` macro expansion. They are the same
type at the concrete level, but are distinct associated types at the
trait level. This means you cannot use an `Arrow::Of` where a
`CloneFn::Of` is expected (or vice versa) without the compiler knowing
the concrete type.

This is correct behavior for the current design: `Arrow` is used in
the optics system for composable functions, while `CloneFn` is used
in the applicative hierarchy for cloneable functions. The concerns
are orthogonal even though the concrete representation is the same.

However, it means that `Arrow::Of` lacks a `Clone` bound (it only
requires `Deref`), while `CloneFn::Of` has `Clone`. If code needs
both composition and cloneability, it must require both
`Arrow + CloneFn` as bounds, even though the concrete type satisfies
both. This is a minor ergonomic cost.

## 5. FnBrand CloneFn<Ref> Implementation Correctness

### Current Implementation

```rust
impl<P: UnsizedCoercible> CloneFn<Ref> for FnBrand<P> {
    type Of<'a, A: 'a, B: 'a> = P::CloneableOf<'a, dyn 'a + Fn(&A) -> B>;
    type PointerBrand = P;
}
```

### Assessment: Correct

The `Of` type is `P::CloneableOf<'a, dyn 'a + Fn(&A) -> B>`, which
for `RcBrand` becomes `Rc<dyn 'a + Fn(&A) -> B>`. This type:

- Implements `Clone` (Rc is Clone).
- Implements `Deref<Target = dyn 'a + Fn(&A) -> B>`.
- The target matches `Ref::Target<'a, A, B> = dyn 'a + Fn(&A) -> B`.

So the `CloneFn<Ref>` trait bounds are satisfied.

### Construction Issue

While the type is correctly defined, there is no trait-based
construction path. As discussed in section 2, users must manually
construct `Rc::new(|x: &A| ...) as Rc<dyn Fn(&A) -> B>`. This is
verbose and pointer-type-aware, defeating the abstraction that
brands provide.

The `UnsizedCoercible::coerce_fn` method takes `impl Fn(A) -> B`
(by-value), not `impl Fn(&A) -> B` (by-ref). There is no
`coerce_ref_fn` despite the plan stating one was added.

## 6. Ref-Mode Construction Gap

### Problem

There is no generic way to construct a `CloneFn<Ref>::Of<'a, A, B>`
value. The current workarounds are:

1. **Manual Rc/Arc construction.** The doc examples all use:

   ```rust
   Rc::new(|x: &i32| *x * 2) as Rc<dyn Fn(&i32) -> i32>
   ```

   This requires knowing the concrete pointer type.

2. **No free function.** There is no `lift_ref_fn_new::<Brand, _, _>`
   equivalent for Ref mode.

### Recommended Solutions

**Option A: Add `coerce_ref_fn` to `UnsizedCoercible`**

```rust
trait UnsizedCoercible: RefCountedPointer + 'static {
    fn coerce_fn<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ...;
    fn coerce_ref_fn<'a, A, B>(f: impl 'a + Fn(&A) -> B)
        -> Self::CloneableOf<'a, dyn 'a + Fn(&A) -> B>;
}
```

This is the simplest fix and was the original plan.

**Option B: Add `LiftRefFn: CloneFn<Ref>` trait**

```rust
trait LiftRefFn: CloneFn<Ref> {
    fn new_ref<'a, A: 'a, B: 'a>(
        f: impl 'a + Fn(&A) -> B
    ) -> <Self as CloneFn<Ref>>::Of<'a, A, B>;
}
```

This is more consistent with `LiftFn: CloneFn<Val>`.

**Option C: Parameterize LiftFn by mode**

```rust
// Not feasible: Fn(A) -> B and Fn(&A) -> B have different
// impl Fn signatures, so the method cannot be generic over mode.
```

Option B is the cleanest. It mirrors the `LiftFn`/`SendLiftFn`
pattern and would have corresponding `SendLiftRefFn: SendCloneFn<Ref>`.

## 7. Coherence Between CloneFn<Val> and CloneFn<Ref>

### Assessment: No Issues

`CloneFn<Val>` and `CloneFn<Ref>` are distinct trait instantiations
due to the different type parameters. Rust's coherence rules treat
`CloneFn<Val>` and `CloneFn<Ref>` as completely separate traits for
impl purposes. Both can be implemented for the same type without
conflict.

The `FnBrand<P>` implementation demonstrates this:

```rust
impl<P: UnsizedCoercible> CloneFn for FnBrand<P> { ... }      // CloneFn<Val>
impl<P: UnsizedCoercible> CloneFn<Ref> for FnBrand<P> { ... }  // CloneFn<Ref>
```

These produce different `Of` types:

- `CloneFn<Val>::Of<'a, A, B>` = `P::CloneableOf<'a, dyn Fn(A) -> B>`
- `CloneFn<Ref>::Of<'a, A, B>` = `P::CloneableOf<'a, dyn Fn(&A) -> B>`

There is no ambiguity because `Val` and `Ref` are distinct types.

### Subtle Point: PointerBrand Duplication

Both `CloneFn<Val>` and `CloneFn<Ref>` define `type PointerBrand = P`.
This is redundant; the pointer brand is mode-independent. If
`PointerBrand` were on a shared supertrait (e.g., a hypothetical
`CloneFnBase`), it would only need to be defined once. As it stands,
it must be identical across all mode impls. Divergence would be a
semantic error, but the type system does not prevent it.

## 8. PointerBrand Usage

### Current Usage

`PointerBrand` on `CloneFn` is used in the optics system:

```rust
type Ptr<FunctionBrand> = <FunctionBrand as CloneFn>::PointerBrand;
```

This appears in `bazaar.rs` to extract the pointer type from a
function brand. The optics system uses it to access the underlying
`RefCountedPointer` capabilities (e.g., for constructing wrapped
functions in lens/prism implementations).

### Assessment: Correctly Used

The usage pattern is sound. The optics code needs to know the pointer
brand to construct new wrapped functions during optic composition.
Since `CloneFn` is the trait that wraps functions in reference-counted
pointers, it is the natural place for this associated type.

However, `CloneFn<Ref>::PointerBrand` is also defined and must match
`CloneFn<Val>::PointerBrand`. The optics code exclusively uses
`CloneFn` (defaulting to `Val`) for `PointerBrand` access. If code
ever needs `PointerBrand` from a `CloneFn<Ref>` bound, it will work
correctly, but the duplication is still a latent inconsistency risk.

## 9. SendCloneFn::Of Missing 'a Bound

### Observation

`CloneFn::Of` has the bound `'a + Clone + Deref<Target = ...>`,
meaning the wrapper type itself must outlive `'a`. `SendCloneFn::Of`
has `Clone + Send + Sync + Deref<Target = ...>` but is missing the
`'a` bound.

Similarly, `Arrow::Of` has `Deref<Target = dyn 'a + Fn(A) -> B>`
but no `'a` bound on the associated type itself, and no `Clone`
bound.

### Impact

For the concrete implementor `FnBrand<P>`, this is irrelevant:
`P::SendOf<'a, dyn 'a + Fn(A) -> B + Send + Sync>` inherently
satisfies `'a` because the `dyn` trait object inside is `'a`-bounded.
But in generic code, a caller with `Brand: SendCloneFn` cannot
assume `Brand::Of<'a, A, B>: 'a` without an explicit where clause.

For `Arrow::Of`, the missing `Clone` bound means that generic code
with `Brand: Arrow` cannot clone arrow values. It must add
`+ CloneFn` to get cloneability. This is intentional (Arrow focuses
on composition, not cloning), but it means Arrow values in generic
contexts are limited to single use unless the concrete type is known.

**Recommendation:** Add `'a` to `SendCloneFn::Of` bounds for
consistency with `CloneFn::Of`:

```rust
type Of<'a, A: 'a, B: 'a>: 'a + Clone + Send + Sync
    + Deref<Target = Mode::SendTarget<'a, A, B>>;
```

## 10. Documentation Inaccuracy

The `LiftFn` trait doc comment states:

> By-reference mode (`CloneFn<Ref>`) uses `coerce_ref_fn` for
> construction instead.

This function does not exist. The doc should either:

- Reference the manual construction approach
  (`Rc::new(|x: &A| ...) as Rc<dyn Fn(&A) -> B>`), or
- Be updated after a `LiftRefFn` or `coerce_ref_fn` is added.

## Summary of Findings

### Design Strengths

1. **ClosureMode parameterization is clean.** The GAT-based approach
   avoids trait duplication while preserving type safety.
2. **LiftFn separation is necessary.** The mode-dependent constructor
   signature requires a separate trait.
3. **No coherence issues.** `CloneFn<Val>` and `CloneFn<Ref>` coexist
   without conflict.
4. **Arrow naming is appropriate.** The supertraits (`Category + Strong`)
   match Haskell's Arrow class structure.
5. **PointerBrand is correctly used** in the optics system for
   pointer-type extraction.

### Issues to Address

| Issue                                                                           | Severity        | Section |
| ------------------------------------------------------------------------------- | --------------- | ------- |
| No generic Ref-mode construction (missing `LiftRefFn` or `coerce_ref_fn`).      | Medium          | 2, 6    |
| `SendCloneFn::Of` missing `'a` lifetime bound.                                  | Low             | 9       |
| `LiftFn` doc references nonexistent `coerce_ref_fn`.                            | Low             | 10      |
| `PointerBrand` duplicated across mode impls with no enforcement of consistency. | Low             | 7, 8    |
| `SendCloneFn` independence causes 4 impls instead of 2 for `FnBrand<P>`.        | Low (trade-off) | 3       |
| Arrow::Of lacks Clone bound, limiting generic usage.                            | Informational   | 4       |
