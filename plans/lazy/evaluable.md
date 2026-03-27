# Evaluable Trait Analysis

**File:** `fp-library/src/classes/evaluable.rs` (120 lines)

## 1. Design

`Evaluable` represents a functor that always contains exactly one extractable value, providing a natural transformation `F ~> Id` (line 29-30). In category theory terms, this is the **counit** of a comonad, or equivalently, `extract` from `Comonad`. The trait is well-motivated by its primary consumer: `Free::evaluate` (line 747 of `free.rs`) uses `Evaluable` to peel off one functor layer during the iterative evaluation loop.

The trait requires `Functor` as a supertrait (line 56), which is appropriate since `Evaluable` is only meaningful for type constructors that participate in the HKT system. The `Functor` bound also aligns with the fact that `Free<F, A>` requires `F: Functor` in its definition (line 114 of `free.rs`), so `Evaluable: Functor` composes cleanly.

**Is it well-motivated?** Yes, but narrowly. The trait exists almost exclusively to serve `Free::evaluate`. It has exactly one implementor (`ThunkBrand`, line 692 of `thunk.rs`) and exactly one call site beyond doc-tests (`Free::evaluate` at line 747 of `free.rs`). The documentation acknowledges this narrow scope (line 37: "Currently only ThunkBrand implements this trait").

**Relationship to Comonad:** `Evaluable::evaluate` is precisely `Comonad::extract` (a.k.a. `counit`). The library does not have a `Comonad` trait. If one were added, `Evaluable` would be a strict subset of it. For now, this is fine because `Thunk` is not a full comonad (it lacks `duplicate`/`extend`), and the name "Evaluable" better communicates the intent of "running an effect" than the more abstract "extract."

## 2. Implementation Quality

### Trait Definition (lines 56-83)

The trait definition is correct. The method signature:

```rust
fn evaluate<'a, A: 'a>(
    fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
) -> A;
```

This takes an `F<A>` and returns `A`, which is exactly the right signature for `extract`/`counit`. The lifetime parameter `'a` allows the functor to hold borrowed data, which is important for `Thunk<'a, A>`.

### ThunkBrand Implementation (lines 692-723 of `thunk.rs`)

The implementation delegates to the inherent `Thunk::evaluate` method (line 721), which simply calls the inner `FnOnce` closure (line 281 of `thunk.rs`). This is correct and minimal.

### Free Function (lines 85-116)

The free function `evaluate` correctly delegates to the trait method. The type parameter order `<'a, F, A>` is consistent with other free functions in the library (e.g., `map::<Brand, _, _>`).

### Usage in Free::evaluate (lines 716-762 of `free.rs`)

The call at line 747 (`<F as Evaluable>::evaluate(fa)`) is correct. The `Wrap` variant contains `F<Free<F, A>>`, and `Evaluable::evaluate` strips the outer `F`, yielding `Free<F, A>`, which becomes the next iteration of the loop. This is the core of the trampolining mechanism.

**Potential concern:** The `Evaluable::evaluate` call at line 747 is not itself stack-safe. If the functor `F`'s `evaluate` implementation were to recursively call back into `Free::evaluate`, it could overflow. However, `Thunk::evaluate` simply calls a closure, so this is not an issue in practice.

## 3. Laws

The documentation states one law (lines 43-55):

> **Naturality:** `evaluate(nat(fa)) == evaluate(fa)`

This law is **incorrectly stated**. As written, it says that for any natural transformation `nat: F ~> G` between two evaluable functors, `evaluate_G(nat(fa)) == evaluate_F(fa)`. This is actually the **coherence condition** for natural transformations between comonads (specifically, that `nat` preserves `extract`). It is a property of `nat`, not of `evaluate` itself.

A more accurate framing would be:

- **Identity/extraction law:** `evaluate(pure(a)) == a` (assuming `F` is also `Pointed`). This states that wrapping and immediately evaluating is the identity.
- **Map-extract law:** `evaluate(map(f, fa)) == f(evaluate(fa))`. This states that mapping before extracting is the same as extracting then applying the function. This is the standard comonad law for `extract`.

The naturality law as stated is not testable in isolation because it constrains natural transformations, not `evaluate` itself. It is also not enforced or tested anywhere in the codebase.

**Recommendation:** Replace or supplement the naturality law with the map-extract law, which is the standard `extract` law and is directly testable.

## 4. API Surface

The API surface is minimal and appropriate:

- One trait method: `evaluate`
- One free function: `evaluate`

There are no unnecessary methods. The `Functor` supertrait is the correct and minimal bound.

**Naming:** The name `evaluate` is well-chosen for this library's context, where the primary use is running deferred computations (thunks). It avoids the category-theoretic jargon of `extract`/`counit` while clearly communicating intent.

**Ergonomics concern:** The free function requires explicit brand annotation: `evaluate::<ThunkBrand, _>(thunk)`. In practice, users will call `thunk.evaluate()` (the inherent method) rather than the free function. The free function's primary utility is in generic code that abstracts over the functor, which is exactly the `Free::evaluate` use case.

## 5. Consistency with Other Type Classes

### Consistent aspects

- **Module structure:** Uses the `#[fp_macros::document_module] mod inner { ... }` pattern with `pub use inner::*`, matching all other type class modules.
- **Documentation macros:** Uses `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`, matching the project's documentation standards.
- **Free function pattern:** Provides a free function alongside the trait method, consistent with `map`, `pure`, `fold_right`, etc.
- **`Kind!` macro usage:** Uses `Apply!(<Self as Kind!(...) >::Of<'a, A>)` in method signatures, consistent with other subtraits of `Functor` (e.g., `Alt` at lines 104-106 of `alt.rs`).

### Inconsistent aspects

- **No `#[kind]` attribute:** Traits like `Functor` (line 87 of `functor.rs`) and `Pointed` (line 23 of `pointed.rs`) use `#[kind(type Of<'a, A: 'a>: 'a;)]`. `Evaluable` does not, instead relying on its `Functor` supertrait. This is actually correct behavior for subtraits; `Alt`, `Semiapplicative`, and `Filterable` also omit `#[kind]` when they have `Functor` as a supertrait. So this is consistent.
- **No property tests:** Other type classes with laws (e.g., `Functor`) have QuickCheck property tests. `Evaluable` has only one unit test (`test_evaluable_via_brand` at line 1286 of `thunk.rs`). Given the law is questionable (see section 3), this is understandable but should be addressed once the laws are corrected.

## 6. Limitations

### Only one implementor

Only `ThunkBrand` implements `Evaluable`. The documentation (lines 38-40) explains why others cannot:

- **`Lazy`:** Cannot implement because its `evaluate()` (actually `force()`) returns `&A`, not owned `A`. This is a fundamental mismatch since memoization requires shared ownership.
- **`Trampoline`:** Cannot participate because it has no brand (due to `'static` requirements of `Free`'s internals conflicting with the HKT system's lifetime polymorphism).
- **`SendThunk`:** Not mentioned in the docs but also cannot implement because HKT trait signatures lack `Send` bounds on closure parameters.

This narrow implementor set raises the question of whether `Evaluable` should be a trait at all, versus a direct method on `Thunk` used by `Free`. The trait is justified by `Free`'s design: `Free<F, A>` is generic over `F`, so it needs a trait bound to call `evaluate`. If future functors (e.g., a hypothetical `OwnedLazy` that clones on extraction) were added, they could implement `Evaluable` without changing `Free`.

### Consumes the value

`evaluate` takes `fa` by value, consuming it. This is correct for `Thunk` (which wraps `FnOnce`) but means `Evaluable` cannot work with types that provide shared access (like `Lazy`). A complementary `RefEvaluable` trait returning `&A` could serve memoized types, though `Free` would need a different evaluation strategy to use it.

### No `fold_free` integration

`Free` has two consumption methods: `evaluate` (requires `Evaluable`) and `fold_free` (requires `NaturalTransformation`). There is no connection between them. Ideally, if `F: Evaluable`, then `fold_free` with the identity natural transformation should be equivalent to `evaluate`. This relationship is not documented or tested.

### `'static` limitation via `Free`

While `Evaluable` itself supports arbitrary lifetimes `'a`, its primary consumer (`Free`) requires `'static` types. So in practice, `Evaluable` with non-`'static` lifetimes is only useful for direct calls, not through `Free`.

## 7. Documentation

### Module-level docs (lines 1-17)

Accurate and concise. The example correctly demonstrates usage.

### Trait-level docs (lines 29-55)

- The description "A functor containing exactly one extractable value" (line 29) is accurate.
- The phrase "providing a natural transformation `F ~> Id`" (line 30) is the correct categorical characterization.
- The explanation of why `Lazy` and `Trampoline` cannot implement (lines 38-40) is helpful and accurate.
- The naturality law (lines 43-55) is problematic as discussed in section 3.

### Method-level docs (lines 57-82)

Adequate. The doc comment "Evaluates the effect, producing the inner value" is clear. The example is correct.

### Free function docs (lines 85-116)

Follows the standard pattern. The link to the trait method is correct.

### Minor documentation issue

Line 37 says "Currently only ThunkBrand implements this trait." This is fragile documentation that will become stale if new implementors are added. Consider phrasing it as "The canonical implementor is ThunkBrand" or removing the statement.

## Summary of Recommendations

1. **Fix the naturality law.** Replace it with the standard comonad `extract` law: `evaluate(map(f, fa)) == f(evaluate(fa))`. The current naturality law describes a property of natural transformations, not of `evaluate` itself.
2. **Add property tests.** Once the law is corrected, add a QuickCheck property test verifying the map-extract law for `ThunkBrand`.
3. **Document the relationship with `fold_free`.** Note that `Free::evaluate` is equivalent to `Free::fold_free` with the identity natural transformation when `F: Evaluable`.
4. **Consider making the "currently only ThunkBrand" note less fragile.** Use phrasing that will not become stale.
5. **No structural changes needed.** The trait is well-designed for its purpose, the implementation is correct, and the API surface is minimal and appropriate.
