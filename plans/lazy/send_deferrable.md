# Analysis: `send_deferrable.rs`

**File:** `fp-library/src/classes/send_deferrable.rs`
**Role:** Defines `SendDeferrable<'a>` trait, the thread-safe counterpart of `Deferrable<'a>`.

## Design

`SendDeferrable<'a>` extends `Deferrable<'a>` with a `Send` bound on the thunk:

```rust
pub trait SendDeferrable<'a>: Deferrable<'a> {
    fn send_defer(f: impl FnOnce() -> Self + Send + 'a) -> Self where Self: Sized;
}
```

This follows the same supertrait pattern used by `SendCloneableFn: CloneableFn`. The `Send` bound (without `Sync`) is correct since deferred computations execute at most once (`FnOnce`), so they only need to be transferable across threads, not shareable.

## Assessment

### Correct decisions

1. **`Send` without `Sync`.** For `FnOnce` closures, `Send` alone is sufficient. This is less restrictive than requiring `Send + Sync`.

2. **Separate trait rather than conditional bound.** Making `SendDeferrable` a separate trait keeps `Deferrable` free of `Send` constraints, allowing `RcLazy` and `Thunk` to implement `Deferrable` without any thread-safety overhead.

3. **Comprehensive test coverage.** Tests cover `SendThunk`, `ArcLazy`, `TrySendThunk`, and `ArcTryLazy`, verifying both transparency and nesting laws.

### Issues

#### 1. The supertrait relationship forces eager `Deferrable` on Send types

Since `SendDeferrable: Deferrable`, types like `ArcLazy` and `SendThunk` must implement `Deferrable::defer` even though the non-`Send` closure cannot be stored. They resolve this by evaluating eagerly. If `SendDeferrable` were independent, this compromise would be unnecessary.

**Impact:** This is the same issue noted in the `deferrable.rs` analysis. The supertrait relationship is the architectural root cause.

#### 2. No law distinguishing `send_defer` from `defer`

The transparency law for `SendDeferrable` is identical to `Deferrable`'s. There is no law that says "send_defer must actually defer." This means a hypothetical implementation of `send_defer` that evaluates eagerly would still be lawful.

**Impact:** Low. In practice, all current `SendDeferrable` implementations do defer properly (since the `Send` closure can be stored). But the law does not guarantee this.

## Relationship to `Deferrable`

The two traits form a clear hierarchy:

- `Deferrable`: any type that can be constructed from a thunk (possibly eagerly).
- `SendDeferrable`: types where the thunk can be `Send`, enabling true deferral for thread-safe types.

The hierarchy is well-motivated but creates the "eager Deferrable" compromise for Send types. This is a design tension with no perfect resolution in Rust's type system.
