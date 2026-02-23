//! Optics for composable data accessors using profunctor encoding.
//!
//! This module provides a trait-based profunctor optic implementation that is a high-fidelity
//! port of PureScript's `purescript-profunctor-lenses`. It allows composing lenses, prisms,
//! and other optics while maintaining type safety and zero-cost abstractions through monomorphization.
//!
//! ### Comparison with PureScript
//!
//! The implementation mirrors the PureScript `Optic` definition closely:
//!
//! | Feature | PureScript | Rust (`fp-library`) |
//! | :--- | :--- | :--- |
//! | **Optic Definition** | `p a b -> p s t` | `trait Optic<'a, P, S, T, A, B>` |
//! | **Lens** | `Strong p => Optic p s t a b` | `struct Lens<'a, P, S, T, A, B>` |
//! | **Lens'** | `Lens s s a a` | `struct LensPrime<'a, P, S, A>` |
//! | **Prism** | `Choice p => Optic p s t a b` | `struct Prism<'a, P, S, T, A, B>` |
//! | **Prism'** | `Prism s s a a` | `struct PrismPrime<'a, P, S, A>` |
//! | **Iso** | `Profunctor p => Optic p s t a b` | `struct Iso<'a, P, S, T, A, B>` |
//! | **Iso'** | `Iso s s a a` | `struct IsoPrime<'a, P, S, A>` |
//! | **AffineTraversal** | `Strong p => Choice p => Optic p s t a b` | `struct AffineTraversal<'a, P, S, T, A, B>` |
//! | **Composition** | `Semigroupoid` / `<<<` | `struct Composed` / `optics_compose` |
//!
//! While PureScript uses the `Semigroupoid` instance of functions for composition,
//! this library uses a specialized `Composed` struct. This allows Rust to perform
//! zero-cost composition through monomorphization while preserving the `Optic` trait
//! boundaries without needing the rank-2 polymorphism that PureScript relies on.
//!
//! ### Module Organization
//!
//! This module is organized into submodules for different optic types:
//!
//! - [`base`] - The core [`Optic`] trait and [`Composed`] type
//! - [`lens`] - [`Lens`] and [`LensPrime`] for product types
//! - [`prism`] - [`Prism`] and [`PrismPrime`] for sum types
//! - [`iso`] - [`Iso`] and [`IsoPrime`] for isomorphisms
//! - [`affine`] - [`AffineTraversal`] and [`AffineTraversalPrime`] for optional focusing
//! - [`helpers`] - Helper functions like [`optics_view`], [`optics_set`], [`optics_over`], [`optics_preview`], [`optics_review`]
//!
//! ### Lifetime Support
//!
//! The optics hierarchy has been updated to include a lifetime parameter `'a`. This allows
//! optics to work with non-static types (e.g., types containing references like `&str`) by
//! ensuring that the captured functions and the types they operate on are valid for the
//! same lifetime.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::optics::*,
//! };
//!
//! // Define a simple struct
//! #[derive(Clone, Debug, PartialEq)]
//! struct Person {
//! 	name: String,
//! 	age: i32,
//! }
//!
//! // Create a lens for the age field
//! let age_lens: LensPrime<RcBrand, Person, i32> = LensPrime::new(
//! 	|p: Person| p.age,
//! 	|(p, age)| Person {
//! 		age,
//! 		..p
//! 	},
//! );
//!
//! let person = Person {
//! 	name: "Alice".to_string(),
//! 	age: 30,
//! };
//! let age = age_lens.view(person.clone());
//! assert_eq!(age, 30);
//!
//! let updated = age_lens.set(person.clone(), 31);
//! assert_eq!(updated.age, 31);
//! ```

mod affine;
mod base;
mod exchange;
mod fold;
mod forget;
mod getter;
mod grate;
mod grating;
mod helpers;
mod iso;
mod lens;
mod market;
mod prism;
mod review;
mod setter;
mod shop;
mod stall;
mod tagged;
mod traversal;

pub use {
	affine::*,
	base::*,
	exchange::*,
	fold::*,
	forget::*,
	getter::*,
	grate::*,
	grating::*,
	helpers::*,
	iso::*,
	lens::*,
	market::*,
	prism::*,
	review::*,
	setter::*,
	shop::*,
	stall::*,
	tagged::*,
	traversal::*,
};
