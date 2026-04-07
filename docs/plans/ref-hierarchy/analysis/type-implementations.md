# Type Implementation Analysis: Ref/SendRef/ParRef Traits

This document analyzes how the Ref, SendRef, and ParRef trait families are
implemented across the library's concrete types, identifying design flaws,
inconsistencies, limitations, and potential improvements.

## 1. Collection Type Consistency (Vec, Option, CatList, Identity)

### 1.1 Trait coverage matrix

| Trait                     | Vec | Option | CatList | Identity |
| ------------------------- | --- | ------ | ------- | -------- |
| RefFunctor                | Yes | Yes    | Yes     | Yes      |
| RefFoldable               | Yes | Yes    | Yes     | Yes      |
| RefTraversable            | Yes | Yes    | Yes     | Yes      |
| RefFilterable             | Yes | Yes    | Yes     | No       |
| RefWitherable             | Yes | Yes    | Yes     | No       |
| RefFunctorWithIndex       | Yes | Yes    | Yes     | Yes      |
| RefFoldableWithIndex      | Yes | Yes    | Yes     | Yes      |
| RefFilterableWithIndex    | Yes | Yes    | Yes     | No       |
| RefTraversableWithIndex   | Yes | Yes    | Yes     | Yes      |
| RefPointed                | Yes | Yes    | Yes     | Yes      |
| RefLift                   | Yes | Yes    | Yes     | Yes      |
| RefSemiapplicative        | Yes | Yes    | Yes     | Yes      |
| RefSemimonad              | Yes | Yes    | Yes     | Yes      |
| RefApplicative (blanket)  | Yes | Yes    | Yes     | Yes      |
| RefMonad (blanket)        | Yes | Yes    | Yes     | Yes      |
| ParRefFunctor             | Yes | No     | Yes     | No       |
| ParRefFoldable            | Yes | No     | Yes     | No       |
| ParRefFilterable          | Yes | No     | Yes     | No       |
| ParRefFunctorWithIndex    | Yes | No     | Yes     | No       |
| ParRefFoldableWithIndex   | Yes | No     | Yes     | No       |
| ParRefFilterableWithIndex | Yes | No     | Yes     | No       |

**Assessment:** The coverage is consistent and well-reasoned.

- Identity correctly lacks RefFilterable/RefWitherable since it does not
  implement Compactable (there is no meaningful "empty" for a single value).
- Option and Identity correctly lack ParRef traits; parallelism over a
  single element is pointless.
- Vec and CatList have full parity across all Ref and ParRef traits.

**No inconsistencies found.** The trait coverage follows the established
design principle: a type implements a Ref trait if and only if it also
implements the corresponding by-value trait. Compound Ref traits (e.g.,
RefWitherable) use the non-Ref version of structural supertraits
(Compactable) combined with the Ref version of element-accessing supertraits
(RefFilterable + RefTraversable).

### 1.2 Missing SendRef traits for collections

No collection type implements SendRef variants (SendRefFunctor,
SendRefPointed, SendRefLift, SendRefSemimonad, etc.). Only
`LazyBrand<ArcLazyConfig>` implements these.

**Is this a gap?** Somewhat, but not a pressing one. The SendRef traits
exist primarily for memoized types that need `Send + Sync` bounds on their
closures due to the Arc-based interior mutability. Collection types do not
have this constraint; their Ref implementations work correctly in
multi-threaded contexts as long as the collection itself is `Send + Sync`
(which `Vec<T>` is whenever `T: Send + Sync`). The ParRef traits serve the
specific parallel-iteration use case that collections actually need.

**Recommendation:** No action needed. SendRef traits for collections would
be redundant with the existing Ref traits, since collection closures do not
require `Send + Sync` bounds for correctness.

## 2. Removing Foldable from Lazy: Correctness and Migration

### 2.1 Was the removal correct?

Yes. The previous `Foldable` impl for `LazyBrand<Config>` was semantically
dishonest: `fold_right` took `fa` by value (consuming it), but internally
called `fa.evaluate()` which borrows through shared interior mutability.
The caller was led to believe the Lazy was consumed, but it was not. The
value remained memoized and accessible through any remaining clone.

`RefFoldable` honestly expresses the semantics: it takes the container by
value (transferring ownership of the `Rc`/`Arc` handle) but passes the
element by reference via `&A`. This matches the reality of what `evaluate()`
provides.

### 2.2 Migration path

Users of `fold_map::<FnBrand, LazyBrand<...>, _, _>(|a: A| ..., lazy)` must
change to `fold_map::<FnBrand, LazyBrand<...>, _, _, _>(|a: &A| ..., lazy)`.

The key changes are:

- The closure receives `&A` instead of `A`.
- The unified `fold_map` free function dispatches automatically based on
  closure type (`|a: A|` vs `|a: &A|`), so call-site turbofish syntax gains
  one underscore for the `Ownership` marker parameter.
- Similarly for `fold_right` and `fold_left`: the accumulation function
  receives `(&A, B)` or `(B, &A)` instead of `(A, B)` or `(B, A)`.

The migration is mechanical but breaking. Users who need the owned value
can dereference or clone within the closure (`*a` for Copy types,
`a.clone()` for Clone types).

### 2.3 TryLazy follows the same pattern

`TryLazy` also had its `Foldable` and `FoldableWithIndex` impls replaced
by `RefFoldable` and `RefFoldableWithIndex`. The same migration path
applies, with the added nuance that the closure in `ref_fold_map` receives
`&A` where `A` is the success type (the `Ok` value from `evaluate()`).

### 2.4 TryThunk retains by-value Foldable

`TryThunk` (the non-memoized fallible thunk) keeps its by-value `Foldable`
implementation. This is correct: `TryThunk` evaluates by consuming the
thunk (`evaluate` takes `self`), so by-value access to the element is
natural. No `RefFoldable` is needed.

## 3. Types That Should (or Should Not) Implement Ref Traits

### 3.1 Pair, Tuple2 (bifunctorial types)

`PairFirstAppliedBrand<L>` and `PairSecondAppliedBrand<L>` have full
by-value Functor/Applicative/Monad/Foldable/Traversable impls but no Ref
variants. The same applies to `Tuple2FirstAppliedBrand<L>` and
`Tuple2SecondAppliedBrand<L>`.

**Should they have Ref impls?** Medium priority. These types are
collections with exactly one element (for the functorial position), so
Ref traits would work identically to how they work for `Identity`. The
implementation would be trivial.

**Missing from the diff:** The pair.rs.diff and tuple_2.rs.diff show only
renames (`CloneableFn` -> `CloneFn`, turbofish count changes) with no new
Ref trait impls added. This is a gap in coverage.

**Recommendation:** Add Ref trait impls for Pair and Tuple2 applied brands
for completeness and consistency. Priority is low since these types are
less commonly used in generic code than Vec/Option.

### 3.2 Result (applied brands)

`ResultErrAppliedBrand<E>` and `ResultOkAppliedBrand<OK>` have full
by-value Functor through Traversable impls but no Ref variants.

**Should they have Ref impls?** Yes, with the same priority as Pair/Tuple2.
`Result` is arguably more commonly used than Pair in generic code, and its
Ref impls would follow the exact same pattern as Option (pattern-match,
pass reference to closure).

**Recommendation:** Add Ref traits for Result applied brands. This is a
medium-priority gap.

### 3.3 Tuple1

`Tuple1Brand` has by-value Functor through Traversable but no Ref variants.
Like Identity, it wraps exactly one value.

**Recommendation:** Add Ref traits for Tuple1 for consistency. Low priority.

### 3.4 Thunk

`ThunkBrand` has by-value Functor/Applicative/Monad/Foldable/Traversable
impls. Thunks are consumed on evaluation (they do not memoize), so by-value
semantics are natural.

**Should it have Ref impls?** No. Unlike Lazy, there is no shared cached
value to reference. Evaluation consumes the thunk. If you `ref_map` a
thunk, you would need to evaluate it (consuming it), then pass a reference
to the result. But this is exactly what the by-value `map` already does;
the "ref" part adds no value because there is no persistent value to borrow
from. The same argument applies to TryThunk.

### 3.5 ConstVal, ControlFlow

`ConstVal` is a phantom functor (maps the function but ignores the value)
used as a type-level tool. `ControlFlow` adapts `std::ops::ControlFlow`.
Neither is a container in the traditional sense.

**Recommendation:** No Ref traits needed. These are structural types, not
data containers.

### 3.6 Endomorphism

`EndomorphismBrand` wraps `Fn(&A) -> A`; it is a profunctor, not a
standard functor. It does not implement Functor and should not implement
RefFunctor.

### 3.7 Coyoneda variants

See section 8.

## 4. Lazy Ref Implementations: Correctness

### 4.1 RefSemimonad for RcLazy

```rust
fn ref_bind(fa, f) {
    f(fa.evaluate())
}
```

This is correct. `fa.evaluate()` returns `&A` (a reference to the memoized
value). The closure `f` receives `&A` and returns a new `Lazy<B>`. The
original `fa` is captured by the closure passed to the new Lazy if needed,
but in this case `ref_bind` evaluates eagerly, which is the intended
semantics for Lazy bind (evaluate the input, then call the continuation).

**Potential concern:** `ref_bind` evaluates `fa` eagerly (not lazily).
This means `ref_bind(lazy_a, f)` forces `lazy_a` immediately, even if
the result is never used. This is semantically correct for a monad (bind
must sequence effects), but it differs from the by-value `Semimonad::bind`
pattern for lazy languages. The design document acknowledges this: Lazy's
ref_bind is intentionally strict because the closure needs the reference
to produce the next computation, and that reference can only come from
evaluating.

### 4.2 RefLift for RcLazy

```rust
fn ref_lift2(func, fa, fb) {
    RcLazy::new(move || func(fa.evaluate(), fb.evaluate()))
}
```

Correct. Both inputs are captured by the new Lazy closure and evaluated
on demand. The function receives references to both memoized values.

### 4.3 RefSemiapplicative for RcLazy

```rust
fn ref_apply(ff, fa) {
    RcLazy::new(move || {
        let f = ff.evaluate();
        let a = fa.evaluate();
        (**f)(a)
    })
}
```

Correct. The double-deref `(**f)` first dereferences the `CloneFn<Ref>::Of`
wrapper (an `Rc<dyn Fn(&A) -> B>`) via `Deref`, then calls it with `a`
(which is `&A` from `evaluate()`).

### 4.4 SendRef variants for ArcLazy

The SendRef implementations mirror the Ref implementations exactly, with
additional `Send + Sync` bounds on type parameters and closures. This is
correct; the only difference is the thread-safety requirement.

### 4.5 RefFoldable for Lazy

```rust
fn ref_fold_map(func, fa) {
    func(fa.evaluate())
}
```

Correct. For a single-element container, fold_map is just applying the
function to the element.

## 5. Performance Concerns

### 5.1 Vec's ref_map creates a new Vec

```rust
fn ref_map(func, fa) {
    fa.iter().map(func).collect()
}
```

This iterates over references and collects into a new `Vec<B>`. This is
the correct and expected behavior; there is no way to map `&A -> B`
in-place without allocation when `A != B`. The cost is O(n) allocation
and O(n) function calls, identical to what `.iter().map(f).collect()`
costs in idiomatic Rust.

**Not a concern.** This is inherent to the operation, not an overhead
of the abstraction.

### 5.2 RefTraversable delegates to by-value Traversable

All four collection types implement `ref_traverse` by delegating to
`Self::traverse`:

```rust
fn ref_traverse(func, ta) {
    Self::traverse::<A, B, F>(move |a: A| func(&a), ta)
}
```

This means `ref_traverse` takes the element by value (via the by-value
traverse), creates a local binding, then passes a reference to the
user's closure. This works correctly but has an implication: the element
is moved into the wrapper closure, so the "by reference" guarantee is
that the user's closure receives `&A`, not that the underlying element
remains in the original container.

**Is this a problem?** For collections that own their elements (Vec,
CatList, Option, Identity), this is fine; the elements are consumed during
iteration either way. The "ref" in RefTraversable means the user's closure
gets references, not that the container is preserved.

**However**, this delegation pattern means `ref_traverse` consumes the
container just like `traverse` does. For a type like `Lazy` where the
whole point of Ref traits is non-consumption, this delegation would be
incorrect. Lazy does not implement RefTraversable, so this is moot;
but if someone added it in the future, they would need a direct
implementation, not delegation.

### 5.3 Option's ref_bind uses `as_ref().and_then()`

```rust
fn ref_bind(fa, f) {
    fa.as_ref().and_then(f)
}
```

This borrows the Option's content via `as_ref()`, then calls `f(&a)`.
The result is `Option<B>`, which is what we want. This is zero-overhead
(no cloning, no allocation beyond what `f` produces).

### 5.4 Vec's ref_bind uses `iter().flat_map()`

```rust
fn ref_bind(fa, f) {
    fa.iter().flat_map(f).collect()
}
```

Correct and idiomatic. Each element is visited by reference, `f(&a)`
returns a `Vec<B>`, and the results are flattened. Allocation is
proportional to the output size.

### 5.5 RefFoldable for Vec: left-associative fold

```rust
fn ref_fold_map(func, fa) {
    fa.iter().fold(Monoid::empty(), |acc, a| Semigroup::append(acc, func(a)))
}
```

This is a left fold using `Semigroup::append`. For monoids like `String`
where `append` is O(n), this produces O(n^2) behavior for large vectors.
This is the same issue as the by-value `fold_map` and is inherent to the
left-fold-over-Semigroup pattern, not specific to the Ref implementation.

**Not a regression**, but worth noting: `ParRefFoldable` for Vec uses
rayon's parallel reduction, which can use tree-structured folding to
reduce the issue.

## 6. RefSemiapplicative: CloneFn<Ref> Construction

### 6.1 How are CloneFn<Ref> values constructed?

`RefSemiapplicative::ref_apply` takes `Of<CloneFn<Ref>::Of<A, B>>`, which
is a container of `Rc<dyn Fn(&A) -> B>` (for RcFnBrand) or
`Arc<dyn Fn(&A) -> B + Send + Sync>` (for ArcFnBrand).

Users construct these values manually using `Rc::new(|x: &i32| ...)` or
`Arc::new(|x: &i32| ...)`. There is no `LiftFn`-like convenience method
for the Ref mode; `LiftFn: CloneFn<Val>` only provides construction for
Val-mode closures.

**Is this a gap?** Partially. The doc examples show manual `Rc::new` /
`Arc::new` construction, which works but is verbose. A `LiftRefFn` trait
or `lift_ref_fn_new` free function could improve ergonomics:

```rust
let f = lift_ref_fn_new::<RcFnBrand, _, _>(|x: &i32| *x + 1);
```

However, `ref_apply` is rarely called directly by users; `ref_lift2` is
the more common entry point and does not require wrapped functions. The
design doc notes that `apply` is the "hard" path and `lift2` is the
"easy" path, which applies equally to their Ref variants.

**Recommendation:** Consider adding a `LiftRefFn` trait for completeness,
but this is low priority since `ref_lift2` covers most use cases.

### 6.2 Correctness of collection RefSemiapplicative impls

Vec:

```rust
fn ref_apply(ff, fa) {
    ff.iter().flat_map(|f| fa.iter().map(move |a| (**f)(a))).collect()
}
```

This iterates over the wrapped functions by reference (each `f` is
`&Rc<dyn Fn(&A) -> B>`), then iterates over the values by reference
(each `a` is `&A`). The double-deref `(**f)` unwraps the `&Rc` to get
the `dyn Fn(&A) -> B`, then calls it with `a: &A`. The Cartesian product
semantics match the by-value `apply`.

Option:

```rust
fn ref_apply(ff, fa) {
    match (ff, fa.as_ref()) {
        (Some(f), Some(a)) => Some((*f)(a)),
        _ => None,
    }
}
```

Note that `ff` is consumed (pattern-matched by value) while `fa` is
borrowed via `as_ref()`. The function `f` is `Rc<dyn Fn(&A) -> B>`,
and `(*f)(a)` calls it with `a: &A`. Correct.

CatList follows the same pattern as Vec. Identity follows the same
pattern as Option.

## 7. RefTraversable: Implementation Status

### 7.1 Was RefTraversable initially deferred?

The plan (step 5) states: "Add RefFoldable, skip RefTraversable initially."
However, step 21 later adds RefTraversable and related traits, and step 22
implements them for all collection types. So RefTraversable was deferred
and then implemented.

### 7.2 Correctness of the delegation pattern

All collection RefTraversable impls delegate to by-value Traversable:

```rust
fn ref_traverse(func, ta) {
    Self::traverse(move |a: A| func(&a), ta)
}
```

This works by converting the reference-taking closure into a value-taking
closure that immediately borrows its argument. The container is still
consumed (elements are moved into the wrapper closure), but the user's
closure sees `&A`.

**Semantic concern:** This delegation means the container is consumed even
though the trait name suggests "by reference" access. This is acceptable
for collections (they are consumed by traverse anyway), but the naming
could be misleading. The "Ref" in RefTraversable refers to how elements
are accessed (by reference), not to how the container is held.

### 7.3 Lazy does not implement RefTraversable

This is correct. `RefTraversable: RefFunctor + RefFoldable` requires both
supertraits, which Lazy implements. However, traverse produces
`F<Brand::Of<B>>`, requiring the output brand to be reconstructible inside
the effect `F`. For Lazy, this would mean producing a `Lazy<B>` inside an
`F` context, which requires a `RefPointed`-like capability that the current
`Traversable` interface does not accommodate (it uses by-value
`Applicative`, not `RefApplicative`, for the output context).

The plan correctly deferred this and no concrete use case has emerged.

## 8. Coyoneda Types: Missing Ref Trait Impls

### 8.1 Current state

The Coyoneda variants (Coyoneda, RcCoyoneda, ArcCoyoneda) implement
by-value Functor, Foldable, Pointed, Semiapplicative, Semimonad, and Lift.
No Ref trait variants were added.

### 8.2 Should they have Ref impls?

The plan (section "Other types" under Phase 4) notes:

> RcCoyoneda and ArcCoyoneda already use `lower_ref` (by-reference
> lowering). They could implement RefFunctor to map over the lowered
> result by reference. Medium priority.

**Analysis:**

- **Coyoneda** (box-based, single-use) implements by-value traits by
  lowering to the underlying functor. Adding RefFunctor would require
  lowering and then mapping by reference, which is possible but loses
  the fusion benefit (Coyoneda fuses multiple maps into one; lowering
  for ref_map defeats this purpose).

- **RcCoyoneda** and **ArcCoyoneda** (clone-based, multi-use) have
  `lower_ref(&self)` which produces the lowered value without consuming
  the Coyoneda. These are natural candidates for RefFunctor:
  ```rust
  fn ref_map(f, fa) {
      RcCoyoneda::lift(F::ref_map(f, fa.lower_ref()))
  }
  ```
  This would evaluate the accumulated maps, then apply `f` by reference.
  It loses fusion but enables by-reference access to the lowered result.

**Recommendation:** Adding RefFunctor to RcCoyoneda and ArcCoyoneda is
reasonable but should be done carefully. The fusion trade-off should be
documented. RefFoldable would also be useful (fold by reference over the
lowered result). RefSemimonad and RefSemiapplicative are less valuable
since these types are primarily used for map fusion.

## 9. SendEndofunction: Design Assessment

### 9.1 Purpose

`SendEndofunction<'a, FnBrand: SendLiftFn, A>` is the `Send + Sync`
counterpart of `Endofunction<'a, FnBrand: LiftFn, A>`. It wraps
`Arc<dyn Fn(A) -> A + Send + Sync>` and provides `Semigroup` (composition)
and `Monoid` (identity) instances.

### 9.2 Does it duplicate functionality?

Partially. `Endofunction<'a, ArcFnBrand, A>` already wraps
`Arc<dyn Fn(A) -> A + Send + Sync>` because `ArcFnBrand` produces
`Arc`-wrapped closures. However, `Endofunction` requires `FnBrand: LiftFn`,
while `SendEndofunction` requires `FnBrand: SendLiftFn`.

The key difference: `Endofunction`'s `Semigroup::append` uses
`<FnBrand as LiftFn>::new(...)` for composition, while
`SendEndofunction`'s uses `<FnBrand as SendLiftFn>::new(...)`. Since
`LiftFn` and `SendLiftFn` are independent traits (no supertrait
relationship), `Endofunction<ArcFnBrand, A>` would require `ArcFnBrand:
LiftFn`, which it does not implement (ArcFnBrand implements `SendLiftFn`).

**So `SendEndofunction` is necessary**, not duplicative. It is the only
way to get monoidal endofunction composition in `Send + Sync` contexts.

### 9.3 Design quality

The implementation is clean and mirrors `Endofunction` exactly. The
`Semigroup` impl requires `A: Send + Sync` (needed because the composed
closure captures both inner closures, which must be `Send + Sync`). The
`Monoid` impl similarly requires `A: Send + Sync` for the identity
closure.

**One concern:** `SendEndofunction` is used internally by the
`SendRefFoldable` and `SendRefFoldableWithIndex` default implementations
for `send_ref_fold_right` and `send_ref_fold_left` (to build the
accumulating function as a monoid). This makes it an infrastructure type,
not primarily a user-facing type. Its API is minimal (just `new`, `Clone`,
`Semigroup`, `Monoid`), which is appropriate.

## 10. Summary of Findings

### Correct design decisions

1. **Removing Foldable from Lazy/TryLazy:** The previous impl was
   semantically dishonest. RefFoldable correctly models the operation.

2. **Collection types implement both by-value and by-ref traits:** This
   gives users the choice without forcing one path.

3. **Identity lacks RefFilterable:** Correct, since Identity does not
   implement Compactable.

4. **Thunk/TryThunk retain by-value Foldable:** These types consume on
   evaluation, so by-value semantics are natural.

5. **SendEndofunction is not duplicative:** It fills a genuine gap where
   `Endofunction<ArcFnBrand, A>` cannot be constructed.

6. **RefTraversable delegation to Traversable:** Correct for collection
   types, avoiding code duplication.

### Gaps and recommendations

| Priority | Item                              | Description                                                                                                                                                                                         |
| -------- | --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Medium   | Result Ref traits                 | `ResultErrAppliedBrand` and `ResultOkAppliedBrand` should implement the full Ref suite (RefFunctor through RefSemimonad, RefFoldable, RefTraversable). The implementation follows Option's pattern. |
| Low      | Pair/Tuple2 Ref traits            | `PairFirstAppliedBrand`, `PairSecondAppliedBrand`, `Tuple2FirstAppliedBrand`, `Tuple2SecondAppliedBrand` should implement Ref traits for consistency. Trivial implementations.                      |
| Low      | Tuple1 Ref traits                 | `Tuple1Brand` should implement Ref traits, mirroring Identity.                                                                                                                                      |
| Low      | RcCoyoneda/ArcCoyoneda RefFunctor | These types could implement RefFunctor and RefFoldable via `lower_ref`, with documented fusion trade-offs.                                                                                          |
| Low      | LiftRefFn convenience             | A construction method for `CloneFn<Ref>::Of` values would reduce boilerplate in `ref_apply` call sites.                                                                                             |

### Potential issues

1. **RefTraversable naming ambiguity:** The name suggests the container is
   not consumed, but the delegation to by-value `traverse` does consume
   it. The "Ref" refers to element access, not container handling. This
   should be documented more clearly to avoid confusion.

2. **ref_fold_map left-associative cost:** Vec and CatList's
   `ref_fold_map` uses a left fold with `Semigroup::append`, which is
   O(n^2) for non-constant-time monoids like `String`. This matches the
   by-value `fold_map` behavior and is not a regression, but it is a known
   performance trap.

3. **RefSemimonad for Lazy is strict:** `ref_bind` evaluates the input
   immediately. This is semantically necessary (the closure needs the
   reference) but differs from what users might expect of a "lazy" type.
   The doc examples demonstrate this, but a more prominent note in the
   trait docs would help.

4. **No `ref_traverse` for Lazy:** This is correctly deferred, but if
   users need to traverse a Lazy value in an applicative context, they
   must use `ref_fold_map` or evaluate manually. This limitation should
   be documented.
