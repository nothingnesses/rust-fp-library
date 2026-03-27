# Analysis: `SendRefFunctor` (`classes/send_ref_functor.rs`)

## Overview

`SendRefFunctor` is the thread-safe counterpart to `RefFunctor`. It provides a `send_ref_map` operation that maps over types whose values are accessed by reference (like memoized lazy types), with the additional constraint that the mapping function and values must be `Send`.

**File:** `fp-library/src/classes/send_ref_functor.rs` (147 lines)

**Implementors:**
- `LazyBrand<ArcLazyConfig>` (in `types/lazy.rs`, line 963)
- `TryLazyBrand<E, ArcLazyConfig>` where `E: 'static + Clone + Send + Sync` (in `types/try_lazy.rs`, line 1455)

## 1. Design

### Separate trait (not a subtrait of RefFunctor): sound and necessary

The decision to make `SendRefFunctor` independent from `RefFunctor` is well-justified and documented in both traits (lines 26-36 of `send_ref_functor.rs`; lines 29-34 of `ref_functor.rs`). The reasoning:

- `ArcLazy::new` requires `Send` on the closure (line 733 of `lazy.rs`), so a generic `RefFunctor::ref_map` (which has no `Send` bound on `func`) cannot construct an `ArcLazy`. Therefore `ArcLazy` cannot implement `RefFunctor`.
- `RcLazy` uses `Rc` (which is `!Send`), so it cannot implement `SendRefFunctor`.
- Making `SendRefFunctor: RefFunctor` would force both constraints simultaneously, excluding both types.

This is the standard Rust pattern for thread-safety trait pairs (like `Fn`/`Send + Fn`, or the library's own `Deferrable`/`SendDeferrable`, `RefCountedPointer`/`SendRefCountedPointer`).

**One notable difference:** `SendDeferrable` extends `Deferrable` (it is a subtrait), while `SendRefFunctor` does NOT extend `RefFunctor`. The asymmetry exists because `SendDeferrable`'s implementors (`SendThunk`, `ArcLazy`, etc.) can satisfy both the `Send` and non-`Send` construction paths, whereas `ArcLazy`'s `new` unconditionally requires `Send` on the closure, making a blanket `RefFunctor` impl impossible. This is a subtle but correct distinction.

### Exclusive implementor sets

- `RefFunctor`: `LazyBrand<RcLazyConfig>`, `TryLazyBrand<E, RcLazyConfig>`
- `SendRefFunctor`: `LazyBrand<ArcLazyConfig>`, `TryLazyBrand<E, ArcLazyConfig>`

The two sets are completely disjoint. No type implements both traits. This means code that is generic over `RefFunctor` cannot use `ArcLazy`, and code generic over `SendRefFunctor` cannot use `RcLazy`. There is no unified abstraction that covers "any lazy type that supports ref_map."

## 2. Implementation Quality

### Trait definition (lines 70-106)

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
    func: impl FnOnce(&A) -> B + Send + 'a,
    fa: Apply!(<Self as Kind!(...)>::Of<'a, A>),
) -> Apply!(<Self as Kind!(...)>::Of<'a, B>);
```

**Correctness of bounds:**

- `A: Send + Sync + 'a`: Required because `fa` (an `ArcLazy<A>`) is captured by the new closure passed to `ArcLazy::new`. For `ArcLazy<A>` (i.e., `Arc<LazyLock<A, ...>>`) to be `Send`, `A` must be `Send + Sync`. This is correct.
- `B: Send + 'a`: The output `B` must be `Send` because the closure producing it must be `Send`. However, `B` does not require `Sync`. This is technically correct for constructing the `ArcLazy<B>`, but the resulting `ArcLazy<B>` will not be `Send` unless `B: Sync` as well (since `Arc<T>: Send` requires `T: Sync`). This means the result of `send_ref_map` may not itself be sendable across threads. See Limitations section.
- `func: impl FnOnce(&A) -> B + Send + 'a`: Correct. `FnOnce` is appropriate since memoized types evaluate the mapping closure at most once. `Send` is required so the closure can be stored inside an `Arc`-backed cell.

### Free function (lines 139-144)

Straightforward delegation with matching bounds. Correct.

### `LazyBrand<ArcLazyConfig>` impl (lazy.rs, lines 963-996)

The implementation delegates to `fa.ref_map(f)` (line 994), which is the inherent method on `ArcLazy` (lazy.rs, line 793). That inherent method:

```rust
pub fn ref_map<B: 'a>(
    self,
    f: impl FnOnce(&A) -> B + Send + 'a,
) -> Lazy<'a, B, ArcLazyConfig>
where
    A: Send + Sync,
{
    ArcLazy::new(move || f(self.evaluate()))
}
```

This captures `self` (the `ArcLazy<A>`) inside a new closure, creates a new `ArcLazy<B>` that evaluates the original, applies `f` to the reference, and memoizes the result. Correct.

### `TryLazyBrand<E, ArcLazyConfig>` impl (try_lazy.rs, lines 1455-1497)

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
    f: impl FnOnce(&A) -> B + Send + 'a,
    fa: ...,
) -> ... {
    ArcTryLazy::new(move || match fa.evaluate() {
        Ok(a) => Ok(f(a)),
        Err(e) => Err(e.clone()),
    })
}
```

Requires `E: 'static + Clone + Send + Sync`. The `Clone` on `E` is needed because when the original `TryLazy` evaluates to `Err`, the error must be cloned into the new cell. This is consistent with the `RefFunctor` impl for `TryLazyBrand<E, RcLazyConfig>` (line 1408), which requires `E: 'static + Clone`. Correct.

## 3. API Surface

### Strengths

- The free function `send_ref_map` provides ergonomic access without needing to name the brand's associated method.
- Documentation includes both module-level and method-level examples with working doc tests.
- The trait name clearly communicates its purpose relative to `RefFunctor`.

### Weaknesses

- **Verbose turbofish syntax:** Callers must write `send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|x: &i32| *x * 2, memo)`, which is quite heavy. This is inherent to the HKT encoding, not specific to this trait, but it is especially noticeable since the only implementors are `LazyBrand<ArcLazyConfig>` and `TryLazyBrand<E, ArcLazyConfig>`. In practice, users will likely prefer the inherent `memo.ref_map(...)` method on `ArcLazy` directly.
- **No blanket/convenience function for ArcLazy specifically:** There is no `arc_lazy_ref_map` convenience alias that hides the brand parameter.

## 4. Consistency with RefFunctor and the Send Pattern

### Structural consistency

The trait definition mirrors `RefFunctor` exactly in structure:

| Aspect | `RefFunctor` | `SendRefFunctor` |
|--------|-------------|-----------------|
| Kind bound | `type Of<'a, A: 'a>: 'a` | `type Of<'a, A: 'a>: 'a` |
| `A` bound | `A: 'a` | `A: Send + Sync + 'a` |
| `B` bound | `B: 'a` | `B: Send + 'a` |
| `func` bound | `impl FnOnce(&A) -> B + 'a` | `impl FnOnce(&A) -> B + Send + 'a` |

The only differences are the added `Send`/`Sync` bounds. This is exactly the expected pattern.

### Consistency with `Deferrable`/`SendDeferrable`

As noted in Design, `SendDeferrable` is a subtrait of `Deferrable` while `SendRefFunctor` is independent of `RefFunctor`. This is an intentional asymmetry. The documentation on both `RefFunctor` (lines 29-34) and `SendRefFunctor` (lines 32-36) explains why.

### Documentation consistency

Both traits document their laws identically (Identity and Composition), with the only difference being the `Send` qualification. Both provide law examples using their respective lazy types. `RefFunctor` has additional documentation sections ("Cache chain behavior" and "Why `FnOnce`?", lines 74-87) that `SendRefFunctor` lacks. These sections apply equally to `SendRefFunctor` and should arguably be present there too.

## 5. Limitations

### `B` does not require `Sync`

The trait bounds `B: Send + 'a` (line 102) without `Sync`. This means:
- The closure `impl FnOnce(&A) -> B + Send` can be stored in an `ArcLazy`.
- The resulting `ArcLazy<B>` wraps `Arc<LazyLock<B, ...>>`.
- For `Arc<LazyLock<B, ...>>` to be `Send`, `B` must be `Send + Sync`.
- If `B` is `Send` but not `Sync`, the resulting `ArcLazy<B>` will be `!Send`, defeating the purpose of using `ArcLazy` over `RcLazy`.

This is not a bug per se (the compiler will catch misuse at the call site), but the trait signature is slightly misleading: it suggests thread-safe mapping but produces a value that may not actually be sendable. Adding `Sync` to the `B` bound would make the guarantee explicit.

However, there may be legitimate use cases where a non-`Send` `ArcLazy<B>` is useful (e.g., shared within a single thread via `Arc` for its `Clone` semantics). Keeping the bound minimal is the conservative choice.

### No unified abstraction over RefFunctor and SendRefFunctor

Code that wants to be generic over "any type supporting ref_map" must choose one trait or the other. There is no supertype that covers both. This could be addressed with a blanket impl or a wrapper trait, but such an approach would complicate the type system for marginal benefit given the small number of implementors.

### Only two implementors

`SendRefFunctor` exists solely for `ArcLazy` and `ArcTryLazy`. No other types in the codebase implement it. This is a very narrow trait, which raises the question of whether a standalone type class is the right abstraction, versus simply having `ArcLazy::ref_map` as an inherent method. The trait form becomes valuable if:
- Users write code generic over `SendRefFunctor` (unlikely given its specificity).
- Future types implement it (plausible if new memoized `Send` types are added).

### Missing documentation sections

`SendRefFunctor` lacks the "Cache chain behavior" and "Why `FnOnce`?" sections present in `RefFunctor` (ref_functor.rs, lines 74-87). Both sections are equally applicable to `SendRefFunctor`:
- Chaining `send_ref_map` on `ArcLazy` creates an `Arc`-referenced chain with the same memory retention characteristics.
- The rationale for `FnOnce` (memoized types evaluate at most once) applies identically.

### Property tests use the inherent method, not the trait

The QuickCheck tests in `lazy.rs` (lines 2239-2276) test `ArcLazy`'s inherent `ref_map` method, not `SendRefFunctor::send_ref_map` or the `send_ref_map` free function. While the trait impl delegates directly to the inherent method (so they are effectively tested), there are no property tests that exercise the trait-level or free-function API paths.

## 6. Documentation Accuracy

### Module doc (lines 1-16)

Accurate. The example compiles and demonstrates the free function usage.

### Trait doc (lines 25-69)

- Line 26: "returning references" is slightly misleading; the function *takes* a reference (`&A`), it does not return a reference. The trait maps `&A -> B`, producing an owned `B` inside the new container.
- Lines 32-36: The explanation of why `SendRefFunctor` is separate is accurate and well-written.
- Lines 40-42: The laws are correctly stated.
- Lines 46-69: The law examples are correct and compile.

### Method doc (lines 72-105)

Accurate. The example demonstrates the associated function call syntax.

### Free function doc (lines 108-144)

Accurate and consistent with the trait method doc.

## Summary

`SendRefFunctor` is a well-designed, correctly implemented, narrowly scoped trait. Its existence as a separate trait from `RefFunctor` is justified by Rust's type system constraints around `Send`. The implementation is correct and consistent with the library's patterns.

Key areas for improvement:
1. Add "Cache chain behavior" and "Why `FnOnce`?" documentation sections (parity with `RefFunctor`).
2. Consider whether `B: Send + Sync` would be a more honest bound (or document why `Sync` is omitted).
3. Add property tests that exercise the trait method and free function directly, not just the inherent method.
4. Minor: line 26 says "returning references" but should say "receiving references."
