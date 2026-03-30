# Analysis: `send_thunk.rs`

**File:** `fp-library/src/types/send_thunk.rs`
**Role:** `SendThunk<'a, A>`, a thread-safe variant of `Thunk` with `Send` on the closure.

## Design

`SendThunk<'a, A>` is a newtype over `Box<dyn FnOnce() -> A + Send + 'a>`. It mirrors `Thunk` but adds a `Send` bound on the inner closure, enabling cross-thread transfer of unevaluated computations.

Properties:

- `Send` (auto-derived from `Box<dyn FnOnce() + Send>`).
- NOT `Sync` (`FnOnce` cannot be shared; only transferred).
- Lifetime-polymorphic like `Thunk`.
- Single-use like `Thunk`.

## Trait Implementations

**HKT (via `SendThunkBrand`):** Foldable, FoldableWithIndex, WithIndex only.

**Not implemented at HKT level (documented):** Functor, Pointed, Semimonad, Semiapplicative, MonadRec, Extract, Extend, Lift, ApplyFirst, ApplySecond. These are excluded because the HKT trait signatures do not require `Send` on closure parameters, so closures passed through the trait would violate the `Send` invariant.

**Value-level:** Deferrable (eager), SendDeferrable (truly deferred), Semigroup, Monoid, Debug.

## Comparison with Thunk

| Aspect            | Thunk                         | SendThunk                            |
| ----------------- | ----------------------------- | ------------------------------------ |
| Inner type        | `Box<dyn FnOnce() -> A + 'a>` | `Box<dyn FnOnce() -> A + Send + 'a>` |
| Send              | No                            | Yes                                  |
| HKT Functor/Monad | Full                          | None                                 |
| Deferrable        | Truly deferred                | Eager                                |
| SendDeferrable    | N/A                           | Truly deferred                       |

The conversion `SendThunk -> Thunk` is zero-cost (unsizing coercion). The reverse `Thunk -> SendThunk` requires eager evaluation (closure is not `Send`).

## Issues

### 1. `Deferrable::defer` eagerly evaluates

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self { f() }
```

The `Deferrable` trait's closure is not `Send`, so `SendThunk` cannot store it. The implementation evaluates `f()` immediately and returns the result. This satisfies the transparency law but violates the spirit of deferral.

**Impact:** Moderate. Generic code using `Deferrable::defer` gets eager evaluation for `SendThunk`, which may be surprising. The `SendDeferrable::send_defer` method provides the correct behavior.

### 2. No HKT trait support limits generic programming

`SendThunk` cannot participate in generic code written against `Functor`, `Monad`, etc. Users must use inherent methods (`map`, `bind`) directly. This is a fundamental limitation of the HKT trait design (no `Send` bounds on closure parameters).

**Impact:** Moderate. This means `SendThunk` is useful as a concrete type but cannot be used in generic FP abstractions.

### 3. `tail_rec_m` documentation incorrectly mentions `Clone`

The doc comment says "The function f must implement Clone because each iteration may need its own copy." However, the actual signature requires `impl Fn(S) -> ...`, which is already multi-callable. `Fn` closures do not need `Clone`; they are callable by shared reference. The `Clone` bound is not present in the signature.

**Impact:** Low. Documentation inaccuracy.

### 4. `arc_tail_rec_m` requires `Sync` on the closure unnecessarily

The signature requires `impl Fn(S) -> ... + Send + Sync + 'a` because `Arc<T>: Send` requires `T: Send + Sync`. However, the `Arc` is never actually shared across threads in the implementation; it is used solely to enable `Fn` semantics for non-`Clone` closures via `Arc::clone`. The `Sync` bound is a consequence of `Arc`'s design, not the algorithm's needs.

**Impact:** Low. Minor ergonomic cost.

### 5. No `into_rc_lazy` method

`Thunk` has both `into_rc_lazy` and `into_arc_lazy`. `SendThunk` only has `into_arc_lazy`. Users wanting `RcLazy` must convert to `Thunk` first. This is logically correct (converting a `Send` type to a non-`Send` type is a downgrade) but creates an asymmetry.

**Impact:** Low.

## Strengths

- Clean `Send` semantics without `Sync`.
- Zero-cost conversion to `Thunk` (unsizing coercion).
- Inherent `map`, `bind`, `tail_rec_m`, `arc_tail_rec_m` provide full functionality despite no HKT.
- Well-documented rationale for the HKT exclusion.
