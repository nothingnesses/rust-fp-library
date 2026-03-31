# Feasibility: Iterative Lowering for Coyoneda

**Verdict: Infeasible** without `unsafe` code or `'static` constraints that would break the existing API.

---

## Problem Recap

`Coyoneda`'s `lower` method recurses through k nested `CoyonedaMapLayer` trait objects. Each layer's `lower` calls `self.inner.lower()`, then applies `F::map(self.func, lowered)`. For k chained maps, this produces k stack frames. Large k overflows the stack.

The goal is to convert this recursion into iteration without introducing `unsafe` code, without requiring `'static`, and without changing the public API.

---

## Sub-approach 1: Collect layers into a Vec, then iterate

The idea: peel each `CoyonedaMapLayer` apart, push the function onto a `Vec`, and keep going until we reach the `CoyonedaBase`. Then fold the `Vec` of functions over the base value.

### Why it fails

Each layer has a different hidden type `B`. The layer chain looks like:

```
CoyonedaMapLayer<F, B_k, A>        -- func: Fn(B_k) -> A
  CoyonedaMapLayer<F, B_{k-1}, B_k>  -- func: Fn(B_{k-1}) -> B_k
    ...
      CoyonedaMapLayer<F, B_1, B_2>    -- func: Fn(B_1) -> B_2
        CoyonedaBase<F, B_1>             -- holds F::Of<'a, B_1>
```

To collect these into a `Vec`, you would need a homogeneous element type. The functions have types `Fn(B_1) -> B_2`, `Fn(B_2) -> B_3`, ..., `Fn(B_k) -> A`. These are all different types. There is no common trait object you can use because:

1. A trait like `Fn(???) -> ???` requires concrete input and output types.
2. You cannot erase both input and output types of a function behind a single trait object without `Any` (which requires `'static`).
3. Even if you boxed each function as `Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>`, the values flowing between them would need to be `Any`, which requires `'static`. The current API supports `'a` lifetimes on the inner values.

A `Vec<Box<dyn SomeErasedFn>>` is not possible without either:

- A single trait that all the functions implement (they do not share one, since each has different input/output types), or
- `Any`-based type erasure (requires `'static`).

**Result: Not feasible.**

---

## Sub-approach 2: Step-based internal protocol (trampoline pattern)

The idea: add a `step` method to `CoyonedaInner` that returns an enum:

```rust
enum LowerStep<'a, F: Kind + 'a, A: 'a> {
    Done(F::Of<'a, A>),
    Continue(/* ??? */),
}
```

The outer loop would call `step` repeatedly until it gets `Done`.

### The type erasure wall

The `Continue` variant must carry two things:

1. The remaining inner `Box<dyn CoyonedaInner<'a, F, B>>` (the next layer down).
2. The function `Fn(B) -> A` from the current layer.

But `B` is existentially quantified; it is different at each layer. The `Continue` variant's type must be fixed, so it cannot mention `B`. You would need something like:

```rust
Continue(
    Box<dyn CoyonedaInner<'a, F, ???>>,   // B is unknown here
    Box<dyn Fn(???) -> A>,                 // B is unknown here too
)
```

You could try to erase `B` by returning a "continuation" that, given the lowered `F::Of<'a, B>`, produces `F::Of<'a, A>`:

```rust
enum LowerStep<'a, F: Kind + 'a, A: 'a> {
    Done(F::Of<'a, A>),
    Continue {
        // A function that takes the lowered inner value and applies F::map
        apply: Box<dyn FnOnce(/* F::Of<'a, B> */ ???) -> F::Of<'a, A> + 'a>,
        // The inner layer to lower next
        inner: Box<dyn CoyonedaInner<'a, F, ???>>,
    },
}
```

The problem remains: `inner` has type `Box<dyn CoyonedaInner<'a, F, B>>` where `B` is existential. If we erase `B` from the `Continue` variant, we cannot call `inner.lower()` because we cannot name its output type `F::Of<'a, B>`, and we cannot feed it to `apply` because `apply` expects `F::Of<'a, B>` but we cannot name `B`.

### Packaging inner + apply together

One could try to package the continuation and the inner layer into a single closure:

```rust
enum LowerStep<'a, F: Kind + Functor + 'a, A: 'a> {
    Done(F::Of<'a, A>),
    // A thunk that, when called, lowers the inner layer and applies F::map
    Defer(Box<dyn FnOnce() -> F::Of<'a, A> + 'a>),
}
```

But `Defer` here is just a deferred version of the recursive call. Converting the recursion to a loop of `Defer` calls does not actually help: the closure inside `Defer` still captures the inner layer and its function, and calling it still recurses through the entire remaining chain. You have not eliminated the recursion; you have just delayed it by one step.

To truly iterate, you would need to peel off one layer at a time, which requires the continuation and the inner layer to be separate, which requires naming `B`, which is existential.

**Result: Not feasible.**

---

## Sub-approach 3: Any-based type erasure

The idea: use `Box<dyn Any>` to erase the intermediate types, then downcast at each step.

### How it would work

1. Add a method `lower_to_any(self: Box<Self>) -> Box<dyn Any>` to `CoyonedaInner`.
2. Each `CoyonedaMapLayer` would:
   - Call `self.inner.lower_to_any()` to get a `Box<dyn Any>`.
   - Downcast to `F::Of<'a, B>`.
   - Apply `F::map(self.func, ...)`.
   - Box the result as `Box<dyn Any>`.

But this approach has two fatal problems:

**Problem 1: `Any` requires `'static`.** The `Any` trait is defined as:

```rust
pub trait Any: 'static { ... }
```

`Coyoneda` supports arbitrary lifetimes `'a`. If we require `F::Of<'a, B>: Any`, then `F::Of<'a, B>: 'static`, which means `'a = 'static` and `B: 'static`. This eliminates the lifetime polymorphism that makes `Coyoneda` useful.

**Problem 2: downcast requires knowing the concrete type.** Even with `Any`, you would need to know the concrete type `F::Of<'a, B>` to downcast. But `B` is existential; the code that would perform the downcast does not know `B`.

Actually, within a single `CoyonedaMapLayer<'a, F, B, A>`, `B` is known. So the downcast could work within each layer. But this still requires `'static`, which is the real blocker.

### Could we build our own non-'static Any?

No, not without `unsafe`. `Any` requires `'static` because `TypeId::of::<T>()` requires `T: 'static`. Building a non-`'static` type ID system would require `unsafe` code (specifically, `std::mem::transmute` or similar) to forge type identities for non-static types.

**Result: Not feasible** without `'static` constraints.

---

## Sub-approach 4: Trait method that returns an opaque "rest" + continuation

The idea: add a method to `CoyonedaInner` that separates "what is below me" from "what I add on top," but packages them in a way that hides `B`.

```rust
trait CoyonedaInner<'a, F, A: 'a> {
    fn step(self: Box<Self>) -> LowerStep<'a, F, A>;
}

enum LowerStep<'a, F: Kind + 'a, A: 'a> {
    Base(F::Of<'a, A>),
    Layer {
        // This closure knows B internally, closes over inner + func,
        // and when given "a way to lower inner," produces the result
        resolve: Box<dyn FnOnce(&dyn Lowerer<'a, F>) -> F::Of<'a, A> + 'a>,
    },
}
```

Where `Lowerer` would be a trait that can lower any `CoyonedaInner`. But the `resolve` closure would still need to call `inner.lower()` (or equivalent) inside itself, which means the recursion has just moved into the closure. The outer loop cannot "drive" the inner lowering because it cannot see through the closure.

The fundamental issue: to separate the inner value from the function and process them independently, the outer loop must be able to hold a value of type `F::Of<'a, B>` where `B` varies at each iteration. Rust's type system requires all loop iterations to work with the same types. Without type erasure (`Any` + `'static`) or `unsafe`, this cannot be done.

**Result: Not feasible.**

---

## Sub-approach 5: CPS (continuation-passing style) transformation

The idea: instead of building a stack of frames and then unwinding, pass a continuation forward.

In the recursive version:

```
lower(MapLayer(inner, f)) = F::map(f, lower(inner))
```

In CPS:

```
lower_cps(MapLayer(inner, f), k) = lower_cps(inner, |fb| k(F::map(f, fb)))
```

This makes the recursion tail-recursive in principle. But in Rust:

- Each iteration of `lower_cps` creates a new closure `|fb| k(F::map(f, fb))` that captures the previous continuation `k`.
- These closures nest: the final continuation is `k_n(k_{n-1}(...k_1(base)...))`.
- When the base case is reached and the continuation chain is invoked, it recurses through k nested closures, producing the same stack depth.

CPS does not eliminate the stack usage; it moves it from the "going down" phase to the "coming back up" phase. In a language with tail-call optimization, the "going down" phase would be O(1) stack, but the "coming back up" phase still requires O(k) stack. Rust does not guarantee TCO.

Even if Rust had TCO, the continuation invocation phase would still recurse through k frames. To avoid that, you would need to represent the continuation chain as a data structure (defunctionalization), which brings us back to the `Vec` of heterogeneous functions problem from sub-approach 1.

**Result: Not feasible.**

---

## Sub-approach 6: Defunctionalized continuation queue (Free monad pattern)

The library's `Free` monad solves a similar problem using `CatList<Continuation<F>>` where each continuation is `Box<dyn FnOnce(Box<dyn Any>) -> Free<F, Box<dyn Any>>>`. This uses `Box<dyn Any>` for type erasure.

Could the same pattern be applied to `Coyoneda`?

Yes, in principle: store the base `F::Of<'a, B_1>` as `Box<dyn Any>`, and each function as `Box<dyn FnOnce(Box<dyn Any>) -> Box<dyn Any>>`. At `lower` time:

1. Extract the base value.
2. Call `F::map` once with a composed function that folds through the continuation queue.

But this requires:

- `Any`, which requires `'static`.
- `F::Of<'static, B>: Any` for all intermediate `B`, meaning all `B: 'static`.
- The lifetime parameter on `Coyoneda` would be forced to `'static`.

This is exactly how `Free` works, and `Free` only supports `'static` types. The plan's constraint table notes: "`Trampoline` requires `'static`." Applying the same technique to `Coyoneda` would impose the same constraint.

**Result: Not feasible** without `'static` constraints.

---

## Root Cause Analysis

The infeasibility stems from a single root cause: **existential quantification via trait objects makes the hidden type `B` inaccessible at the iteration site.**

To iterate rather than recurse, the loop body must handle values of type `F::Of<'a, B>` where `B` changes each iteration. Rust's type system requires loop variables to have a single, statically known type. The only ways to handle varying types at runtime are:

1. **Trait objects** (which is what `CoyonedaInner` already is, and the trait object's method call is the recursion).
2. **`dyn Any` + downcasting** (requires `'static`).
3. **`unsafe` transmute** (explicitly excluded).
4. **Enum dispatch** (requires knowing all possible `B` types at compile time, which is impossible for an open set of user-defined types).

All four options are either already in use (option 1, which is the source of the recursion), require `'static` (option 2), require `unsafe` (option 3), or are impossible (option 4).

---

## Comparison with CoyonedaExplicit

`CoyonedaExplicit` avoids this problem entirely by exposing `B` as a type parameter instead of hiding it existentially. Each `.map` call composes functions at compile time (via generics, not trait objects), producing a single composed function. At `lower` time, one call to `F::map` applies the fully composed function. No recursion, no stack growth.

The trade-off: `CoyonedaExplicit` cannot implement the `Functor` trait via a brand (because the `B` parameter leaks into the type), so it cannot participate in HKT-polymorphic code. `Coyoneda` can, but pays for it with existential quantification, which is the source of the stack overflow risk.

---

## Conclusion

Converting `Coyoneda`'s recursive lowering to an iterative approach is **infeasible** in safe Rust without introducing a `'static` constraint. The existential quantification that makes `Coyoneda` useful (hiding `B` behind a trait object to enable HKT integration) is the same mechanism that forces recursive lowering. Every approach to iteration requires naming or erasing the hidden type `B` at the loop level, and safe Rust provides no mechanism to do this for non-`'static` types.

The recommended mitigation remains **documentation**: note the stack overflow risk for deep chains and recommend `CoyonedaExplicit` (with `.boxed()` for uniform types) when depth is a concern. For users who need both HKT integration and deep chains, the only viable path would be a `'static`-constrained variant (similar to `Free`/`Trampoline`), which would be a separate type rather than a modification to the existing `Coyoneda`.
