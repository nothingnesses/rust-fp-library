# RefFunctor Analysis

## Overview

`RefFunctor` is a type class defined in `fp-library/src/classes/ref_functor.rs` that provides a `ref_map` operation for types whose values can only be accessed by reference. Its single method has the signature:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl FnOnce(&A) -> B + 'a,
    fa: Apply!(...Of<'a, A>),
) -> Apply!(...Of<'a, B>);
```

Current implementors:
- `LazyBrand<RcLazyConfig>` (i.e., `RcLazy`)
- `TryLazyBrand<E, RcLazyConfig>` (i.e., `RcTryLazy`)

## 1. Why RefFunctor Exists Separately from Functor

The separation is well-motivated. `Lazy::evaluate()` returns `&A`, not `A`. The standard `Functor` trait's `map` requires `impl Fn(A) -> B`, which expects an owned `A`. To satisfy `Functor`, `Lazy` would need to either:

1. Clone the cached value (violates zero-cost abstraction principle, adds `Clone` bound).
2. Consume the cached value (defeats the purpose of memoization).
3. Return a reference somehow (impossible with `Functor`'s signature).

`RefFunctor` honestly represents what memoizing types can do: map with `&A -> B`, producing a new `Lazy<B>` that captures the original via `Rc`/`Arc` and reads the cached `&A` on demand.

**Verdict:** The distinction is sound and well-motivated.

## 2. Trait Bounds Assessment

### FnOnce vs Fn

`RefFunctor::ref_map` uses `FnOnce`, while `Functor::map` uses `Fn`. This asymmetry is intentional and correct:

- `Functor::map` uses `Fn` because types like `Vec` call the function multiple times (once per element).
- `RefFunctor::ref_map` uses `FnOnce` because `Lazy` always contains exactly one value, so the mapping function is called at most once.

This is the right choice. `FnOnce` is the weakest bound and maximizes the set of closures that can be passed in (e.g., closures that move captured values).

### Lifetime bounds

The `'a` lifetime parameter threads through correctly: `A: 'a`, `B: 'a`, and `func: ... + 'a` all participate in the same lifetime, matching the `Lazy<'a, A, C>` lifetime.

### No supertraits

`RefFunctor` has no supertraits beyond the `Kind` bound injected by the `#[kind(...)]` attribute. This is appropriate; there is no logical reason it should require `Functor`.

## 3. Design Issues and Inconsistencies

### 3.1. SendRefFunctor Does Not Extend RefFunctor

`SendRefFunctor` is a completely independent trait, not `SendRefFunctor: RefFunctor`. This means:

- `LazyBrand<ArcLazyConfig>` implements `SendRefFunctor` but NOT `RefFunctor`.
- You cannot write generic code over `RefFunctor` and have it work for both `RcLazy` and `ArcLazy`.

The `SendRefFunctor` docs claim "ArcLazy implements both RefFunctor and SendRefFunctor," but this is **inaccurate**. `ArcLazy` only implements `SendRefFunctor`. The comment in `lazy.rs` (lines 776-778) explains why: `RefFunctor` does not require `Send` on the mapping function, but `ArcLazy::new` requires `Send`. So `ArcLazy` cannot satisfy `RefFunctor`'s signature without adding bounds the trait does not require.

This is a real limitation. If you want to write a function generic over "anything that supports ref_map," you cannot use `Brand: RefFunctor` because `ArcLazy` would be excluded. You would need two versions of such a function, or a workaround.

**Possible fix:** Make `SendRefFunctor: RefFunctor` and add a blanket `RefFunctor` impl for `LazyBrand<ArcLazyConfig>` that delegates to `send_ref_map`. This would require the `RefFunctor` impl to add `Send` bounds on `A` and the closure for the `ArcLazy` case, which is impossible since `RefFunctor::ref_map` has no `Send` bounds. So the current design is the only sound option given Rust's trait system. The limitation is inherent, not a bug.

**Documentation fix needed:** The `SendRefFunctor` docs (line 35-36 of `send_ref_functor.rs`) say "`ArcLazy` implements both `RefFunctor` and `SendRefFunctor`." This is false and should be corrected to say "`ArcLazy` implements `SendRefFunctor` (but not `RefFunctor`, because `RefFunctor` lacks the `Send` bounds that `ArcLazy` requires on its closures)."

### 3.2. Identity Law is Subtly Imprecise

The identity law states:

> `ref_map(|x| x.clone(), fa)` evaluates to a value equal to `fa`'s evaluated value.

Two issues:

1. The law says `x.clone()` but the example code uses `|x: &i32| *x` (dereference copy, not clone). For `i32` these are equivalent, but the law text and example should match. More precisely, the identity mapping for `RefFunctor` should be `|x| x.clone()` (since the input is `&A`, identity needs to produce `A`, which requires `Clone`). The law is correct in spirit but the example uses `Copy` dereference instead of `clone()`.

2. The law only holds when `A: Clone`, which is not stated. Unlike `Functor` where identity is simply `|x| x`, `RefFunctor` identity requires converting `&A` to `A`, so the law inherently requires `Clone`. This should be documented explicitly.

### 3.3. Composition Law's Inner Operation is Ambiguous

The composition law states:

> `ref_map(|x| f(&g(x)), fa)` evaluates to the same value as `ref_map(f, ref_map(g, fa))`.

In `ref_map(f, ref_map(g, fa))`, the inner `ref_map(g, fa)` produces a `Lazy<B>` and the outer `ref_map(f, ...)` receives `&B`. This is correct.

In the fused version, `g(x)` where `x: &A` returns a `B` (owned), and `f(&g(x))` takes `&B`. The law is self-consistent.

However, note that this law requires `g` to be usable in both `FnOnce(&A) -> B` position and inside a composed closure. Since the fused version uses `g` inside a single `FnOnce`, and the sequential version uses `g` as its own `FnOnce`, the law is testable in practice. No issue here.

### 3.4. No `ref_map_with_index` or Indexed Variant

There is no `RefFunctorWithIndex` trait analogous to `FunctorWithIndex`. For `Lazy` (which holds exactly one value), an index would be meaningless, so this is not a concern for current implementors. But if `RefFunctor` were ever implemented for a multi-element type, the absence would matter. This is a minor point given the current scope.

## 4. Implementation Correctness

### RcLazy Implementation

```rust
impl RefFunctor for LazyBrand<RcLazyConfig> {
    fn ref_map<'a, A: 'a, B: 'a>(
        f: impl FnOnce(&A) -> B + 'a,
        fa: Lazy<'a, A, RcLazyConfig>,
    ) -> Lazy<'a, B, RcLazyConfig> {
        fa.ref_map(f)  // delegates to inherent method
    }
}
```

The inherent `Lazy::ref_map` creates a new `Lazy` whose initializer captures `self` (the original `Lazy`) and calls `f(self.evaluate())`. This is correct: the new lazy value, when first evaluated, forces the original, reads its cached reference, applies `f`, and caches the result.

### RcTryLazy Implementation

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    f: impl FnOnce(&A) -> B + 'a,
    fa: RcTryLazy<'a, A, E>,
) -> RcTryLazy<'a, B, E> {
    RcTryLazy::new(move || fa.evaluate().map(f).map_err(|e| e.clone()))
}
```

This has the `E: Clone` bound (from `E: 'static + Clone` on the impl block), which is needed to clone the error into the new cell. The implementation is correct: it evaluates the inner `TryLazy`, maps the `Ok` case, and clones the error for the `Err` case.

**Minor concern:** The `E: 'static` bound on `TryLazyBrand` implementations means errors cannot contain borrowed data. This is a pre-existing constraint of `TryLazyBrand`, not specific to `RefFunctor`.

## 5. Ergonomic Issues

### 5.1. Verbose Turbofish

Using `ref_map` as a free function requires specifying the brand:

```rust
ref_map::<LazyBrand<RcLazyConfig>, _, _>(|x: &i32| *x * 2, memo)
```

This is significantly more verbose than the inherent method:

```rust
memo.ref_map(|x| *x * 2)
```

In practice, users will almost always prefer the inherent method. The free function exists for generic programming (writing functions parameterized by `Brand: RefFunctor`), which is the intended use case for all type class free functions in this library.

### 5.2. No Blanket for Clone Types

There is no way to automatically get a `Functor` from a `RefFunctor` when `A: Clone`. A helper like:

```rust
fn ref_map_clone<Brand: RefFunctor, A: Clone + 'a, B: 'a>(f: impl Fn(A) -> B, fa: ...) -> ...
```

could bridge the gap, allowing users to use regular `A -> B` functions when `Clone` is available. This is not a flaw per se, but could be a useful utility function.

### 5.3. Closure Type Annotation Often Required

Because `ref_map` takes `impl FnOnce(&A) -> B` and the compiler must infer `A` from both the closure and the `fa` argument, users often need explicit type annotations on the closure parameter (e.g., `|x: &i32| ...`). This is a minor ergonomic friction inherent to Rust's type inference with generic traits.

## 6. Documentation Quality

### Strengths

- Clear module-level doc with a working example.
- Law documentation with concrete examples.
- Consistent use of `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, and `#[document_returns]` macros.
- Both trait method and free function have examples.

### Issues

- **Inaccurate claim in SendRefFunctor docs:** As noted in 3.1, the claim that "ArcLazy implements both RefFunctor and SendRefFunctor" is false.
- **Identity law does not mention Clone requirement:** The law `ref_map(|x| x.clone(), fa)` implicitly requires `A: Clone`, which should be stated.
- **No doc explaining why FnOnce:** The choice of `FnOnce` over `Fn` is not documented. A brief note explaining that `Lazy` holds exactly one value, so the function is called at most once, would help users understand the design.
- **Missing cross-reference to SendRefFunctor:** The `RefFunctor` trait docs do not mention `SendRefFunctor` as the thread-safe counterpart. A "See also" note would help discoverability.

## 7. Summary of Findings

| Category | Finding | Severity |
|----------|---------|----------|
| Design motivation | Separation from `Functor` is well-justified | OK |
| Trait bounds | `FnOnce` is correct and minimal | OK |
| Hierarchy gap | `SendRefFunctor` does not extend `RefFunctor`; inherent Rust limitation | Accepted limitation |
| Doc accuracy | `SendRefFunctor` docs falsely claim `ArcLazy` implements `RefFunctor` | Medium (misleading) |
| Law precision | Identity law requires `Clone` but does not state it | Low |
| Law precision | Example uses `Copy` deref instead of `clone()` | Low |
| Ergonomics | Verbose turbofish for free function | Low (expected) |
| Missing docs | No mention of `SendRefFunctor` in `RefFunctor` docs | Low |
| Missing docs | No explanation of `FnOnce` choice | Low |
| Missing utility | No `ref_map_clone` bridge function | Low (enhancement) |

## 8. Recommendations

1. **Fix the SendRefFunctor documentation** to accurately state that `ArcLazy` implements only `SendRefFunctor`, not both traits.
2. **Add a `Clone` note to the identity law** or rephrase it to make the requirement explicit.
3. **Add a "See also" cross-reference** from `RefFunctor` to `SendRefFunctor`.
4. **Consider adding a brief note** explaining why `FnOnce` is used (single-element container).
5. **Consider a `ref_map_clone` utility** that bridges `RefFunctor` to `Fn(A) -> B` when `A: Clone`, for convenience.
