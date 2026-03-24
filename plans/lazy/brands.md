# Lazy Hierarchy: Brand Definitions Analysis

## Overview

The lazy evaluation hierarchy defines brands for types that support HKT (higher-kinded type) traits via the Brand pattern. Types requiring `'static` (Trampoline, TryTrampoline, Free) are excluded from the brand system because the Kind traits require lifetime polymorphism (`type Of<'a, A: 'a>: 'a`).

## Brand Inventory

### Brands That Exist

| Brand | Type It Represents | Kind Signature | Config/Params |
|---|---|---|---|
| `ThunkBrand` | `Thunk<'a, A>` | `type Of<'a, A: 'a>: 'a` | None |
| `LazyBrand<Config>` | `Lazy<'a, A, Config>` | `type Of<'a, A: 'a>: 'a` | `Config: LazyConfig` |
| `TryThunkBrand` | `TryThunk<'a, A, E>` (bifunctor) | `type Of<'a, E: 'a, A: 'a>: 'a` | None |
| `TryThunkErrAppliedBrand<E>` | `TryThunk<'a, A, E>` (functor over Ok) | `type Of<'a, A: 'a>: 'a` | `E: 'static` |
| `TryThunkOkAppliedBrand<A>` | `TryThunk<'a, A, E>` (functor over Err) | `type Of<'a, E: 'a>: 'a` | `A: 'static` |
| `TryLazyBrand<E, Config>` | `TryLazy<'a, A, E, Config>` | `type Of<'a, A: 'a>: 'a` | `E: 'static`, `Config: LazyConfig` |

### Types Without Brands (and Why)

| Type | Reason |
|---|---|
| `Trampoline<A>` | Requires `A: 'static`; the Kind trait `type Of<'a, A: 'a>: 'a` demands lifetime polymorphism that `Trampoline` cannot satisfy. |
| `TryTrampoline<A, E>` | Same as Trampoline; wraps `Trampoline<Result<A, E>>` with `A: 'static, E: 'static`. |
| `Free<F, A>` | Requires `F: 'static, A: 'static` due to `Box<dyn Any>` type erasure. Cannot satisfy lifetime-polymorphic Kind signatures. |

---

## 1. Design Analysis

### LazyBrand<Config> and the Config System

The `LazyConfig` trait is well-designed as a strategy pattern that bundles:
- Pointer type (`Rc` vs `Arc`) via the `PointerBrand` associated type.
- Lazy cell type (`LazyCell` vs `LazyLock`).
- Thunk type (`dyn FnOnce() -> A + 'a` vs `dyn FnOnce() -> A + Send + 'a`).

The two concrete configs are:
- `RcLazyConfig`: single-threaded, uses `Rc<LazyCell<...>>`.
- `ArcLazyConfig`: thread-safe, uses `Arc<LazyLock<...>>`, requires `Send` on closures.

**Strengths:**
- The Config parameterization avoids duplicating the entire Lazy type and its methods for Rc vs Arc variants. The type aliases `RcLazy<'a, A>` and `ArcLazy<'a, A>` provide ergonomic access.
- The `PointerBrand` associated type on `LazyConfig` links the config back to the pointer hierarchy, enabling generic code to recover the underlying pointer brand without hard-coding.
- The trait is explicitly documented as open for third-party implementations (e.g., `parking_lot`-based locks, async-aware cells).

**Weaknesses:**
- The Config system introduces a type parameter that propagates through `LazyBrand<Config>` and `TryLazyBrand<E, Config>`, making the brand more complex than simple brands like `ThunkBrand` or `OptionBrand`.
- Trait implementations for `LazyBrand` cannot always be generic over `Config`. In practice, `RefFunctor` is implemented only for `LazyBrand<RcLazyConfig>` and `SendRefFunctor` only for `LazyBrand<ArcLazyConfig>`. Similarly, `Foldable` is implemented separately for each config. This means the "one brand, parameterized by config" design partially breaks down at the trait level, requiring concrete specialization.
- The `impl_kind!` invocation is generic (`impl<Config: LazyConfig> for LazyBrand<Config>`), which is correct and clean. But the trait impls that follow cannot leverage this generality.

### ThunkBrand

Simple and clean. No parameterization needed because `Thunk` has no pointer/config variation. Maps directly to `Thunk<'a, A>` with the standard `type Of<'a, A: 'a>: 'a` signature.

### TryThunkBrand and Its Partial Applications

This follows the same pattern as `ResultBrand` with its `ResultErrAppliedBrand<E>` and `ResultOkAppliedBrand<T>`:

- `TryThunkBrand`: fully unapplied bifunctor brand, `type Of<'a, E: 'a, A: 'a>: 'a = TryThunk<'a, A, E>`. Note the parameter swap: the Kind's `Of` takes `(E, A)` but the concrete type is `TryThunk<'a, A, E>`. This matches the convention from `ResultBrand` where the "last parameter is success."
- `TryThunkErrAppliedBrand<E>`: fixes the error type, creating a functor over the success type. Used for `Functor`, `Monad`, etc.
- `TryThunkOkAppliedBrand<A>`: fixes the success type, creating a functor over the error type.

The `E: 'static` and `A: 'static` bounds on the partially-applied brands deserve scrutiny. These are required because the brand type parameters must be `'static` (brand types themselves are marker types with `PhantomData`, but Rust requires the parameters to outlive the brand). This means you cannot have a `TryThunkErrAppliedBrand<&'a str>`, only `TryThunkErrAppliedBrand<String>` or similar owned types. This is a real limitation but consistent with how `ResultErrAppliedBrand<E>` works elsewhere in the codebase.

### TryLazyBrand<E, Config>

Combines the error-type partial application from `TryThunkErrAppliedBrand` with the Config parameterization from `LazyBrand`. This means `TryLazyBrand` carries two type parameters, making it the most complex brand in the lazy hierarchy. However, no trait implementations exist for this brand beyond the Kind mapping itself. It appears to exist primarily for type-level completeness rather than active use.

### Why Some Types Have Brands and Others Don't

The dividing line is clear and principled: brands exist only for types that can implement the Kind traits, which require lifetime polymorphism (`'a`). Types built on `Free` (which uses `Box<dyn Any>` requiring `'static`) cannot satisfy this. This is explicitly documented in the `ThunkBrand` doc comment and the `Free` module documentation.

---

## 2. Implementation Correctness

### Kind Implementations

All `impl_kind!` invocations are correct:

- `ThunkBrand`: `type Of<'a, A: 'a>: 'a = Thunk<'a, A>` -- straightforward, correct.
- `LazyBrand<Config>`: `type Of<'a, A: 'a>: 'a = Lazy<'a, A, Config>` -- correct, generic over Config.
- `TryThunkBrand`: `type Of<'a, E: 'a, A: 'a>: 'a = TryThunk<'a, A, E>` -- correct parameter ordering (E, A in Kind maps to A, E in concrete type).
- `TryThunkErrAppliedBrand<E>`: `type Of<'a, A: 'a>: 'a = TryThunk<'a, A, E>` with `E: 'static` -- correct.
- `TryThunkOkAppliedBrand<A>`: `type Of<'a, E: 'a>: 'a = TryThunk<'a, A, E>` with `A: 'static` -- correct.
- `TryLazyBrand<E, Config>`: `type Of<'a, A: 'a>: 'a = TryLazy<'a, A, E, Config>` with `E: 'static, Config: LazyConfig` -- correct.

### Type Mapping Issues

No issues found. The `Lazy` struct correctly uses `Config::Lazy<'a, A>` as its inner representation, and the Kind mapping preserves the Config parameter. The `TryLazy` struct similarly uses `Config::TryLazy<'a, A, E>`.

The `TryThunk` struct wraps `Thunk<'a, Result<A, E>>`, and the bifunctor Kind correctly maps `Of<'a, E, A>` to `TryThunk<'a, A, E>` (swapping parameter order to put the "success" type last in the Kind, matching Haskell's `Either e a` convention).

---

## 3. Consistency Analysis

### Consistency Within the Lazy Hierarchy

The lazy hierarchy is internally consistent:

- **Infallible types** (`Thunk`, `Lazy`): one brand each, parameterized by Config where applicable.
- **Fallible types** (`TryThunk`, `TryLazy`): follow the bifunctor partial-application pattern. `TryThunk` has three brands (full bifunctor + two partial applications); `TryLazy` has one brand (only the error-applied form, since it only acts as a functor over the success type).

**Missing: `TryLazyBrand` has no bifunctor brand or ok-applied brand.** Unlike `TryThunk`, which has `TryThunkBrand` (bifunctor), `TryThunkErrAppliedBrand`, and `TryThunkOkAppliedBrand`, `TryLazy` only has `TryLazyBrand<E, Config>` (error-applied). This could be intentional since `TryLazy` has no trait impls beyond the Kind mapping, or it could be an incompleteness.

### Consistency With Other Brands in the File

The lazy brands follow the same patterns established by other brands:

| Pattern | Examples |
|---|---|
| Simple zero-param brand | `ThunkBrand`, `OptionBrand`, `VecBrand` |
| Config-parameterized brand | `LazyBrand<Config>`, `FnBrand<PtrBrand>` |
| Bifunctor brand | `TryThunkBrand`, `ResultBrand`, `StepBrand` |
| Partial application (error fixed) | `TryThunkErrAppliedBrand<E>`, `ResultErrAppliedBrand<E>` |
| Partial application (ok fixed) | `TryThunkOkAppliedBrand<A>`, `ResultOkAppliedBrand<T>` |

All brands derive the standard set: `Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash`. Type aliases (like `RcFnBrand`, `ArcFnBrand`) are not used for the lazy brands, though `RcLazy`/`ArcLazy` aliases exist at the type level.

---

## 4. Limitations

### The `'static` Bound on Partial Application Parameters

`TryThunkErrAppliedBrand<E>` requires `E: 'static`, and `TryThunkOkAppliedBrand<A>` requires `A: 'static`. This prevents using borrowed types as the fixed parameter. The same limitation affects `TryLazyBrand<E, Config>`. This is a fundamental constraint of the brand system since brand types live at the type level and must be constructible without lifetime dependencies.

### No Brands for Stack-Safe Types

`Trampoline`, `TryTrampoline`, and `Free` cannot have brands. This means stack-safe computation types are excluded from generic HKT code. You cannot write `map::<TrampolineBrand, _, _>(f, trampoline)`. Instead, these types provide direct method-based APIs. This is a significant gap: the most powerful computation types in the hierarchy are the ones that cannot participate in generic type class programming.

### LazyBrand Trait Implementations Require Concrete Config

Despite `impl_kind!` being generic over `Config: LazyConfig`, actual trait impls (`RefFunctor`, `SendRefFunctor`, `Foldable`) must be written separately for `LazyBrand<RcLazyConfig>` and `LazyBrand<ArcLazyConfig>`. This is because:
- `RefFunctor` requires non-`Send` semantics (compatible with `Rc`).
- `SendRefFunctor` requires `Send` semantics (compatible with `Arc`).
- Rust cannot express "if Config uses Rc, impl RefFunctor; if Config uses Arc, impl SendRefFunctor" in a single generic impl.

This means adding a new `LazyConfig` implementation also requires manually adding all the trait impls for `LazyBrand<NewConfig>`.

### TryLazy Has Minimal HKT Support

`TryLazyBrand<E, Config>` has a Kind mapping but no trait implementations beyond it. No `RefFunctor`, `Foldable`, or any other type class is implemented for this brand. This makes the brand effectively dormant; it exists for type-level purposes but provides no generic operations.

### No Lazy-Specific Bifunctor Brand

`TryLazy` lacks a full bifunctor brand (analogous to `TryThunkBrand` for `TryThunk`). If someone wanted to bimap over both the success and error types of a `TryLazy`, there is no brand to support this generically.

---

## 5. Alternative Approaches

### Could Trampoline/Free Have Brands?

Theoretically, if the Kind system supported a `'static`-only signature like `type Of<A> = Trampoline<A>` (no lifetime parameter), then `Trampoline` could have a brand. The codebase does define `Kind_ad6c20556a82a1f0` with signature `type Of<A>` (no lifetime). However, most type class traits (`Functor`, `Monad`, etc.) are defined against the lifetime-polymorphic Kind (`type Of<'a, A: 'a>: 'a`), so even with a brand, Trampoline could not implement those traits without a separate set of `'static`-only type classes. This would be a large architectural change with unclear benefits.

### Could the Config System Be Simplified?

One alternative would be to drop the `Config` parameter from `LazyBrand` entirely and have separate `RcLazyBrand` and `ArcLazyBrand` types. Since trait impls already have to be written separately per config, the generic `LazyBrand<Config>` provides less value than it might seem. The main benefit of the current approach is that the `impl_kind!` only needs to be written once, and the Lazy type definition itself is unified. Splitting into separate brands would mean duplicating some definitions but would simplify the brand signatures.

However, the current design is better for forward compatibility: if Rust eventually supports specialization or conditional impls, the generic `LazyBrand<Config>` would allow writing a single generic impl for each trait.

### Could TryLazy Share Infrastructure With TryThunk?

`TryLazy` and `TryThunk` have parallel structures (fallible versions of `Lazy` and `Thunk`). The brand patterns could be more unified. For instance, a generic "try" wrapper brand `TryBrand<InnerBrand, E>` could theoretically represent "the fallible version of any base brand." However, this would require higher-order brand parameterization (brands parameterized by brands), which would significantly complicate the type system.

### Type-Level Aliases for Common Brand Combinations

The codebase uses `type RcFnBrand = FnBrand<RcBrand>` and `type ArcFnBrand = FnBrand<ArcBrand>` for function brands. Similar aliases could be added for lazy brands:

```rust
pub type RcLazyBrand = LazyBrand<RcLazyConfig>;
pub type ArcLazyBrand = LazyBrand<ArcLazyConfig>;
pub type RcTryLazyBrand<E> = TryLazyBrand<E, RcLazyConfig>;
pub type ArcTryLazyBrand<E> = TryLazyBrand<E, ArcLazyConfig>;
```

These would improve ergonomics without changing semantics. Their absence is a minor inconsistency with the `FnBrand` pattern.

---

## Summary

The brand definitions for the lazy hierarchy are well-structured and correct. The Config parameterization for `LazyBrand` is a sound design choice that unifies the Rc/Arc split at the type level, even though trait impls must be specialized. The bifunctor pattern for `TryThunk` brands is consistent with `Result` brands. The main limitations are inherent to the brand system's reliance on lifetime polymorphism (excluding `'static`-only types) and Rust's lack of specialization (requiring duplicate trait impls per config). The most actionable improvement would be adding type aliases for common brand+config combinations (`RcLazyBrand`, `ArcLazyBrand`).
