# Limitations

## Thread Safety and Parallelism

### `Foldable` and `ClonableFn`

Currently, the `Foldable` trait and its default implementations (`fold_right`, `fold_left`) are **not thread-safe** in terms of sending the computation across threads, even when using `ArcFnBrand`. Furthermore, the `Foldable` trait cannot support parallel implementations (like those using `rayon`).

#### The Issue

While `fp-library` provides `ArcFnBrand` (which uses `std::sync::Arc`), the resulting function wrappers are `!Send` (not thread-safe). This means you cannot spawn a thread and pass a `fold_right` operation that uses `ArcFnBrand` into it, nor can you implement a parallel `fold_map`.

#### Root Causes

This limitation stems from the design of the `Function` and `ClonableFn` traits, which prioritize compatibility with `Rc` (single-threaded reference counting).

1.  **`ClonableFn::new` accepts non-`Send` functions:**
    The `ClonableFn` trait defines its constructor as:
    ```rust
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ...
    ```
    The input `f` is **not** required to be `Send`. This is intentional to allow `RcFnBrand` to wrap closures that capture non-thread-safe data (like `Rc` pointers). Because `ArcFnBrand` implements this same trait, it must also accept non-`Send` functions. Since it cannot guarantee the input is `Send`, it cannot wrap it in an `Arc<dyn Fn(...) + Send>`. It is forced to use `Arc<dyn Fn(...)>`, which is `!Send`.

2.  **`Function` Trait Type Constraints:**
    The `Function` trait (which `ClonableFn` extends) enforces strict type equality on its associated type:
    ```rust
    type Of<'a, A, B>: Deref<Target = dyn 'a + Fn(A) -> B>;
    ```
    This prevents `ArcFnBrand` from defining its inner type as `Arc<dyn Fn(...) + Send + Sync>`, because `dyn Fn + Send + Sync` is a different type than `dyn Fn`.

#### Consequences

*   **`fold_right` / `fold_left`:** Even if you use `ArcFnBrand`, the closure created internally by these functions is `!Send`.
*   **`fold_map`:** The `Foldable` trait signature for `fold_map` does not enforce `Send` on the mapping function `F`. Therefore, you cannot implement `Foldable` for a parallel data structure (e.g., using `rayon`) because parallel libraries require `Send` bounds which the trait does not provide.

#### Proposed Solutions

The following solutions are ordered by their effectiveness in addressing the thread safety limitation while minimizing code duplication and maintaining ergonomics.

##### Solution 1: Pure Extension Trait

This solution avoids breaking changes to the `Function` trait by relying solely on the extension trait pattern to provide thread-safe capabilities.

**Rationale:**
Modifying the `Function` trait to relax the `Deref` target is unnecessary because the `Function::new` method accepts `impl Fn`, which is not `Send`. Therefore, the base `Function::Of` type *must* remain compatible with non-`Send` closures (e.g., `Arc<dyn Fn>`). Since `Function::Of` cannot be `Send` anyway, relaxing the `Function` trait provides no benefit. The `SendClonableFn` extension trait introduces a completely separate associated type (`SendOf`), which makes changes to the base `Function` trait redundant.

**The Solution:**

1.  **Keep `Function` and `ClonableFn` unchanged.**

2.  **Add the `SendClonableFn` extension trait:**

```rust
/// Extension trait for brands that support thread-safe function wrappers.
/// Only implemented by brands that can provide `Send + Sync` guarantees.
trait SendClonableFn: ClonableFn {
    /// The Send-capable wrapped function type.
    /// This is distinct from Function::Of and explicitly requires
    /// the deref target to be `Send + Sync`.
    type SendOf<'a, A, B>: Clone
        + Send
        + Sync
        + Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>;

    /// Creates a new Send-capable clonable function wrapper.
    fn new_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B>;
}
```

3.  **Implement for `ArcFnBrand`:**

```rust
impl SendClonableFn for ArcFnBrand {
    type SendOf<'a, A, B> = Arc<dyn 'a + Fn(A) -> B + Send + Sync>;

    fn new_send<'a, A, B>(
        f: impl 'a + Fn(A) -> B + Send + Sync
    ) -> Self::SendOf<'a, A, B> {
        Arc::new(f)
    }
}
// Note: RcFnBrand does NOT implement SendClonableFn
```

**Usage for parallel operations:**

This usage example correctly utilizes the branded function type `SendOf` instead of a raw closure, maintaining the library's HKT abstraction.

```rust
trait ParFoldable<FnBrand: SendClonableFn>: Foldable {
    fn par_fold_map<'a, A, M>(
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
        f: FnBrand::SendOf<'a, A, M>, // Use the Send-capable branded function
    ) -> M
    where
        A: 'a + Clone + Send + Sync,
        M: Monoid + Send + Sync + 'a;
}
```

**Advantages:**
*   **Zero Breaking Changes:** No changes to `Function`, `ClonableFn`, or existing brands.
*   **Clean Separation:** `Send` capabilities are purely additive.
*   **Correct Abstraction:** Uses the branded `SendOf` type, consistent with the library's design.
*   **Explicit Thread-Safety:** The `Deref<Target = dyn ... + Send + Sync>` constraint makes the thread-safety guarantees self-documenting in the trait definition.

---

##### Solution 2: Direct Parallel Methods with Raw Closures

This approach sidesteps the `ClonableFn` abstraction entirely for parallel operations by accepting raw closures that are constrained to `Send + Sync`.

```rust
trait ParFoldable: Foldable {
    /// Parallel fold_map that bypasses ClonableFn entirely.
    /// Uses raw closures with Send + Sync bounds.
    fn par_fold_map<'a, A, M, F>(
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
        f: F,
    ) -> M
    where
        A: 'a + Clone + Send + Sync,
        M: Monoid + Send + Sync + 'a,
        F: Fn(A) -> M + Send + Sync + 'a;
    
    /// Parallel fold_right with raw closure.
    fn par_fold_right<'a, A, B, F>(
        f: F,
        init: B,
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
    ) -> B
    where
        A: 'a + Clone + Send + Sync,
        B: Send + Sync + 'a,
        F: Fn(A, B) -> B + Send + Sync + 'a;
}
```

**Advantages:**
- No changes to existing `Function`, `ClonableFn`, or `Foldable` traits
- Simple and straightforward implementation
- Clear semantic distinction: sequential ops use `ClonableFn`, parallel ops use raw `Fn + Send + Sync`
- Easy integration with Rayon or other parallel libraries

**Disadvantages:**
- Parallel operations lose the "brand" abstraction for function wrappers
- Cannot compose parallel operations using the same patterns as sequential ones
- Some code duplication between sequential and parallel method implementations

---

##### Solution 3: Separate Parallel Hierarchy

This approach creates a complete parallel hierarchy of traits that explicitly require `Send + Sync` bounds throughout.

```rust
/// Send-capable version of Function
trait SendFunction: Category {
    type Of<'a, A, B>: Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync> + Send + Sync;
    
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B + Send + Sync) -> Self::Of<'a, A, B>;
}

/// Send-capable version of ClonableFn
trait SendClonableFn: SendFunction {
    type Of<'a, A, B>: Clone
        + Deref<Target = dyn 'a + Fn(A) -> B + Send + Sync>
        + Send + Sync;
    
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B + Send + Sync) -> Self::Of<'a, A, B>;
}

/// Parallel-capable version of Foldable
trait ParFoldable<FnBrand: SendClonableFn>: Kind_c3c3610c70409ee6 {
    fn par_fold_right<'a, A: 'a + Clone, B: 'a, F>(
        f: F,
        init: B,
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
    ) -> B
    where
        F: Fn(A, B) -> B + Send + Sync + 'a,
        A: Send + Sync,
        B: Send + Sync;
    
    fn par_fold_map<'a, A: 'a + Clone, M, F>(
        f: F,
        fa: Apply!(brand: Self, signature: ('a, A: 'a) -> 'a),
    ) -> M
    where
        M: Monoid + Send + Sync + 'a,
        F: Fn(A) -> M + Send + Sync + 'a,
        A: Send + Sync;
}

// Only ArcFnBrand (or a new SendArcFnBrand) implements these
impl SendFunction for ArcFnBrand { ... }
impl SendClonableFn for ArcFnBrand { ... }
```

**Advantages:**
- No breaking changes to existing traits
- Clear separation between single-threaded and multi-threaded code paths
- Type system enforces thread safety at compile time
- Explicit opt-in to parallelism

**Disadvantages:**
- Significant code duplication across the trait hierarchy
- Potentially need to duplicate `Functor`, `Applicative`, `Monad`, etc. if they also need Send variants
- Users must choose between two parallel hierarchies
- Maintenance burden of keeping both hierarchies in sync

---

##### Rejected: Associated Type Constraints

The originally proposed solution using associated type constraints:

```rust
trait Function {
    type Constraint<A, B>: ?Sized;
    fn new<F>(f: F) -> ...
    where F: Fn(A) -> B + Self::Constraint<A, B>;
}
```

**This approach does not work** due to fundamental Rust limitations:

1. **Invalid syntax:** `F: Fn(A) -> B + Self::Constraint<A, B>` attempts to use an associated type as a trait bound. In Rust, the `+` operator in trait bounds can only combine *traits*, not types.

2. **Type/trait confusion:** The proposal conflates trait object types (`dyn Fn(A) -> B`) with trait bounds (`Fn(A) -> B`). You cannot use a type as a bound on a generic parameter.

3. **No associated traits:** Rust does not support "associated traits" that would allow parameterizing bounds through associated types. This has been discussed in RFCs but is not implemented.

4. **Deref issue unaddressed:** Even if the `new()` signature worked, the `Deref<Target = dyn Fn(A) -> B>` bound would still prevent using `dyn Fn + Send + Sync` variants.

---

##### Rejected: Dual Associated Types

An alternative approach embedding both Send and non-Send capabilities directly in the trait:

```rust
trait ClonableFn: Function {
    /// Standard (potentially non-Send) function type
    type Of<'a, A, B>: Clone + Deref<Target = Self::Target<'a, A, B>>;
    
    /// Send-capable function type
    type SendOf<'a, A, B>: Clone + Send + Sync + Deref<Target = Self::SendTarget<'a, A, B>>;
    
    type Target<'a, A, B>: ?Sized + 'a + Fn(A) -> B;
    type SendTarget<'a, A, B>: ?Sized + 'a + Fn(A) -> B + Send + Sync;
    
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> Self::Of<'a, A, B>;
    fn new_send<'a, A, B>(f: impl 'a + Fn(A) -> B + Send + Sync) -> Self::SendOf<'a, A, B>;
}
```

**This approach is rejected** for the following reasons:

1. **Trait bloat with dead code:** Brands like `RcFnBrand` that cannot support thread safety must still provide `SendOf`, `SendTarget`, and `new_send` implementations. These would either panic at runtime or use placeholder types like `Never`/`!` that can never be constructed. This pollutes the trait with unusable members.

2. **Violates interface segregation:** Every implementor is forced to provide Send-capable machinery even when it's fundamentally impossible for that brand. This creates confusing APIs where methods exist but should never be called.

3. **No compile-time safety for non-Send brands:** Users could accidentally call `new_send` on `RcFnBrand`, leading to runtime panics rather than compile-time errors.

The extension trait approach (Solution 1) is preferred because it cleanly separates capabilities: only brands that can actually provide thread safety implement `SendClonableFn`.
