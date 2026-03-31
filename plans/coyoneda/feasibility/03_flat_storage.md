# Feasibility: Flat Storage via Vec for Coyoneda Lowering

**Verdict: Infeasible** without unsafe code. The type-erasure requirements are fundamentally incompatible with safe Rust's type system, and the constraints introduced by workarounds (`'static`, loss of lifetime flexibility) would break the existing API contract.

---

## 1. The idea

Replace the recursive nesting of `Box<dyn CoyonedaInner>` layers with a flat data structure:

```rust
pub struct Coyoneda<'a, F, A: 'a> {
    /// The original functor value, type-erased.
    base: Box<dyn Any>,
    /// A flat list of type-erased map functions.
    maps: Vec<Box<dyn ErasedMapFn>>,
}
```

At `lower` time, iterate through `maps` and apply each function via `F::map`, avoiding recursive calls entirely.

## 2. The type-erasure problem

Consider a chain `lift(fb).map(f1).map(f2).map(f3)` where:

- `fb: F::Of<'a, B0>`
- `f1: Fn(B0) -> B1`
- `f2: Fn(B1) -> B2`
- `f3: Fn(B2) -> A`

To store these in a `Vec`, every function must have the same type. But `f1`, `f2`, and `f3` have different signatures (`Fn(B0) -> B1`, `Fn(B1) -> B2`, `Fn(B2) -> A`). The input and output types are all different.

### 2.1 Can `std::any::Any` erase the types?

The natural approach is to erase both the function and its input/output types:

```rust
trait ErasedMapFn: 'static {
    fn apply(&self, input: Box<dyn Any>) -> Box<dyn Any>;
}
```

This faces three problems:

**Problem 1: `'static` requirement.** `Any` requires `'static`. But `Coyoneda<'a, F, A>` supports arbitrary lifetimes `'a`. The values inside `F::Of<'a, B>` may contain references. Requiring `'static` would break the API: `Coyoneda::<VecBrand, &str>::lift(vec!["hello"])` would become illegal. This is a hard constraint, not a theoretical concern, since lifetime flexibility is a documented design goal of `Coyoneda` (contrasted with `Trampoline` which requires `'static`).

**Problem 2: The values are not `Any`-compatible at the functor level.** Each intermediate step operates on `F::Of<'a, Bi>`, not on bare `Bi` values. `F::map` takes an `F::Of<'a, Bi>` and produces an `F::Of<'a, Bi+1>`. To iterate through the vec, each step must:

1. Downcast `Box<dyn Any>` to the concrete `F::Of<'a, Bi>`.
2. Call `F::map(fi, value)` to get `F::Of<'a, Bi+1>`.
3. Box the result back to `Box<dyn Any>`.

Step 1 requires knowing the concrete type `F::Of<'a, Bi>` at runtime. `Any::downcast_ref` requires the exact `TypeId`, which is only available at compile time. The whole point of existential quantification in `Coyoneda` is that these intermediate types are erased, so the code calling `lower` does not know what `Bi` is. This is a fundamental contradiction: you need to know the type to downcast, but the purpose of the design is to hide the type.

**Problem 3: `F::map` requires concrete types at compile time.** The signature is:

```rust
fn map<'a, A: 'a, B: 'a>(
    f: impl Fn(A) -> B + 'a,
    fa: F::Of<'a, A>,
) -> F::Of<'a, B>;
```

`A` and `B` are monomorphized at the call site. An iterative loop cannot call `F::map` with different `A`/`B` types on each iteration, since each call site is monomorphized to one specific pair. You would need a single call site that works for all type pairs, which is exactly what generic methods on trait objects would provide, and Rust does not support those (the dyn-compatibility restriction that is the root cause of all Coyoneda limitations).

### 2.2 Can a custom trait object replace `Any`?

One might try:

```rust
trait ErasedStep<'a, F: Functor + 'a>: 'a {
    fn apply_map(self: Box<Self>, input: ???) -> ???;
}
```

The `???` placeholders are the problem. `apply_map` would need to accept `F::Of<'a, SomeType>` and return `F::Of<'a, AnotherType>`, but both `SomeType` and `AnotherType` are type parameters, making the method generic and therefore not dyn-compatible.

A version without generics would need a universal representation:

```rust
trait ErasedStep<'a, F: Functor + 'a>: 'a {
    fn apply_map(self: Box<Self>, input: ErasedFunctorValue) -> ErasedFunctorValue;
}
```

But `ErasedFunctorValue` is just `Box<dyn Any>` (or a raw pointer) in disguise, which brings back all the problems from section 2.1.

### 2.3 What about erasing only the functions, not the functor values?

Suppose we keep the functor value concrete and only erase the map functions:

```rust
pub struct Coyoneda<'a, F, A: 'a> {
    base: F::Of<'a, ???>,  // What type goes here?
    maps: Vec<Box<dyn ???>>,
}
```

The base value is `F::Of<'a, B0>` where `B0` is the original type from `lift`. But `Coyoneda<'a, F, A>` does not have `B0` in its type signature, so `B0` must be erased. We are back to `Box<dyn Any>` for the base.

Even if we accept `'static` for the base, the iteration problem remains: at each step `i`, we need to unwrap the functor value, apply function `i`, and re-wrap. But `F::map` is the only way to apply a function inside the functor context, and `F::map` requires concrete types. The loop body would need to be generic over the step's input/output types, which is impossible in a runtime loop.

## 3. The fundamental obstruction

The recursive nesting in the current implementation is not an accident; it is the only way to preserve type safety across heterogeneous function chains in safe Rust. Each `CoyonedaMapLayer` captures its specific `B` type in its `impl CoyonedaInner for CoyonedaMapLayer<..., B, A>` implementation. The vtable for each layer's `Box<dyn CoyonedaInner>` encodes the monomorphized `lower` implementation that knows how to call `F::map` with the correct types.

A flat `Vec` would need to do the same work (call `F::map` with correct types at each step), but without the vtable dispatch that the recursive encoding provides. The only alternatives are:

1. **Unsafe transmutes or raw pointers** to bypass the type system (explicitly forbidden by the requirements).
2. **`Any`-based downcasting** which requires `'static` and runtime type checks that can fail.
3. **A single composed function** stored alongside the original `F::Of<'a, B0>`, which is exactly what `CoyonedaExplicit` already does.

Option 3 is the correct solution for the flat-storage goal, and it already exists in the codebase. The difference is that `CoyonedaExplicit` exposes `B0` as a type parameter, which prevents full HKT integration but enables single-pass fusion.

## 4. What about a hybrid: Vec of composed closures?

Could we store a single composed `Box<dyn Fn(B) -> A>` and update it with each `map` call, avoiding the nesting?

```rust
pub struct Coyoneda<'a, F, A: 'a> {
    // B is existentially hidden
    inner: Box<dyn CoyonedaInner<'a, F, A> + 'a>,
}
```

This is essentially what `map` would do if it could compose the new function with the stored one. But composing `f: Fn(A) -> C` with the stored `Fn(B) -> A` requires knowing `A` (the current output type of the stored function), which is the hidden intermediate type. On the trait-object boundary, `A` from the caller's perspective is the output type visible in `CoyonedaInner<'a, F, A>`, so composing would require a method like:

```rust
fn compose_and_replace<C>(self: Box<Self>, f: impl Fn(A) -> C) -> Box<dyn CoyonedaInner<'a, F, C>>
```

This method is generic over `C`, making it not dyn-compatible. This is exactly the "cannot compose across the existential boundary" limitation documented in the module.

## 5. Comparison with the recursive approach

| Property            | Recursive nesting (current) | Flat Vec (proposed)              |
| ------------------- | --------------------------- | -------------------------------- |
| Type safety         | Full (compile-time checked) | Requires unsafe or `Any`         |
| Lifetime support    | `'a` (arbitrary)            | `'static` only (via `Any`)       |
| `F::map` dispatch   | Via vtable per layer        | Impossible without type info     |
| Stack overflow risk | Yes (depth k)               | No (iterative)                   |
| API compatibility   | Current API                 | Would break lifetime flexibility |

## 6. Conclusion

Flat storage via `Vec` is **infeasible** in safe Rust for `Coyoneda`'s existentially quantified design. The core obstruction is that each map function has different input/output types, and iterating over a flat collection of such functions requires recovering those types at runtime, which safe Rust cannot do without `'static` constraints and `Any` downcasting.

The recursive nesting is the natural and correct encoding of heterogeneous type-erased function chains in safe Rust. The stack overflow risk it introduces is a real limitation, but the solution lies elsewhere:

- **For users who need deep chains:** Use `CoyonedaExplicit` with `.boxed()`, which stores a single composed function and calls `F::map` once.
- **For `Coyoneda` itself:** Trampolining (approach B from the plan) or documenting the limitation (approach D) are more promising paths. The trampoline approach would convert the recursive `lower` into a continuation-passing loop using an internal protocol, without flattening the storage.
