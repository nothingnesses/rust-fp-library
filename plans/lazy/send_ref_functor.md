# Analysis: `send_ref_functor.rs`

**File:** `fp-library/src/classes/send_ref_functor.rs`
**Role:** Thread-safe variant of `RefFunctor` for `ArcLazy`.

## Design

`SendRefFunctor` mirrors `RefFunctor` with `Send` bounds:

```rust
fn send_ref_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
    func: impl FnOnce(&A) -> B + Send + 'a,
    fa: ...,
) -> ...;
```

The `Send + Sync` bounds on `A` and `B` ensure the values can live inside an `Arc<LazyLock<...>>`.

## Assessment

### Correct decisions

1. **Independent from `RefFunctor`.** The documentation clearly explains why `SendRefFunctor` is not a subtrait: `ArcLazy::new` requires `Send` on the closure, which `RefFunctor` cannot guarantee. Conversely, `RcLazy` cannot implement `SendRefFunctor` because `Rc` is `!Send`.

2. **`Send + Sync` on `A` and `B`.** These bounds are necessary for `Arc<LazyLock<A>>` to be `Send + Sync`.

3. **`FnOnce + Send` on the mapping function.** Correct for the same reasons as `RefFunctor`, with the additional `Send` bound.

### Issues

#### 1. Same limitations as `RefFunctor`

No corresponding `SendRefApplicative` or `SendRefMonad`. The trait hierarchy stops at functor-level mapping.

**Impact:** Same as `RefFunctor`.

#### 2. `Sync` bound on `A` may be overly restrictive

The `A: Send + Sync` bound is needed for `Arc<LazyLock<A>>` to be `Send + Sync`. However, there are types that are `Send` but not `Sync` (e.g., `Cell<i32>`) that could theoretically be used in a lazy context if the `Arc` was only transferred, never shared. The `Sync` requirement excludes these types.

**Impact:** Low. In practice, most types that are `Send` are also `Sync`.

#### 3. Parallel to `RefFunctor` creates maintenance burden

The two traits have nearly identical structure, laws, and documentation. Any change to the functor laws or behavior must be replicated in both. A unified trait with conditional `Send` bounds (e.g., via a marker trait) could reduce this duplication, but Rust's type system makes this difficult.

**Impact:** Low. The duplication is manageable for two traits.
