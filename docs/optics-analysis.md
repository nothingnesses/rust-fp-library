# Optics Implementation Analysis: `fp-library` vs PureScript `purescript-profunctor-lenses`

This document provides a comprehensive analysis comparing the optics implementation in the Rust `fp-library` with its inspiration, the PureScript `purescript-profunctor-lenses` library (specifically `Data.Lens.Types`).

## 1. High-Level Architectural Mapping

The `fp-library` is a remarkably high-fidelity port of `purescript-profunctor-lenses`. Both libraries rely on the **Profunctor Optic Encoding**, where an optic transforms a profunctor over the focus `P a b` to a profunctor over the larger structure `P s t`.

*   **PureScript:** `type Optic p s t a b = p a b -> p s t`
*   **Rust (`fp-library`):** Modeled using the `Optic<'a, P, S, T, A, B>` trait.

### Core Traits (Type Classes)

The constraints required for each family of optics map one-to-one between the languages:

| Optic Family | PureScript Constraint | Rust (`fp-library`) Constraint |
| :--- | :--- | :--- |
| **Iso** | `Profunctor` | `Profunctor` |
| **Lens** | `Strong` | `Strong` |
| **Prism** | `Choice` | `Choice` |
| **Traversal** | `Wander` | `Wander` |
| **Grate** | `Closed` | `Closed` |

### Core Optics Supported

`fp-library` implements all the primary optics families: `Lens`, `Prism`, `Iso`, `AffineTraversal`, `Getter`, `Setter`, `Fold`, `Review`, and `Grate`.

Furthermore, the concrete backing profunctors used to avoid impredicativity in PureScript/GHC—such as `Shop`, `Market`, `Exchange`, `Stall`, `Grating`, `Forget`, and `Tagged`—are all fully implemented in Rust.

---

## 2. Structural Differences (Rust Idiosyncrasies)

Because Rust handles polymorphism, closures, and type dispatch differently than PureScript, several structural deviations exist by design.

### Monomorphization via Traits

PureScript optics are literally just functions, utilizing rank-N polymorphism (`forall p. Constraint p => ...`).
Rust cannot naturally pass functions that are polymorphic over a trait without `Box<dyn ...>` or monomorphization. To maintain static dispatch and zero-cost abstraction, `fp-library` uses a dedicated `Optic` trait with an `evaluate` method. Concrete optics are implemented as distinct structs (e.g., `Lens`, `Prism`).

### Monomorphic Variants (Prime Types)

PureScript uses type aliases for monomorphic variants that don't change the focus type:
```purescript
type Lens' s a = Lens s s a a
```
Rust instead exposes dedicated `...Prime` structs, for example, `LensPrime<'a, P, S, A>` and `PrismPrime<'a, P, S, A>`.

### Composition Strategy

Because optics are functions in PureScript, they are composed natively using standard function composition: `f <<< g`.
In `fp-library`, because optics are distinct structs implementing a trait, they are composed using a specialized `Composed` struct and the `optics_compose` helper function: `first.evaluate(second.evaluate(pab))`.

### Lifetimes and Pointer Brands

PureScript is garbage-collected. Rust relies on static lifetime analysis and ownership constraints.
`fp-library` optic structs carry `'a` lifetimes to track closure validity, and utilize "pointer brands" (like `FnBrand` or `RcBrand`) to handle closure capturing and heap allocation explicitly.

---

## 3. Missing Functionality

While comprehensive, `fp-library` is missing a few features found in `purescript-profunctor-lenses`:

1.  **Indexed Optics:** PureScript features a robust suite of indexed optics (`IndexedOptic`, `IndexedLens`, `IndexedTraversal`, `IndexedFold`, etc.), which allow access to elements alongside their structural indices (like array indices or map keys). There is **no `Indexed` profunctor or indexed optics** implementation in `fp-library`.
2.  **`Bazaar` and `ATraversal`:** `fp-library` completely lacks the `Bazaar` profunctor. Consequently, the `ATraversal` optic is missing.
3.  **Optic Reversing (`Re`):** PureScript includes a `Re` profunctor, which allows reversing optics (e.g., turning a `Getter` into a `Review`). This struct is absent in `fp-library`.
4.  **"A/An" Optic Type Aliases:** PureScript includes explicit aliases like `ALens`, `APrism`, `AnIso` to combat GHC's impredicativity limitations. `fp-library` defines the *backing profunctors* (`Shop`, `Market`, `Exchange`), but does not expose these explicit optic type aliases, as Rust circumvents impredicativity by relying on traits and monomorphization, making the aliases unnecessary wrappers.

---

## 4. Discovered Flaws and Mistakes

A comprehensive review of the codebase revealed a **critical runtime panic flaw** regarding the combination of `Fold` and `Traversal` optics.

### Panic in the `Forget` Profunctor Implementation

In [`fp-library/src/types/optics/forget.rs`](../fp-library/src/types/optics/forget.rs), the `Forget` profunctor (which powers Folds and Getters) wraps a closure as `Box<dyn Fn(A) -> R + 'a>`. Because trait objects like `Box<dyn Fn>` cannot be natively cloned, the author implemented the `Clone` trait with a hardcoded panic:

```rust
impl<'a, R, A, B> Clone for Forget<'a, R, A, B>
where
	R: 'a,
	A: 'a,
{
	fn clone(&self) -> Self {
		panic!("Forget cannot be cloned directly. Use a pointer-wrapped version if cloning is needed.")
	}
}
```

However, inside the exact same file, the `Wander` trait (which is required by `Traversal` and `AffineTraversal` to process multi-focus data structures) is implemented for `ForgetBrand<R>` using the following closure structure:

```rust
impl<R: 'static + Monoid> Wander for ForgetBrand<R> {
	fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a, TFunc>(
		traversal: TFunc,
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	// ...
	{
		use crate::types::const_val::ConstBrand;
		Forget::new(move |s| {
			let pab = pab.clone(); // <--- CRITICAL: PANICS HERE AT RUNTIME
			// ...
		})
	}
}
```

### Impact

Any code attempting to evaluate a **`Fold` or `Getter` over a `Traversal` or `AffineTraversal`** will compile successfully, but **panic at runtime**. When the optic evaluates, the internal closure generated by `Wander` attempts to clone the un-cloneable `Forget` profunctor.

*Note: The author acknowledged in a comment that `Forget` might need to be refactored to use a pointer brand (e.g., `FnBrand`, like the other optic types) to resolve the cloning issue, but the broken `Box<dyn Fn>` implementation was left in place.*