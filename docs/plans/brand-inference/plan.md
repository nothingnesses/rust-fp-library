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
   unambiguous brand. The inference-based calling convention should be
   the primary, default API (`map`, `bind`, etc.).
2. Keep explicit-brand functions available (with an `_explicit` suffix
   like `map_explicit`) for types with multiple brands (Result, Tuple2,
   Pair, ControlFlow) and for disambiguation when needed.
3. Maintain zero-cost abstractions: no dynamic dispatch, no heap allocation.
4. Enable proc-macro generation of the reverse mapping alongside
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

### Naming Convention

The inference-based functions get the clean names (`map`, `bind`, `apply`,
etc.). The explicit-brand functions are renamed with a `_explicit` suffix
(`map_explicit`, `bind_explicit`, `apply_explicit`). This makes the ergonomic path
the default and the explicit path the escape hatch.

Usage:

```rust
// Primary API: brand inferred from container type
let y = map(|x| x + 1, Some(5));
let y = bind(Some(5), |x| Some(x + 1));

// Explicit brand: for ambiguous types or disambiguation
let y = map_explicit::<ResultErrAppliedBrand<String>, _, _, _>(|x| x + 1, Ok(5));
```

### Inference-Based Free Functions

The new `map` function uses `DefaultBrand` to infer the brand:

```rust
/// Maps a function over a functor, inferring the brand from the container type.
pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
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

The old `map` is renamed to `map_explicit` with its signature unchanged.

### Interaction with `FunctorDispatch`

The `FunctorDispatch` trait dispatches between `Functor::map` (Val marker)
and `RefFunctor::ref_map` (Ref marker) based on the closure's argument type.
Brand inference is orthogonal to this dispatch axis:

- `FunctorDispatch` resolves the `Marker` parameter (Val vs Ref).
- `DefaultBrand` resolves the `Brand` parameter.

Both work via trait resolution at compile time. The inferred `map` function
uses `DefaultBrand` to fix Brand, then delegates to `FunctorDispatch`
for the Marker dispatch, composing both inference mechanisms.

### Interaction with Dispatch Extensions

The ref-hierarchy plan (see `docs/plans/ref-hierarchy/plan.md`) extends
dispatch to `bind`, `apply`, and `lift2` via `MonadDispatch`,
`ApplicativeDispatch`, etc. Each dispatch trait gets a corresponding
inference-based wrapper following the same `DefaultBrand` pattern:

- `map` (inferred) / `map_explicit` (explicit) via `FunctorDispatch`
- `bind` (inferred) / `bind_explicit` (explicit) via `MonadDispatch`
- `apply` (inferred) / `apply_explicit` (explicit) via `ApplicativeDispatch`

Each new dispatch trait added by the ref-hierarchy plan gets a
corresponding inference-based wrapper following the same `DefaultBrand`
pattern.

### Interaction with `m_do!` and `a_do!`

The `m_do!` and `a_do!` proc macros currently require a Brand identifier as
the first token:

```rust
m_do!(OptionBrand { x <- Some(5); pure(x + 1) })
```

Brand inference allows an alternative syntax where the macro infers
the brand from the first bind expression:

```rust
m_do!({ x <- Some(5); pure(x + 1) })
```

The macro would generate `bind(expr, ...)` calls (which use `DefaultBrand`
internally) instead of `bind_explicit::<Brand, _, _>(expr, ...)`. This is a
natural follow-on once the core inference functions exist. The
Brand-explicit syntax remains available for ambiguous types.

### Interaction with the `Apply!` Macro

The `Apply!` macro is used in function signatures to project
`<Brand as Kind!(...)>::Of<'a, A>`. In the inference-based functions, the
parameter type is `FA` (the concrete type) rather than a projection, so
`Apply!` is not needed. The trait bound
`<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>` ties
the concrete type back to the brand's `Of` associated type, ensuring
type safety.

### Why `pure` Cannot Use Brand Inference

Unlike `map` and `bind`, `pure` takes only a value, not a container.
There is no `FA` to resolve `DefaultBrand` from. The brand can only be
inferred from the return type, which Rust can sometimes do via
`let x: Option<i32> = pure(5)` but not always. Since a return-type
annotation is no better than a turbofish, `pure` stays as-is with an
explicit Brand parameter. No `pure_explicit` rename is needed.

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

For these types, users must use the `_explicit` suffixed functions.

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

### Error Messages for Types Without `DefaultBrand`

When a user writes `map(f, Ok(5))`, the compiler says
"`DefaultBrand` is not implemented for `Result<i32, _>`." This is correct
but doesn't tell them to use `map_explicit` instead. Use
`#[diagnostic::on_unimplemented]` (stable since Rust 1.78) to provide a
better message:

```rust
#[diagnostic::on_unimplemented(
    message = "`{Self}` has multiple brands and cannot use brand inference",
    note = "use `map_explicit::<YourBrand, _, _, _>(f, x)` to specify the brand explicitly"
)]
pub trait DefaultBrand { ... }
```

### The GAT Equality Bound Must Be Verified

The constraint `<FA as DefaultBrand>::Brand: Kind_cdc7cd43dac7585f<Of<'a, A> = FA>`
requires the compiler to unify `FA` with the brand's `Of` associated type.
This should work when `FA` is a concrete type (e.g., `Option<i32>`) but may
fail with generic `FA` because the compiler cannot determine which
`DefaultBrand` impl applies. This is an unverified assumption that must be
tested in the POC before proceeding.

### Generic Code Still Requires Explicit Brands

`DefaultBrand` resolves at concrete types only. Generic HKT functions
must still parameterize over Brand explicitly. This is not a limitation
to solve; it is inherent to how Rust resolves associated types. The
inference benefit targets leaf-level application code, not library
internals.

```rust
// Works: concrete type
let y = map(|x| x + 1, Some(5));

// Fails: generic FA, compiler cannot resolve DefaultBrand
fn generic_map<FA: DefaultBrand>(fa: FA) { ... }

// Still works: explicit Brand in generic context
fn generic_map<Brand: Functor, A>(fa: Brand::Of<A>) { ... }
```

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
`<Brand as Kind!(...)>::Of<'a, A>`. In the inference-based functions, the
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

`DefaultBrand` impls should be generated by default alongside the `Kind`
impl:

```rust
// Generated by impl_kind!
impl<'a, A: 'a> DefaultBrand for Option<A> {
    type Brand = OptionBrand;
}
```

Types with multiple brands must opt out via a `#[no_default_brand]`
attribute:

```rust
impl_kind! {
    #[no_default_brand]
    for ResultErrAppliedBrand<E> {
        type Of<'a, A: 'a>: 'a = Result<A, E>;
    }
}
```

This makes the common case (single brand) the default. Forgetting
`#[no_default_brand]` on a multi-brand type results in a conflicting impl
compiler error, which is a clear signal.

**Complexity consideration:** The macro would need to extract the type
parameters from the associated type definition and reconstruct them as
generic parameters on the `DefaultBrand` impl. For simple cases
(`Option<A>`, `Vec<A>`), this is straightforward. For parameterized brands
(`LazyBrand<Config>` -> `Lazy<'a, A, Config>`), the macro must merge the
`impl` generics with the associated type generics. This is doable but
requires careful handling of lifetime and type parameter ordering.

## Design Decisions

1. **Inference-based functions get the clean names.**
   `map`, `bind`, `apply`, etc. use `DefaultBrand` for inference. The
   explicit-brand versions are renamed to `map_explicit`, `bind_explicit`, etc.
   This makes the ergonomic path the default, since single-brand types
   are the common case.

2. **`DefaultBrand` on the concrete type, not a `TypeApp`-style witness.**
   `haskell_bits` uses `TypeApp<TCon, T>` on the concrete type, which is
   parameterized by both the type constructor and the element type. This
   supports multiple `TypeApp` impls per type (one per TCon). `DefaultBrand`
   is simpler: one impl per concrete type, providing exactly one canonical
   brand. This matches the design constraint that only unambiguous types
   get inference. The simplicity avoids the `Is` type-equality machinery
   that `haskell_bits` requires.

3. **Default generation via `impl_kind!`, opt-out for exceptions.**
   `impl_kind!` generates `DefaultBrand` by default. Types with multiple
   brands use `#[no_default_brand]` to opt out. The conflicting impl error
   serves as a safety net if the attribute is forgotten.

4. **Generic code still requires explicit brands.**
   `DefaultBrand` resolves at concrete types only. Generic HKT functions
   must still parameterize over Brand explicitly. This is not a limitation
   to solve; it is inherent to how Rust resolves associated types. The
   inference benefit targets leaf-level application code, not library
   internals.

5. **`pure` keeps its current signature.**
   `pure` takes only a value, not a container, so there is no `FA` to
   resolve `DefaultBrand` from. Return-type inference is unreliable and
   no better than turbofish. `pure` is not renamed.

6. **Incremental scope for the `_explicit` rename.**
   The `_explicit` rename happens per-function as inference support is
   added. Functions that have not yet received inference support keep
   their current names unchanged. This avoids a big-bang rename and
   lets each function be validated independently. The core hierarchy
   (`map`, `bind`, `apply`, `lift2`) is first. Other functions
   (`fold_map`, `fold_right`, `traverse`, `filter_map`, `compact`,
   `separate`, `extend`, `extract`, `contramap`, etc.) are extended
   incrementally based on demand.

7. **Parallel functions (`par_map`, etc.) are excluded initially.**
   Parallel operations are niche and used in performance-conscious
   code where explicitness is valued. They keep their current explicit-
   brand signatures. Add inference later if users request it.

8. **The POC function name `map_infer` is temporary.**
   It exists only in the POC step to validate inference before
   committing to the rename. It will not exist in the final API.

## Implementation Order

1. **POC: Verify the GAT equality bound works.** Write a minimal test
   with `DefaultBrand` and a `map_infer` function to confirm type inference
   works with the real `Kind` machinery, `FunctorDispatch`, and both Val/Ref
   markers. Test with `Option`, `Vec`, and `Lazy`. If this fails, the entire
   plan needs rethinking.

2. **Define `DefaultBrand` trait** in `fp-library/src/classes/default_brand.rs`.
   Simple trait with one associated type. Add module to `classes.rs`.

3. **Implement `DefaultBrand` for core types.** Start with `Option`, `Vec`,
   `Identity`, `Thunk`, `SendThunk`, `CatList`, `Tuple1Brand`.

4. **Implement `DefaultBrand` for parameterized types.** `Lazy`, `TryLazy`,
   `Coyoneda`, `RcCoyoneda`, `ArcCoyoneda`, `BoxedCoyonedaExplicit`, `Const`.

5. **Add inference-based `map` and rename the current `map` to `map_explicit`.**
   The POC's temporary `map_infer` name is dropped; the inference-based
   function takes the clean `map` name. Verify that type inference works
   for `Option`, `Vec`, `Identity`, and `Lazy` with both Val and Ref
   dispatch markers. Update all internal call sites that use single-brand
   types to drop the turbofish.

6. **Extend to `bind`, `lift2`, `apply`.** Rename current versions to
   `_explicit` suffix, add inference-based versions following the same pattern.

7. **Add `#[no_default_brand]` support to `impl_kind!`.** Extend the proc
   macro to generate `DefaultBrand` impls by default. Add opt-out attribute.
   Migrate hand-written impls to use the macro.

8. **Extend `m_do!` and `a_do!`.** Add brand-free syntax that generates
   inference-based function calls. The Brand-explicit syntax remains
   available.

9. **Documentation.** Update crate docs, README examples, and
   `fp-library/docs/features.md` to show the inference-based calling
   convention as the primary API, with `_explicit` suffixed functions as the
   escape hatch for ambiguous types.

10. **Tests.** Unit tests for each `DefaultBrand` impl. Compile-fail tests
    confirming that ambiguous types (Result, Tuple2, etc.) do not compile
    with the inference-based functions. Doc tests showing both calling
    conventions.

## References

- `haskell_bits` crate: Demonstrates the `WithTypeArg`/`TypeApp`/`TypeAppParam`
  system for full brand inference in Rust HKT encoding. Each type constructor
  has a module-local `TypeCon` that uniquely identifies it, enabling the
  compiler to infer the type constructor from the concrete type. Avoids the
  multi-brand ambiguity problem by not supporting partial application of
  bifunctors.
- Current `FunctorDispatch` system: `fp-library/src/classes/functor_dispatch.rs`.
  Shows marker-type dispatch for Val/Ref; brand inference composes with this.
- `Kind` trait definitions: `fp-library/src/kinds.rs`.
- Brand definitions: `fp-library/src/brands.rs`.
- `impl_kind!` macro: `fp-macros/src/hkt/impl_kind.rs`.
- `Apply!` macro: `fp-macros/src/hkt/apply.rs`.
- Ref-hierarchy plan: `docs/plans/ref-hierarchy/plan.md`.
  The `FunctorDispatch` proof of concept that this plan builds on.
