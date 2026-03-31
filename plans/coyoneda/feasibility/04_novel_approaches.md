# Novel Approaches for Stack-Safe Coyoneda Lowering

This document evaluates novel approaches for making `Coyoneda`'s `lower` method
stack-safe in Rust, without introducing any unsafe code. Each approach is distinct
from the previously considered options (A: explicit stack/Vec of layers, B:
trampoline/step method, D: document the limitation, E: flat Vec of erased
functions).

## Background: The Core Constraint

The recursive `lower` call chain arises because each `CoyonedaMapLayer` hides a
different intermediate type `B` behind a trait object. When layer k lowers, it
calls `self.inner.lower()` to get `F::Of<'a, B>`, then applies `F::map(self.func,
lowered)` to produce `F::Of<'a, A>`. The type `B` is existentially quantified,
invisible to the caller. You cannot collect these heterogeneous layers into a
homogeneous container, nor can you add a generic `step<C>` method to a
dyn-compatible trait.

`Functor::map` has signature:

```rust
fn map<'a, A: 'a, B: 'a>(f: impl Fn(A) -> B + 'a, fa: F::Of<'a, A>) -> F::Of<'a, B>;
```

This takes an `F::Of<'a, A>` and returns an `F::Of<'a, B>`. The function `f` is
`impl Fn`, not `dyn Fn`, so the concrete types must be known at each call site.

---

## Approach F: Eager Periodic Collapse at `map` Time

### Idea

Instead of deferring all work to `lower`, periodically collapse the layer stack
during `map`. After every N maps (e.g., N=64), call `lower` eagerly to flatten
all accumulated layers into a single `CoyonedaBase`. This bounds the recursion
depth at `lower` time to at most N.

```
map #1: wrap in CoyonedaMapLayer (depth 1)
map #2: wrap in CoyonedaMapLayer (depth 2)
...
map #N: depth reaches threshold -> call lower() internally,
        store result as CoyonedaBase (depth reset to 0)
map #N+1: wrap in CoyonedaMapLayer (depth 1)
...
```

### Implementation Sketch

Add a `depth: usize` field to `Coyoneda`. In `map`:

```rust
pub fn map<B: 'a>(self, f: impl Fn(A) -> B + 'a) -> Coyoneda<'a, F, B>
where
    F: Functor,
{
    let new_depth = self.depth + 1;
    if new_depth >= COLLAPSE_THRESHOLD {
        // Flatten: lower to F::Of<'a, A>, apply f via F::map, re-lift
        let lowered = self.lower();
        let mapped = F::map(f, lowered);
        Coyoneda { inner: Box::new(CoyonedaBase { fa: mapped }), depth: 0 }
    } else {
        Coyoneda {
            inner: Box::new(CoyonedaMapLayer { inner: self.0, func: Box::new(f) }),
            depth: new_depth,
        }
    }
}
```

### Evaluation

- **Feasibility:** Partially feasible.
- **Safety:** No unsafe code required.
- **Key problem: `map` now requires `F: Functor`.** Currently, `Coyoneda::map`
  does not require `F: Functor`; only `lower` does. This is the defining property
  of the free functor construction: `Coyoneda F` is a `Functor` even when `F` is
  not. Requiring `F: Functor` at `map` time destroys this property.
- **Performance:** Introduces periodic `F::map` calls during `map`, turning O(1)
  `map` into amortized O(N/threshold) map calls. For eager containers like `Vec`,
  each collapse traverses the container, which can be expensive.
- **API compatibility:** The `F: Functor` bound on `map` is a breaking API change
  to the `Functor` implementation for `CoyonedaBrand<F>`, which currently has no
  bound on `F` for `map`. The `Coyoneda::map` inherent method also currently
  requires no `F: Functor` bound.
- **Lifetime constraints:** No `'static` requirement added.
- **Verdict:** The approach fundamentally conflicts with Coyoneda's purpose. If
  `F: Functor` is available at `map` time, users should just call `F::map`
  directly. The whole point of Coyoneda is deferring the Functor requirement.

### Variant F': Opt-in Collapse

A variant: keep `map` as-is (no `F: Functor`), but provide an explicit
`collapse(&mut self)` method that requires `F: Functor` and flattens accumulated
layers. Users building deep chains can call `collapse` periodically.

This is feasible and safe but is a manual escape hatch, not automatic stack
safety. It shifts the burden to the caller to know when depth is dangerous. Still,
it is a pragmatic middle ground that could be combined with other approaches.

**Verdict: Partially feasible.** Useful as an ergonomic helper, not a solution.

---

## Approach G: CatList of Type-Erased Functions with Existential Lowering Protocol

### Idea

Inspired by how `Free` achieves stack safety via "Reflection without Remorse,"
redesign `Coyoneda` to store its accumulated map functions in a `CatList` of
type-erased continuations, similar to `Free`'s continuation queue. The base
`F::Of<'a, B>` value and the CatList are stored at the top level. At `lower`
time, an iterative loop applies the functions one at a time by type-erasing and
downcasting intermediate values.

The key insight from `Free`: by storing `Box<dyn Any>` as the intermediate
representation, you can iterate through a heterogeneous chain of functions without
recursion. Each function in the CatList has type
`Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>`, and the intermediate value flows
through as `Box<dyn Any>`.

### How It Would Work

```rust
struct Coyoneda<'a, F, A: 'a> {
    // The original F::Of<'a, B> for some hidden B, erased to Box<dyn Any>
    base_erased: Box<dyn Any>,
    // A function that takes Box<dyn Any> (the erased F::Of<'a, B>), applies
    // F::map with the first function, and returns Box<dyn Any> (the erased
    // F::Of<'a, C>)
    functions: Vec<Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any> + 'a>>,
    // Or use CatList for O(1) snoc
    _phantom: PhantomData<&'a A>,
}
```

At `lower` time:

1. Start with `base_erased`.
2. Iterate through `functions`, applying each one. Each function internally
   knows its concrete input/output types and does the downcast/rebox.
3. Downcast the final `Box<dyn Any>` to `F::Of<'a, A>`.

At `map` time:

1. Capture the concrete types `A` and `B` in a closure that downcasts, calls
   `F::map(f, ...)`, and reboxes.
2. Append the closure to `functions`.

### Evaluation

- **Feasibility:** Infeasible in safe Rust without `'static`.
- **Safety:** No unsafe code, but `Box<dyn Any>` requires `A: 'static`. The
  `Any` trait has the bound `trait Any: 'static`. This means `F::Of<'a, B>` must
  be `'static` for every intermediate type `B`, and the functions themselves must
  be `'static`. This conflicts with Coyoneda's current lifetime-polymorphic
  design (`'a` on all types).
- **The `Free` monad has this exact limitation:** it requires `A: 'static` and
  cannot implement the library's HKT traits. Applying the same technique to
  Coyoneda would impose the same constraint.
- **Could we avoid `Any`?** Without `Any`, we need some other type-erasure
  mechanism for the intermediate values. But Rust's only safe mechanisms for
  heterogeneous type erasure are trait objects (which require dyn-compatibility,
  ruling out generic methods) and `Any` (which requires `'static`). There is no
  third option in safe Rust.
- **Performance:** The iterative loop would be O(k) with no recursion. Each step
  involves one `Box<dyn Any>` downcast (cheap) and one `F::map` call.
- **API compatibility:** Adding `'static` bounds to `Coyoneda` would be a
  breaking change and would prevent it from working with borrowed data.

**Verdict: Infeasible.** The `'static` constraint required by `Any` is
incompatible with Coyoneda's lifetime-polymorphic design. This is the same
fundamental barrier that `Free` faces. A `'static`-only variant could be provided
as a separate type (analogous to `Trampoline` vs `Thunk`), but it would not
replace the general `Coyoneda`.

---

## Approach H: Adaptive Stack Growth via `stacker`

### Idea

Use the `stacker` crate (or `std::thread::Builder::stack_size`) to grow the
stack on demand during `lower`. The `stacker` crate provides
`stacker::maybe_grow(red_zone, stack_size, closure)` which checks remaining stack
space and, if below the red zone threshold, allocates a new stack segment and runs
the closure on it. This is entirely safe Rust (the crate's API is safe; its
internals use platform-specific but well-tested mechanisms).

### Implementation Sketch

Change `CoyonedaMapLayer::lower`:

```rust
fn lower(self: Box<Self>) -> F::Of<'a, A>
where
    F: Functor,
{
    stacker::maybe_grow(32 * 1024, 1024 * 1024, || {
        let lowered = self.inner.lower();
        F::map(self.func, lowered)
    })
}
```

This checks if the remaining stack is less than 32KB; if so, it allocates a 1MB
stack extension and runs the recursive call on it. Each extension supports many
more layers before another extension is needed.

### Evaluation

- **Feasibility:** Feasible.
- **Safety:** The `stacker` crate's public API is entirely safe Rust. The crate
  itself uses platform-specific code internally, but the caller never writes
  unsafe. From the library's perspective, this is safe code calling a safe API.
  (Whether the user considers `stacker`'s internal use of platform APIs
  acceptable is a policy question, not a safety question.)
- **Performance:** Near-zero overhead when the stack is sufficient (just a pointer
  comparison to check remaining space). When a new segment is needed, there is a
  one-time allocation cost per segment. Each 1MB segment supports roughly 10,000+
  additional layers (each frame is ~100-200 bytes), so reallocations are rare.
- **`'static` constraints:** None. Works with arbitrary lifetimes.
- **API compatibility:** Fully backwards-compatible. No signature changes.
- **Platform support:** `stacker` supports Linux, macOS, Windows, and most
  targets Rust supports. It is widely used (e.g., by `rustc` itself).
- **Dependency cost:** Adds one dependency (`stacker`), which could be gated
  behind a feature flag.
- **Trade-offs:**
  - Does not eliminate the recursive structure; it merely makes stack overflow
    unreachable in practice by growing the stack as needed.
  - Adds an external dependency. The library currently has very few dependencies.
  - Could be feature-gated (e.g., `stacker` feature flag) so users who do not
    need deep chains avoid the dependency.
  - The `std::thread::Builder::stack_size` alternative does not require a
    dependency but requires the user to spawn a thread with a large stack, which
    is less ergonomic and not composable.

**Verdict: Feasible.** This is the most pragmatic approach. It requires no
structural changes, preserves all API contracts, imposes no lifetime restrictions,
and has negligible performance impact. The only cost is an optional dependency.
The recursion remains, but stack overflow becomes unreachable in practice.

---

## Approach I: Redesigned Inner Trait with Existential Application Protocol

### Idea

Redesign `CoyonedaInner` so that instead of each layer calling `lower` on its
child (producing recursion), the trait exposes a method that "peels off" one layer
at a time, returning the inner layer and a boxed function to apply afterwards.
The outer `Coyoneda::lower` then drives an iterative loop.

The challenge is typing the return value: the function and inner layer involve
the hidden type `B`. The trick is to define a helper trait that captures the
"apply this function via `F::map` to an `F::Of<'a, B>` and produce `F::Of<'a, A>`"
operation without exposing `B`.

```rust
trait MapApplicator<'a, F, A: 'a>: 'a
where
    F: Functor + Kind_... + 'a,
{
    /// Given the lowered inner value (as a type-erased Box<dyn Any>),
    /// apply F::map with the stored function and return the result
    /// (also type-erased).
    fn apply_map(self: Box<Self>, inner: Box<dyn Any>) -> Box<dyn Any>;
}
```

Then `CoyonedaInner` gains a `peel` method:

```rust
trait CoyonedaInner<'a, F, A: 'a>: 'a {
    fn peel(self: Box<Self>) -> PeelResult<'a, F, A>;
}

enum PeelResult<'a, F, A: 'a> {
    Base(F::Of<'a, A>),
    Layer {
        inner: Box<dyn CoyonedaInner<'a, F, ???>>,  // problem: what is the type?
        applicator: Box<dyn MapApplicator<'a, F, A>>,
    },
}
```

The problem: the `inner` field in `Layer` must be `Box<dyn CoyonedaInner<'a, F,
B>>` for some hidden `B`, but `PeelResult` cannot mention `B` in its type
parameters (it is existential). So we would need _another_ layer of trait-object
indirection to hide `B` in the inner field.

### Reformulation: Type-Erased Peel

We can work around this by type-erasing the inner value in the `Layer` case. Each
layer, when peeled, returns:

1. A `Box<dyn Any>` representing the erased `F::Of<'a, B>` (or the erased inner
   `CoyonedaInner`).
2. A `Box<dyn FnOnce(Box<dyn Any>) -> F::Of<'a, A>>` that, given the lowered
   inner result (type-erased), downcasts and applies `F::map`.

But again, `Box<dyn Any>` requires `'static`, so `F::Of<'a, B>` must be
`'static`. Same barrier as Approach G.

### Alternative: Peel Returns a Continuation Producing the Same Erased Type

What if the inner layer's `lower` is captured in a closure that returns
`Box<dyn Any>`, and the outer layer's `F::map` application is captured in another
closure? Then the iterative loop chains these closures. But building the closure
chain is itself recursive (each layer must wrap the inner layer's closure), so
we have not eliminated the recursion; we have moved it from `lower` to `peel`.

### Evaluation

- **Feasibility:** Infeasible without `'static` constraints.
- **Safety:** No unsafe required, but the `Any`-based variants require `'static`.
- **Core issue:** Every attempt to "peel" layers into an iterative sequence
  requires naming or erasing the hidden type `B`. Naming it is impossible
  (existential). Erasing it requires `Any` (`'static`). Wrapping it in another
  trait object just moves the recursion.
- **Performance:** If it worked, would be O(k) iterative. But it does not work
  without `'static`.

**Verdict: Infeasible.** The hidden type `B` cannot be carried across iterations
without `Any` (requiring `'static`) or another trait object (reintroducing
recursion). This is a fundamental consequence of Rust's type system lacking
first-class existential types.

---

## Approach J: Type-Erased Intermediate Representation with `'static` Opt-in

### Idea

Provide a parallel stack-safe lowering path that is available only when
`F::Of<'a, B>: 'static` for all intermediate types. This is not a universal
solution but covers the common case where `Coyoneda` is used with owned data.

Add a second lowering method `lower_safe` (or make `lower` dispatch to the safe
path when possible) that uses the Approach G technique (CatList/Vec of type-erased
functions) but only when the types are `'static`.

### Implementation Sketch

```rust
impl<F, A: 'static> Coyoneda<'static, F, A>
where
    F: Kind_... + Functor + 'static,
{
    pub fn lower_safe(self) -> F::Of<'static, A> {
        // Use the iterative, type-erased path
    }
}
```

The non-`'static` case falls back to the recursive `lower`.

### Alternative: A Separate `StaticCoyoneda` Type

Similar to how the library has `Thunk<'a, A>` (lifetime-polymorphic) and
`Trampoline<A>` (`'static`-only, stack-safe), provide `CoyonedaSafe<F, A>` that
requires `'static` but guarantees stack-safe lowering.

```rust
pub struct CoyonedaSafe<F, A: 'static> {
    base: Box<dyn Any>,                  // erased F::Of<'static, B>
    fns: Vec<Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>>,
    _phantom: PhantomData<(F, A)>,
}
```

At `map` time, push a new function. At `lower` time, iterate.

### Evaluation

- **Feasibility:** Feasible for the `'static` subset.
- **Safety:** No unsafe code. `Box<dyn Any>` is safe.
- **`'static` constraint:** Required. This means no borrowed data in intermediate
  types. This is acceptable for many use cases (e.g., `Coyoneda<VecBrand, String>`
  where all types are owned).
- **Performance:** O(k) iterative lowering. Each step: one `Any` downcast (very
  cheap), one `F::map` call, one rebox. No recursion.
- **API compatibility:** Provided as a separate type, so no breaking changes.
  Could implement `Functor` via a new brand (`CoyonedaSafeBrand<F>`), or just
  provide inherent methods.
- **HKT integration:** A brand for `CoyonedaSafe` could implement `Functor`
  (the brand's `Kind` signature would use `'static` for the lifetime parameter).
  The library's `Kind` trait requires `type Of<'a, A: 'a>: 'a`, so the brand
  would need to ignore `'a` and use `'static` internally. This may require the
  `Apply!` type to evaluate to a `'static` type regardless of `'a`, which could
  work if `CoyonedaSafe` simply does not reference `'a`.
- **Trade-offs:**
  - Two Coyoneda types to maintain.
  - The `'static` requirement prevents use with borrowed data.
  - Follows the library's existing pattern of paired types (`Thunk`/`Trampoline`,
    `Lazy`/`ArcLazy`, etc.).

**Verdict: Feasible.** This is a viable approach that follows the library's
established pattern of providing `'static`-constrained stack-safe variants
alongside lifetime-polymorphic ones. It does not help the general
lifetime-polymorphic case, but it covers the most common use cases.

---

## Approach K: Spawn-on-Large-Stack via `std::thread`

### Idea

When `lower` detects that the chain is deep (e.g., by tracking depth), spawn a
new thread with a large stack and run `lower` on it. This uses only `std` APIs.

### Implementation Sketch

```rust
pub fn lower(self) -> F::Of<'a, A>
where
    F: Functor,
    // Need Send bounds for cross-thread transfer
{
    if self.depth > THRESHOLD {
        std::thread::Builder::new()
            .stack_size(self.depth * FRAME_SIZE)
            .spawn(move || self.0.lower())
            .unwrap()
            .join()
            .unwrap()
    } else {
        self.0.lower()
    }
}
```

### Evaluation

- **Feasibility:** Infeasible.
- **Safety:** No unsafe, but introduces `Send` bounds.
- **Key problem: `Send` bounds.** Transferring `self` across a thread boundary
  requires `self: Send`, which in turn requires `F::Of<'a, B>: Send` and
  `dyn CoyonedaInner: Send` for all hidden types. The current `Coyoneda` is
  explicitly documented as not `Send`. Adding a `Send` bound would be a breaking
  API change and would prevent use with `Rc`-based types, non-Send closures, etc.
- **Key problem: lifetime bounds.** `std::thread::spawn` requires `'static`
  closures. `std::thread::scope` allows non-`'static` but still requires `Send`.
  With `thread::scope`, we avoid `'static` but still need `Send`.
- **Performance:** Thread creation overhead (~microseconds) is non-trivial and
  disproportionate for what may be a fast operation. Thread pool reuse could
  mitigate this, but adds complexity.
- **Depth tracking:** Requires either a `depth` field on `Coyoneda` (minor space
  overhead) or counting layers at `lower` time (requires a pre-traversal, which
  is itself recursive).

**Verdict: Infeasible.** The `Send` bound requirement is incompatible with
`Coyoneda`'s design, which deliberately supports non-`Send` types. Even with
`thread::scope` (avoiding `'static`), the `Send` requirement is a deal-breaker.

---

## Approach L: CPS Transform of the Lowering Protocol

### Idea

Instead of each layer's `lower` method returning `F::Of<'a, A>` directly,
transform the protocol to continuation-passing style. Each layer receives a
continuation `k: impl FnOnce(F::Of<'a, A>) -> R` and is responsible for calling
`k` with its result, but it can do so in tail position.

In a language with guaranteed tail-call elimination (TCE), this would make
the recursion stack-safe: each layer tail-calls into the inner layer with a
composed continuation. But Rust does not have TCE.

### Formulation

```rust
trait CoyonedaInner<'a, F, A: 'a>: 'a {
    fn lower_cps<R>(
        self: Box<Self>,
        k: impl FnOnce(F::Of<'a, A>) -> R,
    ) -> R
    where
        F: Functor;
}
```

For `CoyonedaMapLayer`:

```rust
fn lower_cps<R>(self: Box<Self>, k: impl FnOnce(F::Of<'a, A>) -> R) -> R {
    self.inner.lower_cps(|fb: F::Of<'a, B>| {
        k(F::map(self.func, fb))
    })
}
```

This composes `k` with `F::map(self.func, -)` and passes it down. Each recursive
call adds a frame for the `lower_cps` call itself, but the continuation is passed
down (not stored on the call stack as a pending return).

### Evaluation

- **Feasibility:** Infeasible (in Rust).
- **Safety:** No unsafe required.
- **Key problem: no TCE in Rust.** Even though the recursive call to
  `self.inner.lower_cps(...)` could conceptually be a tail call (if the closure
  were not capturing `k`), Rust does not guarantee TCE. The `lower_cps` call on
  the inner layer is inside a closure passed to `inner.lower_cps`, so each
  invocation still adds a stack frame. The recursion depth is unchanged.
- **Key problem: `R` is generic.** Adding `lower_cps<R>` to `CoyonedaInner`
  makes it generic over `R`, which breaks dyn-compatibility. The trait object
  `Box<dyn CoyonedaInner>` cannot have methods with unconstrained generic
  parameters. To make it dyn-compatible, `R` would need to be fixed (e.g.,
  `Box<dyn Any>`), which reintroduces the `'static` problem.
- **Even without the dyn-compatibility issue,** the CPS transform does not help
  in Rust because there is no TCE. The stack depth remains O(k).

**Verdict: Infeasible.** CPS relies on tail-call elimination to convert stack
depth into continuation size. Rust lacks TCE, so CPS just rearranges where the
stack frames go without reducing their count. Additionally, the generic `R`
parameter breaks dyn-compatibility.

---

## Approach M: Chunked Composition at `map` Time (No `F: Functor` Required)

### Idea

Instead of building a linear chain of layers (each wrapping the previous),
periodically compose adjacent functions together. Every N maps, compose the last
N functions into a single function, reducing the chain to a single layer per
chunk. This does not require `F: Functor` because it composes the `B -> A`
functions directly, not the functor values.

The challenge: composing `f: B -> A` with `g: C -> B` requires knowing both
types. In the current design, `B` is hidden inside a `CoyonedaMapLayer`. So you
cannot compose functions across the trait-object boundary.

### Reformulation: Compose Before Wrapping

What if, instead of immediately boxing each map function into a new
`CoyonedaMapLayer`, we buffer the last few functions and compose them before
boxing? Each `map` call would compose the new function with a buffer (like
`CoyonedaExplicit`'s compile-time composition), and only create a new layer when
the buffer is flushed.

```rust
pub struct Coyoneda<'a, F, A: 'a> {
    inner: Box<dyn CoyonedaInner<'a, F, A> + 'a>,
}

// map returns a different type that accumulates:
pub fn map<B: 'a>(self, f: impl Fn(A) -> B + 'a) -> CoyonedaBuffered<'a, F, A, B, impl Fn(A) -> B>
```

But this changes the return type of `map`, breaking the uniform `Coyoneda<'a, F,
B>` type. The whole point of the trait-object design is that `map` returns the
same type `Coyoneda<'a, F, B>` regardless of the input type.

### Alternative: Box the Composed Function Periodically

Track the depth in `Coyoneda`. Every N maps, instead of wrapping a new
`CoyonedaMapLayer` around the previous one, compose the last function with the
top layer's function:

```rust
pub fn map<B: 'a>(self, f: impl Fn(A) -> B + 'a) -> Coyoneda<'a, F, B> {
    // If self.inner is a CoyonedaMapLayer and depth < N, compose:
    // Problem: we can't inspect self.inner's type through the trait object
}
```

We cannot "peek inside" the trait object to check if it is a `CoyonedaMapLayer`
and access its function for composition. The trait object erases the concrete
type. We would need a method like `try_compose(&self, f) -> Option<Coyoneda>`
on the inner trait, but this requires generic parameters (for the new output
type), breaking dyn-compatibility.

### Evaluation

- **Feasibility:** Infeasible.
- **Core issue:** Composing functions across the trait-object boundary requires
  "opening" the existential type `B`, which requires a generic method, which
  breaks dyn-compatibility. This is the same fundamental barrier as the
  original map fusion problem.
- **The `CoyonedaExplicit` type already solves this** by not hiding `B`. Any
  attempt to do composition within `Coyoneda` while keeping `B` hidden runs into
  the same wall.

**Verdict: Infeasible.** Function composition across the existential boundary is
exactly the problem that makes `Coyoneda` unable to fuse maps in the first place.
Chunking does not circumvent it.

---

## Summary Table

| Approach                                   | Stack-safe?         | No unsafe? | No `'static`?  | No new `F` bounds?         | API compatible? | Verdict                              |
| ------------------------------------------ | ------------------- | ---------- | -------------- | -------------------------- | --------------- | ------------------------------------ |
| F: Eager periodic collapse                 | Yes (bounded depth) | Yes        | Yes            | No (`F: Functor` on `map`) | No              | Partially feasible (F' variant only) |
| G: CatList of erased functions             | Yes (iterative)     | Yes        | No (`Any`)     | Yes                        | No              | Infeasible                           |
| H: `stacker` adaptive stack growth         | Yes (in practice)   | Yes        | Yes            | Yes                        | Yes             | Feasible                             |
| I: Redesigned inner trait with peel        | Depends             | Yes        | No (`Any`)     | Yes                        | No              | Infeasible                           |
| J: `'static`-only variant (`CoyonedaSafe`) | Yes (iterative)     | Yes        | No (by design) | Yes                        | Yes (new type)  | Feasible                             |
| K: Spawn on large-stack thread             | Yes                 | Yes        | Partial        | No (`Send`)                | No              | Infeasible                           |
| L: CPS transform of lowering               | No (no TCE in Rust) | Yes        | Yes            | Yes                        | No (dyn-compat) | Infeasible                           |
| M: Chunked composition at map time         | N/A                 | Yes        | Yes            | Yes                        | No              | Infeasible                           |

---

## Recommendations

### 1. Primary recommendation: Approach H (`stacker`)

`stacker` is the most pragmatic solution. It requires no structural changes, no
new type parameters, no lifetime restrictions, and no API changes. It makes stack
overflow unreachable in practice by growing the stack on demand. The cost is one
optional dependency, which can be gated behind a feature flag (e.g.,
`stack-safety` or `stacker`).

The `stacker` crate is widely used and battle-tested (used by `rustc`
itself). Its public API is safe Rust. Whether its internal platform-specific
code is acceptable is a policy decision for the library, but from a safety
perspective it meets the "no unsafe in our code" requirement.

### 2. Secondary recommendation: Approach J (`CoyonedaSafe` / `'static`-only variant)

For users who need guaranteed algorithmic stack safety (not just "large enough
stack"), a `'static`-only Coyoneda variant that stores functions in a flat Vec
and iterates at `lower` time is the most principled approach. This follows the
library's established pattern of paired types (`Thunk`/`Trampoline`). It does
not replace the general `Coyoneda` but covers the common case of owned data.

### 3. Tertiary recommendation: Approach F' (opt-in `collapse` method)

As a lightweight ergonomic improvement, an explicit `collapse` method (requiring
`F: Functor`) that flattens accumulated layers can help users manage depth
manually. This is not automatic stack safety but is simple to implement and
useful as a stopgap.

### Combined Strategy

These three approaches are complementary:

1. Gate `stacker` behind a feature flag for automatic stack growth (covers 99% of
   cases).
2. Provide `CoyonedaSafe<F, A>` for guaranteed iterative lowering when types
   are `'static`.
3. Add a `collapse` method for manual depth management in performance-sensitive
   code where periodic eager lowering is acceptable.
