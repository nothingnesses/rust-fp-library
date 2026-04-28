# Limitations and Workarounds

Sections are ordered from most fundamental (language-level constraints that shape the entire library) to most applied (specific implementation trade-offs with workarounds in place).

## The Brand Pattern (No Native HKT)

### The Issue

Rust does not support higher-kinded types natively. You cannot write `impl Functor for Option` because `Option` is a type constructor (`* -> *`), not a type (`*`), and Rust's trait system only operates on concrete types.

The library works around this using the Brand pattern (lightweight higher-kinded polymorphism / type-level defunctionalization): each type constructor has a zero-sized marker type (e.g., `OptionBrand`) that implements `Kind` traits mapping it back to the concrete type.

### Consequences

- **No method syntax.** Type class operations are free functions, not methods on the container. You write `bind(x, f)` not `x.bind(f)`.
- **Generated trait names in errors.** Compiler errors expose the macro-generated `Kind` trait names (e.g., `Kind_cdc7cd43dac7585f`) rather than human-readable names, making diagnostics harder to interpret.
- **Wrapping/unwrapping overhead in generic code.** Generic functions must use `Apply!` macro invocations to convert between the `Kind` associated type and the concrete type, adding syntactic noise.
- **Turbofish for ambiguous types.** Types reachable through multiple brands at a given arity (e.g., `Result` at arity 1) cannot use brand inference and require `explicit::` variants with turbofish: `explicit::map::<ResultErrAppliedBrand<E>, _, _, _, _>(f, x)`.

### Mitigation

**Brand inference:** For types with a single unambiguous brand (Option, Vec, Identity, Thunk, Lazy, etc.), the `InferableBrand` trait enables the compiler to infer the brand from the container type. No turbofish needed: `map(|x| x + 1, Some(5))`. At arity 2 (bifunctor operations), types like `Result` that are ambiguous at arity 1 become unambiguous: `bimap((f, g), Ok(5))`.

**Do-notation:** The `m_do!` and `a_do!` macros provide ergonomic do-notation. In inferred mode (`m_do!({ ... })`), the brand is inferred from container types. In explicit mode (`m_do!(Brand { ... })`), the brand is specified for ambiguous types or to use `pure()`.

The `Pipe` trait allows method-chaining syntax for some operations. The `impl_kind!` and `trait_kind!` macros automate the boilerplate of defining new brands and kind traits.

## Uncurried Semantics (No Zero-Cost Currying)

### The Issue

Most FP languages and libraries use curried functions: `map(f)(fa)`. In Rust, returning a closure from a function requires either boxing it (`Box<dyn Fn>`) or wrapping it in a reference-counted pointer (`Rc<dyn Fn>`, `Arc<dyn Fn>`). Both involve heap allocation and dynamic dispatch, defeating the library's zero-cost abstraction goal.

Every closure in Rust has a unique, anonymous type. A curried `map(f)` would need to return `impl Fn(F::Of<A>) -> F::Of<B>`, but `impl Trait` in return position captures the concrete closure type, making it impossible to store, pass around, or compose without type erasure.

### Consequence

The library uses uncurried semantics throughout: `map(f, fa)` instead of `map(f)(fa)`. This allows the compiler to monomorphize `f` at each call site, enabling inlining and zero heap allocation. The trade-off is that partial application is not directly supported; you must use explicit closures instead (e.g., `|fa| map(f, fa)`).

### Potential Future Resolution

The nightly feature `unboxed_closures` ([rust-lang/rust#29625](https://github.com/rust-lang/rust/issues/29625)) combined with `fn_traits` ([rust-lang/rust#29625](https://github.com/rust-lang/rust/issues/29625)) and particularly `impl_trait_in_fn_trait_return` ([rust-lang/rust#99697](https://github.com/rust-lang/rust/issues/99697)) could enable zero-cost currying by allowing functions to return `impl Fn` without boxing. If stabilized, the library could offer curried variants alongside the uncurried API.

## No Rank-N Types

### The Issue

Rust cannot express rank-2 (or higher) types. You cannot write a type alias or data type that is universally quantified over a trait-bounded type parameter. In PureScript/Haskell, rank-2 types are used pervasively in FP abstractions. Their absence in Rust forces workarounds throughout the library.

### Consequences

#### Profunctor optics

In PureScript, an optic is a rank-2 polymorphic function:

```purescript
type Lens s t a b = forall p. Strong p => p a b -> p s t
```

Composition is ordinary function composition (`<<<`), and the profunctor is chosen at the use site. Rust cannot express this, so the library uses concrete structs (`Lens`, `Prism`, `Iso`, etc.) storing reified internal representations (equivalent to PureScript's `ALens`/`APrism`/`AnIso`), composed via a `Composed` struct with static dispatch rather than function composition. This results in deeply nested types for long composition chains, and generic code must be bounded by optic traits (e.g., `O: LensOptic`) rather than profunctor constraints.

See [Optics Comparison](optics-analysis.md) and [Profunctor Classes Analysis](profunctor-analysis.md) for detailed comparisons.

#### `Wander` trait

PureScript's `wander` takes `forall f. Applicative f => (a -> f b) -> s -> f t`. Rust encodes this via the `TraversalFunc` trait, which provides a concrete `apply` method that the `Wander` implementation calls with specific applicative functors.

#### No `Yoneda` type

PureScript's `Yoneda f a` is `forall b. (a -> b) -> f b`, which requires rank-2 quantification to store as a data type. This cannot be represented in Rust.

#### No `unCoyoneda` eliminator

In Haskell/PureScript, `unCoyoneda :: (forall b. (b -> a) -> f b -> r) -> Coyoneda f a -> r` provides access to the existential intermediate type `b` via a rank-2 continuation. Without this, `Coyoneda::hoist` must lower first (requiring `F: Functor`), transform, then re-lift. PureScript's `hoistCoyoneda` has no `Functor` constraint. `CoyonedaExplicit` avoids this because `B` is an explicit type parameter, not existential.

## Unexpressible Bounds in Trait Method Signatures

### The Issue

Several types (`RcCoyoneda`, `ArcCoyoneda`) cannot implement type class traits at the brand level because their constructors require bounds (like `Clone` or `Send + Sync`) on the `Kind` associated type `F::Of<'a, A>` that cannot be expressed in the trait method signature.

For example, `RcCoyoneda::lift` requires `F::Of<'a, A>: Clone` because the base layer must be clonable for `lower_ref` to work. But the `Pointed` trait's `pure` method has no way to express this:

```rust,ignore
// Pointed::pure signature - no Clone bound on the return type's contents
fn pure<'a, A: 'a>(value: A) -> Self::Of<'a, A>;
```

The same problem affects `Semimonad::bind`, `Semiapplicative::apply`, and `Lift::lift2` for `RcCoyoneda` and `ArcCoyoneda`.

For `ArcCoyoneda`, the problem is compounded: `Functor::map` also cannot be implemented because the HKT `map` signature lacks `Send + Sync` bounds on the closure parameter, so closures passed to `map` cannot be stored inside `Arc`-wrapped layers.

### Consequences

| Type              | Brand-level `Functor`   | Brand-level `Pointed`   | Brand-level `Semimonad` | Reason                                                                                                                                                                                     |
| :---------------- | :---------------------- | :---------------------- | :---------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Coyoneda`        | Yes                     | Yes                     | Yes                     | `Box<dyn FnOnce>` has no extra bounds.                                                                                                                                                     |
| `RcCoyoneda`      | Yes                     | No                      | No                      | Needs `F::Of: Clone`.                                                                                                                                                                      |
| `ArcCoyoneda`     | Yes (via `SendFunctor`) | No                      | No                      | Needs `F::Of: Clone + Send + Sync` and closures `Send + Sync`. `SendFunctor` (closure has `Send + Sync`) closes the by-value `Functor` gap; `SendPointed` / `SendSemimonad` still blocked. |
| `FreeExplicit`    | Yes                     | Yes                     | Yes                     | Concrete recursive enum; `bind` has no Clone bound. `Lift` / `Semiapplicative` / `Applicative` / `Monad` blocked: `lift2` consumes `fb` multiply; `FreeExplicit` is not `Clone`.           |
| `RcFreeExplicit`  | No                      | Yes                     | No                      | `bind` requires per-`A` `Clone` bounds for the shared-inner-state recovery path; `pure` does not.                                                                                          |
| `ArcFreeExplicit` | No                      | Yes (via `SendPointed`) | No                      | Same as `RcFreeExplicit` plus the `Kind<Of<...>: Send + Sync>` auto-derive bound is per-`A` (no HRTB-over-types in stable Rust).                                                           |

The Free Explicit family also has a parallel by-reference classification:

| Type              | Brand-level `RefFunctor`       | Brand-level `RefPointed` | Brand-level `RefSemimonad`     | Reason                                                                                                                                                                                             |
| :---------------- | :----------------------------- | :----------------------- | :----------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `FreeExplicit`    | Yes (requires `F: RefFunctor`) | Yes                      | Yes (requires `F: RefFunctor`) | Recursive helper walks `&fa` via `F::ref_map`. `RefLift` blocked: closure captures shorter-lifetime `&A`, can't satisfy `+ 'a`.                                                                    |
| `RcFreeExplicit`  | Yes (requires `F: RefFunctor`) | Yes                      | Yes (requires `F: RefFunctor`) | Same recursive pattern via `Rc::deref` + `F::ref_map`. `RefLift` blocked for the same reason.                                                                                                      |
| `ArcFreeExplicit` | No (`SendRefFunctor` blocked)  | No                       | No                             | The closure passed to `F::send_ref_map` returns `ArcFreeExplicit<F, B>`, requiring auto-derived `Send + Sync`; that requires the per-`A` `Kind` bound that no HRTB-over-types feature can express. |

The `ArcFreeExplicit` substrate additionally has a Send-aware by-value classification (the `Send*` parallel of the by-value table above). After Phase 2 sub-step 9c migrated the substrate from `F: Functor` to `F: SendFunctor`, the brand-level Send-aware coverage was re-evaluated; the per-`A` `Clone` cascade on `bind`'s where-clause is intrinsic to the `Arc<Inner>` data shape and remains the binding constraint:

| Type              | Brand-level `SendFunctor`     | Brand-level `SendPointed` | Brand-level `SendSemimonad`    | Brand-level `SendLift` | Reason                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| :---------------- | :---------------------------- | :------------------------ | :----------------------------- | :--------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ArcFreeExplicit` | No (inherent `map` available) | Yes                       | No (inherent `bind` available) | No                     | The natural delegation pattern `fa.bind(move \|a\| ArcFreeExplicit::pure(func(a)))` inherits `bind`'s `A: Clone` plus per-`A` `F::Of<'a, ArcFreeExplicit<'a, F, A>>: Clone + Send + Sync` bounds. These are intrinsic to `into_inner_owned`'s shared-`Arc` recovery path and cannot be expressed in `SendFunctor::send_map`'s trait method signature without HRTB-over-types. `SendPointed` is unaffected because `ArcFreeExplicit::pure` has no `Clone` bound. The `SendSemiapplicative` / `SendApplicative` / `SendMonad` cascade depends on `SendLift` and `SendSemimonad`, so it is blocked transitively. Inherent `ArcFreeExplicit::map` and `ArcFreeExplicit::bind` carry the per-`A` bounds explicitly and cover the by-value Send-aware mapping/sequencing surface for callers operating on the concrete type. (The bare name `map`, with `Send + Sync` bounds in the where-clause, follows the established Arc-substrate convention used by `ArcFree::map` / `ArcRunExplicit::map`.) |

Sub-step 9d landed inherent `ArcFreeExplicit::map` as a concrete-type counterpart to the unreachable brand-level `SendFunctor::send_map`. `SendLift::send_lift2` (and the rest of the applicative cascade) has no inherent counterpart on `ArcFreeExplicit`; callers can compose `bind` with `pure` directly when they need lifted binary application.

The parallel `ArcRunExplicitBrand` SendRef-family coverage was re-evaluated in sub-step 9i. The plan's reference shape (delegate `SendRefFunctor::send_ref_map` to the wrapper's inherent `ArcRunExplicit::ref_map`) is rejected by rustc with four blockers, three of which are unresolvable in stable Rust:

| Brand-level trait                                 | Reachable? | Blockers (if any)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| :------------------------------------------------ | :--------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `SendRefPointed`                                  | Yes        | None. Trait's `A: Clone + Send + Sync` matches `ArcRunExplicit::ref_pure`'s; no closure parameter.                                                                                                                                                                                                                                                                                                                                                                                                            |
| `SendRefFunctor`                                  | No         | Closure-bound mismatch: `send_ref_map`'s `Fn(&A) -> B + Send + 'a` is weaker than `ArcRunExplicit::ref_map`'s `Send + Sync` requirement (the substrate stores closures in `Arc<dyn Fn + Send + Sync>`, so `Sync` is non-negotiable). Plus three per-`A` HRTB blockers identical to `ArcFreeExplicitBrand: SendFunctor`'s: `A: Clone`; `<R as Kind>::Of<'_, ArcFreeExplicit<...>>: Clone + Send + Sync`; same on `S` and `NodeBrand<R, S>`. None expressible at the trait method level without `for<T>` HRTBs. |
| `SendRefSemimonad`                                | No         | Same blocker pattern as `SendRefFunctor`: `send_ref_bind`'s closure is `Send`-only; `ArcRunExplicit::ref_bind` carries the same per-`A` `Clone` plus `F::Of<...>: Clone + Send + Sync` cascade.                                                                                                                                                                                                                                                                                                               |
| `SendRefLift` and the rest of the SendRef cascade | No         | `SendRefSemiapplicative: SendRefLift + SendRefFunctor` and the cascade above blanket-derive from these; blocked transitively.                                                                                                                                                                                                                                                                                                                                                                                 |

Adding `A: Clone` to the trait method signature would resolve one blocker but conceptually violate the ref-family contract (`send_ref_map` operates on `&A`, never moving or cloning `A`). Tightening the closure to `Send + Sync` would resolve another but break `LazyBrand<ArcLazyConfig>::send_ref_map` callers that currently pass `Send`-only closures. Neither would resolve the per-`A` HRTB blockers, which are the same fundamental gap that step 4b's resolution and 9d's outcome documented for the by-value `SendFunctor` cascade. The wrapper's inherent `ArcRunExplicit::ref_map` / `ref_bind` / `ref_pure` methods carry the per-`A` bounds explicitly in their where-clauses and remain the user-facing by-reference Send-aware surface. The `im_do!(ref ArcRunExplicit { ... })` macro form (Phase 2 step 7c) desugars to these inherent methods.

### Workaround: Inherent Methods

All affected operations are available as inherent methods with the necessary bounds stated explicitly:

```rust,ignore
// RcCoyoneda - inherent pure with Clone bound
impl<'a, F, A: 'a> RcCoyoneda<'a, F, A> {
	pub fn pure(value: A) -> Self
	where F::Of<'a, A>: Clone { ... }
}

// ArcCoyoneda - inherent map with Send + Sync on closure
impl<'a, F, A: 'a> ArcCoyoneda<'a, F, A> {
	pub fn map<B: 'a>(self, f: impl Fn(A) -> B + Send + Sync + 'a) -> ArcCoyoneda<'a, F, B> { ... }
}
```

This means these types cannot be used generically (e.g., passed to a function expecting `F: Pointed`), but all operations work when used directly. See [Coyoneda Implementations](coyoneda.md) for the full comparison.

### Root Cause

Rust's trait system does not support conditional bounds on associated types. There is no way to write "when `A: Clone`, then `Self::Of<'a, A>` supports `pure`." Each trait method signature is fixed for all implementors. This is a fundamental Rust limitation, not a library design issue.

## Memoized Types Cannot Implement `Functor`

### The Issue

`Lazy::evaluate()` returns `&A` (a reference to the cached value), not an owned `A`. The standard `Functor` trait expects `map` to consume an owned `A`:

```rust,ignore
fn map<'a, A: 'a, B: 'a>(f: impl Fn(A) -> B + 'a, fa: Self::Of<'a, A>) -> Self::Of<'a, B>;
```

Automatically cloning the inner value to satisfy this signature would violate the library's zero-cost abstraction principle, since `Clone` may be expensive and the caller has no control over when it happens.

### Implemented Solution: By-Reference Trait Hierarchy

A complete by-reference type class stack mirrors the by-value hierarchy. Each `Ref*` trait's closures receive `&A` instead of consuming `A`, making the ownership semantics honest for memoized types.

**Core hierarchy:** `RefFunctor`, `RefPointed`, `RefLift`, `RefSemiapplicative`, `RefSemimonad`, `RefApplicative`, `RefMonad`, `RefApplyFirst`, `RefApplySecond`.

**Foldable/Traversable/Filterable:** `RefFoldable`, `RefTraversable`, `RefFilterable`, `RefWitherable`, plus `WithIndex` variants for all.

**Thread-safe:** `SendRefFunctor`, `SendRefPointed`, `SendRefLift`, `SendRefSemiapplicative`, `SendRefSemimonad`, `SendRefApplicative`, `SendRefMonad`, `SendRefFoldable`, etc. These add `Send + Sync` bounds on closures and elements.

**Parallel:** `ParRefFunctor`, `ParRefFoldable`, `ParRefFilterable`, plus `WithIndex` variants. These use rayon for parallel by-reference iteration.

The by-value and by-ref traits are independent (not in a sub/supertrait relationship). A unified `map` free function dispatches to the correct variant based on the closure's argument type (`Fn(A) -> B` routes to `Functor`, `Fn(&A) -> B` routes to `RefFunctor`). The same dispatch pattern extends to `bind`, `lift2`-`lift5`, `fold_map`, `fold_right`, `fold_left`, and semimonad helpers.

Collection types (Vec, Option, CatList, Identity) implement both hierarchies: the by-value traits consume elements, the by-ref traits iterate by reference. `Lazy` only implements the Ref hierarchy since it caches values and can only lend references.

## `Free` and `Trampoline` Require `'static`

### The Issue

`Free` uses `Box<dyn Any>` for type erasure of continuation values. Since `Any` requires `'static`, all types stored in `Free` must be `'static`. This applies to `Trampoline` as well, since `Trampoline<A>` is `Free<ThunkBrand, A>`.

### Consequences

- **No borrowed data.** You cannot create a `Trampoline` that captures a reference to a local variable. All data must be owned or `'static`.
- **No HKT trait integration.** The library's HKT traits require lifetime polymorphism (`type Of<'a, A: 'a>: 'a`). Since `Free` is fixed to `'static`, it cannot implement `Functor`, `Monad`, or other HKT traits at the brand level. Operations like `map`, `bind`, and `pure` are provided as inherent methods instead.
- **Composing with `Lazy` requires cloning.** To memoize a `Trampoline` result via `RcLazy`, you must evaluate first and cache the result, rather than wrapping the `Trampoline` itself (since `Lazy` supports arbitrary lifetimes but `Trampoline` does not).

### Root Cause

Rust's `Any` trait requires `'static` to ensure memory safety (preventing use-after-free of references through downcasting). There is no way to have a lifetime-polymorphic `Any` on stable Rust. `Thunk` and `Lazy` avoid this constraint because they use trait objects with explicit lifetime parameters (`Box<dyn FnOnce() -> A + 'a>`) rather than type erasure via `Any`.

See [Lifetime Ablation Experiment](lifetime-ablation-experiment.md) for a detailed exploration of the trade-offs around lifetime parameters in the lazy evaluation types.

## No Refinement of Associated Type Bounds in Subtraits

### The Issue

Rust does not allow a subtrait to add bounds to an associated type inherited from a supertrait. Given a trait hierarchy where a base trait defines `type Of<T>: Deref`, a subtrait cannot narrow it to `type Of<T>: Clone + Deref` or `type Of<T>: Send + Sync + Deref`. The bounds on an associated type are fixed in the trait that defines it.

This means each level of a trait hierarchy that needs different bounds on its pointer/wrapper type must define its own associated type. For example, `RefCountedPointer` cannot reuse `Pointer::Of` with a `Clone` bound added; it must introduce a separate `Of` with `Clone + Deref` bounds. Similarly, `SendRefCountedPointer` cannot add `Send + Sync` to `RefCountedPointer::Of`; it has its own `Of` with `Clone + Send + Sync + Deref` bounds.

### Consequences

#### Separate associated types per hierarchy level

Each pointer trait level has its own associated type, even though they resolve to the same concrete type for all implementors:

| Trait                   | Associated type | Bounds                        | `ArcBrand` resolves to |
| :---------------------- | :-------------- | :---------------------------- | :--------------------- |
| `RefCountedPointer`     | `Of`            | `Clone + Deref`               | `Arc<T>`               |
| `SendRefCountedPointer` | `Of`            | `Clone + Send + Sync + Deref` | `Arc<T>`               |

The `CloneFn`/`SendCloneFn` split exists for the same reason: `CloneFn::Of` derefs to `dyn Fn(A) -> B`, while `SendCloneFn::Of` derefs to `dyn Fn(A) -> B + Send + Sync`. These are different unsized types in Rust, so they cannot share an associated type.

#### Flat hierarchies instead of linear chains

Because each level needs its own associated type regardless of the hierarchy shape, there is no structural benefit to encoding pointer traits as a linear supertrait chain. The Send variants (`SendRefCountedPointer`, `SendCloneFn`) are independent parallel traits rather than subtraits, since their associated types have fundamentally different bounds. Consumers that need both capabilities list both as bounds (e.g., `P: ToDynCloneFn + ToDynSendFn`).

#### Capability-trait composition is limited

An alternative design using independent "capability traits" composed via blanket-implemented marker traits was considered. Each capability would define its own `Of` type, and markers like `RefCountedPointer` would be blanket-implemented for any type implementing the required capabilities. However, such marker traits cannot have their own associated types in a blanket impl, because the impl cannot know the concrete relationship between different capability traits' `Of` types. If a marker trait needs an associated type that combines bounds from multiple capabilities, it must be manually implemented per type, losing the automatic derivation benefit.

### Root Causes

Three missing Rust features contribute:

1. **No associated type bound refinement in subtraits.** You cannot write `trait Sub: Super where Self::Of: Clone {}` in a way that enriches the inherited associated type's bounds. The associated type is owned by the trait that defines it.

2. **No higher-ranked type bounds.** Rust supports `for<'a>` (higher-ranked over lifetimes) but not `for<T>` (higher-ranked over types). You cannot express "for all `T: Send + Sync`, `Self::Of<T>: Send + Sync`" as a trait bound. This would allow a single associated type to conditionally satisfy different bounds depending on its type parameter.

3. **No associated traits.** Rust has associated types but not associated traits. If a trait could define `trait Bounds` as an associated item, implementors could specify which bounds their `Of` type satisfies, and subtraits could compose those bounds. Without this, the bounds on an associated type are fixed at definition time.

### Consequence for Library Design

The library uses independent parallel traits for Send variants (matching the `CloneFn`/`SendCloneFn` pattern) and separate associated types at each abstraction level. This is the most pragmatic encoding given Rust's trait system. The redundancy at the impl level (defining `Arc<T>` as the associated type in multiple traits) is the cost of expressing different bound requirements on what is concretely the same type.

## Thread Safety and Parallelism

### `Foldable` and `CloneFn`

The `Foldable` trait and its default implementations (`fold_right`, `fold_left`) are **not thread-safe** in terms of sending the computation across threads, even when using `ArcFnBrand`. The `Foldable` trait cannot support parallel implementations (like those using `rayon`).

#### The Issue

While `fp-library` provides `ArcFnBrand` (which uses `std::sync::Arc`), the resulting function wrappers are `!Send` (not thread-safe). This means you cannot spawn a thread and pass a `fold_right` operation that uses `ArcFnBrand` into it, nor can you implement a parallel `fold_map`.

#### Root Causes

This limitation stems from the design of the `Arrow` and `CloneFn` traits, which prioritize compatibility with `Rc` (single-threaded reference counting).

1.  **`CloneFn::new` accepts non-`Send` functions:**
    The `CloneFn` trait defines its constructor as:

    ```rust,ignore
    fn new<'a, A, B>(f: impl 'a + Fn(A) -> B) -> ...
    ```

    The input `f` is **not** required to be `Send`. This is intentional to allow `RcFnBrand` to wrap closures that capture non-thread-safe data (like `Rc` pointers). Because `ArcFnBrand` implements this same trait, it must also accept non-`Send` functions. Since it cannot guarantee the input is `Send`, it cannot wrap it in an `Arc<dyn Fn(...) + Send>`. It is forced to use `Arc<dyn Fn(...)>`, which is `!Send`.

2.  **`Function` Trait Type Constraints:**
    The `Arrow` trait enforces strict type equality on its associated type:
    ```rust,ignore
    type Of<'a, A, B>: Deref<Target = dyn 'a + Fn(A) -> B>;
    ```
    This prevents `ArcFnBrand` from defining its inner type as `Arc<dyn Fn(...) + Send + Sync>`, because `dyn Fn + Send + Sync` is a different type than `dyn Fn`.

#### Consequences

- **`fold_right` / `fold_left`:** Even if you use `ArcFnBrand`, the closure created internally by these functions is `!Send`.
- **`fold_map`:** The `Foldable` trait signature for `fold_map` does not enforce `Send` on the mapping function `F`. Therefore, you cannot implement `Foldable` for a parallel data structure (e.g., using `rayon`) because parallel libraries require `Send` bounds which the trait does not provide.

#### Implemented Solution: Parallel Traits

The library addresses this with independent parallel traits that provide thread-safe capabilities without breaking existing code:

- [`SendCloneFn`](../src/classes/send_clone_fn.rs): A separate trait (not a supertrait of `CloneFn`) that mirrors `CloneFn` with `Send + Sync` bounds. It has its own `Of` associated type that wraps `dyn Fn + Send + Sync` (a different unsized type than `CloneFn::Of`'s `dyn Fn` target). `FnBrand<P>` implements both traits when the pointer `P` supports it (`ArcFnBrand` implements both; `RcFnBrand` implements only `CloneFn`).
- [`ParFoldable`](../src/classes/par_foldable.rs): Parallel fold operations using `impl Fn + Send + Sync` closures directly, bypassing the `CloneFn` abstraction for parallel paths.

This approach keeps `Arrow` and `CloneFn` unchanged, cleanly separates `Send` capabilities as independent traits, and provides compile-time safety (only brands that can actually provide thread safety implement `SendCloneFn`).
