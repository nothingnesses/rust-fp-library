# Trampoline Analysis

**File:** `fp-library/src/types/trampoline.rs`

## 1. Type Design

`Trampoline<A>` is defined as a newtype wrapper around `Free<ThunkBrand, A>`:

```rust
pub struct Trampoline<A: 'static>(Free<ThunkBrand, A>);
```

This is notably *not* a type alias; it is a proper newtype struct. This is a good design choice because it:

- Provides a focused, ergonomic API without exposing the full `Free` machinery.
- Prevents users from accidentally mixing raw `Free` operations with `Trampoline` operations.
- Allows `Trampoline`-specific trait implementations (e.g., `Semigroup`, `Monoid`, `Debug`, `Deferrable`) that would otherwise conflict with or clutter `Free`'s own API.
- Enables dedicated `tail_rec_m` and `arc_tail_rec_m` methods that are tailored to the `Trampoline` use case.

### Trade-offs

**Pro:** Maximum code reuse. All the difficult parts (CatList-based bind, iterative evaluate, type erasure, stack-safe Drop) live in `Free` and are inherited by delegation.

**Con:** Every `Trampoline` method wraps/unwraps the newtype, creating a thin layer of indirection in the source. In practice this should be zero-cost after inlining, but it means the `Trampoline` API is a manual re-export of `Free`'s API. Each method (`pure`, `bind`, `map`, `evaluate`, `defer`) simply delegates to the inner `Free`. This creates maintenance overhead: if `Free`'s signature changes, every `Trampoline` method must be updated.

**Structural note:** The CLAUDE.md table describes `Trampoline<A>` as a type alias `Free<ThunkBrand, A>`, but the actual code uses a newtype wrapper. The table should be updated to reflect this, since it matters for API visibility and trait implementation.

## 2. Stack Safety

### Mechanism

Stack safety is achieved through the "Reflection without Remorse" technique implemented in `Free`:

1. **CatList-based continuation queue**: Each `bind` appends a continuation to a `CatList` in O(1) time via `snoc`. This avoids building deeply nested closures.
2. **Iterative evaluate loop**: `Free::evaluate` runs an iterative loop that processes `Pure`, `Wrap`, and `Bind` variants without recursion. `Wrap` layers are unwrapped via `Evaluable::evaluate` (which for `ThunkBrand` simply runs the thunk). `Bind` layers merge their continuation lists via `CatList::append` in O(1).
3. **`defer` for safe recursion**: `Trampoline::defer` wraps a computation-producing closure in `Free::wrap(Thunk::new(...))`, ensuring the recursive call itself is deferred rather than immediately executed on the stack.
4. **Iterative Drop**: `Free`'s `Drop` implementation uses a worklist to iteratively dismantle `Bind` chains and `Wrap` chains, preventing stack overflow during cleanup of deeply nested structures.

### Is the guarantee solid?

Yes, with a caveat. The guarantee holds for:

- Deep `bind` chains (handled by CatList + iterative evaluate).
- Deep `defer` chains (each `defer` becomes a `Wrap` node that the evaluate loop handles iteratively).
- Deep `tail_rec_m` recursion (each step uses `defer`, so it trampolines).
- Deep `Drop` (iterative dismantling via worklist).

The one weakness: `Free::hoist_free` is documented as *not* stack-safe for deeply nested `Wrap` chains, because it recurses once per `Wrap` layer. However, this method is on `Free`, not exposed through `Trampoline`, so it does not affect `Trampoline`'s safety guarantees directly.

## 3. HKT Support

### Why no brand?

`Trampoline` has no brand type and does not participate in the library's HKT system. This is a direct consequence of the `'static` requirement:

- The `Kind` trait requires `type Of<'a, A: 'a>: 'a`, meaning the type constructor must accept *any* lifetime `'a`.
- `Free` (and therefore `Trampoline`) requires `A: 'static` because it uses `Box<dyn Any>` for type erasure in the CatList of continuations. `dyn Any` requires `'static`.
- This creates an irreconcilable conflict: `Trampoline` cannot satisfy `Kind`'s lifetime polymorphism.

### Should it have one?

No, not under the current architecture. Adding a brand would require either:

1. Removing the `'static` requirement from `Free` (which would require a fundamentally different implementation strategy, likely sacrificing O(1) bind or stack safety).
2. Creating a `Kind` variant that only works for `'static` lifetimes (which would break the uniformity of the HKT system and create a confusing second-class citizen).

The current design correctly identifies this as a fundamental limitation and does not paper over it. Users who need HKT-compatible lazy computation should use `Thunk<'a, A>` (which has a brand via `ThunkBrand`), accepting the trade-off of less-than-guaranteed stack safety.

## 4. Type Class Implementations

### Currently implemented

| Trait | Notes |
|-------|-------|
| `Deferrable<'static>` | Delegates to `Trampoline::defer`. |
| `Semigroup` (where `A: Semigroup`) | Combines via `lift2` + `Semigroup::append`. |
| `Monoid` (where `A: Monoid`) | Uses `Trampoline::pure(Monoid::empty())`. |
| `Debug` | Constant string `"Trampoline(<unevaluated>)"`, does not force. |

### Conversions

| From | To | Notes |
|------|----|-------|
| `Lazy<'static, A, Config>` (where `A: Clone`) | `Trampoline<A>` | Clones the memoized value. |
| `Thunk<'static, A>` | `Trampoline<A>` | Wraps thunk evaluation in `Trampoline::new`. |
| `Trampoline<A>` | `Lazy<'a, A, RcLazyConfig>` | Evaluates lazily on first access. |
| `Trampoline<A>` | `Lazy<'a, A, ArcLazyConfig>` (where `A: Send + Sync`) | Evaluates eagerly (Trampoline is `!Send`). |
| `Trampoline<A>` | `TryTrampoline<A, E>` | Wraps in `Ok`. |

### Coverage assessment

The coverage is appropriate given the constraints. `Trampoline` cannot implement `Functor`, `Monad`, `Applicative`, or `MonadRec` as HKT traits because it lacks a brand. Instead, it provides equivalent functionality through inherent methods (`map`, `bind`, `pure`, `lift2`, `then`, `tail_rec_m`).

**Potential additions:**

- `Eq` / `PartialEq` could be derived for `Trampoline<A>` where `A: Eq`, but this would require eagerly evaluating, which conflicts with laziness semantics. The current `Debug` approach (not evaluating) is correct.
- `From<A>` for `Trampoline<A>` as a convenience (equivalent to `Trampoline::pure`). This is a stylistic choice; `pure` is already clear.

### Missing: `Foldable` and `Traversable`

PureScript's `Free` implements both `Foldable` and `Traversable`. This library's `Free` does not, and by extension neither does `Trampoline`. This is reasonable because:

- `Foldable` on `Free<ThunkBrand, A>` would just extract the single value (equivalent to `evaluate`), adding little value.
- `Traversable` would require HKT support, which is unavailable.

## 5. The `'static` Requirement

### Why it exists

The `'static` bound is a direct consequence of using `Box<dyn Any>` for type erasure in `Free`'s continuation queue. The Rust trait `Any` has an inherent `'static` bound:

```rust
pub trait Any: 'static { ... }
```

This is necessary because `Any` uses `TypeId` for runtime type identification, and `TypeId` can only distinguish types that are fully owned (no borrows). A `&'a str` with different lifetimes `'a` would have different types at the language level but the same `TypeId`, creating unsoundness.

### Is this fundamental?

Yes, given the "Reflection without Remorse" approach. The technique requires storing heterogeneous continuations in a single queue, which requires type erasure. In Rust, safe type erasure (`dyn Any` + `downcast`) requires `'static`. Alternative approaches exist:

1. **`unsafeCoerce` (PureScript's approach):** PureScript uses `unsafeCoerce` instead of `dyn Any`, bypassing the type system. Rust could use `unsafe` transmute, but this would sacrifice memory safety guarantees.
2. **GATs/existentials:** A hypothetical Rust feature for safe existential types could potentially eliminate the need for `dyn Any`, but no such feature exists today.
3. **Naive Free:** A `Free` without CatList could avoid type erasure entirely, but at the cost of O(n) bind and no stack safety guarantee.

The `'static` requirement is the correct trade-off for a Rust library prioritizing safety. The practical impact is limited: most computations use owned data, and the few that need borrowed references can use `Thunk<'a, A>` instead.

## 6. Comparison to PureScript

### PureScript's Trampoline

```purescript
type Trampoline = Free ((->) Unit)
```

PureScript defines `Trampoline` as a type *alias* for `Free` over the `Unit -> _` functor (a thunk). Key differences:

| Aspect | PureScript | Rust |
|--------|-----------|------|
| Definition | Type alias | Newtype wrapper |
| Base functor | `(->) Unit` (function from Unit) | `ThunkBrand` (brand for `Box<dyn FnOnce() -> A>`) |
| HKT | Full (inherits all of Free's instances: Functor, Monad, MonadRec, Foldable, Traversable, etc.) | None (cannot participate in HKT system) |
| Lifetime restriction | None (GC handles everything) | `'static` only |
| Type erasure | `unsafeCoerce` | `Box<dyn Any>` with safe `downcast` |
| API surface | Thin (3 functions: `done`, `delay`, `runTrampoline`) | Rich (pure, new, defer, bind, map, evaluate, lift2, then, append, empty, tail_rec_m, arc_tail_rec_m, into_rc_lazy, into_arc_lazy) |

### Structural comparison

PureScript's approach is remarkably minimal because `Trampoline` inherits everything from `Free`, which in turn inherits from `Functor`, `Monad`, `MonadRec`, `Semigroup`, `Monoid`, `Foldable`, `Traversable`, etc. The Rust version must reimplement each of these as inherent methods because the HKT system is inaccessible.

PureScript's `runTrampoline` is:
```purescript
runTrampoline = runFree (_ $ unit)
```

This is a single-line definition that unwraps one functor layer at a time. Rust's `Trampoline::evaluate` delegates to `Free::evaluate`, which uses the iterative trampoline loop with CatList processing.

### Key insight

PureScript can use `Free` as a universal abstraction (AST interpretation, trampolining, DSL embedding) because type erasure is free (via `unsafeCoerce` and GC). Rust's `Free` is more specialized: it is primarily a stack-safety mechanism, with `fold_free` offering limited interpretation capability. The Rust `Trampoline` wrapper makes this specialization explicit by providing a focused API.

## 7. Free-Based Approach: Overhead Analysis

### Overhead of delegating to Free

Each `Trampoline` operation pays for `Free`'s full machinery:

1. **`Trampoline::new(f)`** creates `Free::wrap(Thunk::new(move || Free::pure(f())))`. This involves:
   - One `Box` allocation for the `Thunk` closure.
   - One `FreeInner::Wrap` variant wrapping it.
   - The `Option<FreeInner>` wrapper for linear consumption.

2. **`Trampoline::bind(f)`** creates a type-erased continuation via `Free::bind`:
   - One `Box` allocation for the erased closure.
   - One `CatList::singleton` / `CatList::snoc` operation.
   - One `downcast` per continuation during evaluation.

3. **`Trampoline::evaluate()`** calls `Free::evaluate`:
   - `erase_type()` is called first, wrapping `A` in `Box<dyn Any>`.
   - Each continuation requires a `downcast` (safe but involves a `TypeId` comparison).
   - Each `Wrap` layer requires `Evaluable::evaluate` (for `ThunkBrand`, this calls the thunk).

### Compared to a dedicated Trampoline

A dedicated `Trampoline` enum (without Free) could look like:

```rust
enum Trampoline<A> {
    Done(A),
    More(Box<dyn FnOnce() -> Trampoline<A>>),
}
```

This avoids:
- Type erasure (`Box<dyn Any>`, `downcast`).
- The `Option` wrapper for linear consumption.
- CatList overhead for short chains.

But it loses:
- O(1) bind (naive `Trampoline` has O(n) left-associated bind).
- The CatList-based reassociation that prevents quadratic blowup.

For the common case of `defer`-based recursion (not heavy `bind` chains), the naive approach would be simpler and slightly faster. For monadic pipeline use cases with many `bind`s, the `Free`-based approach is strictly better due to O(1) bind.

### Verdict

The `Free`-based approach is the right choice for a general-purpose library. The overhead of type erasure is constant per operation and dwarfed by the algorithmic improvement from O(1) bind. The alternative would be to maintain two separate Trampoline implementations (one naive for simple recursion, one Free-based for monadic pipelines), which is not worth the complexity.

## 8. Documentation Quality

### Strengths

- Module-level documentation is clear and concise, explaining what trampolining is and why it exists.
- The "When to Use" section correctly differentiates `Trampoline` from `Thunk`.
- The `Memoization` section proactively warns users that `Trampoline` does not memoize and shows how to wrap in `Lazy`.
- Every public method has `#[document_signature]`, `#[document_parameters]`, `#[document_returns]`, and `#[document_examples]` annotations.
- The `tail_rec_m` documentation includes a complete Fibonacci example.
- The `defer` documentation includes a recursive sum example demonstrating stack safety.
- The `into_arc_lazy` documentation explains *why* eager evaluation is required (Trampoline is `!Send`).

### Weaknesses

- The `Clone` bound on `tail_rec_m`'s `f` parameter is documented but could benefit from a more concrete example of what fails without `Clone` and why `arc_tail_rec_m` exists.
- The module-level doc example could be slightly more motivating; it shows a trivial chain that does not actually demonstrate stack safety.
- No cross-reference to `TryTrampoline` for the fallible variant.
- The `Debug` implementation always returns `"Trampoline(<unevaluated>)"` even for `Trampoline::pure(42)` where the value is immediately available. This is a reasonable simplification but could be documented.

### Test coverage

Tests are comprehensive:
- Basic operations: `pure`, `new`, `bind`, `map`, `defer`.
- `tail_rec_m` and `arc_tail_rec_m` correctness.
- `lift2` and `then` combinators.
- `Semigroup` / `Monoid` via `append` and `empty`.
- Conversions: `From<Lazy>`, `From<Thunk>` (both `Rc` and `Arc` variants).
- `!Send` types (`Rc<T>`) across all operations.
- QuickCheck property tests for Functor laws (identity, composition) and Monad laws (left identity, right identity, associativity).
- Stack safety stress tests at 200,000 iterations for both `tail_rec_m` and `arc_tail_rec_m`.

## 9. Issues, Limitations, and Design Flaws

### 9.1. No `Send` variant

`Trampoline` is `!Send` because `Free` uses `Box<dyn FnOnce()>` (not `Box<dyn FnOnce() + Send>`) for type-erased continuations. There is no `SendTrampoline` equivalent. Users needing thread-safe stack-safe recursion must use `SendThunk` (which is not stack-safe) or find a workaround. The `into_arc_lazy` method handles the output side (converting a computed result to a `Send` type), but there is no way to *build* a stack-safe computation that is `Send`.

This is a genuine gap in the hierarchy. A `SendFree` or `SendTrampoline` would require `Send`-bounded continuations throughout `Free`, which would be a significant refactor.

### 9.2. `tail_rec_m` requires `Clone` on the step function

The `Clone` bound on `tail_rec_m`'s function parameter is a Rust-specific limitation. Each iteration of the inner `go` function captures `f` by value (via `move` in the `Trampoline::defer` closure), so `f` must be cloneable. The `arc_tail_rec_m` variant works around this by wrapping in `Arc`, but at the cost of atomic reference counting overhead.

In PureScript, this is a non-issue because closures are implicitly shared (GC-managed).

### 9.3. `map` is implemented via `bind`

`Trampoline::map` delegates to `Free::map`, which is implemented as `self.bind(|a| Free::pure(f(a)))`. This means every `map` creates a type-erased continuation and a `CatList` entry, even though map is semantically simpler than bind. A dedicated `FreeInner::Map` variant could optimize this, but it would add complexity to `Free`'s evaluate loop. This is an acceptable trade-off for simplicity.

### 9.4. `From<Lazy>` requires `Clone`

Converting `Lazy<'static, A, Config>` to `Trampoline<A>` requires `A: Clone` because `Lazy::evaluate` returns `&A` (a reference), not an owned `A`. The `Trampoline::new(move || lazy.evaluate().clone())` pattern copies the memoized value. This is documented but could surprise users with expensive `Clone` implementations.

### 9.5. No `flatten` / `join`

There is no `Trampoline<Trampoline<A>> -> Trampoline<A>` method. This can be achieved via `.bind(|x| x)`, but a dedicated `flatten` method would improve discoverability.

### 9.6. Debug does not differentiate variants

The `Debug` implementation always outputs `"Trampoline(<unevaluated>)"` regardless of whether the value is `Pure`, `Wrap`, or `Bind`. This prevents users from inspecting the structure during debugging. A more informative implementation could show `"Trampoline(Pure(...))"` for pure values (where `A: Debug`), though this would require evaluating for `Wrap`/`Bind` variants which conflicts with laziness semantics.

### 9.7. No `apply` / `ap` method

While `lift2` is provided, there is no `apply` method (`Trampoline<Fn(A) -> B>` applied to `Trampoline<A>`). This is a minor omission since `lift2` covers most use cases, but `apply` is a fundamental operation in functional programming.

## 10. Alternatives: Dedicated Trampoline Enum vs. Free Alias

### Option A: Current approach (newtype over Free)

```rust
pub struct Trampoline<A: 'static>(Free<ThunkBrand, A>);
```

**Pros:**
- Code reuse: CatList, evaluate loop, type erasure, stack-safe Drop are all shared with `Free`.
- O(1) bind via CatList reassociation.
- `fold_free` and `hoist_free` are available on the inner `Free` (not exposed but could be).
- Consistent with the library's overall design philosophy of building on `Free`.

**Cons:**
- Pays for `Free`'s generality even when only trampolining is needed.
- `'static` requirement inherited from `Free`'s `dyn Any` usage.
- No HKT support.
- `Box<dyn Any>` downcast overhead per continuation.

### Option B: Dedicated Trampoline enum

```rust
enum Trampoline<A> {
    Done(A),
    More(Box<dyn FnOnce() -> Trampoline<A>>),
}
```

**Pros:**
- Simpler implementation (no type erasure, no CatList, no Option wrapper).
- Could potentially support non-`'static` types (no `dyn Any`).
- Slightly less allocation per step.
- Easier to understand.

**Cons:**
- O(n) left-associated bind leads to quadratic blowup for monadic pipelines.
- Must separately implement stack-safe evaluate (simple iterative loop, but duplicates logic).
- Must separately implement stack-safe Drop.
- No path to `fold_free` or `hoist_free`.
- Would need a third variant `Bind(Box<Trampoline<dyn Any>>, Box<dyn FnOnce(dyn Any) -> Trampoline<A>>)` to get O(1) bind, at which point it has reimplemented Free.

### Option C: Dedicated Trampoline with CatList

A dedicated enum that embeds CatList directly for O(1) bind, without the `Free` abstraction layer:

```rust
enum Trampoline<A: 'static> {
    Done(A),
    More(Box<dyn FnOnce() -> Trampoline<A>>),
    Bind {
        head: Box<Trampoline<Box<dyn Any>>>,
        continuations: CatList<Box<dyn FnOnce(Box<dyn Any>) -> Trampoline<Box<dyn Any>>>>,
        _marker: PhantomData<A>,
    },
}
```

This is essentially `Free<ThunkBrand, A>` with `ThunkBrand` inlined. It would eliminate the `Evaluable` trait dispatch but otherwise be structurally identical. The marginal benefit is near zero, and it would duplicate all of `Free`'s evaluate/Drop logic.

### Recommendation

The current approach (Option A) is correct. The overhead of delegating to `Free` is negligible, and the code reuse benefit is substantial. Option B would only make sense if the library also wanted to support non-`'static` trampolining (which would sacrifice O(1) bind). Option C would be premature optimization with no practical benefit.

The one change worth considering is exposing `resume` and `fold_free` through the `Trampoline` API for users who need to inspect or interpret the computation structure. Currently these are only available on the inner `Free`.
