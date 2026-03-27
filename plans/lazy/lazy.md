# Analysis of `fp-library/src/types/lazy.rs`

## 1. Design

### Config-Parameterized Approach

The `LazyConfig` trait is a well-chosen design that cleanly abstracts over the `Rc<LazyCell>` vs `Arc<LazyLock>` distinction. It avoids code duplication by parameterizing the `Lazy` struct over a config type, while keeping the two variants' type-level differences (e.g., `Send` bounds) explicit at the impl level.

Strengths:

- The separation of `LazyConfig` (infallible) and `TryLazyConfig` (fallible) is sound. Third-party implementors can choose to implement only the infallible variant.
- The `PointerBrand` associated type provides a clean link to the pointer hierarchy, enabling generic code to discover the pointer strategy.
- The `Thunk` associated type correctly uses `?Sized` to allow `dyn FnOnce()` trait objects.

Weaknesses:

- The `lazy_new` function takes `Box<Self::Thunk<'a, A>>`, meaning every `Lazy` construction allocates a boxed closure. For `RcLazy`, the value is then wrapped in `Rc<LazyCell<A, Box<dyn FnOnce() -> A>>>`, so there are two allocations: one for the `Box<dyn FnOnce>` and one for the `Rc`. This is unavoidable given `LazyCell`'s generic parameter, but worth noting.
- The config approach means `Lazy<'a, A, Config>` is a three-parameter generic, which makes type signatures verbose. The `RcLazy` and `ArcLazy` type aliases mitigate this well.
- The trait is open for extension (documented), but the doc examples only show `RcLazyConfig` and `ArcLazyConfig`. This is fine since those are the primary use cases.

### Overall Structure

The struct `Lazy<'a, A, Config>` is a newtype over `Config::Lazy<'a, A>` with `pub(crate)` visibility on the inner field. This correctly encapsulates the implementation while allowing sibling modules (like `try_lazy.rs`) to access internals.

## 2. Correctness

### Memoization Semantics

The memoization is correct. `Clone` on `Lazy` clones the `Rc`/`Arc`, sharing the underlying cell. All clones see the same computed value. The `evaluate` method correctly delegates to `Config::evaluate`, which calls `LazyCell::force` or `LazyLock::force`.

### `Deferrable` for `ArcLazy`: Eager Evaluation

The `Deferrable` impl for `ArcLazy` calls `f()` eagerly:

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self
where
    Self: Sized, {
    f()
}
```

This is documented with a warning in the `Deferrable` trait docs and in the impl's doc comment. The justification is that `Deferrable::defer` does not require `Send` on the thunk, but `ArcLazy::new` does. The `SendDeferrable` impl provides proper lazy behavior. This is a pragmatic compromise, but it means `Deferrable` for `ArcLazy` violates the spirit of "deferrable" (the computation is not deferred). The transparency law technically holds (the value is the same), but semantically this is surprising.

### `Deferrable` for `RcLazy`: Clone Requirement

The `Deferrable` impl for `RcLazy` requires `A: Clone`:

```rust
fn defer(f: impl FnOnce() -> Self + 'a) -> Self {
    RcLazy::new(move || f().evaluate().clone())
}
```

This clones the evaluated value out of the inner lazy. The `Clone` bound is necessary because `evaluate()` returns `&A`, and the new `RcLazy` needs to own an `A`. This is correct but limits which types can use `Deferrable` with `RcLazy`.

### Fix Combinators: Cycle Correctness

The `rc_lazy_fix` and `arc_lazy_fix` functions use `OnceCell`/`OnceLock` to create self-referential lazy cells. The invariant that the cell is set immediately after closure creation is maintained by the code structure. The documented caveats (memory leak if dropped without evaluation, panic/deadlock on reentrant access) are accurate.

However, the claim that "This cycle is broken when the lazy cell is first evaluated" deserves scrutiny. When the lazy cell is evaluated, the `FnOnce` closure is consumed, which drops the `cell_clone: Rc<OnceCell<RcLazy>>`. If this was the last reference to the `OnceCell`, the cycle breaks. But the `OnceCell` also contains the `RcLazy` itself (via `cell.set(lazy.clone())`). After evaluation:

1. The `LazyCell` consumes the closure, dropping `cell_clone`.
2. The `OnceCell` still holds the `RcLazy` clone.
3. The outer `cell: Rc<OnceCell<...>>` goes out of scope after `rc_lazy_fix` returns.

`cell` is a local variable. After `rc_lazy_fix` returns, only `lazy` survives. The `Rc<OnceCell>` has one strong reference (from `cell_clone` inside the closure). When the closure is consumed by `LazyCell::force`, the closure is dropped, which drops `cell_clone`, which drops the `Rc<OnceCell>`. But the `OnceCell` contains a clone of the `RcLazy`, which references the same `LazyCell`. After the closure runs:

- The `FnOnce` is consumed (dropped).
- `cell_clone` is dropped.
- `cell` (the original) is already out of scope.
- The `OnceCell` should be deallocated since both `Rc` references (`cell` and `cell_clone`) are gone.

This seems correct; the cycle does break on evaluation. If never evaluated, `cell_clone` inside the closure keeps the `OnceCell` alive, which keeps the `RcLazy` clone alive, which keeps the `LazyCell` alive, which keeps the closure alive, forming a cycle. The documentation accurately describes this.

### Semigroup/Monoid: Clone Requirement

The `Semigroup` and `Monoid` impls require `A: Clone`, which is necessary because `evaluate()` returns `&A` and the implementations need owned values. This is correct.

### Foldable: Clone Requirement

The `Foldable` impl clones the evaluated value (`fa.evaluate().clone()`). This is consistent with how the trait signature requires `A: Clone`. Correct.

### From Conversions

The `From<Thunk>` for `RcLazy` correctly wraps the thunk evaluation in a new lazy cell. The `From<Thunk>` for `ArcLazy` eagerly evaluates because `Thunk` is `!Send`. The `From<Trampoline>` conversions follow the same pattern. These are all correct.

## 3. Type Class Instances

### Implemented

- `Clone` (manual, shared cache semantics).
- `Deferrable` (RcLazy and ArcLazy, with different strategies).
- `SendDeferrable` (ArcLazy only).
- `RefFunctor` (RcLazy via `LazyBrand<RcLazyConfig>`).
- `SendRefFunctor` (ArcLazy via `LazyBrand<ArcLazyConfig>`).
- `Foldable` (generic over Config).
- `Semigroup` (RcLazy and ArcLazy separately).
- `Monoid` (RcLazy and ArcLazy separately).
- `PartialEq`, `Eq`, `PartialOrd`, `Ord` (generic over Config).
- `Hash` (generic over Config).
- `Display` (generic over Config, forces evaluation).
- `Debug` (generic over Config, does NOT force evaluation).
- `From<Thunk>` (both RcLazy and ArcLazy).
- `From<Trampoline>` (both RcLazy and ArcLazy).

### Missing (compared to PureScript)

PureScript's `Data.Lazy` implements:

- **Functor**: Not implementable for `Lazy` because `evaluate` returns `&A`. `RefFunctor` is the correct substitute, and it is implemented.
- **Apply/Applicative**: Not implemented. PureScript has `apply f x = defer \_ -> force f (force x)` and `pure a = defer \_ -> a`. In Rust, `Apply` and `Applicative` require `Functor` as a supertrait, which `Lazy` cannot implement. This is a fundamental limitation, not a missing impl.
- **Bind/Monad**: Same situation; requires `Functor`.
- **Extend/Comonad**: These traits do not exist in this library, so not applicable.
- **Traversable**: Not implemented. PureScript's `traverse f l = defer <<< const <$> f (force l)` re-wraps the result in `Lazy`. In Rust, `Traversable` requires `Functor`, which `Lazy` cannot implement.
- **FunctorWithIndex (Unit)**: Not implemented. Would require `Functor`.
- **FoldableWithIndex (Unit)**: Could potentially be implemented since it only requires `Foldable + WithIndex`. The `WithIndex` trait requires specifying an `Index` type (which would be `()` for `Lazy`). This is a reasonable addition.
- **Foldable1**: Not present in the library.
- **Traversable1/TraversableWithIndex**: Would require `Traversable`.
- **Semiring/Ring/CommutativeRing/EuclideanRing**: Not implemented. PureScript implements these by deferring the arithmetic. Could be added if `Semiring` etc. traits exist in the library (they do, per the `classes/` listing).
- **Bounded**: Not implemented. Could be added as `Lazy::new(|| top)` / `Lazy::new(|| bottom)` if a `Bounded` trait exists.
- **HeytingAlgebra/BooleanAlgebra**: Not implemented. Would require the corresponding traits in the library.
- **Invariant**: Not implemented. PureScript derives it from `Functor`.
- **Show**: `Display` is implemented (forces evaluation). PureScript wraps the display in `(defer \_ -> ...)`.

### Potentially Addable

- **`FoldableWithIndex` for `LazyBrand<Config>`**: This would be a straightforward impl with index type `()`. The library has `FoldableWithIndex` and `WithIndex` traits, so this could be added.
- **Semiring-family traits**: If the library has `Semiring`, `Ring`, etc. (it does, based on the `classes/` listing), these could be implemented for `Lazy` following PureScript's pattern.
- **`Pointed`**: The library has a `Pointed` trait. `Lazy` has a `pure` method but does not implement `Pointed` at the type-class level. However, `Pointed` likely requires `Functor` as a supertrait, which would block this.

## 4. API Surface

### Well-Designed Aspects

- `new`, `pure`, `evaluate`, `ref_map` provide a clean minimal API.
- The `RcLazy` and `ArcLazy` type aliases make common usage concise.
- The `fix` combinators are a thoughtful addition for recursive lazy values.
- `From` conversions from `Thunk` and `Trampoline` provide good interop within the lazy hierarchy.

### Missing Operations or Conversions

- **`RcLazy` to `ArcLazy` conversion**: There is no `From<RcLazy<A>> for ArcLazy<A>`. This would require forcing the `RcLazy` (since `Rc` is `!Send`) and wrapping the result. Could be useful.
- **`ArcLazy` to `RcLazy` conversion**: Downgrading from `Arc` to `Rc` would be useful when thread safety is no longer needed.
- **`into_inner` / `try_into_inner`**: No way to extract the value if the `Lazy` has a unique reference count. `Rc::try_unwrap` or `Arc::try_unwrap` could enable this.
- **`is_evaluated` / `is_forced`**: No way to check whether the lazy value has already been computed without forcing it. `LazyCell` does not expose this, so it would require a different internal representation (e.g., wrapping in an additional cell).
- **`map_ref` on `Lazy<'a, A, Config>` (generic over Config)**: Currently `ref_map` is defined separately for `RcLazy` and `ArcLazy` impl blocks. A generic inherent method is not possible because the closure bounds differ (`Send` for Arc).
- **`zip` / `zip_with`**: Combining two lazy values. PureScript provides this via `Apply`, which is not available here. An inherent method or standalone function would be useful.
- **`SendThunk` to `ArcLazy` conversion**: `SendThunk` is `Send`, so a lazy (non-eager) conversion should be possible.

### API Friction

- Creating an `RcLazy` requires `Lazy::<_, RcLazyConfig>::new(|| ...)` or `RcLazy::new(|| ...)`. The latter is clean, but the former requires turbofish with an underscore. This is an inherent limitation of default type parameters.

## 5. Memoization

### Approach

The memoization leverages `std::cell::LazyCell` (Rust 1.80+) and `std::sync::LazyLock` for thread-safe variants. This is the correct modern approach; these types handle the initialization-once logic correctly, including proper synchronization for `LazyLock`.

### Efficiency

- **Double allocation**: As noted, `Lazy` allocates both a `Box<dyn FnOnce>` (for the closure) and an `Rc`/`Arc` (for the shared cell). The `Box` is consumed on first evaluation, but the `Rc`/`Arc` persists. This is one more allocation than strictly necessary; a custom `Rc<UnsafeCell<MaybeUninit<A>>>` could combine them, but at the cost of reimplementing `LazyCell`'s logic.
- **Closure retention**: After evaluation, the `LazyCell`/`LazyLock` drops the closure and stores the value. This is correct and avoids retaining closure captures longer than necessary.
- **Clone cost**: Cloning a `Lazy` is an `Rc::clone` / `Arc::clone`, which is O(1). Good.
- **`ref_map` chaining**: Each `ref_map` creates a new `Lazy` that captures the previous one. This creates a linked list of `Rc`/`Arc` references. The `RefFunctor` trait docs correctly warn about this. Evaluating the chain forces all predecessors.

### Comparison with `once_cell`

The stdlib `LazyCell`/`LazyLock` are essentially the same as `once_cell::unsync::Lazy` / `once_cell::sync::Lazy`, so this is the canonical approach.

## 6. Documentation

### Strengths

- The module-level doc clearly explains why `Lazy` does not implement `Functor` and points to `RefFunctor`/`SendRefFunctor` as alternatives.
- The `LazyConfig` trait has good extensibility documentation.
- The fix combinator docs accurately describe the caveats (memory leaks, panics, deadlocks).
- The `Debug` impl explicitly does not force evaluation, which is documented.
- The `Display` impl does force evaluation, which is documented.
- Panic behavior (poisoned `LazyCell`/`LazyLock`) is documented on the `Lazy` struct.

### Issues

- The doc comment on `ArcLazy`'s `Deferrable` says "The thunk `f` is called eagerly to obtain the inner `ArcLazy`, which is then returned directly." This is accurate but could be clearer about the implication: the `defer` call is not actually deferred at all, making this impl semantically misleading.
- The `LazyConfig` doc says "This design leverages Rust 1.80's `LazyCell` and `LazyLock` types," which is a minor historical note that may become stale.
- Several `#[document_type_parameters]` annotations on outer `impl` blocks describe parameters as "The lifetime of the reference" when they should say "The lifetime of the computation." This inconsistency appears in the `Deferrable` for `ArcLazy` impl block (line 853) and the `Display` impl block (line 1001).
- The `LazyConfig::evaluate` method documents a lifetime parameter `'b` ("The borrow lifetime") but this is the standard Rust borrow lifetime and does not need special documentation.

## 7. Consistency with the Rest of the Library

### Positive

- Uses `impl_kind!` macro for HKT registration, consistent with other types.
- Follows the brand pattern (`LazyBrand<Config>`) with proper type aliases (`RcLazyBrand`, `ArcLazyBrand`).
- Implements `Semigroup`, `Monoid`, `Foldable` following the same patterns as other types.
- Uses the `#[fp_macros::document_module]` and `mod inner` / `pub use inner::*` pattern.
- Documentation attributes (`#[document_signature]`, `#[document_parameters]`, etc.) are used throughout.
- Wrapping everything in `mod inner` with `pub use inner::*` is consistent with the codebase convention.

### Minor Inconsistencies

- The `Foldable` impl is generic over `Config`, which is good. But the `Semigroup` and `Monoid` impls are duplicated for `RcLazyConfig` and `ArcLazyConfig` separately due to different `Send` bounds. This is necessary but creates duplication.
- `pure` is defined as an inherent method rather than through a `Pointed` type class. Other types in the library may handle this differently.

## 8. Limitations and Issues

### Fundamental Limitations

1. **No `Functor`/`Monad`**: The reference-returning `evaluate` makes `Functor` impossible. This is the most significant deviation from PureScript and limits composability with the rest of the type class hierarchy. `RefFunctor` is a partial mitigation but does not integrate into the `Functor` -> `Applicative` -> `Monad` tower.

2. **No `Applicative`/`Apply`**: Cannot lift functions into `Lazy` context or combine lazy values applicatively. Users must manually force and re-wrap.

3. **`Clone` required for most operations**: Since `evaluate()` returns `&A`, extracting owned values requires `Clone`. This excludes non-cloneable types from `Semigroup`, `Monoid`, `Foldable`, and `Deferrable`.

4. **No stack safety**: `ref_map` chaining creates deeply nested closures. Unlike `Trampoline`, there is no trampolining mechanism, so deeply chained `ref_map` calls can overflow the stack on first evaluation.

5. **Panic poisoning**: If the initializer panics, the `Lazy` becomes permanently poisoned. There is no recovery mechanism short of `TryLazy`.

### Design Trade-offs

- **`Deferrable` for `ArcLazy` is eager**: This is a known compromise. The `SendDeferrable` trait exists as the proper alternative, but the `Deferrable` instance may surprise users.

- **Two-trait functor hierarchy (`RefFunctor` / `SendRefFunctor`)**: The decision to keep these independent (rather than making `SendRefFunctor: RefFunctor`) is correct but means generic code cannot abstract over both. A higher-level abstraction (e.g., a trait parameterized by send-ness) could help, but would add complexity.

### Potential Improvements

- **`Deref` impl**: Implementing `Deref<Target = A>` for `Lazy` would allow `*lazy` to force evaluation, providing more ergonomic access. However, this would make accidental forcing easier, which may be undesirable.
- **`AsRef<A>` impl**: Less dangerous than `Deref`, could allow `lazy.as_ref()` as an alternative to `lazy.evaluate()`.

## 9. Comparison with PureScript

| Aspect | PureScript `Data.Lazy` | Rust `Lazy` |
|--------|----------------------|-------------|
| **Core type** | `Lazy :: Type -> Type` (FFI, opaque) | `Lazy<'a, A, Config>` (newtype over config cell) |
| **Construction** | `defer :: (Unit -> a) -> Lazy a` | `Lazy::new(\|\| a)` |
| **Evaluation** | `force :: Lazy a -> a` (returns owned) | `evaluate(&self) -> &A` (returns reference) |
| **Functor** | Yes | No (uses `RefFunctor` instead) |
| **Apply/Applicative** | Yes | No |
| **Bind/Monad** | Yes | No |
| **Extend/Comonad** | Yes (`extract = force`) | No (traits do not exist in library) |
| **Foldable** | Yes | Yes |
| **Traversable** | Yes | No (requires `Functor`) |
| **Semigroup/Monoid** | Yes | Yes |
| **Eq/Ord** | Yes | Yes |
| **Show** | Yes (wraps in `(defer \_ -> ...)`) | `Display` (shows raw value), `Debug` (shows `Lazy(..)`) |
| **Semiring family** | Yes (Semiring, Ring, CommutativeRing, EuclideanRing) | No |
| **HeytingAlgebra** | Yes | No |
| **Lazy (self-reference)** | `class Lazy` with `defer` and `fix` | `Deferrable` trait + standalone `rc_lazy_fix` / `arc_lazy_fix` |
| **WithIndex variants** | Yes (FunctorWithIndex Unit, FoldableWithIndex Unit, TraversableWithIndex Unit) | No |
| **Thread safety** | N/A (single-threaded runtime) | Yes, via `ArcLazyConfig` |
| **Lifetime support** | N/A (GC-managed) | Yes, `'a` parameter |

The most significant difference is that PureScript's `force` returns an owned value (the runtime handles sharing), while Rust's `evaluate` returns a reference. This single difference cascades into the inability to implement `Functor` and everything above it in the type class hierarchy. The `RefFunctor` workaround is sound but fundamentally less composable.

The Rust version compensates with features PureScript does not need: lifetime parameterization, thread-safe variants, and explicit control over sharing semantics. The config-parameterized design is a Rust-specific innovation that handles the `Rc`/`Arc` split cleanly.

### Missing PureScript Instances That Could Be Added

- `FoldableWithIndex` (index type `()`): straightforward, only requires `Foldable + WithIndex`.
- Semiring-family traits: if `Semiring`, `Ring`, etc. exist in the library, these follow the same pattern as `Semigroup`/`Monoid`.
- `Invariant`: if the library has an `Invariant` trait, it could be implemented via `RefFunctor` (using `ref_map` for the covariant direction and ignoring the contravariant function, mirroring PureScript's `imapF`).

## Summary

The `lazy.rs` implementation is well-designed and correct within its constraints. The config-parameterized approach is a clean solution to the `Rc`/`Arc` split. The fundamental limitation (reference-returning `evaluate` preventing `Functor`) is inherent to Rust's ownership model and is handled as well as possible via `RefFunctor`. The documentation is thorough with minor inconsistencies. The main areas for improvement are:

1. Adding `FoldableWithIndex` with index type `()`.
2. Adding Semiring-family instances if the traits exist.
3. Adding conversion functions between `RcLazy` and `ArcLazy`.
4. Considering a `SendThunk -> ArcLazy` conversion that preserves laziness.
5. Clarifying the `Deferrable` for `ArcLazy` documentation to make the eager-evaluation behavior more prominent.
