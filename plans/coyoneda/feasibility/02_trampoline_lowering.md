# Approach B: Trampoline the Recursion in Coyoneda::lower

## Problem Statement

`Coyoneda::lower` is recursive through the layered trait-object chain. For k
chained `map` calls, each `CoyonedaMapLayer::lower` calls `self.inner.lower()`
before applying its own function, producing k stack frames. This risks stack
overflow for large k.

The question: can the recursion be restructured to use a trampoline or
continuation-passing style, keeping the iteration on the heap instead of the
stack, without introducing unsafe code?

## Background: How lower Works Today

```
CoyonedaMapLayer { inner: Box<dyn CoyonedaInner>, func: Box<dyn Fn(B) -> A> }
```

Each layer's `lower(self: Box<Self>) -> F::Of<'a, A>` does:

1. Call `self.inner.lower()` to get `F::Of<'a, B>`.
2. Call `F::map(self.func, lowered_b)` to get `F::Of<'a, A>`.

The recursion bottoms out at `CoyonedaBase`, which returns `self.fa` directly.
`CoyonedaNewLayer` applies `F::map` once without recursing further.

The call stack for k layers looks like:

```
lower_k -> lower_{k-1} -> ... -> lower_1 -> lower_base
```

Each frame holds a `Box<dyn Fn(B) -> A>` and a `F::Of<'a, B>` as it unwinds.

## Sub-approach 1: Internal LowerStep Enum

The idea: replace the recursive `lower` with a `lower_step` method that returns
either a final value or a "keep going" continuation, then drive it with a loop.

### Sketch

```rust
enum LowerStep<'a, F: Kind + 'a, A: 'a> {
    Done(F::Of<'a, A>),
    Continue {
        inner: Box<dyn CoyonedaInner<'a, F, ???>>,
        apply: Box<dyn FnOnce(F::Of<'a, ???>) -> F::Of<'a, A> + 'a>,
    },
}
```

### The Type-Erasure Problem

The `???` in `Continue` is the hidden existential type `B` from `CoyonedaMapLayer`.
Each layer has a _different_ `B`. The driver loop would need to chain together
steps where each step's output type matches the next step's input type:

```
step_base: Done(F::Of<'a, B0>)
step_1:    apply_1: F::Of<'a, B0> -> F::Of<'a, B1>
step_2:    apply_2: F::Of<'a, B1> -> F::Of<'a, B2>
...
step_k:    apply_k: F::Of<'a, B_{k-1}> -> F::Of<'a, A>
```

The intermediate types `B0, B1, ..., B_{k-1}` are all different and unknown
at compile time. To store them in a uniform data structure (like a `Vec`), the
types must be erased. But `F::Of<'a, Bi>` is a _functor application_, not a
plain value. You cannot erase `F::Of<'a, Bi>` to `Box<dyn Any>` because:

1. `F::Of<'a, Bi>` may not be `'static`, so it cannot satisfy `Any`'s bound.
2. Even if it were `'static`, you would need to downcast it back to apply the
   next function, and the concrete type is unknown.
3. You also need to erase the _function_ `F::Of<'a, Bi> -> F::Of<'a, B_{i+1}>`,
   which is doubly polymorphic in types that are not statically known.

**Could we use a trait object for the "continue" payload?** We would need
something like:

```rust
trait ErasedStep<'a, F: Kind + 'a, A: 'a>: 'a {
    fn finish(self: Box<Self>) -> F::Of<'a, A>;
}
```

But `finish` would need to call `F::map` (requiring `F: Functor`), and the
implementation for a chain of k steps would just re-introduce the same recursive
nesting we are trying to eliminate. The trait object hides the type but not the
recursion.

**Verdict for sub-approach 1: Infeasible.** The existential type `B` at each
layer creates a type-level chain that cannot be linearized into a homogeneous
loop without either (a) erasing `F::Of<'a, B>` via `Any` (blocked by lifetime
constraints), or (b) some form of unsafe transmute/cast.

## Sub-approach 2: Collect Functions, Apply in a Single Pass

The idea: instead of calling `F::map` at each layer, extract all the `Box<dyn Fn>` closures into a list, compose them, then call `F::map` once.

### The Same Type-Erasure Problem

Each function has type `Box<dyn Fn(Bi) -> B_{i+1}>`. These have different
argument and return types. Composing them requires knowing the intermediate
types. You cannot store `Fn(B0) -> B1` and `Fn(B1) -> B2` in the same
`Vec<Box<dyn Fn(???) -> ???>>` without erasing both sides.

Function composition across type-erased boundaries is exactly the problem that
`CoyonedaExplicit` solves (by keeping the function type parameter explicit). But
for the dyn-based `Coyoneda`, this cannot work.

You could try erasing via `Box<dyn Any>`:

```rust
Box<dyn Fn(Box<dyn Any>) -> Box<dyn Any>>
```

But this requires:

- All intermediate types `Bi` to be `'static` (they may hold references).
- Boxing/unboxing at every step, with downcasts that can only be verified at
  runtime.
- The inner `F::Of<'a, B0>` to be mapped through a single composed function
  `B0 -> A`, which requires `F::map` to accept a function that internally
  does k downcasts. This is technically possible for `'static` types but
  fragile, and it changes the semantics: instead of k calls to `F::map`,
  you get one call to `F::map` with a k-step internally-boxing function.

Even if you accept the `'static` restriction and the boxing overhead, you still
cannot extract `F::Of<'a, B0>` from the base layer without knowing `B0`. The
base layer is behind `Box<dyn CoyonedaInner>`, and the only way to get the
value out is to call `lower()`, which returns `F::Of<'a, A>` (the final type),
not the intermediate `F::Of<'a, B0>`.

**Verdict for sub-approach 2: Infeasible.** Same root cause as sub-approach 1.

## Sub-approach 3: Use the Library's Free/Trampoline

The idea: restructure `lower` to return `Trampoline<F::Of<'a, A>>` or use
`Free<ThunkBrand, F::Of<'a, A>>`, converting the recursive descent into a
trampoline-style computation.

### Lifetime Conflict

`Free<F, A>` requires `A: 'static`. `Trampoline<A>` is `Free<ThunkBrand, A>`,
so it also requires `A: 'static`.

`Coyoneda<'a, F, A>` is parameterized over lifetime `'a`. The return type of
`lower` is `F::Of<'a, A>`, which may borrow data with lifetime `'a`. Wrapping
this in `Trampoline` would require `F::Of<'a, A>: 'static`, which is only
satisfied when `'a = 'static`.

This would restrict `Coyoneda::lower` to only work with `'static` values,
which is a breaking API change and defeats much of the purpose of lifetime
polymorphism.

### Recursion Structure Mismatch

Even ignoring the lifetime issue, the recursion in `lower` is not the kind that
trampolining typically addresses. Trampolining helps when you have _tail
recursion_ or _monadic sequencing_ that can be deferred. But `lower`'s recursion
is _structural_, following the nesting of trait objects:

```
lower(layer_k) = F::map(f_k, lower(layer_{k-1}))
```

This is not tail-recursive. The `F::map` call wraps the recursive result. To
trampoline this, you would need to somehow defer the `F::map(f_k, _)` call and
accumulate it. But `F::map` operates on `F::Of<'a, B>`, a functor-wrapped
value, and each layer has a different `B`. The trampoline would need to store
partially-applied `F::map` calls with their type-erased arguments, which
circles back to the type-erasure problem from sub-approach 1.

**Verdict for sub-approach 3: Infeasible.** The `'static` requirement of
`Free`/`Trampoline` conflicts with `Coyoneda`'s lifetime polymorphism, and the
non-tail-recursive structure of `lower` does not fit the trampoline pattern.

## Sub-approach 4: Custom Iterative Driver Without Free/Trampoline

The idea: build a bespoke iterative loop that avoids both the existing
`Free`/`Trampoline` machinery and the type-erasure via `Any`. Use a stack-like
data structure on the heap to store the pending `F::map` applications.

### The Fundamental Issue

An iterative driver loop needs a homogeneous "work item" type:

```rust
loop {
    match work_stack.pop() {
        Some(item) => { /* process item, maybe push more */ }
        None => return result,
    }
}
```

Each "work item" would be a pending `F::map(f_i, _)` application. But these
have different types:

```
F::map(f_1, _): F::Of<'a, B0> -> F::Of<'a, B1>
F::map(f_2, _): F::Of<'a, B1> -> F::Of<'a, B2>
```

To store these in a stack, you need a uniform type. A trait object like
`dyn FnOnce(F::Of<'a, ???>) -> F::Of<'a, ???>` cannot be written because the
input and output types vary.

You could try double-boxing through `Any`:

```rust
type AnyFunctor = Box<dyn Any>;  // erased F::Of<'a, Bi>
type Step = Box<dyn FnOnce(AnyFunctor) -> AnyFunctor>;
```

But `F::Of<'a, Bi>` is not `Any`-compatible when `'a != 'static`. And even
with `'static`, you need to box the functor value _itself_ (`F::Of<'a, Bi>`)
into `Box<dyn Any>`, which requires knowing the concrete type to box it. The
base layer knows `B0`, and the map layer knows `Bi` and `B_{i+1}`, but there
is no way to tell each layer "box your intermediate result as `dyn Any`" through
the current `CoyonedaInner` trait without adding a method that returns
`Box<dyn Any>`, which again requires `'static`.

**Verdict for sub-approach 4: Infeasible.** Same root cause as all above.

## Root Cause Analysis

All sub-approaches fail for the same fundamental reason: **the existential type
`B` hidden inside each `CoyonedaMapLayer` creates a type-level chain that
cannot be linearized without erasing intermediate functor values
`F::Of<'a, B>`.**

Erasing these values requires either:

1. **`Any`-based erasure**, which demands `'static` and breaks lifetime
   polymorphism.
2. **Unsafe transmute/cast**, which was explicitly excluded from scope.
3. **A method on `CoyonedaInner` that returns the erased intermediate**, but
   any such method would need to be generic (to handle the varying `B` types),
   which breaks dyn-compatibility.

The recursion in `lower` is inherently tied to the _nested existential types_
in the layered trait-object encoding. This is a qualitatively different problem
from tail-recursive computations or monadic bind chains, which is what
`Free`/`Trampoline` are designed to handle.

## What Would Work Instead (Out of Scope for This Analysis)

For completeness, approaches that _could_ provide stack safety for `lower`:

- **stacker crate**: Dynamically extends the stack when it gets too deep.
  Requires no API changes and no unsafe in user code (the crate itself uses
  platform-specific unsafe internally). Does not change the recursion structure.
- **Rewrite to CoyonedaExplicit**: `CoyonedaExplicit` keeps the function type
  parameter explicit, enabling true function composition. Its `lower` calls
  `F::map` exactly once. No stack depth issue.
- **Limit chain depth**: A runtime or compile-time limit on the number of
  chained `map` calls. Pragmatic but not a general solution.
- **Iterative lowering with unsafe type erasure**: Erase `F::Of<'a, B>` via
  pointer casts, collect functions, apply iteratively. Would require careful
  unsafe code and is explicitly out of scope.

## Verdict

**Infeasible.** Trampolining or continuation-passing cannot make `Coyoneda::lower`
stack-safe without either introducing unsafe code or adding a `'static`
constraint that would break the existing API. The root cause is that the
existential types hidden in each map layer create a heterogeneous type chain
that cannot be linearized into a homogeneous iterative loop in safe Rust. The
library's `Free`/`Trampoline` machinery is designed for a fundamentally
different kind of recursion (monadic bind chains with type-erased continuations)
and cannot be adapted to this problem.
