# Coyoneda Implementation Analysis

This document analyzes the two Coyoneda implementations in the library, identifies
flaws, limitations, and issues, and proposes approaches to address each one.

Files under analysis:

- `/home/nixos/projects/rust-fp-lib/fp-library/src/types/coyoneda.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/types/coyoneda_explicit.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/classes/functor.rs`
- `/home/nixos/projects/rust-fp-lib/fp-library/src/brands.rs`

---

## 1. CoyonedaExplicit boxes every composed function, defeating "zero-cost" claims

**Location:** `coyoneda_explicit.rs`, lines 130-138 (`new`), 167-175 (`map`), 434-439 (`lift`).

**Problem:** The module documentation (line 1) claims "zero-cost map fusion" and the
comparison table (line 22) claims "0" heap allocations per map. However, every
construction path boxes the function:

- `lift` at line 438: `Box::new(identity)`
- `new` at line 137: `Box::new(f)`
- `map` at line 174: `Box::new(compose(f, self.func))`

Each call to `map` allocates a new `Box<dyn Fn(B) -> C>`. While the `compose` function
itself (in `functions.rs` line 88-93) returns an `impl Fn(A) -> C` with no allocation,
wrapping it in `Box::new(...)` immediately allocates on the heap. For k chained maps,
there are k+1 heap allocations (one for lift, one per map). The old box is dropped each
time, but the allocation still happens.

This is strictly better than `Coyoneda` (which allocates 2 boxes per map: one for the
function and one for the `CoyonedaMapLayer` struct), but the documentation's "zero-cost"
and "0 heap allocations per map" claims are inaccurate.

**Why the box exists:** The `func` field must be stored in the struct, and its type changes
with each `map` call (the composed closure has a different, unnameable type). Since Rust
structs have fixed layouts, the function must be type-erased somehow. `Box<dyn Fn>` is the
standard approach.

### Approach A: Use a generic function type parameter instead of `Box<dyn Fn>`

Change the struct to:

```rust
pub struct CoyonedaExplicit<'a, F, B, A, Func>
where
    F: Kind_cdc7cd43dac7585f + 'a,
    Func: Fn(B) -> A + 'a,
{
    fb: <F as Kind_cdc7cd43dac7585f>::Of<'a, B>,
    func: Func,
}
```

Each `map` returns a `CoyonedaExplicit<'a, F, B, C, impl Fn(B) -> C>` whose `Func` is
the composed closure. No boxing, no heap allocation, truly zero-cost.

**Trade-offs:**

- (+) Genuinely zero-cost: no heap allocation, no dynamic dispatch.
- (+) Compiler can inline the entire composed function chain.
- (-) The type becomes complex and unnameable after several maps because `Func` is a
  nested `impl Fn` type. Users cannot store the value in a struct field without
  type-erasing it themselves.
- (-) `lift` would need to use `identity` as the function, which means the initial type
  is `CoyonedaExplicit<'a, F, A, A, fn(A) -> A>`. This is fine.
- (-) Methods like `into_coyoneda`, `hoist`, `apply`, and `bind` that construct new
  `CoyonedaExplicit` values would need to propagate the `Func` parameter, increasing
  complexity.

**Recommendation:** This is the correct approach if the type is meant to be used in
pipelines (constructed, chained, lowered in one expression). The unnameable type is not a
problem in that usage pattern. Provide a `.boxed()` method that type-erases the function
for users who need to store the value. This matches how Rust futures work (unnameable
types with `.boxed()` for storage).

### Approach B: Correct the documentation

Change the documentation claims to accurately describe the allocation behavior: one heap
allocation per map (the `Box<dyn Fn>`), but only one call to `F::map` at lower time.

**Trade-offs:**

- (+) Simple, no code change.
- (-) The type loses its distinguishing advantage over manual `compose` + single `map`.

**Recommendation:** If Approach A is not pursued, at minimum the documentation must be
corrected. The table at line 22 should say "1 box" not "0" for heap allocation per map.

---

## 2. Coyoneda does not perform map fusion at all

**Location:** `coyoneda.rs`, lines 207-250 (`CoyonedaMapLayer`), lines 394-402 (`map`).

**Problem:** Each call to `Coyoneda::map` creates a new `CoyonedaMapLayer` containing:

- `Box<dyn CoyonedaInner>` (the previous layer) -- one heap allocation
- `Box<dyn Fn(B) -> A>` (the new function) -- one heap allocation

At `lower` time (line 244-249), each layer calls `F::map` independently:

```rust
fn lower(self: Box<Self>) -> ... {
    let lowered = self.inner.lower();
    F::map(self.func, lowered)
}
```

For k chained maps on a `Vec` of n elements, this performs k separate traversals of the
collection, creating k intermediate `Vec` values. This is O(k \* n) allocations and work,
exactly as bad as calling `F::map` directly k times. The `Coyoneda` wrapper provides no
performance benefit whatsoever for eager functors.

The documentation at lines 38-44 already acknowledges this, which is good. However, the
existence of a type called `Coyoneda` that does not perform fusion may mislead users who
are familiar with Haskell or PureScript's Coyoneda, where fusion is the primary purpose.

**Root cause:** Rust's trait objects cannot have generic methods. The `CoyonedaInner` trait
cannot have a `map_inner<C>(&self, f: impl Fn(A) -> C) -> ...` method because that would
make it non-dyn-compatible. Without this, there is no way to compose functions across the
existential boundary.

### Approach A: Enum-based Coyoneda with a fixed set of composition depths

Instead of layered trait objects, use an enum that stores the original value and a single
boxed function:

```rust
pub struct Coyoneda<'a, F, A>(Box<dyn CoyonedaInner<'a, F, A> + 'a>);
```

becomes:

```rust
pub struct Coyoneda<'a, F, A: 'a> where F: Kind_cdc7cd43dac7585f + 'a {
    inner: CoyonedaCore<'a, F, A>,
}
enum CoyonedaCore<'a, F, A: 'a> where F: Kind_cdc7cd43dac7585f + 'a {
    Base { fa: <F as Kind_cdc7cd43dac7585f>::Of<'a, A> },
    Mapped { inner: Box<dyn ErasedCoyoneda<'a, F, A> + 'a> },
}
```

where `ErasedCoyoneda` has a `lower` method and a `fold_map` method (not generic over
the monoid, but using a callback approach). This still cannot compose functions, but could
at least combine `fold_map` into a single pass.

**Trade-offs:**

- (+) Could enable `Foldable` without `Functor`.
- (-) Still cannot achieve map fusion for the general `lower` case.
- (-) More complex implementation.

**Recommendation:** This approach only partially addresses the problem. The fundamental
issue (no generic methods on trait objects) remains.

### Approach B: Deprecate Coyoneda in favor of CoyonedaExplicit for fusion use cases

Make the documentation very clear that `Coyoneda` is for HKT polymorphism (getting a
`Functor` instance for free), not for performance. Direct users to `CoyonedaExplicit`
for fusion. Consider renaming:

- `Coyoneda` -> keep as-is (it serves the "free functor for HKT" role)
- `CoyonedaExplicit` -> `FusedMap` or `MapFusion` (emphasizes its purpose)

**Trade-offs:**

- (+) Clear separation of concerns.
- (-) Users familiar with the Coyoneda name from other languages will still expect fusion.

**Recommendation:** Keep both types but improve documentation and naming to make the
trade-offs obvious. Consider a prominent module-level note in `coyoneda.rs` directing
users to `CoyonedaExplicit` when they want fusion.

---

## 3. CoyonedaExplicit::map has nested-closure stack overflow risk

**Location:** `coyoneda_explicit.rs`, line 174.

**Problem:** Each `map` call composes functions via `compose(f, self.func)`, which
produces `move |a| f(g(a))`. After k maps, the composed function is:

```
move |a| fk(move |a| f(k-1)(move |a| ... (move |a| f1(identity(a))) ... ))
```

Each invocation recurses through k nested closures. While the compiler may inline some
of these, it is not guaranteed, especially when the closures are behind `Box<dyn Fn>`.
Dynamic dispatch through `Box<dyn Fn>` prevents inlining entirely. For very deep chains
(thousands of maps), this could overflow the stack when the composed function is finally
called.

The test at line 558-564 chains 100 maps successfully, but this is a modest depth.

**Note:** The module documentation table at line 23 claims "No" for stack overflow risk.
This claim is inaccurate for very deep chains, because the composed function nests to
depth k and each call adds a stack frame when dispatched through `Box<dyn Fn>`.

### Approach A: Use an iterative function composition strategy

Store functions in a `Vec<Box<dyn Fn>>` and apply them iteratively at lower time instead
of nesting closures:

```rust
// Pseudocode
struct CoyonedaExplicit<'a, F, B, A> {
    fb: F::Of<'a, B>,
    // Instead of a single composed function, store a chain
    funcs: Vec<Box<dyn FnOnce(???) -> ??? + 'a>>,
}
```

**Trade-offs:**

- (-) Cannot be done type-safely in Rust without `Any` and downcasting, because each
  function has a different type signature.
- (-) Would lose the type-level tracking that makes the current design elegant.

**Recommendation:** This approach is impractical in Rust's type system.

### Approach B: Document the stack depth limitation

Add a note that for chains deeper than a few thousand maps, the composed function may
overflow the stack. Suggest using `CoyonedaExplicit` with the generic `Func` parameter
(Approach A from Issue 1), where the compiler can inline and optimize the chain.

**Trade-offs:**

- (+) Honest documentation.
- (-) Does not fix the problem.

**Recommendation:** If the boxed approach is kept, document this limitation. If the
generic `Func` approach from Issue 1 is adopted, the compiler can flatten the closure
chain through inlining, making the stack overflow much less likely (though still possible
for sufficiently deep chains, the threshold would be much higher).

### Approach C: Trampoline the composed function

Wrap the function application in a trampoline for stack safety. At lower time, instead
of calling `F::map(self.func, self.fb)`, use the library's existing `Trampoline` type.

**Trade-offs:**

- (-) Trampolining adds overhead per element, defeating the fusion purpose.
- (-) Requires `'static` lifetime (the library's `Trampoline` requires `'static`).

**Recommendation:** Not recommended. The overhead would negate the fusion benefit.

---

## 4. CoyonedaExplicit uses `Fn` instead of `FnOnce`, preventing move semantics

**Location:** `coyoneda_explicit.rs`, lines 96, 131, 168, 199.

**Problem:** The stored function is `Box<dyn Fn(B) -> A>` and all function parameters
use `impl Fn`. This means:

1. The function must be callable multiple times, even though in many cases (e.g.,
   `Option::map`) it is called at most once.
2. Closures that move out of captured variables cannot be used. For example:

```rust
let s = String::from("hello");
// This would fail: closure moves `s` but `Fn` requires `&self`
CoyonedaExplicit::<OptionBrand, _, _>::lift(Some(1)).map(move |_| s)
```

This is a consequence of `F::map` in the `Functor` trait taking `impl Fn(A) -> B`
(line 121-124 of `functor.rs`). Since `lower` delegates to `F::map`, the function must
be `Fn`, not `FnOnce`.

### Approach A: Change the Functor trait to use `FnOnce`

This would be a fundamental change to the library's core type class.

**Trade-offs:**

- (+) More flexible, allows move semantics.
- (-) Breaks existing code throughout the library.
- (-) `FnOnce` closures cannot be stored in `Box<dyn Fn>` for reuse.
- (-) `Foldable::fold_map` needs the function to be called multiple times.

**Recommendation:** Not recommended as a wholesale change. `Fn` is the correct choice
for the general Functor trait because functors like `Vec` call the function multiple
times.

### Approach B: Accept this as an inherent limitation

The `Fn` bound is correct for the general case. `Vec::map` calls the function n times.
`Fn` is the only safe choice.

**Recommendation:** Document this as a known constraint. Users who need move semantics
should compose their pipeline to avoid capturing non-Clone, non-Copy values in the
mapping functions, or clone when necessary.

---

## 5. CoyonedaExplicit::apply and bind destroy fusion

**Location:** `coyoneda_explicit.rs`, lines 358-366 (`apply`), lines 395-402 (`bind`).

**Problem:** Both `apply` and `bind` call `self.lower()` immediately, which forces the
accumulated function to be applied via `F::map`. The result is then re-lifted with
`CoyonedaExplicit::lift(...)`, resetting the fusion pipeline. This means:

```rust
CoyonedaExplicit::<VecBrand, _, _>::lift(vec![1, 2, 3])
    .map(|x| x + 1)    // accumulated, not applied yet
    .map(|x| x * 2)    // accumulated, not applied yet
    .bind(|x| ...)      // forces lower() -> 1 call to F::map, then F::bind
    .map(|x| x + 10)   // accumulated again
    .map(|x| x * 3)    // accumulated again
    .lower()            // 1 call to F::map
```

The fusion is broken at every `apply`/`bind` boundary. For `apply`, both `ff` and `fa`
are lowered, which means two calls to `F::map` plus one call to `F::apply`.

This is semantically correct, but the user might expect that accumulated maps are somehow
preserved through `apply`/`bind`.

### Approach: Document the fusion boundary behavior

This is an inherent limitation. `bind` introduces data-dependent control flow that
prevents static fusion. `apply` requires the wrapped function and value to be in `F`'s
context, so lowering is unavoidable.

**Recommendation:** Add documentation to `apply` and `bind` explaining that they force
a fusion boundary. This is already partially done (lines 327 and 372 mention "the fusion
pipeline is reset"), but should be more prominent.

---

## 6. Coyoneda::Foldable requires F: Functor, unlike PureScript

**Location:** `coyoneda.rs`, line 533.

**Problem:** The `Foldable` implementation for `CoyonedaBrand<F>` requires
`F: Functor + Foldable + 'static`. PureScript's `Foldable` for `Coyoneda` only requires
`Foldable f` because it can open the existential via `unCoyoneda` and compose the fold
function with the accumulated mapping function directly.

The implementation at line 579 first lowers (`fa.lower()`, requiring `F: Functor`), then
folds the result. This creates an unnecessary intermediate `F A` allocation for eager
functors.

`CoyonedaExplicit` handles this correctly at line 281-291: it composes the fold function
with the accumulated function and folds the original `F B` directly, requiring only
`F: Foldable`.

### Approach A: Add a fold_inner method to CoyonedaInner

Add a method to the inner trait that accepts a type-erased fold callback:

```rust
trait CoyonedaInner<'a, F, A: 'a> {
    fn lower(self: Box<Self>) -> ... where F: Functor;
    fn fold_map_erased(
        self: Box<Self>,
        func: &dyn Fn(&dyn std::any::Any) -> Box<dyn std::any::Any>,
    ) -> Box<dyn std::any::Any> where F: Foldable;
}
```

**Trade-offs:**

- (-) Requires `Any` downcasting, which is unsafe-adjacent and loses type safety.
- (-) Requires `'static` bounds on the element types.
- (-) Extremely ugly and error-prone.

**Recommendation:** Not recommended.

### Approach B: Use CoyonedaExplicit internally for fold operations

When a user needs to fold a Coyoneda, suggest converting to CoyonedaExplicit first
via a conversion method. However, this is impossible because the Coyoneda hides the
intermediate type `B`, and CoyonedaExplicit requires it to be named.

**Recommendation:** Accept this limitation as documented. The layered trait-object design
fundamentally cannot support this. Users who need `Foldable` without `Functor` should
use `CoyonedaExplicit`.

---

## 7. Coyoneda::hoist requires F: Functor, unlike PureScript

**Location:** `coyoneda.rs`, lines 443-450.

**Problem:** `hoist` lowers to `F A` (requiring `F: Functor`), applies the natural
transformation, then re-lifts. PureScript applies the natural transformation directly
to the hidden `F B` via `unCoyoneda`, requiring no `Functor` constraint.

This means Coyoneda cannot be hoisted between non-Functor type constructors, which
defeats one of the key uses of the free functor construction: providing functor-like
operations for non-functor types, then transforming between them.

### Approach A: Add a hoist_inner method to CoyonedaInner

```rust
trait CoyonedaInner<'a, F, A: 'a> {
    fn lower(self: Box<Self>) -> ... where F: Functor;
    fn hoist<G>(self: Box<Self>, nat: &dyn ...) -> Box<dyn CoyonedaInner<'a, G, A>>;
}
```

**Trade-offs:**

- (-) The method is generic over `G`, making it non-dyn-compatible.

This is the fundamental problem. The method cannot exist on a trait object.

### Approach B: Store the natural transformation for deferred application

Instead of applying the natural transformation immediately, wrap the Coyoneda in another
layer that stores the natural transformation and applies it at lower time.

```rust
struct HoistedLayer<'a, F, G, A: 'a> {
    inner: Coyoneda<'a, F, A>,
    nat: Box<dyn NaturalTransformation<F, G>>,
}
```

But this `HoistedLayer` would need to implement `CoyonedaInner<'a, G, A>`, and its
`lower` method would need `F: Functor` anyway to lower the inner Coyoneda before
applying the nat.

**Recommendation:** Accept this limitation. Document it clearly and point users to
`CoyonedaExplicit::hoist`, which does not require `F: Functor`.

---

## 8. CoyonedaExplicit lacks an HKT brand and Functor instance

**Location:** `coyoneda_explicit.rs` (entire file).

**Problem:** `CoyonedaExplicit` cannot have a brand or implement `Functor` because its
type has four parameters (`F`, `B`, `A`, and the lifetime), but `Kind_cdc7cd43dac7585f`
requires `type Of<'a, A: 'a>: 'a` -- a single type parameter. The intermediate type `B`
and the function type are part of the struct's identity, so they cannot be hidden in a
brand.

This means `CoyonedaExplicit` cannot be used in code that is generic over `Functor`. It
is a standalone pipeline builder, not a participant in the type class hierarchy.

### Approach A: Provide a brand over (F, B) pairs

```rust
pub struct CoyonedaExplicitBrand<F, B>(PhantomData<(F, B)>);
impl_kind! {
    impl<F: ..., B: 'static> for CoyonedaExplicitBrand<F, B> {
        type Of<'a, A: 'a>: 'a = CoyonedaExplicit<'a, F, B, A>;
    }
}
```

The `Functor` implementation's `map` would delegate to `CoyonedaExplicit::map`.

**Trade-offs:**

- (+) Enables HKT integration.
- (-) The brand is parameterized by `B`, which changes after each `map`. After mapping,
  the actual type is `CoyonedaExplicit<'a, F, B, C>` but the brand still says
  `CoyonedaExplicitBrand<F, B>`, and `Of<'a, C>` would produce
  `CoyonedaExplicit<'a, F, B, C>`. This actually works correctly because the `B` stays
  fixed (it is the original input type), and only `A` (the output) varies.
- (-) Requires `B: 'static` for the brand.

**Recommendation:** This is worth implementing. The brand `CoyonedaExplicitBrand<F, B>`
correctly represents "a functor that holds an `F B` and a function from `B` to the
output type." The Functor instance would compose functions without calling `F::map`,
achieving true fusion through the type class interface.

---

## 9. CoyonedaExplicit::fold_map requires B: Clone unnecessarily

**Location:** `coyoneda_explicit.rs`, line 286.

**Problem:** The `fold_map` method has a `B: Clone` bound:

```rust
pub fn fold_map<FnBrand, M>(self, func: impl Fn(A) -> M + 'a) -> M
where
    B: Clone,
    ...
```

This `B: Clone` bound comes from `Foldable::fold_map` requiring `A: Clone` (seen at
`foldable.rs` line 211: `A: 'a + Clone`). Since `CoyonedaExplicit` calls
`F::fold_map::<FnBrand, B, M>(...)`, the element type being folded is `B`, so `B: Clone`
is required.

However, this is a limitation of the library's `Foldable` trait design, not of the
Coyoneda construction itself. In PureScript, `foldMap` has no `Clone` equivalent because
values are immutable and shared by default.

### Approach: Accept as inherent to the Rust Foldable design

The `Clone` bound on `Foldable::fold_map` exists because `fold_right` (the default
implementation) needs to pass elements to a cloneable callback. This is a library-wide
design decision.

**Recommendation:** No change needed specifically for Coyoneda. If the `Foldable` trait's
`Clone` bound is ever relaxed, this bound can be relaxed too.

---

## 10. Neither implementation is Send or Sync

**Location:** `coyoneda.rs` line 269 (`Box<dyn CoyonedaInner>`),
`coyoneda_explicit.rs` line 96 (`Box<dyn Fn(B) -> A + 'a>`).

**Problem:** Both types store `Box<dyn Fn(B) -> A + 'a>` or `Box<dyn CoyonedaInner>`,
which are not `Send` or `Sync`. This means:

- Neither type can be sent across threads.
- Neither type can be used with `rayon` or other parallel execution contexts.
- Neither type can be stored in an `Arc` for shared access.

### Approach A: Add Send variants

Create `SendCoyoneda` and `SendCoyonedaExplicit` that use `Box<dyn ... + Send>`:

```rust
pub struct SendCoyonedaExplicit<'a, F, B: 'a, A: 'a> {
    fb: F::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + Send + 'a>,
}
```

**Trade-offs:**

- (+) Enables thread-safe usage.
- (-) Doubles the API surface.
- (-) All closures passed to `map` must be `Send`.

### Approach B: Parameterize over Send-ness

Use a marker trait or feature flag to toggle `Send` bounds.

**Trade-offs:**

- (-) Complex, may require significant type machinery.

### Approach C: Use the generic Func approach (Issue 1, Approach A)

If `CoyonedaExplicit` uses a generic `Func` parameter instead of `Box<dyn Fn>`, the
`Send`-ness is determined by the concrete closure type. If the closure is `Send`, the
whole struct is `Send` (assuming `F::Of<'a, B>: Send`).

**Recommendation:** If the generic `Func` approach from Issue 1 is adopted, this problem
is solved automatically. Otherwise, provide `Send` variants following the library's
existing pattern (`Thunk`/`SendThunk`, `RcLazy`/`ArcLazy`).

---

## 11. Coyoneda::new creates an unnecessary extra layer

**Location:** `coyoneda.rs`, lines 306-316.

**Problem:** `Coyoneda::new` creates a `CoyonedaMapLayer` wrapping a `CoyonedaBase`:

```rust
pub fn new<B: 'a>(f: impl Fn(B) -> A + 'a, fb: F::Of<'a, B>) -> Self {
    Coyoneda(Box::new(CoyonedaMapLayer {
        inner: Box::new(CoyonedaBase { fa: fb }),
        func: Box::new(f),
    }))
}
```

This allocates 3 boxes: one for the outer `Coyoneda`, one for `CoyonedaBase`, and one
for the function. The `CoyonedaBase` is unnecessary because it just wraps `fb` and
returns it unchanged on `lower`. A single struct that stores both `fb` and `func` would
suffice.

### Approach: Combine into a single layer

Create a `CoyonedaSingle` struct:

```rust
struct CoyonedaSingle<'a, F, B: 'a, A: 'a> {
    fb: F::Of<'a, B>,
    func: Box<dyn Fn(B) -> A + 'a>,
}
impl CoyonedaInner for CoyonedaSingle { ... }
```

Then `Coyoneda::new` becomes:

```rust
pub fn new<B: 'a>(f: impl Fn(B) -> A + 'a, fb: F::Of<'a, B>) -> Self {
    Coyoneda(Box::new(CoyonedaSingle { fb, func: Box::new(f) }))
}
```

This saves one heap allocation.

**Trade-offs:**

- (+) One fewer allocation.
- (+) Simpler.
- (-) Introduces a third inner struct, slightly more code. However, `CoyonedaBase` could
  be removed if `lift` also uses `CoyonedaSingle` with `identity` as the function.

**Recommendation:** Implement this. It is a straightforward improvement with no
downsides.

---

## 12. CoyonedaExplicit::lift allocates a box for the identity function

**Location:** `coyoneda_explicit.rs`, lines 434-439.

**Problem:** `lift` stores `Box::new(identity)`, allocating a box for a function that
does nothing. Every `CoyonedaExplicit` starts with this unnecessary allocation.

### Approach A: Use an enum to distinguish "no function" from "has function"

```rust
enum Func<'a, B, A> {
    Identity, // phantom: no allocation
    Boxed(Box<dyn Fn(B) -> A + 'a>),
}
```

At `map` time, if the current function is `Identity`, just box the new function directly
instead of composing with identity.

**Trade-offs:**

- (+) Saves one allocation for the common `lift` -> `map` -> ... pattern.
- (-) Adds a branch at `map` and `lower` time.
- (-) `Identity` variant only works when `B == A`.

### Approach B: Use the generic Func approach (Issue 1, Approach A)

With a generic `Func` parameter, `lift` would store the actual `fn(A) -> A` (the
`identity` function pointer), which is a zero-sized type. No heap allocation.

**Recommendation:** The generic `Func` approach solves this naturally. If the boxed
approach is kept, Approach A is a reasonable optimization.

---

## 13. Missing type class instances for Coyoneda

**Location:** `coyoneda.rs`, lines 453-581.

**Problem:** `Coyoneda` currently implements only `Functor`, `Pointed`, and `Foldable`.
The module documentation at lines 70-73 notes that PureScript provides `Apply`,
`Applicative`, `Bind`, `Monad`, `Traversable`, `Extend`, `Comonad`, `Eq`, `Ord`, and
others.

Key missing instances:

- **Semiapplicative / Applicative (Apply):** Would require lowering both sides, which
  needs `F: Functor + Semiapplicative`. `CoyonedaExplicit` has an ad-hoc `apply` method
  but `Coyoneda` does not.
- **Semimonad / Monad (Bind):** Would require lowering, binding, and re-lifting. Needs
  `F: Functor + Semimonad`. `CoyonedaExplicit` has `bind` but `Coyoneda` does not.
- **Traversable:** Requires `Clone` on the inner type, which `Box<dyn CoyonedaInner>`
  does not support.
- **Eq, Ord:** Would require lowering and comparing, needing `F: Functor` and the
  concrete type to implement `Eq`/`Ord`.

### Approach: Implement the feasible instances

For `Coyoneda`:

```rust
impl<F: Functor + Semiapplicative + 'static> Semiapplicative for CoyonedaBrand<F> {
    fn apply<'a, A, B>(ff: Coyoneda<F, CloneableFn::Of<A, B>>, fa: Coyoneda<F, A>) -> Coyoneda<F, B> {
        Coyoneda::lift(F::apply(ff.lower(), fa.lower()))
    }
}
```

**Trade-offs:**

- (+) More complete type class coverage.
- (+) Enables `Coyoneda` to participate in applicative/monadic code.
- (-) All instances require `F: Functor` for lowering, unlike PureScript where only
  `lower` itself requires `Functor`.
- (-) Each operation forces a lower/re-lift cycle, creating intermediate allocations.

**Recommendation:** Implement `Semiapplicative` and `Semimonad` for `CoyonedaBrand<F>`
with the `F: Functor` constraint. These are the most useful missing instances. Skip
`Traversable` (requires `Clone`) and `Extend`/`Comonad` (niche use cases) for now.

---

## 14. CoyonedaExplicit::into_coyoneda loses fusion

**Location:** `coyoneda_explicit.rs`, lines 314-316.

**Problem:** `into_coyoneda` converts the explicit form into a `Coyoneda` by calling
`Coyoneda::new(self.func, self.fb)`. This creates a `CoyonedaMapLayer` wrapping a
`CoyonedaBase`, which means any subsequent `map` calls on the resulting `Coyoneda` will
not compose with the accumulated function. The accumulated function becomes a single
layer, and new maps add new layers on top.

This is semantically correct but means the conversion is a one-way "escape hatch" that
abandons fusion. Any maps accumulated in `CoyonedaExplicit` are preserved (they become
the single composed function in the `CoyonedaMapLayer`), but future maps after conversion
do not fuse with them.

### Approach: Document clearly

This behavior is inherent to the Coyoneda design. No code change needed, but the
documentation should note that conversion is a fusion boundary.

**Recommendation:** Add a note to the `into_coyoneda` documentation: "After conversion,
further maps on the resulting `Coyoneda` do not fuse with the previously accumulated
function."

---

## 15. CoyonedaExplicit has no conversion from Coyoneda

**Location:** `coyoneda_explicit.rs` (absent method).

**Problem:** There is `into_coyoneda` to go from `CoyonedaExplicit` to `Coyoneda`, but
no `from_coyoneda` to go the other way. This makes sense because `Coyoneda` hides the
intermediate type `B`, and `CoyonedaExplicit` requires it. However, a
`from_coyoneda_lowered` method could lower the `Coyoneda` and lift the result:

```rust
pub fn from_coyoneda(coyo: Coyoneda<'a, F, A>) -> Self where F: Functor {
    Self::lift(coyo.lower())
}
```

### Approach: Add a from_coyoneda method

**Trade-offs:**

- (+) Enables round-tripping (with fusion loss).
- (+) Consistent API.
- (-) Requires `F: Functor` for the lower step.

**Recommendation:** Add this method for API completeness. It is a simple convenience that
avoids users having to manually lower and re-lift.

---

## Summary of Recommendations

| Issue                                    | Severity | Recommendation                                            |
| ---------------------------------------- | -------- | --------------------------------------------------------- |
| 1. CoyonedaExplicit boxes every function | High     | Use generic Func parameter; provide .boxed() for storage  |
| 2. Coyoneda has no map fusion            | Medium   | Accept; improve docs directing users to CoyonedaExplicit  |
| 3. Nested closure stack overflow         | Medium   | Document; mitigated by generic Func approach              |
| 4. Fn instead of FnOnce                  | Low      | Accept; inherent to Functor trait design                  |
| 5. apply/bind destroy fusion             | Low      | Document fusion boundaries more prominently               |
| 6. Foldable requires Functor (Coyoneda)  | Medium   | Accept; point users to CoyonedaExplicit                   |
| 7. hoist requires Functor (Coyoneda)     | Medium   | Accept; point users to CoyonedaExplicit                   |
| 8. No HKT brand for CoyonedaExplicit     | Medium   | Add CoyonedaExplicitBrand<F, B>                           |
| 9. fold_map requires B: Clone            | Low      | Accept; inherent to Foldable design                       |
| 10. Not Send or Sync                     | Medium   | Solved by generic Func approach; else add Send variants   |
| 11. Coyoneda::new extra allocation       | Low      | Combine into single-layer struct                          |
| 12. lift boxes identity                  | Low      | Solved by generic Func approach; else use enum            |
| 13. Missing type class instances         | Medium   | Implement Semiapplicative and Semimonad for CoyonedaBrand |
| 14. into_coyoneda loses fusion           | Low      | Document the fusion boundary                              |
| 15. No from_coyoneda                     | Low      | Add convenience method                                    |

The single most impactful change would be Issue 1's Approach A: making `CoyonedaExplicit`
generic over the function type. This would deliver on the "zero-cost" promise, eliminate
heap allocations per map, enable compiler inlining of the composed function chain, and
automatically solve the `Send`/`Sync` issue (Issue 10) and the identity boxing issue
(Issue 12). It would also substantially mitigate the stack overflow concern (Issue 3)
since inlined closures do not add stack frames.
