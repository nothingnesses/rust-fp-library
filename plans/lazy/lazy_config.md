# Analysis: `lazy_config.rs`

**File:** `fp-library/src/classes/lazy_config.rs`
**Role:** Defines the `LazyConfig` trait, the strategy pattern for memoization.

## Design

`LazyConfig` bundles the pointer type, lazy cell type, and thunk type needed by `Lazy<'a, A, Config>`. It has two built-in implementations: `RcLazyConfig` (single-threaded) and `ArcLazyConfig` (thread-safe).

```rust
pub trait LazyConfig: 'static {
    type PointerBrand: crate::classes::RefCountedPointer;
    type Lazy<'a, A: 'a>: Clone;
    type Thunk<'a, A: 'a>: ?Sized;
    fn lazy_new<'a, A: 'a>(f: Box<Self::Thunk<'a, A>>) -> Self::Lazy<'a, A>;
    fn evaluate<'a, 'b, A: 'a>(lazy: &'b Self::Lazy<'a, A>) -> &'b A;
}
```

## Assessment

### Correct decisions

1. **Strategy pattern over enum dispatch.** Using a trait rather than an `enum { Rc, Arc }` enables zero-cost dispatch and extensibility. Third-party crates can implement custom configs.

2. **`'static` bound on the trait itself.** This ensures config types are marker-like (no borrowed data), which is appropriate since configs are phantom type parameters.

3. **`Clone` bound on `Lazy` associated type.** This ensures the memoized cell can be shared (via `Rc::clone` or `Arc::clone`).

4. **`?Sized` on `Thunk` associated type.** This allows `dyn FnOnce() -> A` as the thunk type, avoiding the need for a concrete closure type.

5. **`PointerBrand` associated type.** This links configs to the pointer hierarchy, enabling generic code to recover the pointer brand from a `LazyConfig`.

### Issues

#### 1. No way to create an already-evaluated lazy cell

`lazy_new` always takes a thunk (`Box<Self::Thunk<'a, A>>`). There is no `lazy_from_value` method to create a pre-initialized cell. This means `Lazy::pure(a)` must wrap the value in a closure and box it, adding unnecessary overhead for known values.

The standard library's `LazyCell` and `LazyLock` do not expose `from_value` constructors, so this limitation partly comes from the standard library. However, `LazyConfig` could add an optional method with a default implementation.

**Impact:** Low. The overhead of wrapping a value in `Box::new(move || a)` is minimal, but it prevents true zero-cost `pure` for lazy types.

#### 2. `evaluate` takes `&Self::Lazy<'a, A>` but some use cases want owned values

The trait only provides `evaluate` returning `&A`. There is no `evaluate_owned` or `into_inner` method. Types like `Thunk` can consume themselves to return `A`, but `LazyConfig::evaluate` always returns a reference. This forces `Clone` bounds wherever owned values are needed.

**Impact:** Moderate. This is the root cause of many `Clone` bounds throughout the lazy hierarchy (in `Deferrable`, `Semigroup`, `Foldable`, etc.).

#### 3. No `Send`/`Sync` bounds on associated types

`LazyConfig` does not constrain `Lazy<'a, A>` to be `Send` or `Sync`. For `ArcLazyConfig`, the resulting `Arc<LazyLock<...>>` IS `Send + Sync` (when `A: Send + Sync`), but this is not expressed in the trait. Generic code over `LazyConfig` cannot assume thread safety.

**Impact:** Low. In practice, code that needs thread safety uses `ArcLazyConfig` directly rather than abstracting over `LazyConfig`.

#### 4. Extensibility claim may be overstated

The documentation claims third-party crates can implement custom configs (e.g., `parking_lot`-based locks). However, the associated type `Lazy<'a, A>` must satisfy `Clone`, and the two methods must match specific semantics. The trait does not constrain atomicity, ordering guarantees, or panic safety, making it unclear what contract a third-party implementation must uphold.

**Impact:** Very low. Extensibility is nice to have, and the interface is simple enough that implementations would be straightforward.
