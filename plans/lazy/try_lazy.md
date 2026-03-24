# Analysis: `try_lazy.rs`

## Overview

`TryLazy<'a, A, E, Config>` is the fallible counterpart of `Lazy<'a, A, Config>`. It wraps a `Result<A, E>` in a memoized lazy cell, with `RcTryLazy` (single-threaded) and `ArcTryLazy` (thread-safe) variants. The file weighs in at ~1,790 lines, roughly split between implementation (~900 lines) and tests (~250 lines), with documentation comprising the bulk of the rest.

## 1. Design Assessment

### What Works Well

- **Config trait pattern.** The `TryLazyConfig` trait (defined in `lazy.rs`) cleanly separates the Rc/Arc choice from the `TryLazy` type itself, matching the infallible `Lazy` design exactly.
- **Evaluate returns `Result<&A, &E>`.** This is the correct design for memoized fallible values; returning references avoids unnecessary cloning.
- **Comprehensive conversion suite.** `From` impls cover `TryThunk`, `TryTrampoline`, `Lazy`, and `Result`, with correct eager-evaluation semantics for the `!Send` to `Arc` conversions.
- **Panic safety.** The `catch_unwind` and `catch_unwind_with` methods are a thoughtful addition, covering both convenience (`E = String`) and custom error conversion cases. This aligns with the `Lazy` docs that explicitly recommend `TryLazy` for panic-safe memoization.
- **Test coverage.** Tests cover caching (Ok/Err), clone sharing, panic poisoning, thread safety, conversions, `Deferrable`/`SendDeferrable`, QuickCheck properties, and convenience constructors.

### Structural Consistency with `Lazy`

The design follows `Lazy` closely:
- Same `Config` parameter with default `RcLazyConfig`.
- Same pattern of separate `impl` blocks for `RcLazyConfig` and `ArcLazyConfig`.
- Same `Clone`, `Debug`, `Semigroup`, `Monoid`, `Deferrable`, `SendDeferrable`, `RefFunctor`, `SendRefFunctor`, `Foldable` implementations.
- Same approach to `From` conversions with eager evaluation for `!Send` to `ArcLazyConfig`.

This structural parallelism is good for learnability and maintenance.

## 2. Issues and Concerns

### 2.1. `map_err` Clones the Success Side Unnecessarily via `.cloned()`

In the `RcTryLazy::map_err` implementation (line 283):

```rust
RcTryLazy::new(move || self.evaluate().cloned().map_err(f))
```

This calls `.cloned()` on the `Result<&A, &E>`, which clones _both_ the `&A` and `&E` into owned values, then `map_err(f)` transforms only the error. The success value `A` gets cloned even though `f` only operates on the error. Compare with `map` (line 246):

```rust
RcTryLazy::new(move || self.evaluate().map(f).map_err(|e| e.clone()))
```

Here `map` only clones the error when it is `Err`, and applies `f` (which takes `&A`) when it is `Ok`. This is asymmetric: `map` avoids cloning the untouched side via `&A`, but `map_err` does not use a `&E`-taking `f` to avoid cloning `A`.

The issue is that `map_err`'s `f` takes `&E` (consistent with `map`'s `f` taking `&A`), but then uses `.cloned()` which clones both sides. A more efficient implementation would be:

```rust
RcTryLazy::new(move || match self.evaluate() {
    Ok(a) => Ok(a.clone()),
    Err(e) => Err(f(e)),
})
```

This clones only the side that needs cloning, matching the efficiency of `map`. The same issue applies to `ArcTryLazy::map_err` (line 727).

### 2.2. `Deferrable` for `RcTryLazy` Requires `Clone` on Both `A` and `E`

The `Deferrable` impl for `RcTryLazy` (line 825) requires `A: Clone + 'a, E: Clone + 'a`. The `defer` body is:

```rust
Self::new(move || f().evaluate().cloned().map_err(Clone::clone))
```

The `.cloned()` call on `Result<&A, &E>` clones the `Ok` side, and `.map_err(Clone::clone)` is redundant since `.cloned()` already cloned both sides. This looks like a bug: `.cloned()` on `Result<&A, &E>` produces `Result<A, &E>` (clones only `Ok`), so `.map_err(Clone::clone)` is needed. Actually, checking the std docs: `Result::cloned()` returns `Result<A, E>` when both `A` and `E` are references, so it clones both. Wait, `Result<&A, &E>::cloned()` is not a standard method. Let me reconsider.

Actually, `Result<&A, &E>` does not have a `.cloned()` method in std. The `.cloned()` method exists on `Result<&T, E>` and produces `Result<T, E>` by cloning the `Ok` variant. So `self.evaluate().cloned()` on `Result<&A, &E>` would produce `Result<A, &E>`, requiring `A: Clone`. Then `.map_err(Clone::clone)` clones the `E`. This is correct, though the two-step approach is slightly confusing. It could be written more clearly as:

```rust
Self::new(move || match f().evaluate() {
    Ok(a) => Ok(a.clone()),
    Err(e) => Err(e.clone()),
})
```

The same pattern appears in `SendDeferrable::send_defer` (line 945).

### 2.3. `Deferrable` for `ArcTryLazy` Calls `f()` Eagerly

The `Deferrable` for `ArcTryLazy` (line 872) calls `f()` eagerly:

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self
where Self: Sized {
    f()
}
```

The documentation explains this is because `Deferrable::defer` does not require `Send` on the thunk, while `ArcTryLazy::new` does. This is correct and matches `Lazy<ArcLazyConfig>`'s approach exactly. However, this means `defer` for `ArcTryLazy` is not actually lazy at the outer level; it immediately calls `f`, though the returned `ArcTryLazy` itself may still be lazy. This is a necessary compromise but could surprise users.

### 2.4. No `and_then` / `or_else` Methods

`TryLazy` has `map` and `map_err` but no `and_then` (flat-map on the success side) or `or_else` (flat-map on the error side). For a fallible type, these are standard combinators that users would expect. An `and_then` would take `FnOnce(&A) -> Result<B, E>` and produce a new `TryLazy<B, E>`. This is a missing ergonomic feature.

### 2.5. No `Bifunctor` Implementation

Since `TryLazy` has both `map` (over success) and `map_err` (over error), it is a natural candidate for a `Bifunctor` implementation via the HKT system. The library has `TryThunk` which may have a `Bifunctor` brand; `TryLazy` does not. This could be a deliberate omission (since the mapping functions take `&A` / `&E` rather than owned values, which does not match the standard `Bifunctor` signature), but it is worth noting.

### 2.6. No `fix` Combinator

`Lazy` provides `rc_lazy_fix` and `arc_lazy_fix` for tying recursive knots. `TryLazy` has no equivalent. A `try_lazy_fix` could be useful for recursive fallible computations. The absence is understandable (fallible fixed points are less common), but it is a gap in the API surface compared to `Lazy`.

### 2.7. `impl_kind!` Requires `E: 'static`

Line 860-865:

```rust
impl_kind! {
    impl<E: 'static, Config: TryLazyConfig> for TryLazyBrand<E, Config> {
        type Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>;
    }
}
```

The `E: 'static` bound means `TryLazyBrand` cannot be used in HKT contexts when the error type borrows data. This is a real limitation for error types like `&str` or types containing borrowed references. Compare with `LazyBrand` which has `impl<Config: LazyConfig>` with no extra lifetime constraints. The `'static` bound on `E` comes from the HKT machinery (brand types must be `'static`), so this may be inherent, but it is a limitation worth documenting.

### 2.8. `Foldable` Silently Discards Errors

The `Foldable` implementation for `TryLazyBrand` treats `Err` as "empty" (returns the initial accumulator or `M::empty()`). This is semantically treating `TryLazy` as "zero or one elements" (like `Option`), which is a valid interpretation but has a subtle implication: errors are silently lost during folding. Users folding over a `TryLazy` that evaluates to `Err` will get no indication that an error occurred.

This is consistent with how `Foldable` works for `Option` and `Result` in Haskell/PureScript, so the design is principled. However, it means that `fold_right`, `fold_left`, and `fold_map` are all "optimistic" operations that ignore failure. The documentation does state "If `Err`, returns the initial accumulator unchanged," which is accurate.

One concern: the `Foldable` impl requires `E: Clone` on the brand (line 1190: `E: 'static + Clone`), but the fold methods never actually clone `E`. The `Clone` bound comes from the `RefFunctor` impl sharing the same brand, not from `Foldable` itself. This is a consequence of the brand-level trait organization rather than a code error.

Actually, looking more carefully, the `Foldable` impl at line 1190 has `impl<E: 'static + Clone> Foldable for TryLazyBrand<E, RcLazyConfig>`. The `Clone` bound on `E` is not needed by the fold methods themselves (they never clone `E`). This bound seems inherited from the idea that `TryLazyBrand` needs `E: Clone` for most operations, but `Foldable` does not actually require it. If the `Foldable` impl could use a weaker bound (`E: 'static` only), it would be more general.

### 2.9. `Semigroup::append` Evaluates Both Sides Before Short-Circuiting

In the `Semigroup` impl (line 1078):

```rust
RcTryLazy::new(move || match (a.evaluate(), b.evaluate()) {
    (Ok(va), Ok(vb)) => Ok(Semigroup::append(va.clone(), vb.clone())),
    (Err(e), _) => Err(e.clone()),
    (_, Err(e)) => Err(e.clone()),
})
```

Both `a` and `b` are evaluated even if `a` is `Err`. A short-circuiting implementation would check `a` first:

```rust
match a.evaluate() {
    Err(e) => Err(e.clone()),
    Ok(va) => match b.evaluate() {
        Ok(vb) => Ok(Semigroup::append(va.clone(), vb.clone())),
        Err(e) => Err(e.clone()),
    },
}
```

For memoized values the cost is small (just a reference lookup after first eval), but conceptually, short-circuiting is the more principled behavior for fallible `Semigroup`. In the left-biased error propagation model used here, evaluating `b` when `a` already failed is wasted work.

### 2.10. `From<Lazy> for TryLazy` Requires `A: Clone`

The `From<Lazy<'a, A, RcLazyConfig>> for TryLazy<'a, A, E, RcLazyConfig>` impl (line 428) requires `A: Clone` because it does `Ok(memo.evaluate().clone())`. This is necessary because `Lazy::evaluate` returns `&A`. However, it means you cannot convert a `Lazy<NonCloneType>` into a `TryLazy`. An alternative design could share the underlying lazy cell, but that would require deeper integration between the two types.

## 3. Documentation Assessment

### Strengths

- The module-level doc comment is clear and concise.
- The `TryLazy` struct doc includes "When to Use" guidance and "Cache Chain Behavior" explanation.
- The panic poisoning behavior is documented on `evaluate`.
- `map` and `map_err` both document their `Clone` requirements with "Why `E: Clone`?" / "Why `A: Clone`?" sections.
- All doc examples include assertions and compile.

### Gaps

- The module doc does not mention the `Foldable` behavior (treating `Err` as empty). Given the surprising nature of silently discarding errors, this should be called out.
- The `Deferrable` for `ArcTryLazy` documents that `f()` is called eagerly, which is good. The `RcTryLazy` `Deferrable` documentation says "The inner `TryLazy` is computed only when the outer `TryLazy` is evaluated," which is accurate.
- Missing guidance on when to use `TryLazy` vs. `Result<Lazy, E>` vs. `Lazy<Result<A, E>>`. These are three distinct designs with different trade-offs.

## 4. Comparison with Infallible `Lazy`

| Aspect | `Lazy` | `TryLazy` | Notes |
|--------|--------|-----------|-------|
| Core method | `evaluate() -> &A` | `evaluate() -> Result<&A, &E>` | Consistent. |
| HKT brand | `LazyBrand<Config>` | `TryLazyBrand<E, Config>` | Extra `E` parameter is necessary. |
| `RefFunctor` | Yes (`RcLazyConfig`) | Yes (`RcLazyConfig`, `E: Clone`) | `TryLazy` needs `Clone` for error propagation. |
| `SendRefFunctor` | Yes (`ArcLazyConfig`) | Yes (`ArcLazyConfig`, `E: Clone + Send + Sync`) | Consistent. |
| `Foldable` | Always applies func | Applies func on `Ok`, returns initial on `Err` | Consistent with `Option`/`Result` semantics. |
| `Semigroup` | Always combines | Short-circuits on `Err` (but evaluates both) | Could be improved. |
| `Monoid` | `empty()` wraps `A::empty()` | `empty()` wraps `Ok(A::empty())` | Consistent. |
| `Deferrable` (Rc) | Clones inner value | Clones both `A` and `E` | Consistent approach. |
| `Deferrable` (Arc) | Calls `f()` eagerly | Calls `f()` eagerly | Same compromise. |
| `fix` combinators | Yes (`rc_lazy_fix`, `arc_lazy_fix`) | No | Gap. |
| `pure` constructor | Yes | No (uses `ok`/`err` instead) | Appropriate naming for fallible type. |
| `ref_map` inherent | Yes | `map` (inherent, takes `&A`) | Naming differs; `TryLazy::map` is conceptually `ref_map` on the `Ok` side. |

The naming difference for the inherent mapping method (`ref_map` on `Lazy` vs. `map` on `TryLazy`) is slightly inconsistent. `Lazy::ref_map` is named to distinguish from a hypothetical `Functor::map` (which it cannot implement); `TryLazy::map` does not carry this distinction even though it also takes `&A`. This could confuse users who expect `TryLazy::map` to take an owned `A` by analogy with `Result::map`.

## 5. Summary of Recommendations

1. **Fix `map_err` efficiency.** Use an explicit `match` instead of `.cloned()` to avoid cloning the untouched `Ok` side.
2. **Short-circuit `Semigroup::append`.** Avoid evaluating `b` when `a` is already `Err`.
3. **Relax `Foldable` bounds.** Remove the `E: Clone` requirement if possible (it is not used by the fold methods).
4. **Add `and_then` / `or_else`.** These are standard combinators for fallible types.
5. **Consider renaming `map` to `ref_map`** (or `try_ref_map`) for consistency with `Lazy::ref_map`, or document the naming rationale.
6. **Document `Foldable` error-discarding behavior** in the module-level doc comment.
7. **Document when to use `TryLazy` vs. `Lazy<Result<A, E>>`** to help users choose.
8. **Consider adding `try_lazy_fix` combinators** for recursive fallible memoized computations, if there are use cases.
