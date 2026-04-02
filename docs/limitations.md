# Limitations

## Thread Safety and Parallelism

### `Foldable` and `CloneableFn`

The `Foldable` trait and its default implementations (`fold_right`, `fold_left`) are **not thread-safe** in terms of sending the computation across threads, even when using `ArcFnBrand`. The `Foldable` trait cannot support parallel implementations (like those using `rayon`).

#### The Issue

While `fp-library` provides `ArcFnBrand` (which uses `std::sync::Arc`), the resulting function wrappers are `!Send` (not thread-safe). This means you cannot spawn a thread and pass a `fold_right` operation that uses `ArcFnBrand` into it, nor can you implement a parallel `fold_map`.

#### Root Causes

This limitation stems from the design of the `Function` and `CloneableFn` traits, which prioritize compatibility with `Rc` (single-threaded reference counting).

1.  **`CloneableFn::new` accepts non-`Send` functions:**
    The `CloneableFn` trait defines its constructor as:

    ```rust
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ...
    ```

    The input `f` is **not** required to be `Send`. This is intentional to allow `RcFnBrand` to wrap closures that capture non-thread-safe data (like `Rc` pointers). Because `ArcFnBrand` implements this same trait, it must also accept non-`Send` functions. Since it cannot guarantee the input is `Send`, it cannot wrap it in an `Arc<dyn Fn(...) + Send>`. It is forced to use `Arc<dyn Fn(...)>`, which is `!Send`.

2.  **`Function` Trait Type Constraints:**
    The `Function` trait (which `CloneableFn` extends) enforces strict type equality on its associated type:
    ```rust
    type Of<'a, A, B>: Deref<Target = dyn 'a + Fn(A) -> B>;
    ```
    This prevents `ArcFnBrand` from defining its inner type as `Arc<dyn Fn(...) + Send + Sync>`, because `dyn Fn + Send + Sync` is a different type than `dyn Fn`.

#### Consequences

- **`fold_right` / `fold_left`:** Even if you use `ArcFnBrand`, the closure created internally by these functions is `!Send`.
- **`fold_map`:** The `Foldable` trait signature for `fold_map` does not enforce `Send` on the mapping function `F`. Therefore, you cannot implement `Foldable` for a parallel data structure (e.g., using `rayon`) because parallel libraries require `Send` bounds which the trait does not provide.

#### Implemented Solution: Extension Traits

The library addresses this with extension traits that provide thread-safe capabilities without breaking existing code:

- [`SendCloneableFn`](../fp-library/src/classes/send_cloneable_fn.rs): Extends `CloneableFn` with a separate `SendOf` associated type that wraps `dyn Fn + Send + Sync`. Only implemented by `ArcFnBrand`.
- [`ParFoldable`](../fp-library/src/classes/par_foldable.rs): Parallel fold operations using `impl Fn + Send + Sync` closures directly, bypassing the `CloneableFn` abstraction for parallel paths.

This approach keeps `Function` and `CloneableFn` unchanged, cleanly separates Send capabilities as additive traits, and provides compile-time safety (only brands that can actually provide thread safety implement `SendCloneableFn`).
