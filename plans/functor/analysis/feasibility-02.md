# Feasibility: Send/Sync and Library Integration for Generic Func Parameter

## 1. Send/Sync Auto-Derivation

With a generic `Func`, Rust automatically derives `Send` when `Func: Send` AND
`F::Of<'a, B>: Send`. No `unsafe impl` needed. This subsumes the
`SendThunk`/`SendCoyoneda` pattern entirely. The library's existing pattern of creating
separate `Send` types (e.g., `Thunk` vs `SendThunk`) exists because `Box<dyn Fn(...)>`
erases the concrete type, so the `+ Send` bound must be baked into the trait object. With
a generic `Func`, a single type handles both cases; the compiler enforces `Send` at the
call site. Note: there is no existing `SendCoyoneda` or `SendCoyonedaExplicit` in the
library today, so this is purely preventive; the generic approach avoids ever needing to
create one.

## 2. Interaction with the Pointer Brand System

`FnBrand<P>` and the generic `Func` parameter are complementary, not conflicting.
`FnBrand` operates in the type class layer where functions must be `Clone` (e.g.,
`Semiapplicative::apply` wraps closures in `Rc`/`Arc`). The generic `Func` operates in
the data structure layer where a single closure is stored and consumed once. The
`fold_map` and `apply` methods already take `FnBrand` as a separate type parameter for
the cloneable functions they need internally; this does not change.

## 3. Interaction with From Impls

**CoyonedaExplicit -> Coyoneda**: Works cleanly. `Coyoneda::new` takes
`impl Fn(B) -> A + 'a`, and a generic `Func` satisfying that bound can be passed
directly.

**Coyoneda -> CoyonedaExplicit**: The conversion lowers then lifts, so the result type
is always `CoyonedaExplicit<..., fn(A) -> A>` (identity). The extra `Func` parameter
does not complicate this.

**Type of Func for lift**: `fn(A) -> A` (a function pointer), which is
`Copy + Send + Sync + 'static`.

## 4. Interaction with Foldable, apply, bind

The extra `Func` parameter is always inferred from context (the receiver or argument
types), so call sites do not change. For `apply`, two extra type parameters (`FuncF`,
`FuncA`) appear in the signature, but both are inferred from the arguments. The return
type of `apply` and `bind` is `CoyonedaExplicit<..., fn(C) -> C>` because they lower
and re-lift. Error messages may show the full composed closure type, but `.boxed()`
provides an escape hatch to erase back to `Box<dyn Fn(B) -> A>`.

## 5. Does .boxed() Produce a Compatible Type?

Yes. `Box<dyn Fn(B) -> A>` implements `Fn(B) -> A` via a blanket impl in the standard
library (`impl<Args, F: Fn<Args> + ?Sized> Fn<Args> for Box<F>`). So
`CoyonedaExplicit<'a, F, B, A, Box<dyn Fn(B) -> A + 'a>>` satisfies the struct's
`Func: Fn(B) -> A + 'a` bound. All methods work. The boxed variant is `!Send`; a
`.boxed_send()` returning `Box<dyn Fn(B) -> A + Send + 'a>` would be `Send`.

## Summary

| Question                       | Finding                                                      |
| ------------------------------ | ------------------------------------------------------------ |
| Send/Sync auto-derivation      | Works automatically. No separate Send variant needed.        |
| Pointer brand interaction      | Complementary, not conflicting. Orthogonal concerns.         |
| From impls                     | CoyonedaExplicit -> Coyoneda works. Reverse unaffected.      |
| Foldable/apply/bind signatures | No inference regressions. Func is always inferred.           |
| .boxed() compatibility         | `Box<dyn Fn(B)->A>` implements `Fn(B)->A`. Fully compatible. |
