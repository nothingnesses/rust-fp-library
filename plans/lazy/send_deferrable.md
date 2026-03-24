# Analysis: `SendDeferrable` trait

**File:** `fp-library/src/classes/send_deferrable.rs`

## Summary

`SendDeferrable<'a>` is a subtrait of `Deferrable<'a>` that adds `Send + Sync` bounds on the thunk closure, enabling thread-safe lazy construction. It has a single method `send_defer` and a free function of the same name. Four types implement it: `SendThunk`, `ArcLazy`, `TrySendThunk`, and `ArcTryLazy`.

## Overall assessment

The trait is well-structured and follows the library's established patterns. The separation from `Deferrable` is justified. However, there is one clear design flaw (unnecessary `Sync` bound) and several minor issues worth addressing.

## 1. The `Sync` bound on the thunk is unnecessary

The `send_defer` signature requires `impl FnOnce() -> Self + Send + Sync + 'a`, but `Sync` is never needed by any of the four implementations:

- **`SendThunk`:** calls `SendThunk::new(move || f().evaluate())`, where `SendThunk::new` requires `FnOnce() -> A + Send + 'a` (no `Sync`).
- **`ArcLazy`:** calls `ArcLazy::new(move || f().evaluate().clone())`, where `ArcLazy::new` requires `FnOnce() -> A + Send + 'a` (no `Sync`). The underlying `ArcLazyConfig::Thunk` is `dyn FnOnce() -> A + Send + 'a`.
- **`TrySendThunk`:** calls `TrySendThunk(SendThunk::new(move || f().evaluate()))`, same as `SendThunk`.
- **`ArcTryLazy`:** calls `Self::new(move || ...)`, same as `ArcLazy`.

In all cases, the closure `f` is captured by a `move ||` closure and only used via `FnOnce`. A `FnOnce` closure that captures a `Send` value is itself `Send`; `Sync` adds nothing here. The `Sync` bound unnecessarily restricts callers, for example, closures capturing `Cell<T>` or `Rc<T>`-like values that are `Send` but not `Sync` would be rejected.

**Recommendation:** Remove `Sync` from the trait method and free function signatures, changing `Send + Sync` to just `Send`. This is a breaking change but only in the relaxing direction (all existing call sites will continue to compile).

Note: The documentation states this follows "the same supertrait pattern used by `SendCloneableFn: CloneableFn`." However, `SendCloneableFn` deals with `Fn` (multi-call), where `Sync` can matter for shared references. `SendDeferrable` deals with `FnOnce` (single-call), where `Sync` is irrelevant.

## 2. The trait separation from `Deferrable` is justified

The `Deferrable` / `SendDeferrable` split is necessary because:

- `Deferrable::defer` accepts `impl FnOnce() -> Self + 'a` (no `Send`), which is essential for types like `Thunk`, `RcLazy`, `TryThunk`, `RcTryLazy`, `Trampoline`, `Free<ThunkBrand, A>`, and `TryTrampoline` that are not `Send`.
- Thread-safe types (`SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`) implement both traits. Their `Deferrable::defer` implementations call `f()` eagerly (because the non-`Send` closure cannot be stored in a `Send` container), while their `SendDeferrable::send_defer` implementations wrap the closure lazily.

This eager-vs-lazy behavioral difference is the real motivation for the split and is well-documented in the implementation comments (e.g., "The thunk `f` is called eagerly because `Deferrable::defer` does not require `Send` on the closure").

## 3. Asymmetry in `Deferrable` implementations for `Send` types

For `SendThunk` and `TrySendThunk`, `Deferrable::defer` calls `f()` eagerly. For `ArcLazy` and `ArcTryLazy`, `Deferrable::defer` also calls `f()` eagerly. But for `RcLazy`, `Deferrable::defer` wraps the closure lazily via `RcLazy::new(move || f().evaluate().clone())`.

This is correct: `RcLazy::new` accepts any `FnOnce() -> A + 'a` (no `Send` needed), so it can wrap the non-`Send` closure lazily. The `Send` types cannot do this through the `Deferrable` interface because their constructors require `Send` closures. This asymmetry is inherent to the design and not a bug.

## 4. Missing re-export in `functions.rs`

The `send_defer` free function is not re-exported in `functions.rs`. The `generate_function_re_exports!` macro presumably scans `src/classes/` but does not pick up `send_defer`. Similarly, `defer` from `deferrable.rs` is also absent from `functions.rs`.

Despite this, the module-level examples use `functions::*` to import `send_defer`, which suggests it should be accessible from there. If the macro does auto-export these, then this is fine; if not, the examples in the module doc comment are misleading because they show `use fp_library::{brands::*, functions::*, types::*}` but `send_defer` may only be available through `classes::send_deferrable::send_defer` or `classes::*`.

**Recommendation:** Verify that `send_defer` and `defer` are actually reachable via `functions::*`. If not, either fix the re-export or fix the examples.

## 5. `SendDeferrable` is never used as a generic bound

Neither `Deferrable` nor `SendDeferrable` appear as trait bounds in any generic function or struct in the codebase (outside their own free function wrappers). Every usage is either a concrete impl block or a direct method call on a concrete type. This means the traits currently serve only as:

1. A naming convention and documentation anchor.
2. A way to provide the `send_defer` / `defer` free functions with type inference.

This is not necessarily a problem; the traits establish a conceptual vocabulary and could be used as bounds in future generic code. But it does mean the supertrait relationship `SendDeferrable: Deferrable` is currently untested in generic contexts. For example, it is untested whether `fn foo<D: SendDeferrable<'a>>(x: D)` can actually call `D::defer(...)` through the supertrait.

## 6. Documentation quality

The documentation is thorough and follows the project's conventions well:

- The transparency law is stated and demonstrated with a doc-test.
- The "Why there is no generic `fix`" section correctly explains the design rationale and points to the concrete `arc_lazy_fix` function.
- The `SendCloneableFn` analogy is mentioned for motivation.
- The `document_signature`, `document_parameters`, `document_returns`, `document_examples` macros are used consistently.

Minor issues:

- The trait doc says "following the same supertrait pattern used by `SendCloneableFn: CloneableFn`," but this analogy is imprecise as noted in finding 1 (`FnOnce` vs `Fn` semantics differ regarding `Sync`).
- The module-level example uses `ArcLazy::new(|| 42)` inside `send_defer`, which is a realistic but somewhat odd pattern (it creates a lazy value inside a deferred lazy value). A simpler example like `send_defer(|| ArcLazy::pure(42))` would be clearer, and is in fact what the trait-level example uses.

## 7. Trait bounds are nearly minimal

The trait definition itself is clean:

```rust
pub trait SendDeferrable<'a>: Deferrable<'a> {
    fn send_defer(f: impl FnOnce() -> Self + Send + Sync + 'a) -> Self
    where
        Self: Sized;
}
```

The `Sized` bound on `Self` in the where clause is necessary because `impl FnOnce() -> Self` requires a sized return type. The `Deferrable<'a>` supertrait is appropriate. The only excess is the `Sync` bound discussed in finding 1.

## 8. No blanket impl or default method

The trait has no default implementation of `send_defer` in terms of `Deferrable::defer`. This is correct because there is no sound way to provide one: you cannot call `defer(f)` when `f: impl FnOnce() + Send + Sync` because `defer` would just call `f()` eagerly for `Send` types (defeating the purpose of `send_defer`), and you cannot construct the type's internal `Send` storage from a generic `Deferrable` interface.

## Recommendations summary

| Priority | Issue | Action |
|----------|-------|--------|
| High | `Sync` bound on `send_defer` is unnecessary | Remove `Sync` from trait method and free function |
| Medium | `send_defer` may not be reachable via `functions::*` | Verify and fix re-export or fix doc examples |
| Low | `SendCloneableFn` analogy is imprecise regarding `FnOnce` vs `Fn` | Adjust or remove the comparison |
| Low | Module-level example uses nested lazy construction | Simplify to `send_defer(\|\| ArcLazy::pure(42))` |
| None | Trait separation, supertrait relationship, law, and overall design | Sound; no changes needed |
