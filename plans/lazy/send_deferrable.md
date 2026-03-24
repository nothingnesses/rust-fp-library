# Analysis: `SendDeferrable` Trait

**File:** `fp-library/src/classes/send_deferrable.rs`

## Overview

`SendDeferrable<'a>` is the thread-safe counterpart to `Deferrable<'a>`. It adds `Send + Sync` bounds to the thunk parameter, enabling deferred construction of values that can be shared across threads. It is implemented by `Lazy<'a, A, ArcLazyConfig>` (alias `ArcLazy`) and `TryLazy<'a, A, E, ArcLazyConfig>` (alias `ArcTryLazy`).

## 1. Design

### Motivation

The trait is well-motivated. Rust's `Send`/`Sync` bounds are not additive after the fact; you cannot retroactively require a closure to be `Send` if the trait signature does not demand it. Since `Deferrable::defer` accepts `impl FnOnce() -> Self + 'a` without any thread-safety bounds, a separate trait is needed for thread-safe deferred construction. This mirrors the library's established pattern with `SendRefFunctor`, `SendCloneableFn`, and `SendRefCountedPointer`.

### Relationship to PureScript

PureScript's `Control.Lazy` class has a single `defer :: (Unit -> l) -> l` method and a generic `fix` combinator. In PureScript, there is no concept of thread safety, so there is no `Send` variant. The Rust `Deferrable`/`SendDeferrable` split is a necessary adaptation to Rust's ownership model. The `fix` combinator is intentionally omitted from both Rust traits (see `Deferrable`'s documentation for the rationale), with concrete `rc_lazy_fix`/`arc_lazy_fix` functions provided instead.

### Relationship to `Deferrable`

`SendDeferrable` is a standalone trait with no supertrait relationship to `Deferrable`. This is a deliberate design choice that diverges from the pattern used by some other `Send*` traits:

- `SendCloneableFn: CloneableFn` (has supertrait).
- `SendRefCountedPointer: RefCountedPointer` (has supertrait).
- `SendRefFunctor` (no supertrait, standalone).

`SendDeferrable` follows the `SendRefFunctor` pattern. Both operate on concrete types rather than brand-parameterized HKT, which makes the supertrait relationship less natural. However, see the "Alternatives" section for discussion.

## 2. Implementation

### Correctness

The trait definition and free function are correct. No bugs or subtle issues are present.

The trait method signature is:
```rust
fn send_defer(f: impl FnOnce() -> Self + Send + Sync + 'a) -> Self
where
    Self: Sized;
```

This correctly constrains the closure to be `Send + Sync` while preserving the lifetime bound `'a`. The `Sized` bound is appropriate since the return type is `Self`.

### Implementations

Both implementations (`ArcLazy` and `ArcTryLazy`) follow the same pattern as their `Deferrable` counterparts, with the addition of `Send + Sync` bounds on the type parameters. The implementation bodies (`ArcLazy::new(move || f().evaluate().clone())`) are semantically identical to the `Deferrable` versions, which is correct.

### Free function

The free function `send_defer` correctly delegates to `D::send_defer(f)`. It follows the standard library pattern for free functions.

## 3. Consistency

### Consistent patterns

- Module structure uses the `#[fp_macros::document_module] mod inner { ... } pub use inner::*;` pattern, matching all other trait modules.
- Documentation uses the standard `#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` macro annotations.
- The free function is defined alongside the trait, following the library convention.
- The `send_` prefix naming convention matches `send_ref_map`, `send_cloneable_fn_new`, etc.

### Inconsistencies

1. **No re-export alias in `functions.rs`.** The `generate_function_re_exports!` macro should auto-discover `send_defer` from `send_deferrable.rs` and re-export it in `functions.rs`. The doc examples use `functions::*` and call `send_defer`, suggesting this works. However, `defer` (from `deferrable.rs`) is also not listed as a manual alias in `functions.rs`, so both appear to be auto-discovered. This is consistent.

2. **Missing laws documentation.** `Deferrable` documents a "Transparency" law (`defer(|| x)` is observationally equivalent to `x` when evaluated). `SendDeferrable` does not document any laws. The same transparency law should apply, and the `TryLazy` property test (`deferrable_transparency`) already tests it. The trait doc should state this law explicitly.

3. **Missing `#[document_examples]` on the trait.** `Deferrable` has `#[document_examples]` on the trait itself with a transparency law example. `SendDeferrable` lacks this. Minor but noticeable.

## 4. Limitations

### Inherent limitations

- **No supertrait on `Deferrable`.** A type implementing `SendDeferrable` does not automatically provide `Deferrable`, meaning generic code cannot call `defer` on a `SendDeferrable` type. This is inherent to the design choice but could be revisited (see Alternatives).
- **Only two implementors.** Currently only `ArcLazy` and `ArcTryLazy` implement `SendDeferrable`. Types like `Thunk` and `Trampoline` do not have `Send` variants, so this is expected given the current type landscape.
- **No blanket impl.** There is no blanket `impl<T: SendDeferrable> Deferrable for T`, meaning `ArcLazy` must implement both traits separately. The `ArcLazy` implementation does implement `SendDeferrable` but not `Deferrable`, so you cannot use `defer` with `ArcLazy`. This is likely intentional (forcing users to use `send_defer` to make thread-safety visible), but it could surprise users.

### Addressable limitations

- **`ArcLazy` does not implement `Deferrable`.** Looking at the implementations, `Deferrable` is only implemented for `Lazy<'a, A, RcLazyConfig>` and `SendDeferrable` is only implemented for `Lazy<'a, A, ArcLazyConfig>`. This means `ArcLazy` cannot be used in generic code that requires `Deferrable`. If `SendDeferrable` had `Deferrable` as a supertrait, this would be resolved. Alternatively, `ArcLazy` could implement both traits.

## 5. Alternatives

### Supertrait approach

Making `SendDeferrable: Deferrable` would allow `ArcLazy` to be used in generic `Deferrable` contexts while also providing the `send_defer` method for thread-safe contexts. The implementation would be:

```rust
pub trait SendDeferrable<'a>: Deferrable<'a> {
    fn send_defer(f: impl FnOnce() -> Self + Send + Sync + 'a) -> Self
    where
        Self: Sized;
}
```

This would require `ArcLazy` to also implement `Deferrable`, which is straightforward (the closure just would not require `Send + Sync`). The benefit is that any `SendDeferrable` type can be used where `Deferrable` is expected.

The downside: `Deferrable::defer` on an `ArcLazy` would accept non-`Send` closures, which could lead to surprising runtime behavior if the user expects thread safety. However, `ArcLazy::new` already accepts non-`Send` closures in its inherent method, so this is consistent.

This mirrors the `SendCloneableFn: CloneableFn` pattern already in the library.

### Unified trait with conditional bounds

An alternative would be a single `Deferrable` trait parameterized by the pointer brand, but this would require significant refactoring and adds complexity without clear benefit.

### Status quo justification

The current standalone approach is simpler and makes the `Send`/non-`Send` distinction explicit at the trait level. It avoids accidental misuse where someone passes a non-`Send` closure to `defer` on a type intended for cross-thread use. This is a reasonable trade-off.

## 6. Documentation

### Strengths

- The trait doc clearly states the relationship to `Deferrable`.
- Method documentation follows the standard template with signature, parameters, and return value annotations.
- Doc examples compile and are representative of real usage.
- The free function doc correctly cross-references the trait method.

### Weaknesses

- **No laws section.** The transparency law from `Deferrable` applies equally here and should be documented. The `TryLazy` tests already verify this law for `ArcTryLazy`.
- **No discussion of when to use `SendDeferrable` vs `Deferrable`.** A brief note explaining the choice criteria would help users.
- **The trait-level doc lacks `#[document_examples]`.** Unlike `Deferrable`, there is no trait-level example demonstrating the transparency law.
- **Missing `fix` discussion.** `Deferrable` has a detailed "Why there is no generic `fix`" section and references `arc_lazy_fix`. `SendDeferrable` does not mention `fix` at all, even though `arc_lazy_fix` is directly relevant to `SendDeferrable` types.

## Summary of Recommendations

| Priority | Item |
|----------|------|
| Medium | Add transparency law documentation to the trait. |
| Medium | Add a note about `arc_lazy_fix` for self-referential construction, mirroring `Deferrable`'s `fix` discussion. |
| Low | Add `#[document_examples]` with a transparency law example to the trait itself. |
| Low | Consider whether `ArcLazy` should also implement `Deferrable` (and whether `SendDeferrable` should have `Deferrable` as a supertrait), following the `SendCloneableFn: CloneableFn` precedent. This is a design decision with trade-offs in both directions. |
| Low | Add a brief usage note distinguishing when to prefer `send_defer` over `defer`. |
