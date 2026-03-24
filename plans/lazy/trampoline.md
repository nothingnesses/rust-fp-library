# Trampoline Analysis

**File:** `fp-library/src/types/trampoline.rs`
**PureScript references:** `Data.Lazy`, `Control.Lazy`

## 1. Design Decision: Newtype vs Type Alias

`Trampoline<A>` is a newtype struct wrapping `Free<ThunkBrand, A>`, not a type alias. This is the correct choice for Rust. A newtype:

- Provides a clean, focused API surface that hides the `Free` machinery.
- Allows inherent methods (`Trampoline::new`, `Trampoline::defer`, etc.) without polluting `Free`'s API.
- Enables `From` conversions (`From<Thunk>`, `From<Lazy>`) and trait implementations (`Deferrable`, `Debug`) specific to trampolines.
- Prevents users from accidentally using low-level `Free` operations (like `wrap`, `lift_f`, `resume`, `erase_type`) on what should be a simple stack-safe computation type.

The zero-cost abstraction is maintained since the newtype is transparent at runtime.

**Verdict:** Sound design.

## 2. The `'static` Requirement

The `'static` bound on `A` is an unavoidable consequence of `Free`'s use of `Box<dyn Any>` for type erasure (required by the "Reflection without Remorse" technique). This is well-documented in both the `Trampoline` and `Free` module docs.

### Impact

- Cannot hold borrowed references (e.g., `Trampoline<&str>` is impossible).
- Cannot participate in the library's HKT trait hierarchy, which requires lifetime polymorphism (`type Of<'a, A: 'a>`).

### Is this acceptable?

Yes, for the intended use case. `Trampoline` is positioned as the "heavy-duty" type for deep recursion, not as a general-purpose HKT container. Users who need lifetimes use `Thunk`; users who need stack safety use `Trampoline`. The hierarchy table in CLAUDE.md makes this tradeoff explicit.

### Could it be relaxed?

Not without fundamental changes to `Free`. The `Any` trait requires `'static` for memory safety. Alternative approaches (unsafe transmutes, custom vtables) would sacrifice safety or introduce unsoundness. The tradeoff is appropriate.

## 3. Comparison with PureScript

### PureScript's `Data.Lazy`

PureScript's `Lazy a` is a memoized, lazy container: it computes at most once and caches the result. It implements a full suite of type classes: `Functor`, `Apply`, `Applicative`, `Bind`, `Monad`, `Extend`, `Comonad`, `Foldable`, `Traversable`, `Eq`, `Ord`, `Show`, `Semigroup`, `Monoid`, `Semiring`, `Ring`, `BooleanAlgebra`, `HeytingAlgebra`, and the `Lazy` class from `Control.Lazy`.

### PureScript's `Control.Lazy`

The `Lazy` class (`defer :: (Unit -> l) -> l`) is a type class for deferred construction, plus `fix :: Lazy l => (l -> l) -> l` for lazy self-reference.

### How Rust's `Trampoline` relates

Rust's `Trampoline` is **not** an analogue of PureScript's `Data.Lazy`. Instead:

- PureScript `Data.Lazy` corresponds to Rust's `Lazy<'a, A, Config>` (memoized, single-evaluation).
- PureScript `Control.Lazy` (the class) corresponds to Rust's `Deferrable` trait.
- Rust's `Trampoline` corresponds more closely to PureScript's `Aff` or the trampoline pattern from `Control.Monad.Free` / `Control.Monad.Trampoline`.

The file is not really "based on" `Data.Lazy.purs` in a direct sense. The relationship is structural: both ecosystems need a way to defer and sequence computations. The Rust implementation splits PureScript's single `Lazy` concept across multiple types (`Thunk`, `Trampoline`, `Lazy`) due to Rust's ownership model and lack of pervasive laziness.

## 4. API Surface Evaluation

### What is provided

| Method | Purpose | Complexity |
|--------|---------|------------|
| `pure(a)` | Wrap an immediate value | O(1) |
| `new(f)` | Defer a computation | O(1) |
| `defer(f)` | Defer construction of a `Trampoline` itself | O(1) |
| `bind(f)` | Monadic bind | O(1) amortized |
| `map(f)` | Functor map | O(1) |
| `evaluate()` | Force evaluation | O(n) in bind chain length |
| `lift2(other, f)` | Combine two trampolines | O(1) construction |
| `then(other)` | Sequence, discard first | O(1) construction |
| `append(other)` | Semigroup combine | O(1) construction |
| `empty()` | Monoid identity | O(1) |
| `tail_rec_m(f, init)` | Stack-safe tail recursion | O(n) in iterations |
| `arc_tail_rec_m(f, init)` | Same, for non-Clone closures | O(n) in iterations |
| `memoize()` | Convert to `Lazy<RcLazyConfig>` | O(1) |
| `memoize_arc()` | Eagerly evaluate, wrap in `ArcLazy` | O(n) + O(1) |

### What is missing or could be improved

1. **No `ap` / `apply` method.** The `Applicative` pattern (`Trampoline<F> applied to Trampoline<A>`) is missing. `lift2` partially fills this gap, but a direct `ap` would be more compositional. This is a minor gap since `bind` subsumes `ap`.

2. **No `flatten` / `join`.** `Trampoline<Trampoline<A>> -> Trampoline<A>` would be a natural convenience method. Currently achievable via `t.bind(|inner| inner)`.

3. **No `and_then` alias.** Rust convention for monadic bind is `and_then` (see `Option`, `Result`). Having only `bind` is consistent with the FP library's naming, but a Rust-idiomatic alias could improve discoverability.

4. **`memoize_arc` eagerly evaluates.** The documentation correctly notes this ("evaluated eagerly because its inner closures are not `Send`"), but this is a sharp edge. Users might expect `memoize_arc` to defer evaluation like `memoize` does. The naming could be clearer, e.g., `evaluate_into_arc_lazy`.

5. **`From<Lazy>` requires `Clone`.** Converting `Lazy -> Trampoline` clones the cached value. This is documented but could surprise users. The `Clone` bound makes it impossible to convert a `Lazy<'static, Box<dyn SomeTrait>>` to a `Trampoline`.

## 5. Implementation Correctness

### `Trampoline::new`

```rust
pub fn new(f: impl FnOnce() -> A + 'static) -> Self {
    Trampoline(Free::wrap(Thunk::new(move || Free::pure(f()))))
}
```

This creates a `Wrap(Thunk(|| Pure(f())))` structure. On evaluation, the thunk is forced, producing a `Pure` value. This is correct and efficient.

### `Trampoline::defer`

```rust
pub fn defer(f: impl FnOnce() -> Trampoline<A> + 'static) -> Self {
    Trampoline(Free::wrap(Thunk::new(move || f().0)))
}
```

This unwraps the inner `Free` from the produced `Trampoline`. This is correct and is the key to stack-safe recursion: the recursive call is inside a thunk, so the stack frame is released before the next iteration.

### `Trampoline::bind`

Delegates to `Free::bind`, which appends to the `CatList`. O(1) amortized. Correct.

### `Trampoline::map`

Delegates to `Free::map`, which uses the `Map` variant. This avoids the type-erasure roundtrip of going through `bind`. Correct and efficient.

### `tail_rec_m`

```rust
fn go<A: 'static, B: 'static, F>(f: F, a: A) -> Trampoline<B>
where F: Fn(A) -> Trampoline<Step<A, B>> + Clone + 'static {
    let f_clone = f.clone();
    Trampoline::defer(move || {
        f(a).bind(move |step| match step {
            Step::Loop(next) => go(f_clone.clone(), next),
            Step::Done(b) => Trampoline::pure(b),
        })
    })
}
```

This is stack-safe because:
- Each iteration is deferred via `Trampoline::defer`, which wraps in a `Thunk`.
- The `bind` is O(1) thanks to `CatList`.
- Recursion in `go` is inside the deferred closure, so it does not grow the Rust call stack during construction.

However, the `Clone` requirement on `f` is a notable ergonomic burden. The `arc_tail_rec_m` variant addresses this by wrapping in `Arc`, which is the right approach.

**Potential issue:** Each recursive step clones `f` twice: once for `f_clone` and once inside the `bind` closure (`f_clone.clone()`). For closures that capture large state, this could be expensive. The `Arc` variant avoids this since `Arc::clone` is cheap.

## 6. Documentation Quality

### Strengths

- Clear explanation of when to use `Trampoline` vs `Thunk`.
- Explicit about the `'static` requirement and why.
- Good examples for each method, including a recursive sum for `defer`.
- The `memoize` example shows the `Lazy` wrapping pattern.
- All doc examples appear to be testable and correct.

### Issues

1. **Module-level doc says "Built on the `Free` monad"** but does not link to `Free`'s documentation for users who want to understand the internals.

2. **The `Trampoline::new` doc says "does NOT memoize"** which is good, but the memoization example uses `Lazy::<_, RcLazyConfig>::new(|| Trampoline::new(|| 1 + 1).evaluate())`. This works but is verbose. The `memoize()` method is more ergonomic and should be mentioned here instead.

3. **`Debug` implementation always prints `"Trampoline(<unevaluated>)"`**, even for `Trampoline::pure(42)` where the value is immediately available. This is a deliberate simplification (since inspecting `Free` variants would be complex), but it could be confusing for debugging. A `Pure` case could show the value if `A: Debug`.

4. **The `#[document_parameters]` on `evaluate` is empty** (line 241: `#[document_parameters]` with no arguments). Since `evaluate` takes `self`, this is technically fine but slightly inconsistent with other methods that document `self` as "The Trampoline instance."

## 7. Missing Trait Implementations

### What `Trampoline` implements

- `Deferrable<'static>` (via inherent `defer` + trait impl).
- `From<Thunk<'static, A>>`.
- `From<Lazy<'static, A, Config>>` (requires `A: Clone`).
- `Debug` (always prints `<unevaluated>`).

### What is notably absent

1. **No `Semigroup` / `Monoid` trait implementations.** The `append` and `empty` methods exist as inherent methods, but `Trampoline` does not implement the library's `Semigroup` and `Monoid` traits. This means `Trampoline` cannot be used generically where `Semigroup` or `Monoid` bounds are required. This is likely because `Trampoline` lacks a brand and cannot participate in HKT, making trait impls for it somewhat orphaned from the rest of the type class hierarchy.

2. **No `Eq` / `PartialEq`.** Comparing trampolines would require evaluating them, which is destructive (consumes `self`). PureScript's `Lazy` implements `Eq` because `force` is non-destructive (memoized). This is an inherent limitation of non-memoized lazy computation in Rust.

3. **No `Display` or `Show`-equivalent.** Same reason as `Eq`.

4. **No `Clone`.** `Free` contains `Box<dyn FnOnce(...)>` continuations that cannot be cloned. This is inherent.

5. **No `Send` / `Sync`.** `Thunk` uses `Box<dyn FnOnce()>` which is `!Send`. This is by design; the library provides `SendThunk` for thread-safe scenarios, but there is no `SendTrampoline`. This is a gap: users who need stack-safe computation across threads have no direct option. They must use `Trampoline` on one thread and transfer the result.

## 8. Edge Cases and Performance Concerns

### Allocation overhead

Every `Trampoline::new` allocates:
- One `Box<dyn FnOnce()>` for the `Thunk`.
- One `Box<Free<ThunkBrand, A>>` for the inner `Wrap` variant (indirectly via `Free`).

For very short computations (e.g., wrapping a constant), this overhead is non-trivial. `Trampoline::pure` avoids the thunk allocation but still allocates for the `FreeInner::Pure` variant (though this is just the value itself inside an `Option<FreeInner>`).

### CatList amortized cost

The `CatList` provides O(1) amortized `snoc` and `append`, but individual `uncons` operations during evaluation may trigger O(n) rebalancing. This is the standard amortized guarantee and is well-understood.

### Memory usage for deep chains

Each `bind` adds a boxed closure to the `CatList`. For very long chains (millions of binds), this can consume significant heap memory. This is the expected tradeoff for stack safety.

### Drop safety

`Free` has a custom `Drop` implementation that iteratively walks `Bind` and `Map` chains to prevent stack overflow during destruction. This is important and correct. Without it, dropping a deep `Free` chain would recurse through destructors and overflow the stack.

## 9. Comparison with Other Ecosystem Trampolines

### `tailcall` crate

The `tailcall` crate provides a proc macro `#[tailcall]` that rewrites recursive functions into loops. This is zero-cost but only works for tail-recursive functions. `Trampoline` is more general: it supports arbitrary monadic composition, not just tail recursion.

### Manual trampolining

A simpler trampoline pattern in Rust is:

```rust
enum Trampoline<A> {
    Done(A),
    More(Box<dyn FnOnce() -> Trampoline<A>>),
}
```

This is simpler but has O(n) bind (each bind wraps another layer). The `Free`-based approach with `CatList` provides O(1) amortized bind, which matters for left-associated bind chains.

## 10. Summary of Findings

### Sound aspects

- Newtype over `Free<ThunkBrand, A>` is the right design.
- `'static` requirement is well-justified and unavoidable given the implementation strategy.
- API surface covers the essential operations (pure, new, defer, bind, map, evaluate, tail_rec_m).
- O(1) bind via CatList is a significant advantage over naive trampolines.
- Documentation is thorough with good examples.
- Tests cover basic operations, monad laws (via QuickCheck), `!Send` types, stack safety stress tests, and conversions.
- The `tail_rec_m` / `arc_tail_rec_m` split is pragmatic.

### Issues and improvement opportunities

1. **`memoize_arc` eagerly evaluates** without making this obvious from the name. Consider renaming to `evaluate_into_arc_lazy` or adding a prominent warning.
2. **Double-clone in `tail_rec_m`**: each recursive step clones `f` twice. This is not incorrect but is suboptimal for closures with expensive-to-clone captures. Consider restructuring to clone once per iteration.
3. **No `SendTrampoline`**: users needing stack-safe computation across thread boundaries have no direct solution. This is a notable gap in the hierarchy.
4. **`Debug` is uninformative for `Pure` values**: could show the value when `A: Debug`.
5. **Module doc memoization example is verbose**: should reference `memoize()` method instead of manual `Lazy` wrapping.
6. **`Semigroup`/`Monoid` as inherent methods only**: cannot be used generically with trait bounds. This is a consequence of no HKT brand but worth noting.
