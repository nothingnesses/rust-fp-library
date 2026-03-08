# Profunctor Classes: Rust vs PureScript Analysis

This document compares the profunctor class hierarchy in `fp-library` against the PureScript reference implementations in `purescript-profunctor` and `purescript-profunctor-lenses`, and analyses naming consistency within the Rust codebase.

---

## 1. Class Hierarchy Comparison

| PureScript | Rust | Superclass | Status |
|------------|------|------------|--------|
| `Profunctor p` | `Profunctor` | — | Complete |
| `Strong p` | `Strong` | `Profunctor` | Complete |
| `Choice p` | `Choice` | `Profunctor` | Complete |
| `Closed p` | `Closed<FunctionBrand: CloneableFn>` | `Profunctor` | Complete (parameterized) |
| `Cochoice p` | `Cochoice` | `Profunctor` | Complete |
| `Costrong p` | `Costrong` | `Profunctor` | Complete |
| `Wander p` | `Wander` | `Strong + Choice` | Complete |

The class hierarchy is faithfully reproduced. All superclass relationships match.

---

## 2. Method Comparison

### 2.1 Profunctor

| PureScript | Rust | Notes |
|------------|------|-------|
| `dimap :: (a -> b) -> (c -> d) -> p b c -> p a d` | `dimap<A, B, C, D, FuncAB, FuncCD>(ab, cd, pbc)` | Identical semantics |
| — | `lmap<A, B, C, FuncAB>(ab, pbc)` (default impl) | Trait method in Rust |
| — | `rmap<A, B, C, FuncBC>(bc, pab)` (default impl) | Trait method in Rust |

PureScript defines only `dimap` as a class method. `lcmap` (PureScript's equivalent of `lmap`) and `rmap` are standalone free functions. Rust promotes both to trait methods with default implementations derived from `dimap`. This is a reasonable Rust idiom — it allows implementors to override with more efficient versions.

### 2.2 Strong

| PureScript | Rust | Notes |
|------------|------|-------|
| `first :: p a b -> p (Tuple a c) (Tuple b c)` | `first<A, B, C>(pab)` | Identical. Uses native tuples. |
| `second :: p b c -> p (Tuple a b) (Tuple a c)` | `second<A, B, C>(pab)` | Identical |

### 2.3 Choice

| PureScript | Rust | Notes |
|------------|------|-------|
| `left :: p a b -> p (Either a c) (Either b c)` | `left<A, B, C>(pab) -> ...Result<C, A>, Result<C, B>` | `Either` → `Result` (see §3.1) |
| `right :: p b c -> p (Either a b) (Either a c)` | `right<A, B, C>(pab) -> ...Result<A, C>, Result<B, C>` | Same adaptation |

### 2.4 Closed

| PureScript | Rust | Notes |
|------------|------|-------|
| `closed :: p a b -> p (x -> a) (x -> b)` | `closed<A, B, X>(pab) -> ...FunctionBrand::Of<X, A>, FunctionBrand::Of<X, B>` | Parameterized over `FunctionBrand` (see §3.2) |

### 2.5 Cochoice

| PureScript | Rust | Notes |
|------------|------|-------|
| `unleft :: p (Either a c) (Either b c) -> p a b` | `unleft<A, B, C>(pab)` | Identical (with `Either` → `Result`) |
| `unright :: p (Either a b) (Either a c) -> p b c` | `unright<A, B, C>(pab)` | Identical |

### 2.6 Costrong

| PureScript | Rust | Notes |
|------------|------|-------|
| `unfirst :: p (Tuple a c) (Tuple b c) -> p a b` | `unfirst<A, B, C>(pab)` | Identical |
| `unsecond :: p (Tuple a b) (Tuple a c) -> p b c` | `unsecond<A, B, C>(pab)` | Identical |

### 2.7 Wander

| PureScript | Rust | Notes |
|------------|------|-------|
| `wander :: (forall f. Applicative f => (a -> f b) -> s -> f t) -> p a b -> p s t` | `wander<S, T, A, B, TFunc>(traversal, pab)` | Rank-2 type replaced by `TFunc: TraversalFunc` (see §3.3) |

---

## 3. Rust-Specific Adaptations

### 3.1 `Either` → `Result`

PureScript's `Either a b` maps to Rust's `Result<B, A>` with reversed parameter order:

| PureScript | Rust | Semantic role |
|------------|------|--------------|
| `Left a` | `Err(A)` | Active/focus variant |
| `Right c` | `Ok(C)` | Passthrough variant |

This means `Choice::left` acts on the `Err` variant and `Choice::right` acts on the `Ok` variant. The naming is faithful to PureScript's conventions but may be unintuitive to Rust users who associate `Ok` with the "primary" case.

### 3.2 `Closed` parameterization

PureScript's `Closed` uses bare function types `x -> a`. Rust cannot express this directly — closures must be wrapped in `Rc<dyn Fn>` or `Arc<dyn Fn>`. The Rust `Closed<FunctionBrand: CloneableFn>` trait takes an extra type parameter `FunctionBrand` to abstract over the function wrapping strategy.

### 3.3 `fan_out` requires `A: Clone`

PureScript's `fanout` duplicates the input with `\a -> Tuple a a`. Rust's move semantics require `A: Clone` to achieve this. The same justification applies as for `Closed::closed`'s `X: Clone`.

### 3.4 `Wander` and rank-2 types

PureScript's `wander` uses a rank-2 type `(forall f. Applicative f => ...)`. Rust lacks rank-2 polymorphism, so this is encoded via the `TraversalFunc` trait, which provides a concrete `apply` method that the `Wander` implementation calls with specific applicative functors.

---

## 4. Free Function Comparison

### 4.1 Present in both

| PureScript | Rust | Notes |
|------------|------|-------|
| `dimap` | `dimap` | Identical |
| `lcmap` | `lmap` | **Name difference** (see §5.1) |
| `rmap` | `rmap` | Identical |
| `first` | `first` | Identical |
| `second` | `second` | Identical |
| `left` | `left` | Identical |
| `right` | `right` | Identical |
| `closed` | `closed` | Identical |
| `unleft` | `unleft` | Identical |
| `unright` | `unright` | Identical |
| `unfirst` | `unfirst` | Identical |
| `unsecond` | `unsecond` | Identical |
| `wander` | `wander` | Identical |
| `arr` | `arrow` | **Name difference**; free function with `Category + Profunctor` bounds |
| `splitStrong` (`***`) | `split_strong` | snake_case; `Semigroupoid + Strong` bounds |
| `fanout` (`&&&`) | `fan_out` | snake_case; `Semigroupoid + Strong` bounds; `A: Clone` (see §3.4) |
| `splitChoice` (`+++`) | `split_choice` | snake_case; `Semigroupoid + Choice` bounds |
| `fanin` (`\|\|\|`) | `fan_in` | snake_case; `Semigroupoid + Choice` bounds |

---

## 5. Naming Consistency

### 5.1 `lmap` vs PureScript's `lcmap`

PureScript renamed `lmap` to `lcmap` to avoid conflict with `Data.Functor.Contravariant.cmap`. The Rust library uses `lmap`, which matches the name used in Haskell's `profunctors` package and in the profunctor literature. This is a deliberate and reasonable choice.

### 5.2 Free function brand parameter naming

All profunctor free functions consistently use `Brand` for the profunctor type parameter:

```rust
pub fn dimap<'a, Brand: Profunctor, ...>(...) { ... }
pub fn first<'a, Brand: Strong, ...>(...) { ... }
pub fn closed<'a, Brand: Closed<FunctionBrand>, FunctionBrand: CloneableFn, ...>(...) { ... }
```

This is consistent within the profunctor module. However, the optics code uses `P` for profunctor parameters (e.g., `Optic::evaluate<P: Profunctor>`). This is a minor naming difference between the two modules — `Brand` in profunctor classes vs `P` in optic traits — but both are single-concept identifiers and the bounds disambiguate.

### 5.3 Type parameter conventions summary

| Concept | In profunctor classes | In optic traits/structs | In optic free functions |
|---------|----------------------|------------------------|------------------------|
| Profunctor brand | `Self` (trait) / `Brand` (free fn) | `P` (method-level) | `P` |
| Pointer brand | — | `PointerBrand` | `PointerBrand` |
| Function brand | `FunctionBrand` (`Closed` only) | `FunctionBrand` | `FunctionBrand` |
| Focus types | `A, B` | `A, B` | `A, B` |
| Structure types | — | `S, T` | `S, T` |
| Auxiliary types | `C` (passthrough), `X` (input) | `R` (result/monoid) | `R` |

---

## 6. Summary

### What matches PureScript

- All 7 profunctor classes present with correct superclass hierarchy
- All method names identical (except `lcmap` → `lmap`, which matches Haskell convention)
- Type parameter ordering within methods matches PureScript
- `lmap`/`rmap` default implementations match PureScript's free function definitions

### Rust-specific adaptations (justified)

- `Closed<FunctionBrand>` parameterization (Rust needs explicit function wrapping)
- `Result<C, A>` instead of `Either a c` (Rust lacks a standard `Either`)
- `TFunc: TraversalFunc` instead of rank-2 types in `Wander`
- `X: Clone` on `Closed::closed` (Rust needs explicit cloning in nested closures)
- `A: Clone` on `fan_out` (same justification — Rust needs explicit cloning to duplicate input)
- `arrow` is a free function with `Category + Profunctor` bounds (no `Arrow` trait)
- `lmap`/`rmap` as trait methods instead of free functions

### Naming inconsistencies to consider

1. **`Brand` in profunctor free functions vs `P` in optic traits** — different naming for profunctor type parameter (minor, both are clear from context)
