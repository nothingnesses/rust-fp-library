# Analysis: `Lazy` (RcLazy and ArcLazy)

**File:** `fp-library/src/types/lazy.rs`
**PureScript reference:** `Data.Lazy` from `purescript-lazy`

## 1. Type Design

### Core Structure

`Lazy<'a, A, Config>` is a newtype wrapper around `Config::Lazy<'a, A>`, which resolves to:

- **RcLazy:** `Rc<LazyCell<A, Box<dyn FnOnce() -> A + 'a>>>`
- **ArcLazy:** `Arc<LazyLock<A, Box<dyn FnOnce() -> A + Send + 'a>>>`

This design is sound. `LazyCell` (stabilized in Rust 1.80) and `LazyLock` are the standard library's "compute at most once" primitives, and wrapping them in `Rc`/`Arc` provides shared ownership so that clones observe the same cached value. The pointer + cell combination is exactly what is needed for memoized lazy evaluation with shared semantics.

### LazyConfig Parameterization

The `LazyConfig` trait bundles three associated items:

- `Lazy<'a, A>` - the concrete lazy cell type.
- `Thunk<'a, A>` - the initializer closure type (a `?Sized` trait object).
- `PointerBrand` - links the config to the pointer hierarchy.

Two methods complete the interface: `lazy_new` (construction) and `evaluate` (forcing).

**Strengths:**
- The config trait is open for extension; third-party implementations can swap in alternative cell or pointer types (e.g., `parking_lot`-based or async-aware cells).
- The `PointerBrand` associated type lets generic code discover the underlying pointer brand without hard-coding `RcBrand` or `ArcBrand`.
- `TryLazyConfig` extends `LazyConfig` cleanly for fallible memoization, with its own pair of associated types and methods.

**Weaknesses:**
- The `?Sized` `Thunk` associated type exists only to define the closure trait object (`dyn FnOnce() -> A + 'a` vs `dyn FnOnce() -> A + Send + 'a`). It is never used outside of `lazy_new`'s signature. This could arguably be inlined, but the current design keeps it visible for documentation and extensibility purposes.
- Constructors take `Box<Self::Thunk<'a, A>>`, requiring boxing at every call site. This is inherent to the trait-object approach and probably unavoidable if the config trait is to remain object-safe-ish, but it does impose one heap allocation per `Lazy` construction on top of the `Rc`/`Arc` allocation.

**Verdict:** The parameterization is well-designed, not over-engineered. Two concrete configs (`RcLazyConfig`, `ArcLazyConfig`) capture the Rc-vs-Arc dichotomy cleanly, and the open trait allows future extension without breaking changes.

## 2. Memoization Semantics

Memoization is the defining characteristic that distinguishes `Lazy` from `Thunk`. A `Lazy` evaluates its thunk at most once and caches the result; all clones share the cache via reference counting.

### Interaction with the FP Type Class Hierarchy

Memoization creates a fundamental tension with standard FP abstractions:

1. **Shared ownership prevents consumption.** `evaluate()` returns `&A`, not `A`. The cached value lives inside the `Rc`/`Arc` and cannot be moved out without breaking the sharing invariant. This means any operation that needs an owned `A` must clone.

2. **Mapping creates new cells, not in-place mutations.** `ref_map(f, lazy)` produces a *new* `Lazy` that, when forced, forces the original and applies `f` to the reference. This creates a linked chain of `Rc`/`Arc` cells where each cell keeps its predecessor alive. Long chains accumulate memory.

3. **No Applicative or Monad.** PureScript's `Lazy` has `pure`, `apply`, and `bind`, but these require the ability to produce owned values from the lazily-computed contents. In Rust, `bind : Lazy<A> -> (A -> Lazy<B>) -> Lazy<B>` would need to move `A` out of the memoized cell (impossible without `Clone`) or take `&A` (changing the signature away from the standard Monad interface). This is an inherent limitation of Rust's ownership model interacting with shared memoization.

### Clone Requirements

Several implementations (`Deferrable`, `Semigroup`, `Monoid`, `Foldable`) require `A: Clone` because they need to extract values from the shared cache. For example, `Semigroup::append` forces both sides and clones before combining. This `Clone` requirement pervades the API and is unavoidable given the design.

## 3. HKT Support

### What Is Implemented

`LazyBrand<Config>` has an `impl_kind!` that maps `Of<'a, A> = Lazy<'a, A, Config>`, so it participates in the HKT system. The trait implementations are:

| Trait | RcLazy | ArcLazy | Notes |
|-------|--------|---------|-------|
| `RefFunctor` | Yes | No (inherent `ref_map` only) | `RefFunctor` does not require `Send` on the closure. |
| `SendRefFunctor` | No | Yes | Separate trait with `Send` bound. |
| `Foldable` | Yes | Yes | Generic over `Config`. |
| `FoldableWithIndex` | Yes | Yes | Index type is `()`. |
| `WithIndex` | Yes | Yes | `type Index = ()`. |
| `Deferrable` | Yes | Yes (eager fallback) | ArcLazy's `Deferrable::defer` evaluates `f()` immediately. |
| `SendDeferrable` | No | Yes | Properly deferred with `Send` closure. |
| `Semigroup` | Yes (A: Clone) | Yes (A: Clone + Send + Sync) | |
| `Monoid` | Yes (A: Clone) | Yes (A: Clone + Send + Sync) | |

### What Is NOT Implemented and Why

| Missing Trait | Reason |
|---------------|--------|
| **Functor** | `map : (A -> B) -> F<A> -> F<B>` requires an owned `A`. `evaluate()` returns `&A`. Cannot move the cached value out of `Rc`/`Arc`. |
| **Applicative** | Requires `Functor` as a supertrait. Same ownership problem. |
| **Monad** | Requires `Applicative`. Same ownership problem. |
| **Traversable** | Requires `Functor + Foldable`. Blocked by `Functor`. |
| **Evaluable** | Requires `Functor` and returns an owned value. Explicitly documented as impossible for `Lazy`. |
| **Comonad/Extend** | Not defined in the library at all, but even if they were, `extract` would need an owned value. |

### Is Partial HKT the Right Trade-off?

Yes. The `RefFunctor`/`SendRefFunctor` split is the correct compromise:

- `RefFunctor` captures the reference-based mapping semantics precisely.
- `SendRefFunctor` as a separate (non-subtrait) trait avoids poisoning `RcLazy` with `Send` bounds it cannot satisfy.
- `Foldable` works because folding consumes the structure to produce a summary value; the `Clone` bound on `A` allows extracting the cached value.

### Could Full Functor Be Supported?

There are a few theoretical approaches, none entirely satisfactory:

1. **Require `A: Clone` in the `Functor` impl.** This would violate the trait's contract, which makes no assumption about `A`. The library's `Functor` trait signature is `fn map<'a, A: 'a, B: 'a>(f: impl Fn(A, ) -> B + 'a, fa: ...) -> ...`, and the `A` is taken by value. You would need to clone internally, but the trait does not permit adding a `Clone` bound per-instance.

2. **Change `Functor::map` to take `&A`.** This would break every other `Functor` implementation in the library and is the wrong direction.

3. **Make `Lazy` non-shared (consume-once).** This defeats the purpose of memoization.

4. **Use `Rc::try_unwrap` to attempt ownership.** This only works when the reference count is exactly 1, which is not guaranteed by the type system.

**Conclusion:** Full `Functor` cannot be supported for `Lazy` without either changing the `Functor` trait's signature (breaking the rest of the library) or adding a `Clone` bound (impossible without GAT-based conditional bounds or a separate `CloneFunctor` trait).

## 4. Comparison to PureScript's Lazy

PureScript's `Data.Lazy` implements the full tower: `Functor`, `FunctorWithIndex`, `Foldable`, `FoldableWithIndex`, `Traversable`, `TraversableWithIndex`, `Foldable1`, `Traversable1`, `Apply`, `Applicative`, `Bind`, `Monad`, `Extend`, `Comonad`, `Semigroup`, `Monoid`, `Eq`, `Ord`, `Show`, `Bounded`, `Semiring`, `Ring`, `CommutativeRing`, `EuclideanRing`, `HeytingAlgebra`, `BooleanAlgebra`, `Invariant`, and the `Lazy` (self-referential construction) class.

### Why PureScript Can Do This

PureScript has:

1. **Pervasive immutability with implicit sharing.** All values are immutable and garbage-collected. There is no ownership or borrowing. `force` can hand out a value without worrying about who else holds a reference.

2. **No distinction between owned and borrowed.** PureScript's `force :: Lazy a -> a` returns the value directly. In a GC'd runtime, this is a pointer to the cached heap object; there is no move/copy distinction.

3. **No Send/Sync.** PureScript runs single-threaded (no green threads with shared memory), so there is no need for a `Send` variant.

4. **`Lazy` is a foreign type.** It is implemented in the runtime (JavaScript/C++ backend), not in PureScript itself. The runtime handles the memoization cell directly.

### Fundamental vs. Design Choice

The gap is **fundamentally caused by Rust's ownership model**, not a design choice. Specifically:

- `Functor::map` in Rust takes owned `A` (by value) because that is the natural and zero-cost way to transform data. PureScript's `map` works on "values" that are really GC pointers, so the distinction does not arise.
- `evaluate()` in Rust must return `&A` because the value is shared behind `Rc`/`Arc`. In PureScript, `force` returns the value (which is a GC pointer) without ownership transfer.
- Thread safety concerns force the Rc/Arc split, which PureScript does not face.

The library's response, introducing `RefFunctor` as a reference-based alternative to `Functor`, is the idiomatic Rust solution to this fundamental mismatch.

### What Rust's Lazy Is Missing Compared to PureScript

Beyond the type class tower (Functor, Monad, etc.), PureScript's `Lazy` also provides:

- **Algebraic instances** (`Semiring`, `Ring`, `HeytingAlgebra`, `BooleanAlgebra`, `Bounded`). These could potentially be added to the Rust version, since they would just require constructing new `Lazy` values that force both operands. The library already does this for `Semigroup` and `Monoid`. However, these algebraic traits may not exist in the Rust library's class hierarchy.
- **`Extend` / `Comonad`.** In PureScript, `extract = force` and `extend f x = defer \_ -> f x`. The Rust version could potentially implement `Comonad`-like behavior if those traits existed in the library; `evaluate` serves as `extract` (returning `&A`, which is close but not identical to an owned value).

## 5. Type Class Implementations: Detailed Analysis

### RefFunctor (RcLazy only)

```rust
fn ref_map<'a, A: 'a, B: 'a>(
    f: impl FnOnce(&A) -> B + 'a,
    fa: Lazy<'a, A, RcLazyConfig>,
) -> Lazy<'a, B, RcLazyConfig> {
    fa.ref_map(f)  // delegates to inherent method
}
```

Correctly creates a new `RcLazy` that captures the original and applies `f` to the reference when forced. Uses `FnOnce` since the mapped lazy evaluates at most once.

### SendRefFunctor (ArcLazy only)

Same pattern but with `Send` bounds on `f`, `A`, and `B`. Delegates to `ArcLazy::ref_map`.

Note: `ArcLazy` does NOT implement `RefFunctor`. The doc comment on `ArcLazy::ref_map` explains why: `RefFunctor::ref_map` does not require `Send` on the mapping function, but `ArcLazy::new` does. This is the correct decision.

### Foldable (generic over Config)

Generic implementation for all `LazyBrand<Config>`. Clones the value out of the cache to fold. The `Clone` bound on `A` is necessary and appropriate.

### Deferrable

- **RcLazy:** `defer(f)` creates a new `RcLazy` that, when forced, calls `f()` to get an inner `RcLazy`, forces *that*, and clones the result. Requires `A: Clone`. This is correct: it flattens `Lazy<Lazy<A>>` into `Lazy<A>`.
- **ArcLazy:** `defer(f)` calls `f()` **eagerly** and returns the result directly. This is a compromise because `Deferrable::defer` does not require `Send` on the thunk, but `ArcLazy::new` does. The documentation explicitly warns about this. This is an honest trade-off, well-documented.

### SendDeferrable (ArcLazy only)

`send_defer(f)` creates a properly deferred `ArcLazy` because the `Send` bound on `f` satisfies `ArcLazy::new`'s requirements. Flattens by forcing the inner and cloning. Requires `A: Clone + Send + Sync`.

### Semigroup and Monoid

Both create new `Lazy` values that force both operands and combine. Require `A: Clone` to extract values from the shared cache. The implementations are straightforward and correct.

### Standard Library Traits

`PartialEq`, `PartialOrd`, `Eq`, `Ord`, `Hash`, `Display` all force evaluation to delegate to the inner value. `Debug` intentionally does NOT force evaluation, printing `"Lazy(..)"` instead. This is a thoughtful design choice: `Debug` should be safe to call for diagnostics without side effects.

`Clone` shares the underlying cache (increments the reference count). This is correct and essential.

## 6. LazyConfig Trait: Detailed Assessment

### Architecture

```
LazyConfig (infallible)
    |-- PointerBrand
    |-- Lazy<'a, A>
    |-- Thunk<'a, A>
    |-- lazy_new()
    |-- evaluate()
    |
    +-- TryLazyConfig (fallible, extends LazyConfig)
        |-- TryLazy<'a, A, E>
        |-- TryThunk<'a, A, E>
        |-- try_lazy_new()
        |-- try_evaluate()
```

**Pros:**
- Clean separation of infallible and fallible memoization.
- Open for third-party extension without modifying existing code.
- The `PointerBrand` link allows traversing from a config to the pointer hierarchy.
- The `'static` bound on `LazyConfig` itself prevents lifetime-infected configs, keeping the trait object-friendly.

**Cons:**
- Two configs (`RcLazyConfig`, `ArcLazyConfig`) is the only realistic usage. The extensibility story is nice in theory but unlikely to be exercised in practice.
- The separation of `LazyConfig` and `TryLazyConfig` means that `TryLazy` has its own parallel set of associated types that structurally duplicate the infallible ones (just wrapping in `Result`). This is some conceptual overhead, though it keeps the infallible path clean for consumers who do not need error handling.

**Verdict:** This is appropriate engineering for a library that explicitly targets the Rc/Arc abstraction boundary. The extensibility is a low-cost bonus, not wasted complexity.

## 7. Lifetime Handling

`Lazy<'a, A, Config>` is generic over lifetime `'a`, which bounds the initializer closure. This allows capturing references with limited lifetimes:

```rust
let local = String::from("hello");
let lazy = RcLazy::new(|| local.len()); // 'a ties to local's scope
```

Key observations:

- `LazyConfig: 'static` means the config types themselves carry no borrowed data. Good.
- `A: 'a` is enforced everywhere, ensuring the computed value outlives the closure's borrows.
- `ArcLazyConfig`'s thunk type adds `Send + 'a`, correctly threading both bounds.
- The `From<Trampoline<A>> for RcLazy` requires `A: 'static` because `Trampoline` itself is `'static`-only. This is correctly propagated.
- The `From<Thunk> for ArcLazy` eagerly evaluates the thunk because `Thunk` is `!Send`. This is the correct handling of the lifetime/Send interaction.

The lifetime design is sound and consistent throughout.

## 8. Documentation Quality

### Strengths

- The module-level doc comment clearly explains why `Lazy` does not implement `Functor` and directs users to `RefFunctor` / `SendRefFunctor`.
- Every public function, trait impl, and method has doc comments with the library's documentation macros (`#[document_signature]`, `#[document_type_parameters]`, `#[document_parameters]`, `#[document_returns]`, `#[document_examples]`).
- Code examples are provided for virtually every public API surface, and they include assertions.
- The `Deferrable` impl for `ArcLazy` explicitly documents the eager-evaluation compromise.
- The `fix` combinators document both the memory-leak hazard (dropping without evaluation) and the panic/deadlock hazard (reentrant evaluation).
- The `RefFunctor` trait doc explains why `RefFunctor` and `SendRefFunctor` are independent traits rather than a subtype hierarchy.
- Cache chain behavior is documented on the `RefFunctor` trait.

### Weaknesses

- The `Lazy` struct's doc comment does not mention the `Clone` requirement that pervades many of its trait impls (`Deferrable`, `Semigroup`, `Monoid`, `Foldable`). A user might try to use `Lazy` with a non-`Clone` type and be surprised by the restricted API surface.
- The `ArcLazy::ref_map` inherent method has a good doc comment explaining why it is not a blanket `RefFunctor` impl, but a user discovering the type through the trait system might not find this explanation.
- `pure` is an inherent method, not from any trait. This could be confusing for users expecting `Applicative::pure`.

### Test Coverage

The test suite is thorough:
- Memoization caching and sharing (both Rc and Arc).
- Thread safety for `ArcLazy`.
- All `From` conversions (Thunk, Trampoline, SendThunk, RcLazy-to-ArcLazy, ArcLazy-to-RcLazy).
- Eager evaluation on cross-boundary conversions.
- QuickCheck property tests for round-trip preservation, RefFunctor laws, Deferrable transparency, Semigroup associativity, Monoid identity.
- Fix combinator tests (constant, memoization, clone sharing, thread safety).
- Foldable and FoldableWithIndex tests with consistency checks.
- SendRefFunctor laws (identity, composition, memoization preservation).

Overall documentation quality is high.

## 9. Issues, Limitations, and Design Flaws

### Issue 1: ArcLazy's `Deferrable` is Semantically Broken

`ArcLazy::defer(f)` calls `f()` eagerly, making it not actually deferred at all. While documented, this violates the spirit of the `Deferrable` trait. The transparency law ("defer(|| x) is identical to x") is technically satisfied because the result equals `x`, but the trait's name and purpose imply deferred evaluation.

**Severity:** Medium. Users relying on `Deferrable` for lazy construction will get eager evaluation with `ArcLazy`. The `SendDeferrable` alternative is properly deferred, but the two traits have different closure requirements (`FnOnce` vs `FnOnce + Send`), so users cannot always switch.

### Issue 2: Memory Chains from ref_map

Each `ref_map` call creates a new `Lazy` that retains a reference to its predecessor. A chain of N `ref_map` calls creates N+1 `Rc`/`Arc` cells, all kept alive. This is documented on the `RefFunctor` trait, but there is no mechanism to collapse chains or eagerly evaluate intermediate steps.

**Severity:** Low. This is inherent to the reference-based memoization design and matches how PureScript's `Lazy` works under the hood. Users can manually force and re-wrap if they need to break chains.

### Issue 3: Fix Combinators Leak Memory If Not Evaluated

`rc_lazy_fix` and `arc_lazy_fix` create reference cycles (`Rc`/`Arc` cycles through `OnceCell`/`OnceLock`). The cycle is broken when the lazy value is first evaluated, but if it is dropped without evaluation, the memory leaks. This is documented but is a real footgun.

**Severity:** Low-Medium. The fix combinators are an advanced API for recursive definitions. The documentation is clear about the limitation. There is no good alternative in Rust without weak references (which would add complexity).

### Issue 4: Box Allocation in Constructor

`lazy_new` takes `Box<Self::Thunk<'a, A>>`, meaning every `Lazy::new(f)` boxes the closure before wrapping it in `Rc`/`Arc`. This is two allocations: one for the `Box` and one for the `Rc`/`Arc`. `LazyCell::new` takes a closure by value, so `Rc::new(LazyCell::new(f))` would be a single allocation if the closure were passed directly. The boxing exists because the `LazyConfig` trait uses a `dyn FnOnce()` trait object to erase the closure type, which is necessary for the config trait to be generic over the closure's concrete type.

**Severity:** Low. The extra allocation is small and constant. The trait-based design requires type erasure, making this unavoidable without specialization or associated type GATs that can carry the closure type.

### Issue 5: Semigroup/Monoid Require Clone

`Semigroup::append` for `Lazy` requires `A: Clone` because it must extract values from two shared caches to combine them. This propagates `Clone` requirements to any downstream usage (e.g., `fold` over a list of `Lazy` values using `Semigroup`). In PureScript, `Semigroup` for `Lazy` does not require any equivalent constraint because GC handles ownership.

**Severity:** Low. This is a fundamental consequence of Rust's ownership model and is shared by many Rust FP encodings.

### Issue 6: No Eq/Ord for Inner Lazy Cell

`PartialEq`, `Eq`, `PartialOrd`, and `Ord` force evaluation of both sides. This means comparing two `Lazy` values is not referentially transparent for side-effecting closures (e.g., a closure that increments a counter). This is documented behavior and matches PureScript.

**Severity:** Informational. This is the expected semantics.

## 10. Alternatives and Improvements

### Alternative 1: CloneFunctor Trait

Introduce a `CloneFunctor` trait with `map` that requires `A: Clone`:

```rust
trait CloneFunctor: Kind!(...) {
    fn clone_map<'a, A: Clone + 'a, B: 'a>(
        f: impl FnOnce(A) -> B + 'a,
        fa: Self::Of<'a, A>,
    ) -> Self::Of<'a, B>;
}
```

This would let `Lazy` participate in a mapping abstraction that acknowledges the `Clone` cost. It would be a separate trait from `Functor` but could enable more generic programming over `Lazy` types.

**Trade-off:** Adds a new trait to the hierarchy. May cause confusion about when to use `Functor` vs `CloneFunctor`.

### Alternative 2: Weak References for Fix Combinators

Use `Rc::downgrade` / `Arc::downgrade` in the fix combinators to avoid reference cycles. The self-reference would be a `Weak<...>` that is upgraded when needed. This eliminates the memory leak if the lazy value is dropped without evaluation, at the cost of a potential upgrade failure (which should never happen in practice if the lazy value is still alive).

**Trade-off:** Slightly more complex implementation. Upgrade failure must be handled (panic or return an error).

### Alternative 3: Inline the Config

Instead of the `LazyConfig` trait, use a `const generic` or feature-flag approach to select Rc vs Arc. This would eliminate the trait indirection and the box allocation.

**Trade-off:** Less extensible, but simpler and potentially more performant. The current design's extensibility may not be exercised in practice.

### Alternative 4: `evaluate_owned` for `A: Clone`

Add an inherent method `evaluate_owned(&self) -> A where A: Clone` that returns a cloned owned value. This would make the `Clone` extraction pattern more discoverable and explicit, rather than having users write `lazy.evaluate().clone()` everywhere.

**Trade-off:** Minimal; this is a convenience method that does not change the semantics.

### Alternative 5: Lazy with Interior Mutability (Take Semantics)

Use `Rc<RefCell<Option<A>>>` or similar to allow "taking" the value out (converting from lazy to owned). After taking, the lazy becomes empty. This would enable a one-shot `Functor` impl.

**Trade-off:** Fundamentally changes the shared memoization semantics. The value would not be available to other clones after the first take. This contradicts the core design goal.

## Summary

`Lazy` is a well-designed memoized lazy evaluation type that makes the correct trade-offs for Rust's ownership model. The `LazyConfig` parameterization cleanly separates the Rc/Arc concern. The `RefFunctor` / `SendRefFunctor` split is the right response to the fundamental tension between shared memoization and Rust's `Functor` contract. The documentation is thorough, the test suite is comprehensive, and the limitations (no full `Functor`, `Clone` requirements, memory chains from `ref_map`, leak hazard in fix combinators) are inherent to the design space rather than bugs or oversights. The most actionable improvement would be adding an `evaluate_owned` convenience method and potentially using weak references in the fix combinators to eliminate the leak hazard.
