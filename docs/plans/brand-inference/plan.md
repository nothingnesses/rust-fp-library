# Plan: Optional Brand Inference for Free Functions

## Motivation

Every free function in fp-library (`map`, `bind`, `pure`, `lift2`, `apply`,
`fold_right`, `traverse`, etc.) requires the caller to specify the Brand via
turbofish:

```rust
map::<OptionBrand, _, _, _>(|x| x + 1, Some(5))
bind::<OptionBrand, _, _>(Some(5), |x| Some(x + 1))
pure::<OptionBrand, _>(5)
```

The Brand parameter cannot be inferred because the `Kind` trait's associated
type `Of<'a, A>` is a forward mapping (Brand -> concrete type), and the
compiler has no reverse mapping to recover Brand from a concrete type like
`Option<A>`. Multiple brands can map to the same concrete type (e.g.,
`Result<A, E>` is reached by `ResultErrAppliedBrand<E>`,
`ResultOkAppliedBrand<A>`, and `BifunctorFirstAppliedBrand<ResultBrand, E>`),
so inference is fundamentally ambiguous in the general case.

However, many types have exactly one brand. For these, the turbofish is
unnecessary boilerplate that hurts readability and makes the library harder
to learn. The `haskell_bits` crate demonstrates that a `TypeApp` trait
providing a reverse mapping (concrete type -> type constructor) enables
inference, letting users write `map(|x| x + 1, Some(5))` without any
turbofish at all.

## Goals

1. Allow users to omit the Brand turbofish for types with a single
   unambiguous brand.
2. Keep the turbofish available for types with multiple brands (Result,
   Tuple2, Pair, ControlFlow) and for disambiguation when needed.
3. Maintain zero-cost abstractions: no dynamic dispatch, no heap allocation.
4. Make the change purely additive (no breaking changes to existing code).
5. Enable proc-macro generation of the reverse mapping alongside
   `impl_kind!`.

## Design

### The `DefaultBrand` Trait

A new trait provides the reverse mapping from concrete types to their
canonical brand:

```rust
/// Maps a concrete type back to its canonical brand.
///
/// Only implemented for types where the brand is unambiguous (one brand
/// per concrete type). Types reachable through multiple brands (Result,
/// Tuple2, Pair, ControlFlow) do not implement this trait.
pub trait DefaultBrand {
    /// The canonical brand for this type.
    type Brand: Kind_cdc7cd43dac7585f;
}
```

Implementations for unambiguous types:

```rust
impl<A> DefaultBrand for Option<A> {
    type Brand = OptionBrand;
}

impl<A> DefaultBrand for Vec<A> {
    type Brand = VecBrand;
}

impl<A> DefaultBrand for Identity<A> {
    type Brand = IdentityBrand;
}

impl<'a, A> DefaultBrand for Thunk<'a, A> {
    type Brand = ThunkBrand;
}
// etc.
```

### Wrapper Free Functions with Inference

The existing free functions keep their signatures unchanged. New wrapper
functions use `DefaultBrand` to infer the brand:

```rust
/// Maps a function over a functor, inferring the brand from the container type.
///
/// This is a convenience wrapper around `map` that eliminates the need for
/// a Brand turbofish when the container type has a single unambiguous brand.
pub fn map_infer<'a, FA, A: 'a, B: 'a, Marker>(
    f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>,
    fa: FA,
) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
where
    FA: DefaultBrand + 'a,
    <FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>,
{
    f.dispatch(fa)
}
```

Usage:

```rust
// Before: turbofish required
let y = map::<OptionBrand, _, _, _>(|x| x + 1, Some(5));

// After: brand inferred from Option<i32>
let y = map_infer(|x| x + 1, Some(5));

// Turbofish still works for ambiguous types
let y = map::<ResultErrAppliedBrand<String>, _, _, _>(|x| x + 1, Ok(5));
```

### Alternative: Overloading `map` Itself

Instead of a separate `map_infer`, a single `map` function could accept
either a Brand turbofish or infer it. This is harder in Rust because a
function cannot have "optional" type parameters. Two approaches:

**Approach A: Trait-based dispatch on the container.**
Add a `MapInfer` trait that mirrors `FunctorDispatch` but resolves Brand
from the container. The unified `map` function would need a different
generic structure. This risks inference failures when both impls are
available.

**Approach B: Default type parameter on a newtype.**
Rust does not support default type parameters on functions, only on structs
and traits. This approach does not work directly.

The recommended path is to start with separate `_infer` suffixed functions,
then evaluate whether a unified signature is feasible once the trait is
proven.

### Interaction with `FunctorDispatch`

The `FunctorDispatch` trait in `functor_dispatch.rs` dispatches between
`Functor::map` (Val marker) and `RefFunctor::ref_map` (Ref marker) based
on the closure's argument type. Brand inference is orthogonal to this
dispatch axis:

- `FunctorDispatch` resolves the `Marker` parameter (Val vs Ref).
- `DefaultBrand` resolves the `Brand` parameter.

Both work via trait resolution at compile time. The `map_infer` wrapper
uses `DefaultBrand` to fix Brand, then delegates to `FunctorDispatch`
for the Marker dispatch, composing both inference mechanisms.

### Interaction with `m_do!` and `a_do!`

The `m_do!` and `a_do!` proc macros currently require a Brand identifier as
the first token:

```rust
m_do!(OptionBrand { x <- Some(5); pure(x + 1) })
```

Brand inference could allow an alternative syntax where the macro infers
the brand from the first bind expression:

```rust
m_do!({ x <- Some(5); pure(x + 1) })
```

However, this is significantly more complex because the macro would need to
either:

- Defer brand resolution to the type checker (by generating code that uses
  `DefaultBrand` internally), or
- Require the first expression's type to be statically known at macro
  expansion time (which proc macros cannot do).

The `DefaultBrand`-based approach works: the macro could generate
`bind_infer(expr, ...)` calls instead of `bind::<Brand, _, _>(expr, ...)`.
This is a natural follow-on once the core `_infer` functions exist.

### Interaction with the `Apply!` Macro

The `Apply!` macro expands `<Brand as Kind!(...)>::Of<'a, A>` into a
concrete associated type projection. With `DefaultBrand`, the `Apply!`
macro is not involved at call sites because the wrapper functions handle
the projection internally. The trait bound
`<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>` ties
the concrete type back to the brand's `Of` associated type, ensuring
type safety.

## Types That CAN Implement `DefaultBrand`

These types have exactly one brand and an unambiguous reverse mapping:

| Concrete type                        | Brand                         | Notes                         |
| ------------------------------------ | ----------------------------- | ----------------------------- |
| `Option<A>`                          | `OptionBrand`                 |                               |
| `Vec<A>`                             | `VecBrand`                    |                               |
| `Identity<A>`                        | `IdentityBrand`               |                               |
| `Thunk<'a, A>`                       | `ThunkBrand`                  |                               |
| `SendThunk<'a, A>`                   | `SendThunkBrand`              |                               |
| `CatList<A>`                         | `CatListBrand`                |                               |
| `Lazy<'a, A, Config>`                | `LazyBrand<Config>`           | Parameterized by Config       |
| `TryLazy<'a, A, E, Config>`          | `TryLazyBrand<E, Config>`     | Parameterized by E and Config |
| `Coyoneda<'a, F, A>`                 | `CoyonedaBrand<F>`            | Parameterized by F            |
| `RcCoyoneda<'a, F, A>`               | `RcCoyonedaBrand<F>`          | Parameterized by F            |
| `ArcCoyoneda<'a, F, A>`              | `ArcCoyonedaBrand<F>`         | Parameterized by F            |
| `BoxedCoyonedaExplicit<'a, F, B, A>` | `CoyonedaExplicitBrand<F, B>` | Parameterized by F and B      |
| `Const<'a, R, A>`                    | `ConstBrand<R>`               | Parameterized by R            |
| `(A,)`                               | `Tuple1Brand`                 | Single-element tuple          |
| `TryThunk<'a, A, E>`                 | Ambiguous                     | See below                     |

Note: `TryThunk<'a, A, E>` maps to both `TryThunkErrAppliedBrand<E>` (functor
over A) and `TryThunkOkAppliedBrand<A>` (functor over E), plus the bifunctor
`TryThunkBrand`. It should NOT implement `DefaultBrand`.

Types that do NOT have `impl_kind!` and therefore have no brand at all
(no `DefaultBrand` needed): `Free`, `Trampoline`, `TryTrampoline`,
`TrySendThunk`, `Additive`, `Multiplicative`, `Conjunctive`, `Disjunctive`,
`Dual`, `First`, `Last`, `Endomorphism`, `Endofunction`.

## Types That CANNOT Implement `DefaultBrand`

These types are reachable through multiple brands:

| Concrete type         | Brands                                                                                                                                                                     | Reason                                            |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------- |
| `Result<A, E>`        | `ResultErrAppliedBrand<E>`, `ResultOkAppliedBrand<A>`, `BifunctorFirstAppliedBrand<ResultBrand, E>`, `BifunctorSecondAppliedBrand<ResultBrand, A>`                         | Two type parameters, each can be the "mapped" one |
| `(First, Second)`     | `Tuple2FirstAppliedBrand<First>`, `Tuple2SecondAppliedBrand<Second>`, `BifunctorFirstAppliedBrand<Tuple2Brand, First>`, `BifunctorSecondAppliedBrand<Tuple2Brand, Second>` | Same two-parameter ambiguity                      |
| `Pair<First, Second>` | `PairFirstAppliedBrand<First>`, `PairSecondAppliedBrand<Second>`, `BifunctorFirstAppliedBrand<PairBrand, First>`, `BifunctorSecondAppliedBrand<PairBrand, Second>`         | Same two-parameter ambiguity                      |
| `ControlFlow<B, C>`   | `ControlFlowBreakAppliedBrand<B>`, `ControlFlowContinueAppliedBrand<C>`                                                                                                    | Break vs Continue functors                        |
| `TryThunk<'a, A, E>`  | `TryThunkErrAppliedBrand<E>`, `TryThunkOkAppliedBrand<A>`                                                                                                                  | Success vs error functors                         |

For these types, users must continue using the explicit Brand turbofish.

## How `haskell_bits` Achieves This

The `haskell_bits` crate uses three traits to enable brand-free inference:

1. **`WithTypeArg<T>`** (on the brand/type constructor): Maps Brand + T to
   the concrete type. This is equivalent to fp-library's `Kind` trait with
   its `Of` associated type.

2. **`TypeApp<TCon, T>`** (on the concrete type): The reverse mapping.
   `Option<T>: TypeApp<TypeCon, T>` tells the compiler that `Option<T>` is
   the application of `TypeCon` to `T`. This is what fp-library lacks.

3. **`TypeAppParam`** (on the concrete type): Extracts the type parameter.
   `Option<T>::Param = T`.

The key insight is that `haskell_bits` uses a _single_ `TypeCon` per module
(e.g., `option::TypeCon`, `vec::TypeCon`). Each concrete type implements
`TypeApp<TypeCon, T>` for exactly one `TypeCon`. The free function `lmap`
constrains `X: TypeApp<TCon, TIn>`, and Rust infers `TCon` from `X` because
there is only one `TypeApp` impl per concrete type.

The `DefaultBrand` trait proposed here is analogous to `TypeApp`, but
simplified: rather than parameterizing on the type argument, it directly
associates the brand. This avoids the need for `TypeAppParam` and the
`Is` trait that `haskell_bits` uses for type equality witnesses.

### Key Difference

`haskell_bits` does not face the ambiguity problem for `Result` because each
module defines its own `TypeCon`. There is no shared `ResultBrand` with
multiple partial applications. In fp-library, the brand system explicitly
supports multiple partial applications of bifunctors, which is more
expressive but makes universal inference impossible.

## Potential Issues

### Coherence and Orphan Rules

`DefaultBrand` impls for library-defined types (`Identity`, `Thunk`, etc.)
are straightforward because both the trait and the type are in the same
crate. For `Option` and `Vec`, the trait is defined in fp-library and the
types are in `std`, so this is also fine (downstream impl, upstream type).

Users cannot implement `DefaultBrand` for their own types unless the trait
is public and unsealed, which is the intended design. However, if a user
defines a type that maps to an fp-library brand via `impl_kind!`, they would
also want to implement `DefaultBrand`. This is valid under orphan rules as
long as the user's type is local.

### Type Inference Failures

The constraint `<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>`
requires the compiler to unify `FA` with the `Of` associated type. This
should work when `FA` is a concrete type (e.g., `Option<i32>`) but may fail
with generic `FA` because the compiler cannot determine which `DefaultBrand`
impl applies.

This means `map_infer` works at concrete call sites but not in generic
functions. Generic code must still use the explicit-Brand versions:

```rust
// Works: concrete type
let y = map_infer(|x| x + 1, Some(5));

// Fails: generic FA, compiler cannot resolve DefaultBrand
fn generic_map<FA: DefaultBrand>(fa: FA) { ... }

// Still works: explicit Brand in generic context
fn generic_map<Brand: Functor, A>(fa: Brand::Of<A>) { ... }
```

This is acceptable because generic HKT code is inherently Brand-explicit.
The inference benefit targets concrete call sites, which are the majority
of user-facing code.

### Lifetime Complications

Some types carry a lifetime parameter (e.g., `Thunk<'a, A>`,
`Lazy<'a, A, Config>`). The `DefaultBrand` impl must be generic over the
lifetime:

```rust
impl<'a, A: 'a> DefaultBrand for Thunk<'a, A> {
    type Brand = ThunkBrand;
}
```

This should work because `ThunkBrand` itself is `'static` (brands are
zero-sized marker types) and the lifetime only appears in the `Kind` trait's
`Of<'a, A>` associated type.

### Interaction with `Apply!` Macro Type Projections

The `Apply!` macro is used in function signatures to project
`<Brand as Kind!(...)>::Of<'a, A>`. In the `_infer` wrappers, the function
parameter type is `FA` (the concrete type) rather than a projection, so
`Apply!` is not needed. The trait bound ties `FA` to the brand's `Of` type,
ensuring the two representations are equivalent.

## Proc-Macro Generation via `impl_kind!`

The `impl_kind!` macro already parses the brand-to-type mapping:

```rust
impl_kind! {
    for OptionBrand {
        type Of<'a, A: 'a>: 'a = Option<A>;
    }
}
```

It could optionally generate a `DefaultBrand` impl alongside the `Kind`
impl:

```rust
// Generated by impl_kind!
impl<'a, A: 'a> DefaultBrand for Option<A> {
    type Brand = OptionBrand;
}
```

To control when this is generated, `impl_kind!` could accept an attribute:

```rust
impl_kind! {
    #[default_brand]
    for OptionBrand {
        type Of<'a, A: 'a>: 'a = Option<A>;
    }
}
```

Types with multiple brands would omit the attribute. This keeps the
generation opt-in and avoids conflicting impls.

**Complexity consideration:** The macro would need to extract the type
parameters from the associated type definition and reconstruct them as
generic parameters on the `DefaultBrand` impl. For simple cases
(`Option<A>`, `Vec<A>`), this is straightforward. For parameterized brands
(`LazyBrand<Config>` -> `Lazy<'a, A, Config>`), the macro must merge the
`impl` generics with the associated type generics. This is doable but
requires careful handling of lifetime and type parameter ordering.

## Whether This Is a Breaking Change

This design is purely additive:

- The `DefaultBrand` trait is new. No existing types or traits change.
- The `_infer` wrapper functions are new. The existing `map`, `bind`,
  `pure`, etc. keep their exact signatures.
- The turbofish syntax continues to work everywhere.
- Existing code compiles without modification.

The only risk is if a future change tries to unify the `_infer` wrappers
with the originals (e.g., overloading `map` to work both ways). That would
require more careful design to avoid inference regressions.

## Design Decisions

1. **Separate `_infer` functions rather than overloading the originals.**
   Overloading `map` to accept either an explicit Brand or infer it would
   require a dispatch trait with two impls (explicit vs inferred), risking
   ambiguity when the compiler cannot tell which path to take. Separate
   functions make the ergonomics/precision trade-off explicit at the call
   site. Revisit unification after the trait is proven.

2. **`DefaultBrand` on the concrete type, not a `TypeApp`-style witness.**
   `haskell_bits` uses `TypeApp<TCon, T>` on the concrete type, which is
   parameterized by both the type constructor and the element type. This
   supports multiple `TypeApp` impls per type (one per TCon). `DefaultBrand`
   is simpler: one impl per concrete type, providing exactly one canonical
   brand. This matches the design constraint that only unambiguous types
   get inference. The simplicity avoids the `Is` type-equality machinery
   that `haskell_bits` requires.

3. **Opt-in generation via `impl_kind!` attribute.**
   Not all `impl_kind!` invocations should generate `DefaultBrand`. Types
   with multiple brands, optics profunctors, and partially-applied bifunctor
   adapters must not generate conflicting impls. An explicit `#[default_brand]`
   attribute makes the intent clear and prevents accidental conflicts.

4. **Generic code still requires explicit brands.**
   `DefaultBrand` resolves at concrete types only. Generic HKT functions
   must still parameterize over Brand explicitly. This is not a limitation
   to solve; it is inherent to how Rust resolves associated types. The
   inference benefit targets leaf-level application code, not library
   internals.

5. **`pure` needs special handling.**
   Unlike `map` and `bind`, `pure` takes only a value, not a container.
   There is no `FA` to resolve `DefaultBrand` from. `pure_infer` would need
   to infer the brand from the return type, which Rust can sometimes do
   via `let x: Option<i32> = pure_infer(5)` but not always. The `_infer`
   variant of `pure` may not be worth providing, or it could be provided
   with the understanding that a return-type annotation is often needed.

## Implementation Order

1. **Define `DefaultBrand` trait** in `fp-library/src/classes/default_brand.rs`.
   Simple trait with one associated type. Add module to `classes.rs`.

2. **Implement `DefaultBrand` for core types.** Start with `Option`, `Vec`,
   `Identity`, `Thunk`, `SendThunk`, `CatList`, `Tuple1Brand`.

3. **Implement `DefaultBrand` for parameterized types.** `Lazy`, `TryLazy`,
   `Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`, `BoxedCoyonedaExplicit`, `Const`.

4. **Add `map_infer` wrapper.** Verify that type inference works for
   `Option`, `Vec`, `Identity`, and `Lazy` with both Val and Ref dispatch
   markers.

5. **Add `bind_infer`, `lift2_infer`, `apply_infer` wrappers.** Follow the
   same pattern as `map_infer`.

6. **Evaluate `pure_infer`.** Test whether return-type inference is
   sufficient for common usage patterns. Add if practical, skip if not.

7. **Add `#[default_brand]` attribute to `impl_kind!`.** Extend the proc
   macro to optionally generate `DefaultBrand` impls. Migrate hand-written
   impls to use the attribute.

8. **Extend `m_do!` and `a_do!`.** Add an optional brand-free syntax that
   generates `bind_infer` / `map_infer` calls instead of explicit-Brand
   versions.

9. **Documentation.** Update crate docs, README examples, and the
   `docs/features.md` file to show the inference-based calling convention
   alongside the turbofish convention.

10. **Tests.** Unit tests for each `DefaultBrand` impl. Compile-fail tests
    confirming that ambiguous types (Result, Tuple2, etc.) do not compile
    with `_infer` functions. Doc tests showing both calling conventions.

## References

- `haskell_bits` crate: `/home/jessea/Documents/projects/haskell_bits/src/typeapp.rs`,
  `/home/jessea/Documents/projects/haskell_bits/src/functor.rs`.
  Demonstrates the `WithTypeArg`/`TypeApp`/`TypeAppParam` system for
  full brand inference in Rust HKT encoding.
- Current `FunctorDispatch` system:
  `fp-library/src/classes/functor_dispatch.rs`.
  Shows marker-type dispatch for Val/Ref; brand inference composes with this.
- `Kind` trait definitions: `fp-library/src/kinds.rs`.
- Brand definitions: `fp-library/src/brands.rs`.
- `impl_kind!` macro: `fp-macros/src/hkt/impl_kind.rs`.
- `Apply!` macro: `fp-macros/src/hkt/apply.rs`.
- Ref-hierarchy plan: `docs/plans/ref-hierarchy/plan.md`.
  Demonstrates the plan document format and the `FunctorDispatch` proof of
  concept that this plan builds on.
