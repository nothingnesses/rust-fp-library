# SendDeferrable Trait Analysis

## Overview

`SendDeferrable<'a>` is a trait for deferred lazy evaluation with thread-safe thunks. It extends `Deferrable<'a>` by adding a `Send` bound on the closure parameter. The trait lives at `fp-library/src/classes/send_deferrable.rs` and has a single method, `send_defer`, plus a corresponding free function.

**Signature:**

```rust
pub trait SendDeferrable<'a>: Deferrable<'a> {
    fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
    where
        Self: Sized;
}
```

**Implementors:**

| Type | Bounds on impl | Behavior |
|------|---------------|----------|
| `SendThunk<'a, A>` | `A: Send + 'a` | Delegates to `SendThunk::defer(f)`, which flattens via `SendThunk::new(move \|\| f().evaluate())`. |
| `TrySendThunk<'a, A, E>` | `A: Send + 'a, E: Send + 'a` | Flattens via `TrySendThunk(SendThunk::new(move \|\| f().evaluate()))`. |
| `Lazy<'a, A, ArcLazyConfig>` (ArcLazy) | `A: Clone + Send + Sync + 'a` | Creates a new `ArcLazy` via `ArcLazy::new(move \|\| f().evaluate().clone())`. |
| `TryLazy<'a, A, E, ArcLazyConfig>` (ArcTryLazy) | `A: Clone + Send + Sync + 'a, E: Clone + Send + Sync + 'a` | Creates a new `ArcTryLazy` via `Self::new(move \|\| match f().evaluate() { ... })`. |

## 1. Trait Design: Supertrait Relationship

### Strengths

The `SendDeferrable<'a>: Deferrable<'a>` supertrait approach is well-motivated and consistent with the library's conventions:

- **Liskov substitution:** Any `SendDeferrable` type can be used in code that only requires `Deferrable`. This is documented explicitly: "Every `SendDeferrable` type is also `Deferrable`, so generic code written against `Deferrable` accepts both single-threaded and thread-safe types."
- **Mirrors the `CloneableFn`/`SendCloneableFn` pattern:** The library already establishes this convention for the `Send`/non-`Send` split, so users encounter a familiar structure.
- **Differs from `RefFunctor`/`SendRefFunctor`:** The docs for `RefFunctor` explicitly explain that `SendRefFunctor` is NOT a subtrait of `RefFunctor` because `ArcLazy::new` requires `Send` on the closure, preventing a valid `RefFunctor` impl for `ArcLazy`. The `Deferrable`/`SendDeferrable` pair avoids this issue because `Deferrable::defer` takes `FnOnce() -> Self`, which does not constrain the inner closure of the type being produced, only the outer thunk itself. Each Send-capable type can satisfy `Deferrable` by eagerly calling the non-Send outer thunk, then wrapping the result.

### The Eager-Evaluation Compromise

The supertrait relationship introduces a subtle semantic consequence: `Deferrable::defer` for Send types (`SendThunk`, `ArcLazy`, `TrySendThunk`, `ArcTryLazy`) must call the thunk **eagerly** because the outer thunk `f` is not guaranteed `Send`.

- For `SendThunk`'s `Deferrable` impl: `fn defer(f) -> Self { f() }` calls `f` immediately.
- For `ArcLazy`'s `Deferrable` impl: `fn defer(f) -> Self { f() }` calls `f` immediately.
- The `SendDeferrable::send_defer` implementations, by contrast, can truly defer because the `Send` bound on the closure allows wrapping it in a new `SendThunk::new(...)` or `ArcLazy::new(...)`.

This is well-documented in the `Deferrable` trait itself (the "Warning" section) and in each eager impl's doc comment. The documentation correctly steers users toward `SendDeferrable` when they need guaranteed deferred evaluation with thread-safe types.

### Assessment

The design is sound. The supertrait relationship maintains useful subtyping at the cost of eager evaluation in the `Deferrable` fallback, which is explicitly documented and reasonable.

## 2. Method Signatures

### `send_defer` Signature

```rust
fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self
where
    Self: Sized;
```

**Correctness of bounds:**

- `FnOnce()`: Correct. Deferred computations are executed at most once. Using `Fn` or `FnMut` would be unnecessarily restrictive.
- `Send`: Correct. This is the distinguishing requirement over `Deferrable::defer`. The closure must be transferable across thread boundaries.
- Not `Sync`: Correct. The docs explicitly justify this: "this trait accepts a `FnOnce` closure that only needs to be `Send` (not `Sync`), since deferred computations are executed at most once." A `FnOnce` closure is consumed on call, so `Sync` (shared-reference thread safety) is irrelevant.
- `'a`: Correct. Matches the lifetime parameter on the trait, allowing the closure to borrow data with lifetime `'a`.
- `Self: Sized`: Standard for trait methods that return `Self` by value. Prevents calling on `dyn SendDeferrable` trait objects, which is acceptable since the library uses static dispatch throughout.

**No `Send` bound on `Self`:** The trait does not require `Self: Send`. This is correct because the trait is about constructing a value from a `Send` thunk, not about the resulting value being `Send`. Whether the constructed type is `Send` depends on the concrete type (e.g., `SendThunk` is `Send` when `A: Send`, `ArcLazy` is always `Send`).

### Free Function Signature

```rust
pub fn send_defer<'a, D: SendDeferrable<'a>>(f: impl FnOnce() -> D + Send + 'a) -> D
```

Correctly mirrors the trait method. The generic parameter `D` allows type inference to determine the concrete type.

## 3. Consistency with Library-Wide Send/non-Send Patterns

The library uses three distinct patterns for the Send/non-Send split:

| Pattern | Supertrait? | Reason |
|---------|------------|--------|
| `CloneableFn` / `SendCloneableFn` | Yes (`SendCloneableFn: CloneableFn`) | The wrapper type changes (`Rc` vs `Arc`), but both can exist for the same brand. |
| `RefFunctor` / `SendRefFunctor` | No (independent) | `ArcLazy::new` requires `Send` closure, so `ArcLazy` cannot implement `RefFunctor` (which does not impose `Send`). |
| `Deferrable` / `SendDeferrable` | Yes (`SendDeferrable: Deferrable`) | Eager evaluation fallback allows Send types to satisfy the non-Send trait. |

`SendDeferrable` follows the `SendCloneableFn` pattern (supertrait), not the `SendRefFunctor` pattern (independent). This is correct because the eager-evaluation workaround makes it possible to implement `Deferrable` for all Send types, whereas `RefFunctor` has no such workaround for `ArcLazy`.

**Naming convention** is consistent: the `Send` variant prefixes the non-Send name with `Send` and the free function prefixes with `send_`.

**Documentation cross-references** between `Deferrable` and `SendDeferrable` are thorough, with `Deferrable`'s Warning section pointing users to `SendDeferrable`, and `SendDeferrable` referencing the `SendCloneableFn: CloneableFn` pattern as precedent.

## 4. Documentation Quality

### Strengths

- **Clear purpose statement:** "A trait for deferred lazy evaluation with thread-safe thunks."
- **Good cross-references:** Links to `Deferrable`, `SendCloneableFn`, and the concrete `arc_lazy_fix` function.
- **FnOnce vs Fn justification:** Explicitly explains why `FnOnce` is used rather than `Fn`, and why `Send` but not `Sync` is required.
- **Law is stated:** Transparency law is clearly specified.
- **"Why there is no generic `fix`" section:** Correctly explains that lazy self-reference requires shared ownership and interior mutability.
- **Working examples:** Both the module-level example and the doc examples on the trait and free function compile and demonstrate real usage.
- **Uses the library's documentation macro system** (`#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`) consistently.

### Weaknesses

- **No explicit mention of the eager-evaluation tradeoff in `SendDeferrable` itself.** The `Deferrable` trait documents the Warning about eager evaluation, and each concrete impl documents it, but the `SendDeferrable` trait docs could benefit from a note explaining that `send_defer` guarantees truly deferred evaluation, contrasting with `defer` on Send types.
- **Single law only.** `Deferrable` has two QuickCheck tests (transparency and nesting). `SendDeferrable` has zero QuickCheck tests and states only the transparency law. The nesting law (`send_defer(|| send_defer(|| x))` equals `send_defer(|| x)`) is not mentioned, though it arguably follows from transparency.
- **Module-level example is narrow.** Uses only `ArcLazy`; could show `SendThunk` as well to demonstrate the trait across multiple types.

## 5. Issues and Limitations

### 5.1 No Property Tests

`Deferrable` has a `#[cfg(test)]` module with two QuickCheck properties (transparency and nesting). `SendDeferrable` has no tests at all. There are only individual unit tests inside the impl modules for `SendThunk` and `TrySendThunk`. A property-based test for `SendDeferrable` transparency across all implementors would strengthen the test suite.

### 5.2 Clone Bound Asymmetry

The `ArcLazy` `SendDeferrable` impl requires `A: Clone`, while its `Deferrable` impl does not. This is because `send_defer` calls `f().evaluate().clone()` to extract the inner value from the intermediate `ArcLazy` (which returns `&A`), whereas the `Deferrable::defer` impl just calls `f()` eagerly and returns the result as-is. The `Clone` bound is a genuine limitation: types that are `Send + Sync` but not `Clone` cannot use `send_defer` with `ArcLazy`. The same asymmetry applies to `ArcTryLazy`. This is inherent to the memoized types' design (`evaluate` returns `&A`) and not a bug, but it is worth calling out.

### 5.3 Not Used as a Bound in Generic Code

A search for `SendDeferrable` shows it is never used as a trait bound in generic functions or other traits. It is only implemented and called at concrete types. This suggests the abstraction is currently useful primarily for documentation and for a uniform API surface (`send_defer`), rather than for writing polymorphic code. If the library evolves to have generic functions that accept `SendDeferrable`, the trait will become more valuable. Currently its primary benefit is naming the pattern and providing the free function.

### 5.4 No `Evaluable` Integration

The `send_defer` implementations all call `.evaluate()` internally. There is an `Evaluable` trait in the library, but `SendDeferrable` does not require `Evaluable` as a supertrait. This is fine since `Evaluable` may have a different evaluation signature (returning `&A` vs `A`), but it means the flattening behavior in each `send_defer` impl is ad-hoc rather than expressed through the type system.

### 5.5 Trampoline and Free Do Not Implement SendDeferrable

`Trampoline<A>` and `Free<ThunkBrand, A>` implement `Deferrable<'static>` but not `SendDeferrable`. This makes sense because `Thunk` (their underlying functor) is `!Send`. If a `SendThunk`-based `Free` variant were added, it could implement `SendDeferrable`.

## 6. Alternatives Considered

### 6.1 Blanket Impl Instead of Separate Trait

One alternative: drop `SendDeferrable` and instead provide a blanket impl of `Deferrable` that adds `Send` bounds. This does not work in Rust because you cannot conditionally add bounds to an existing trait method. The separate trait is the correct approach.

### 6.2 Conditional Impls via Marker Traits

Another approach: have a single `Deferrable` trait with an associated type or const that indicates whether `Send` is required, then use conditional compilation or specialization. This is overly complex and specialization is unstable. The current two-trait approach is simpler and idiomatic.

### 6.3 Making `Deferrable::defer` Require `Send`

If `Deferrable::defer` required `Send` on the closure, `SendDeferrable` would be unnecessary. However, this would exclude `Thunk`, `RcLazy`, `TryThunk`, `Free`, and `Trampoline` from implementing `Deferrable`, since their closures are `!Send`. This is clearly worse.

### 6.4 Independent Traits (Like RefFunctor/SendRefFunctor)

Making `SendDeferrable` independent from `Deferrable` (no supertrait) would lose the ability to use Send types in `Deferrable`-bounded generic code. The supertrait approach is strictly better here because the eager-evaluation fallback makes it possible for all Send types to implement `Deferrable`.

### 6.5 A Generic `Deferrable` Parameterized by Thread Safety

Something like `Deferrable<'a, Safety = NotSend>` / `Deferrable<'a, Safety = IsSend>`. This is theoretically elegant but adds complexity to every bound site and does not compose well with Rust's trait system. The two-trait approach is more pragmatic.

## 7. Summary

`SendDeferrable` is a well-designed, focused trait that correctly extends `Deferrable` with `Send` bounds on the closure. The supertrait relationship is justified by the eager-evaluation fallback that allows Send types to satisfy `Deferrable`. The `FnOnce + Send` (not `Sync`) bound is precisely correct for single-use deferred computations.

**Key strengths:** Correct bounds, consistent naming, good documentation, follows established library patterns.

**Areas for improvement:**
1. Add QuickCheck property tests for the transparency law (and ideally the nesting law).
2. Add a note in the `SendDeferrable` trait docs explicitly contrasting guaranteed-deferred evaluation in `send_defer` with eager evaluation in `defer` for Send types.
3. Consider whether the `Clone` bound on `ArcLazy`/`ArcTryLazy` implementations could be relaxed (likely not without changing the `Lazy` evaluation model, but worth documenting the limitation in the trait docs).
