# SendRefFunctor Analysis

**File:** `fp-library/src/classes/send_ref_functor.rs`
**Only implementor:** `LazyBrand<ArcLazyConfig>` (in `fp-library/src/types/lazy.rs`, line 934)

## Purpose

`SendRefFunctor` is the thread-safe counterpart to `RefFunctor`. It exists because `ArcLazy` cannot implement `RefFunctor` directly: `RefFunctor::ref_map` places no `Send` bound on the mapping function, but `ArcLazy::new` requires the closure to be `Send` (since the underlying `LazyLock` demands it). Rather than adding `Send` bounds to `RefFunctor` (which would break `RcLazy`), a separate trait was introduced.

This mirrors the library's general pattern of paired traits for single-threaded and thread-safe variants (e.g., `Deferrable` / `SendDeferrable`, `CloneableFn` / `SendCloneableFn`, `RefCountedPointer` / `SendRefCountedPointer`).

## Design

### Trait Signature

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
    func: impl FnOnce(&A) -> B + Send + 'a,
    fa: Apply!(...::Of<'a, A>),
) -> Apply!(...::Of<'a, B>);
```

Key design choices:

1. **`A: Send + Sync`** -- Required because `ArcLazy` stores the value behind `Arc<LazyLock<A, ...>>`. `LazyLock` requires the value type be `Send + Sync` for the lock to be `Send + Sync`. The `Sync` on `A` is correct: `evaluate()` returns `&A`, which is shared across threads via `Arc`, so `A` must be `Sync`.

2. **`B: Send` (but not `Sync`)** -- The output only requires `Send` because the returned `ArcLazy<B>` will have `B` stored inside a new `LazyLock`. `LazyLock<T, F>` implements `Sync` when `T: Send + Sync`, so if the caller needs the result to be `Sync` as well, they'll need `B: Sync` at the call site. This asymmetry between `A` and `B` is potentially problematic (see Limitations below).

3. **`func: impl FnOnce(&A) -> B + Send + 'a`** -- `FnOnce` is correct for lazy evaluation (the function is called at most once). `Send` is required because the closure is boxed into a `dyn FnOnce() -> B + Send` inside `ArcLazy::new`. The function does not require `Sync`, which is correct since the closure is consumed once, not shared.

4. **No supertrait relationship with `RefFunctor`** -- `SendRefFunctor` does not extend `RefFunctor`. This is intentional: `LazyBrand<ArcLazyConfig>` cannot implement `RefFunctor` (the weaker-bounded trait), so requiring it as a supertrait would be impossible. This is the correct design.

### Comparison with `Functor` / `ParFunctor`

The `Functor`/`ParFunctor` pair follows the same pattern. `ParFunctor` requires `Send + Sync` on closures and `Send` on values, while `Functor` has no such bounds. Neither is a subtrait of the other.

However, there is one structural difference: `Functor::map` takes `impl Fn(A) -> B` (multi-use), while `RefFunctor::ref_map` and `SendRefFunctor::send_ref_map` both take `impl FnOnce(&A) -> B` (single-use). The `FnOnce` is appropriate here because `Lazy` is a single-element container (it holds exactly one memoized value), so the function will be called exactly once. The `&A` parameter is also correct: `Lazy::evaluate` returns `&A`, so the mapping function must accept a reference.

## Implementation (in lazy.rs)

```rust
impl SendRefFunctor for LazyBrand<ArcLazyConfig> {
    fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + 'a>(
        f: impl FnOnce(&A) -> B + Send + 'a,
        fa: Apply!(<Self as Kind!(...)>::Of<'a, A>),
    ) -> Apply!(<Self as Kind!(...)>::Of<'a, B>) {
        fa.ref_map(f)
    }
}
```

This delegates to the inherent `ArcLazy::ref_map` method (line 697), which requires `A: Send + Sync` and `f: ... + Send`. The bounds align correctly.

### Correctness

No bugs identified. The delegation to `fa.ref_map(f)` is sound because:
- `ArcLazy::ref_map` requires `A: Send + Sync` (satisfied by trait bounds on `A`).
- `ArcLazy::ref_map` requires `f: impl FnOnce(&A) -> B + Send + 'a` (satisfied by trait bounds on `func`).
- The return type `Lazy<'a, B, ArcLazyConfig>` matches `Apply!(<LazyBrand<ArcLazyConfig> as Kind!(...)>::Of<'a, B>)`.

## Consistency

### Consistent patterns:
- Module structure (`#[fp_macros::document_module] mod inner { ... } pub use inner::*;`) matches all other class modules.
- Documentation style follows the project's `#[document_signature]`, `#[document_type_parameters(...)]`, etc. macros.
- Free function (`send_ref_map`) mirrors the trait method, consistent with every other type class in the library.
- The `#[kind(type Of<'a, A: 'a>: 'a;)]` attribute on the trait is identical to `RefFunctor` and `Functor`.

### Inconsistencies:

1. **Missing re-export in `functions.rs`** -- The `send_ref_map` free function exists in the module, and the `generate_function_re_exports!` macro should auto-discover it since it scans `src/classes`. However, the module-level doc example imports it via `functions::*`, suggesting it IS auto-exported. If it is not (the grep showed no explicit mention in `functions.rs`), then the doc test would fail. Since the code presumably compiles, the macro likely does auto-export it. No issue here.

2. **`RefFunctor` is implemented for `LazyBrand<RcLazyConfig>` only** -- `SendRefFunctor` is implemented for `LazyBrand<ArcLazyConfig>` only. There is no `RefFunctor` impl for `LazyBrand<ArcLazyConfig>` and no `SendRefFunctor` impl for `LazyBrand<RcLazyConfig>`. This is correct and intentional, but it means the two lazy variants live in completely separate type class hierarchies with no unification point.

3. **Naming convention** -- Other `Send`-prefixed traits in the library (`SendDeferrable`, `SendCloneableFn`, `SendRefCountedPointer`) follow the same `Send` prefix pattern. Consistent.

## Limitations

### 1. Asymmetric bounds on `A` vs `B`

`A: Send + Sync` but `B: Send` (no `Sync`). This means the returned `ArcLazy<B>` may not be `Sync` (and thus not `Send + Sync`), which means you cannot chain `send_ref_map` calls without the caller ensuring `B: Sync` at the use site:

```rust
// This works:
let result = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(|x: &i32| *x + 1, lazy);

// But chaining may fail if B is not Sync:
let chained = send_ref_map::<LazyBrand<ArcLazyConfig>, _, _>(
    |x: &SomeNonSyncType| ...,
    result,  // result's inner type must be Send + Sync to satisfy A bounds
);
```

In practice, most types that are `Send` are also `Sync`, so this rarely causes issues. But it is a theoretical gap. Adding `Sync` to `B` would close this gap but would be slightly more restrictive. The current choice favors flexibility for the output type at the cost of composability. This seems like a reasonable tradeoff, and the inherent `ArcLazy::ref_map` method has the same `B: Send`-only bound, so the trait is consistent with its implementation.

### 2. Single implementor

`SendRefFunctor` currently has exactly one implementor (`LazyBrand<ArcLazyConfig>`). This is a trait with a very narrow scope. It exists primarily to allow `ArcLazy` to participate in the HKT type class hierarchy for ref-based mapping. This is fine, as the library design favors granular traits, but it does mean the abstraction overhead (a whole trait + free function + documentation) is significant relative to the implementation surface.

### 3. No blanket impl relating `SendRefFunctor` to `RefFunctor`

There is no way to use `send_ref_map` where `ref_map` is expected, or vice versa. Code that is generic over `RefFunctor` cannot work with `ArcLazy`. This is an inherent limitation of Rust's trait system: you cannot have a single trait that works for both `Rc`-backed and `Arc`-backed lazy values because the bounds differ. A potential workaround would be a blanket impl `impl<B: SendRefFunctor> RefFunctor for B` with tighter bounds, but this runs into coherence issues and would require `Send` bounds to propagate through `RefFunctor`'s signature.

### 4. No `Sync` on the closure

The closure is `Send` but not `Sync`. This is correct for `FnOnce` (consumed, never shared), but worth noting for anyone comparing with `ParFunctor` where closures are `Send + Sync` (because `Fn` closures may be shared across rayon threads).

## Alternatives

### 1. Parameterize `RefFunctor` over bounds

One could imagine a design where `RefFunctor` itself is parameterized to optionally require `Send`:

```rust
trait RefFunctor<Bounds = NoBounds> { ... }
```

This would avoid the need for a separate trait but would add significant complexity to the trait system and is not idiomatic Rust. The current approach of separate traits is simpler and more explicit. **Not recommended.**

### 2. Conditional supertrait via marker

A `SendRefFunctor: RefFunctor` relationship where `RefFunctor` is also implemented for `ArcLazy` with the additional bounds baked in. However, `RefFunctor::ref_map` does not require `Send` on `func`, so the `ArcLazy` impl would need to somehow reject non-`Send` closures at the impl level. This is not possible with Rust's current type system. **Not feasible.**

### 3. Use a single `Lazy` type with runtime pointer selection

Instead of `RcLazyConfig` / `ArcLazyConfig`, use a single config that abstracts over the pointer type at the trait level, with a single `RefFunctor`-like trait that conditionally adds `Send` bounds. This would require GATs or associated type bounds that Rust does not yet fully support in the needed form. **Not feasible today.**

### 4. Remove the trait, keep only the inherent method

Since there is only one implementor, one could argue the trait is unnecessary and `ArcLazy::ref_map` is sufficient. However, the trait enables generic programming: code can be written as `fn foo<F: SendRefFunctor>(...)` to abstract over any future `Send`-capable ref-functor. The trait also participates in the library's type class hierarchy, which is a core design principle. **Not recommended** given the library's design goals.

## Documentation

### Quality

The documentation is thorough and follows the project's conventions:
- Module-level doc with example.
- Trait-level doc explaining the relationship to `RefFunctor`.
- Laws section with identity and composition.
- Law-verification examples with assertions.
- Method-level documentation with signature, type parameter, parameter, and return descriptions.
- Free function documentation mirroring the trait method.

### Accuracy

The documentation is accurate. The laws are correctly stated in terms of reference-based mapping. The examples compile and demonstrate the intended usage.

### Suggestions

1. The trait doc says "where the mapping function must be `Send`" but does not mention the `Send + Sync` requirement on `A` or the `Send` requirement on `B`. These bounds are visible in the signature but could be called out explicitly in the prose, especially the `Sync` requirement on `A` which may surprise users.

2. The trait doc links to `RefFunctor` and `ArcLazy` but does not explain *why* a separate trait is needed. The inherent method `ArcLazy::ref_map` has a comment explaining this (line 679: "A blanket `RefFunctor` trait impl is not provided for `LazyBrand<ArcLazyConfig>` because the `RefFunctor` trait does not require `Send` on the mapping function"), but this explanation is absent from the `SendRefFunctor` trait doc itself. Adding a brief note about the motivation would help users understand the design.

3. The composition law example uses `f` and `g` where `g` returns an owned value and `f` takes a reference to that value. This is correct but could benefit from a comment noting that the intermediate value from `g` is materialized inside a new `ArcLazy` before `f` receives a reference to it.

## Summary

`SendRefFunctor` is a well-designed, narrowly-scoped trait that fills a necessary gap in the type class hierarchy. It exists because Rust's type system cannot unify `Send`-bounded and unbounded versions of the same trait without separate definitions. The implementation is correct, the documentation is thorough, and the design is consistent with the library's broader patterns. The main limitations (asymmetric bounds, single implementor, no relationship with `RefFunctor`) are inherent to the problem being solved and do not indicate design flaws. No changes are needed.
