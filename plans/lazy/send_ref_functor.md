# Analysis: `SendRefFunctor` trait

**File:** `fp-library/src/classes/send_ref_functor.rs`

## Summary

`SendRefFunctor` is a type class for reference-based mapping with thread-safe (`Send`) function bounds. It exists as the `Send`-capable counterpart to `RefFunctor`, following the same split pattern used by `Deferrable`/`SendDeferrable` and `CloneableFn`/`SendCloneableFn`.

Currently implemented by:
- `LazyBrand<ArcLazyConfig>` (for `ArcLazy`)
- `TryLazyBrand<E, ArcLazyConfig>` (for `ArcTryLazy`)

## Design Assessment

### The trait split is justified

Keeping `RefFunctor` free of `Send` bounds allows `RcLazy` (which uses `Rc`, a `!Send` type) to implement `RefFunctor`, while `ArcLazy` implements `SendRefFunctor`. A single trait with `Send` bounds would exclude `RcLazy` entirely. The documentation explains this rationale clearly.

### Missing supertrait relationship

`SendDeferrable` has `Deferrable` as a supertrait:

```rust
pub trait SendDeferrable<'a>: Deferrable<'a> { ... }
```

`SendCloneableFn` has `CloneableFn` as a supertrait:

```rust
pub trait SendCloneableFn: CloneableFn { ... }
```

But `SendRefFunctor` does **not** have `RefFunctor` as a supertrait:

```rust
pub trait SendRefFunctor { ... }  // no `: RefFunctor`
```

This is a structural inconsistency with the rest of the library's "Send-extension" pattern. The consequences are concrete:

1. **`LazyBrand<ArcLazyConfig>` implements `SendRefFunctor` but not `RefFunctor`.** The documentation on the trait says "ArcLazy implements both RefFunctor and SendRefFunctor," but this is false in practice; there is no `impl RefFunctor for LazyBrand<ArcLazyConfig>`.

2. **Generic code written against `RefFunctor` cannot accept `ArcLazy`.** If a function takes `Brand: RefFunctor`, it can use `RcLazy` but not `ArcLazy`, which is the opposite of what you would expect from a subtyping relationship. With `SendDeferrable: Deferrable`, generic code against `Deferrable` works with both `Thunk` (via `Deferrable`) and `ArcLazy` (via `SendDeferrable` implying `Deferrable`).

3. **`TryLazyBrand<E, ArcLazyConfig>` does implement both `RefFunctor` and `SendRefFunctor`**, making this inconsistency even more confusing: the `TryLazy` family handles it correctly but the `Lazy` family does not.

**Recommendation:** Either add `RefFunctor` as a supertrait of `SendRefFunctor` (preferred, to match the library's pattern), or at minimum add a blanket or manual `impl RefFunctor for LazyBrand<ArcLazyConfig>`. The supertrait approach is cleaner and self-documenting.

### Asymmetric bounds on `A` vs `B`

The trait signature requires:

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
    func: impl FnOnce(&A) -> B + Send + 'a,
    fa: ...,
) -> ...;
```

- `A: Send + Sync` - `Sync` is needed because `ArcLazy` stores data behind `Arc<LazyLock<A>>`, and `LazyLock<A>` requires `A: Sync` for shared access via `&A`.
- `B: Send` - `B` only needs `Send` (not `Sync`) because the newly created `ArcLazy<B>` is fresh and there are no concurrent readers yet at construction time.

This asymmetry is technically correct for the current implementations. However, it is arguably over-specialized to `ArcLazy`'s internal representation. If a future implementor needs `B: Sync` (which is likely once the returned `ArcLazy<B>` is itself shared across threads), the trait would need a breaking change. In practice, `B: Send + Sync` would be no more restrictive for real-world use since most `Send` types are also `Sync`, and it would make the bounds symmetric and more future-proof.

This is a minor point; keeping `B: Send` is defensible as "minimal bounds."

### No relationship to `Functor`

Neither `RefFunctor` nor `SendRefFunctor` extends `Functor`. This is correct because `Lazy` types return `&A` (not `A`) from `evaluate()`, making them fundamentally incompatible with `Functor`'s `map` which expects ownership transfer of inner values. The documentation does not call this out, but it is an acceptable omission.

## Documentation Quality

### Accurate and thorough

- The module-level docs, trait docs, and method docs all follow the library's documentation template.
- The "Why a Separate Trait?" section is clear and well-motivated.
- Laws (identity and composition) are stated and demonstrated with runnable doctests.

### Inaccurate claim in the trait docs

The docs state:

> "ArcLazy implements both RefFunctor and SendRefFunctor"

As noted above, `LazyBrand<ArcLazyConfig>` does **not** implement `RefFunctor`. This statement is aspirational rather than factual.

### Composition law phrasing

The composition law is stated as:

> `send_ref_map(|x| f(&g(x)), fa)` evaluates to the same value as `send_ref_map(f, send_ref_map(g, fa))`

This is correct, but note that `g` in the example is `|x: &i32| *x * 2` which returns `i32`, then `f` takes `&i32`. The intermediate type changes from `&A` to `B` to `&B`, which makes the law slightly harder to follow. The doctest itself is correct and demonstrates the law properly.

## Edge Cases and Ergonomics

### Verbose turbofish syntax

Using `send_ref_map` requires specifying the brand type:

```rust
send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|x: &i32| *x * 2, memo)
```

This is a known ergonomic limitation of the HKT-via-brands approach and is consistent with the rest of the library. No improvement possible without language-level changes.

### `FnOnce` prevents reuse

The mapping function is `FnOnce`, meaning `send_ref_map` consumes the function. This is consistent with `RefFunctor::ref_map` and is correct for lazy types that evaluate at most once. A `Fn` bound would be unnecessarily restrictive on callers.

### Only two implementors

`SendRefFunctor` is implemented only for `LazyBrand<ArcLazyConfig>` and `TryLazyBrand<E, ArcLazyConfig>`. This is a narrow trait serving a specific niche (thread-safe memoized ref-mapping). The narrowness is acceptable given the library's design; not every trait needs many implementors.

## Recommendations

1. **Add `RefFunctor` as a supertrait** of `SendRefFunctor`, matching the `SendDeferrable: Deferrable` and `SendCloneableFn: CloneableFn` patterns. Then add `impl RefFunctor for LazyBrand<ArcLazyConfig>`, or derive it via a blanket impl.

2. **Fix the documentation claim** that "ArcLazy implements both RefFunctor and SendRefFunctor" to match reality, or make it true by adding the missing `RefFunctor` impl (which recommendation 1 would accomplish).

3. **Consider `B: Send + Sync`** for symmetry and future-proofing, though this is low priority and the current `B: Send` is defensible.

4. No other structural changes are needed. The file is well-organized, well-documented, and correctly implements the pattern.
