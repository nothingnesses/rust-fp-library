# RefFunctor Analysis

## Overview

`RefFunctor` (defined in `fp-library/src/classes/ref_functor.rs`) is a type class for mapping over types where the contained value is accessed by reference rather than by ownership. Its sole current implementors are `LazyBrand<RcLazyConfig>` (line 928 of `lazy.rs`) and `TryLazyBrand<E, RcLazyConfig>` (line 1408 of `try_lazy.rs`).

The trait provides a single method:

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    func: impl FnOnce(&A) -> B + 'a,
    fa: Apply!(<Self as Kind!(...)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
```

## 1. Design: Why RefFunctor Exists Separately from Functor

**The core reason is sound.** `Lazy::evaluate` returns `&A` (line 532 of `lazy.rs`), not an owned `A`. The standard `Functor::map` signature requires `impl Fn(A) -> B`, which demands ownership of `A`. For a memoized type that only exposes `&A`, this would require either:

- Cloning the cached value on every `map` call (wasteful, requires `Clone`).
- Consuming/invalidating the cache (defeats the purpose of memoization).

By introducing `RefFunctor` with `impl FnOnce(&A) -> B`, the mapping function borrows the cached value. This is the correct design for memoized types.

**The independence from `SendRefFunctor` is also justified.** As documented at lines 29-34 of `ref_functor.rs`, `ArcLazy::new` requires `Send` on the closure. A `RefFunctor` supertrait relationship would require `ArcLazy` to accept non-`Send` closures, which `ArcLazy` cannot do (its underlying `LazyLock` requires `Send`). Keeping the two traits independent means `RcLazy` implements only `RefFunctor`, and `ArcLazy` implements only `SendRefFunctor`, with neither forced to satisfy the other's constraints.

**Limitation of this approach.** The split means there is no trait-level polymorphism over "any lazy type that supports ref_map." Generic code cannot abstract over both `RcLazy` and `ArcLazy` using a single bound. A hypothetical unifying trait parameterized by a marker (e.g., `RefFunctor<ThreadSafety>`) could address this, but at significant complexity cost.

## 2. Implementation Quality and Correctness

### Trait definition (ref_functor.rs, lines 88-124)

The trait definition is correct. Key observations:

- **`FnOnce` is the right choice** (documented at lines 82-87). Since `Lazy` evaluates the closure at most once (the result is memoized), `FnOnce` is sufficient and strictly more permissive for callers than `Fn` or `FnMut`. This is a deliberate and correct divergence from `Functor::map`, which uses `impl Fn(A) -> B` (line 122 of `functor.rs`). `Functor` needs `Fn` because types like `Vec` call the function multiple times; `Lazy` calls it exactly once.

- **The `Kind` machinery** (`#[kind(type Of<'a, A: 'a>: 'a;)]` on line 88) correctly mirrors `Functor`'s kind annotation (line 87 of `functor.rs`), ensuring consistent HKT encoding.

### `LazyBrand<RcLazyConfig>` implementation (lazy.rs, lines 928-961)

The implementation delegates to the inherent `ref_map` method (line 959: `fa.ref_map(f)`). The inherent method (lines 607-612) creates a new `RcLazy` that captures `self` (the original `Lazy`) and the closure:

```rust
RcLazy::new(move || f(self.evaluate()))
```

This is correct: on first evaluation of the mapped `Lazy`, it forces the original, passes the reference to `f`, and memoizes the result. The `self` capture by move means the original `Lazy` (which is `Rc`-backed and `Clone`) is kept alive by the new one, which is correctly documented in the "Cache chain behavior" section (lines 74-80).

### `TryLazyBrand<E, RcLazyConfig>` implementation (try_lazy.rs, lines 1408-1450)

The implementation handles the `Result` semantics correctly:

```rust
RcTryLazy::new(move || match fa.evaluate() {
    Ok(a) => Ok(f(a)),
    Err(e) => Err(e.clone()),
})
```

The `E: Clone` bound on the impl (line 1408: `impl<E: 'static + Clone>`) is necessary because on the error path, the error must be cloned into the new cell. This is a reasonable requirement.

### Free function (ref_functor.rs, lines 157-162)

The free function `ref_map` is a thin wrapper that dispatches to `Brand::ref_map(func, fa)`. This follows the same pattern as `map` in `functor.rs` (lines 158-163). Correct.

## 3. Laws

The documented laws (lines 38-45) are:

- **Identity:** `ref_map(|x| x.clone(), fa)` is equivalent to `fa`, given `A: Clone`.
- **Composition:** `ref_map(|x| g(&f(x)), fa)` is equivalent to `ref_map(g, ref_map(f, fa))`.

### Assessment

These are the correct functor laws adapted for reference semantics. The identity law necessarily requires `Clone` because `ref_map` receives `&A` but must produce a value; `|x| x.clone()` is the closest analogue to the standard identity function in this context.

The composition law is correctly stated: since the inner `ref_map(f, fa)` produces a new `Lazy<B>` and the outer `ref_map(g, ...)` receives `&B`, the composed version must also take `&A` and produce the final value via `g(&f(x))`.

**Subtlety worth noting:** The identity law holds up to value equality (as tested in the doc example at line 60: `assert_eq!(*mapped.evaluate(), *fa.evaluate())`), not referential equality. Two distinct `Lazy` instances with the same evaluated value are "equivalent" for law purposes, but they are different allocations. This is inherent to the design and correctly reflected in the examples.

**Missing:** There are no property-based tests for these laws. The only validation is through doc tests. Property tests (QuickCheck) would strengthen confidence, especially for composition across chained `ref_map` calls.

## 4. API Surface

### Strengths

- The free function `ref_map` (lines 157-162) provides a convenient calling convention matching other type class free functions (`map`, `bind`, etc.).
- The inherent method `Lazy::ref_map` (line 607 of `lazy.rs`) provides an ergonomic method-call syntax for direct use.
- The function re-export through `functions.rs` works via the `generate_function_re_exports!` macro (line 25 of `functions.rs`), so `ref_map` is available through `fp_library::functions::*`.

### Concerns

1. **Verbosity of generic free function calls.** Callers must write `ref_map::<LazyBrand<RcLazyConfig>, _, _>(|x: &i32| ..., fa)`, which is considerably more verbose than the inherent `fa.ref_map(|x| ...)`. In practice, most users will prefer the inherent method. The free function's primary value is for generic programming over `RefFunctor` bounds, but since the only implementors are `Lazy` variants, the generic use case is narrow.

2. **No `Fn` version.** The trait only offers `FnOnce`. While this is correct for `Lazy` (single evaluation), if a future implementor needed multi-evaluation semantics (e.g., a hypothetical `LazyStream`), it could not use `RefFunctor`. This is acceptable given the current scope.

## 5. Consistency with Other Type Classes

**Consistent aspects:**

- Uses the same `#[kind(...)]` annotation as `Functor`, `Foldable`, etc.
- Follows the same `trait + free function` pattern in an inner `document_module` block.
- Documentation follows the same template: laws section, examples, `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`.
- Re-exported through `functions.rs` like other free functions.

**Inconsistent aspects:**

- `Functor::map` uses `impl Fn(A) -> B` while `RefFunctor::ref_map` uses `impl FnOnce(&A) -> B`. The `FnOnce` choice is justified (see section 2), but this means `RefFunctor` is not a drop-in replacement for `Functor`. The relationship between the two traits is "similar but parallel" rather than hierarchical.
- `RefFunctor` and `SendRefFunctor` are fully independent traits with no shared supertrait. By contrast, many other trait pairs in the codebase have supertrait relationships (e.g., `Monad: Applicative`, `SendDeferrable: Deferrable`). The `SendDeferrable: Deferrable` relationship works because `Deferrable` does not impose bounds that conflict with `Send`; `RefFunctor` cannot follow this pattern because `ArcLazy::new` requires `Send` on the closure.

## 6. Limitations and Alternatives

### Current limitations

1. **No supertrait relationship with `Functor`.** Code that is generic over `Functor` cannot work with `Lazy`. This means `Lazy` is excluded from any combinator or algorithm built on `Functor`, `Applicative`, `Monad`, etc. The entire standard functor/monad tower is unavailable for `Lazy`.

2. **Only two implementors.** `RefFunctor` is implemented only for `LazyBrand<RcLazyConfig>` and `TryLazyBrand<E, RcLazyConfig>`. This limits its utility as a generic abstraction. It is effectively a "Lazy-specific Functor."

3. **No `Applicative` or `Monad` analogue.** There is no `RefApplicative` or `RefMonad`. If users want to combine multiple `Lazy` values, they must do so manually or use `ref_map` with captured references.

4. **Cache chain memory behavior.** As documented (lines 74-80), chaining `ref_map` creates a linked list of `Rc`-held cells. Each mapped value retains its predecessor. Long chains accumulate memory. This is inherent and well-documented, but users may not expect it.

### Alternative: Functor with Clone

One alternative would be to implement `Functor` for `Lazy` where `A: Clone`, using `map(f, fa)` = `Lazy::new(move || f(fa.evaluate().clone()))`. This would integrate `Lazy` into the standard functor tower.

**Tradeoffs:**
- Pro: `Lazy` could participate in `Functor`-generic code.
- Pro: No need for a separate `RefFunctor` trait.
- Con: Forces a `Clone` on every `map`, which may be expensive.
- Con: Rust's trait system does not easily support conditional `Functor` impls where `A: Clone`; the `Kind` trait's `Of<'a, A>` has no `Clone` bound, and adding one would affect all `Functor` implementors.

**Verdict:** The `Clone`-based approach would be worse. It imposes unnecessary costs and cannot be expressed cleanly in the current HKT encoding without polluting the `Functor` trait itself. `RefFunctor` is the right solution for this constraint.

### Alternative: Functor with interior mutability

Another approach: make `Lazy::evaluate` return an owned value via `Clone` internally, storing `A: Clone` as a bound on the `Kind` impl. This is essentially a variation of the above, with the same drawbacks.

### Alternative: Comonad-style `extract`

In Haskell, `Comonad` provides `extract :: w a -> a`. `Lazy` could be a `Comonad` where `extract` forces evaluation and clones the result. Then `extend :: (w a -> b) -> w a -> w b` would be the natural mapping operation, which is essentially `ref_map` but with the full context available. This is a more principled categorical framing, but requires implementing the full `Comonad` trait and still requires `Clone` for `extract`.

## 7. Documentation Quality

### Strengths

- The module-level docs (lines 1-15) provide a working example.
- The trait-level docs (lines 24-87) are thorough, covering:
  - Why `RefFunctor` exists (lines 24-27).
  - Why it is independent from `SendRefFunctor` (lines 29-34).
  - Laws with explanation of the `Clone` requirement (lines 38-45).
  - Working law examples (lines 48-72).
  - Cache chain behavior warning (lines 74-80).
  - Why `FnOnce` is used (lines 82-87).
- The free function docs (lines 126-162) follow the standard template.

### Issues

1. **Composition law documentation ambiguity (line 45).** The composition law states `ref_map(|x| g(&f(x)), fa)` is equivalent to `ref_map(g, ref_map(f, fa))`. In the doc example (lines 63-64), `f` is `|x: &i32| *x * 2` and `g` is not present in the composed form in the same way. The notation is fine but could be clearer about the types: in the composed form, `f` returns an owned `B`, and `g` takes `&B`. The doc example correctly demonstrates this (line 66: `|x: &i32| f(&g(x))`), but the abstract law statement on line 45 uses `f` and `g` in the opposite order from the example, which could confuse readers. In the law, `f` is applied first (inner), but in the example, `g` is applied first. This is technically correct (the law uses mathematical composition order), but the inconsistency between the law statement and the example is a minor readability issue.

2. **The identity law example (line 59) uses `|x: &i32| *x` instead of `|x: &i32| x.clone()`.** For `i32` these are equivalent (dereferencing a `Copy` type), but the law as stated says `x.clone()`. Using `*x` is fine for the example but slightly diverges from the stated law.

## Summary of Findings

| Aspect | Assessment |
|--------|-----------|
| Design rationale | Sound; `RefFunctor` is necessary for memoized types that expose `&A`. |
| Independence from `SendRefFunctor` | Correctly justified by `Send` bound incompatibility. |
| Implementation correctness | Correct for both `Lazy` and `TryLazy`. |
| `FnOnce` choice | Correct and well-reasoned. |
| Laws | Correct adaptation of functor laws for reference semantics. |
| Property tests | Missing; only doc tests validate laws. |
| API surface | Adequate; inherent methods provide ergonomic access. Free function useful for generic code. |
| Consistency | Mostly consistent with other type classes; the `FnOnce` vs `Fn` difference is justified. |
| Documentation | Thorough and accurate, with minor composition law readability issue. |
| Limitations | Cannot integrate with Functor tower; only two implementors; no RefApplicative/RefMonad. |

### Recommendations

1. Add property-based tests for `RefFunctor` identity and composition laws.
2. Consider clarifying the composition law documentation to use consistent variable naming between the abstract law and the example.
3. The overall design is correct and should be kept. The alternative of `Functor` with `Clone` bounds is strictly worse for this use case.
