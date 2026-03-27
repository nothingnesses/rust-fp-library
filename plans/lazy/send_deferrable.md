# SendDeferrable Analysis

**File:** `fp-library/src/classes/send_deferrable.rs`

## Overview

`SendDeferrable<'a>` is a trait for types that support deferred lazy evaluation with thread-safe (`Send`) thunks. It extends `Deferrable<'a>` as a supertrait, adding a `send_defer` method that requires the closure to be `Send`. The module also provides a free function `send_defer` that dispatches to the trait method.

## 1. Design

### Supertrait relationship

The `SendDeferrable<'a>: Deferrable<'a>` supertrait bound (line 64) is well-motivated. It mirrors the `SendCloneableFn: CloneableFn` pattern used elsewhere in the codebase (documented in the trait doc at line 26). The key property this enables: any generic code written against `Deferrable` accepts both single-threaded and thread-safe types. This is the standard Rust idiom for "progressive capability," where thread safety is an opt-in extension.

However, the supertrait design creates a practical tension. For `SendThunk`, the `Deferrable::defer` implementation (at `send_thunk.rs:422-427`) must evaluate the thunk eagerly (`f()`) because the `Deferrable` trait signature does not require `Send` on the closure, and `SendThunk` internally requires `Send`. This means calling `defer` on a `SendThunk` loses laziness, which undercuts the "transparency" law that `Deferrable` promises. The `Deferrable` doc (at `deferrable.rs:39-45`) mitigates this with a warning, and the transparency law at `deferrable.rs:23-24` is carefully worded to say it "does not constrain when evaluation occurs." Still, this is an inherent awkwardness of the supertrait design: a `SendDeferrable` type's `Deferrable` implementation may behave less lazily than expected.

The same tension applies to `ArcLazy`'s `Deferrable::defer` (`lazy.rs:884-888`), which also evaluates eagerly (`f()`), while its `SendDeferrable::send_defer` (`lazy.rs:921-924`) defers properly.

### Alternative: no supertrait

An alternative would be to make `SendDeferrable` standalone (not a supertrait of `Deferrable`). This would avoid the awkward eager-`defer` implementations. But it would also mean generic code could not accept both thread-safe and single-threaded types through a single bound, which is a significant ergonomic loss. The current design is the better trade-off.

### Verdict

The supertrait design is sound and consistent with the library's established patterns. The eager-evaluation compromise in `Deferrable` impls for `Send` types is well-documented.

## 2. Implementation Quality

### Trait definition (lines 64-86)

The trait is minimal, containing a single method:

```rust
fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
where
    Self: Sized;
```

This is correct. The `FnOnce` bound (rather than `Fn`) is appropriate for computations that execute at most once. The `Send` bound (without `Sync`) is justified and explicitly documented at lines 31-34: deferred computations are executed at most once, so they do not need `Sync`. This is a meaningful distinction from `SendCloneableFn`, which wraps multi-use `Fn` closures and requires `Send + Sync`.

### Free function (lines 113-115)

```rust
pub fn send_defer<'a, D: SendDeferrable<'a>>(f: impl FnOnce() -> D + Send + 'a) -> D {
    D::send_defer(f)
}
```

Correct, minimal delegation. Follows the same pattern as `defer` in `deferrable.rs:111-113`.

### Implementors

Four types implement `SendDeferrable`:

1. **`SendThunk`** (`send_thunk.rs:433-457`): Delegates to the inherent `SendThunk::defer` (line 455), which calls `SendThunk::new(move || f().evaluate())` (line 150-152). This is correctly lazy; the outer thunk captures `f` and defers both the outer thunk call and the inner evaluation. Requires `A: Send`.

2. **`ArcLazy`** (`lazy.rs:895-924`): Creates a new `ArcLazy` via `ArcLazy::new(move || f().evaluate().clone())`. This is correctly lazy (deferred until first access). Requires `A: Clone + Send + Sync`.

3. **`TrySendThunk`** (`try_send_thunk.rs:880`): Fallible counterpart of `SendThunk`.

4. **`ArcTryLazy`** (`try_lazy.rs:1362`): Fallible counterpart of `ArcLazy`.

All implementations are consistent and correct.

### Potential issue: `SendThunk::send_defer` method resolution

At `send_thunk.rs:455`, the call `SendThunk::defer(f)` resolves to the *inherent* method `SendThunk::defer` (line 150), not the `Deferrable::defer` trait method (line 422), because inherent methods take priority in Rust's method resolution. This is the correct behavior, but it relies on an implicit resolution rule. A more explicit approach would be `SendThunk::new(move || f().evaluate())` directly, avoiding any ambiguity. This is a minor style concern, not a bug.

## 3. API Surface

### Completeness

The API surface is intentionally minimal: one trait method and one free function. This is appropriate. `SendDeferrable` is a capability trait, not a rich interface. The actual computation-building API lives on the concrete types (`SendThunk::map`, `SendThunk::bind`, etc.).

### Discoverability

The free function `send_defer` is auto-exported via the `generate_function_re_exports!` macro in `functions.rs:25-35`, making it available through `use fp_library::functions::*`. The module-level example at lines 6-13 demonstrates usage with this import path. This is good.

### Missing: no `send_defer` in `functions.rs` re-exports list

The `generate_function_re_exports!` macro scans `src/classes/` and automatically finds free functions, so `send_defer` does not need a manual entry. This is fine; the macro handles it.

## 4. Consistency

### With `Deferrable`

The trait structure exactly mirrors `Deferrable`:

| Aspect | `Deferrable` | `SendDeferrable` |
|---|---|---|
| Trait method | `defer(f: impl FnOnce() -> Self + 'a)` | `send_defer(f: impl FnOnce() -> Self + Send + 'a)` |
| Free function | `defer<'a, D>(f)` | `send_defer<'a, D>(f)` |
| Module pattern | `document_module` + `pub use inner::*` | Same |
| Law | Transparency | Transparency (same) |

The only difference is the `+ Send` bound on the closure. This is exactly what one would expect.

### With the Send pattern elsewhere

The library uses a consistent pattern for thread-safe extensions:

- `CloneableFn` / `SendCloneableFn` (referenced in doc, line 26)
- `RefFunctor` / `SendRefFunctor`
- `Deferrable` / `SendDeferrable`
- `RefCountedPointer` / `SendRefCountedPointer`

`SendDeferrable` follows this established convention.

### Naming

The method is named `send_defer` rather than `defer` (which would shadow the supertrait method). This avoids ambiguity and is consistent with `SendCloneableFn` which uses `SendOf` as its associated type name rather than `Of`.

## 5. Limitations

### No blanket impl

There is no blanket `impl<T: SendDeferrable<'a>> Deferrable<'a> for T`. This is correct; each type provides its own `Deferrable` impl (some eager, some not), so a blanket impl would be wrong. But it does mean every `SendDeferrable` implementor must also manually implement `Deferrable`, which is a small maintenance burden.

### `FnOnce` prevents retry/multi-evaluation

The `FnOnce` bound means the deferred computation can only be produced once. If the thunk fails (e.g., panics), there is no way to retry. This is inherent to `FnOnce` and is the correct trade-off for move semantics.

### No `Sync` bound on the trait itself

`SendDeferrable` only requires the closure to be `Send`, not `Sync`. The trait itself has no `Send` or `Sync` bound on `Self`. This means a `SendDeferrable` type is not necessarily `Send` itself (e.g., `ArcLazy` is `Send + Sync` because of `Arc`, but the trait does not guarantee this). This is fine; the trait only governs construction, not the thread-safety of the resulting value.

### `'static` limitation for some implementors

`Trampoline<A>` (which is `Free<ThunkBrand, A>`) is listed in the CLAUDE.md table as implementing `Deferrable` but is absent from `SendDeferrable` implementors. This is because `Trampoline` requires `'static` and is not `Send`. This limitation is inherent to the type, not to `SendDeferrable`.

### No default implementation

The trait provides no default `send_defer` in terms of `defer`. This is correct because `defer` has weaker bounds (no `Send`), so it cannot delegate to `send_defer`, and going the other direction (`send_defer` delegating to `defer`) would lose the `Send` guarantee.

## 6. Documentation

### Trait doc (lines 22-63)

Thorough and accurate. Specifically:

- **Lines 24-26:** Correctly describes the supertrait relationship and cites the analogous `SendCloneableFn: CloneableFn` pattern.
- **Lines 28-29:** Correctly states that `Deferrable` code accepts both single-threaded and thread-safe types.
- **Lines 31-34:** Important clarification that `FnOnce` only needs `Send` (not `Sync`), with a clear rationale. This is a notable design decision that deserves this level of documentation.
- **Lines 38-39:** The transparency law is stated. It mirrors `Deferrable`'s law.
- **Lines 41-47:** The "Why there is no generic `fix`" section correctly explains that lazy self-reference requires shared ownership and interior mutability, pointing to `arc_lazy_fix` as the concrete solution. This mirrors the equivalent section in `Deferrable`'s doc (`deferrable.rs:28-37`).
- **Lines 51-63:** The example demonstrates the transparency law with `ArcLazy`.

### Module doc (lines 1-14)

The module-level example uses `ArcLazy` and demonstrates the `send_defer` free function. Accurate.

### Free function doc (lines 88-115)

Correctly describes itself as a dispatch wrapper. Includes type parameter and parameter documentation.

### Minor documentation observations

- The transparency law at line 39 says `send_defer(|| x)` is "observationally equivalent to `x` when evaluated." This matches `Deferrable`'s wording. However, unlike `Deferrable`, which warns that some implementations may evaluate eagerly, `SendDeferrable` does not include this caveat. This is correct since `SendDeferrable` implementations are expected to be truly lazy (the whole point of the trait is to provide the `Send` bound that enables true laziness for types like `ArcLazy` and `SendThunk`).

## Summary

`SendDeferrable` is a well-designed, minimal trait that follows the library's established patterns for thread-safe extensions. The supertrait relationship with `Deferrable` is sound, the implementation is correct across all four implementors, and the documentation is thorough. The main inherent tension (eager evaluation in `Deferrable` impls for `Send` types) is a necessary trade-off that is well-documented. No bugs or significant design issues were found.
