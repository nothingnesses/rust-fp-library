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
//! | **Iso** | `Profunctor p => Optic p s t a b` | `struct Iso<'a, PointerBrand, S, T, A, B>` |
//! | **Iso'** | `Iso s s a a` | `struct IsoPrime<'a, PointerBrand, S, A>` |
//! | **Lens** | `Strong p => Optic p s t a b` | `struct Lens<'a, PointerBrand, S, T, A, B>` |
//! | **Lens'** | `Lens s s a a` | `struct LensPrime<'a, PointerBrand, S, A>` |
//! | **Prism** | `Choice p => Optic p s t a b` | `struct Prism<'a, PointerBrand, S, T, A, B>` |
//! | **Prism'** | `Prism s s a a` | `struct PrismPrime<'a, PointerBrand, S, A>` |
//! | **AffineTraversal** | `Strong p => Choice p => Optic p s t a b` | `struct AffineTraversal<'a, PointerBrand, S, T, A, B>` |
//! | **AffineTraversal'** | `AffineTraversal s s a a` | `struct AffineTraversalPrime<'a, PointerBrand, S, A>` |
//! | **Traversal** | `Wander p => Optic p s t a b` | `struct Traversal<'a, PointerBrand, S, T, A, B, F>` |
//! | **Traversal'** | `Traversal s s a a` | `struct TraversalPrime<'a, PointerBrand, S, A, F>` |
//! | **Getter** | `forall r. Fold r s t a b` | `struct Getter<'a, PointerBrand, S, T, A, B>` |
//! | **Getter'** | `Getter s s a a` | `struct GetterPrime<'a, PointerBrand, S, A>` |
//! | **Setter** | `Optic Arrow s t a b` | `struct Setter<'a, PointerBrand, S, T, A, B>` |
//! | **Setter'** | `Setter s s a a` | `struct SetterPrime<'a, PointerBrand, S, A>` |
//! | **Fold** | `Optic (Forget r) s t a b` | `struct Fold<'a, PointerBrand, S, T, A, B, F>` |
//! | **Fold'** | `Fold r s s a a` | `struct FoldPrime<'a, PointerBrand, S, A, F>` |
//! | **Review** | `Optic Tagged s t a b` | `struct Review<'a, PointerBrand, S, T, A, B>` |
//! | **Review'** | `Review s s a a` | `struct ReviewPrime<'a, PointerBrand, S, A>` |
//! | **Grate** | `Closed p => Optic p s t a b` | `struct Grate<'a, PointerBrand, S, T, A, B>` |
//! | **Grate'** | `Grate s s a a` | `struct GratePrime<'a, PointerBrand, S, A>` |
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
//! - **Core:** The [`crate::classes::optics::Optic`] trait and [`Composed`] / [`optics_compose`]
//! - **Optic Types:**
//!   - [`Iso`] / [`IsoPrime`]: Isomorphisms
//!   - [`Lens`] / [`LensPrime`]: Product types (get/set a field)
//!   - [`Prism`] / [`PrismPrime`]: Sum types (match/construct a variant)
//!   - [`AffineTraversal`] / [`AffineTraversalPrime`]: Optional focusing (Lens + Prism)
//!   - [`Traversal`] / [`TraversalPrime`]: Multiple foci
//!   - [`Getter`] / [`GetterPrime`]: Read-only access
//!   - [`Setter`] / [`SetterPrime`]: Write-only modification
//!   - [`Fold`] / [`FoldPrime`]: Collecting multiple values (read-only)
//!   - [`Review`] / [`ReviewPrime`]: Constructing values
//!   - [`Grate`] / [`GratePrime`]: Closed/zipping optics
//!   - [`ReversedOptic`]: Reversed/inverted optic
//! - **Indexed Optics:**
//!   - [`IndexedLens`] / [`IndexedLensPrime`]
//!   - [`IndexedTraversal`] / [`IndexedTraversalPrime`]
//!   - [`IndexedGetter`] / [`IndexedGetterPrime`]
//!   - [`IndexedFold`] / [`IndexedFoldPrime`]
//!   - [`IndexedSetter`] / [`IndexedSetterPrime`]
//! - **Internal Profunctors:** [`Exchange`], [`Shop`], [`Market`], [`Stall`], [`Forget`], [`Tagged`], [`Grating`], [`Zipping`], [`Bazaar`], [`Indexed`], [`Reverse`]
//! - **Helper Functions:**
//!   - Lens: [`optics_view`], [`optics_set`], [`optics_over`]
//!   - Prism/Fold: [`optics_preview`], [`optics_review`]
//!   - Iso: [`optics_from`], [`optics_to`]
//!   - Grate: [`zip_with_of`]
//!   - Indexed: [`optics_indexed_view`], [`optics_indexed_over`], [`optics_indexed_set`], [`optics_indexed_preview`], [`optics_indexed_fold_map`]
//!   - Reindexing: [`optics_un_index`], [`optics_as_index`], [`optics_reindexed`]
//!   - Other: [`optics_eval`], [`positions`], [`reverse`]
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
//! 	brands::{
//! 		optics::*,
//! 		*,
//! 	},
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
//! let age_lens: LensPrime<RcBrand, Person, i32> = LensPrime::from_view_set(
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

// Smart-pointer function types (Rc<dyn Fn>/Arc<dyn Fn>) require wrapping closures
// for deref-and-call; profunctor-encoded optics inherently produce deeply nested types.
#![allow(clippy::redundant_closure, clippy::type_complexity)]

mod affine;
mod bazaar;
mod composed;
mod exchange;
mod fold;
mod forget;
mod functions;
mod getter;
mod grate;
mod grating;
mod indexed;
mod indexed_fold;
mod indexed_getter;
mod indexed_lens;
mod indexed_setter;
mod indexed_traversal;
mod iso;
mod lens;
mod market;
mod prism;
mod reverse;
mod review;
mod setter;
mod shop;
mod stall;
mod tagged;
mod traversal;
mod zipping;

pub use {
	affine::*,
	bazaar::*,
	composed::*,
	exchange::*,
	fold::*,
	forget::*,
	functions::*,
	getter::*,
	grate::*,
	grating::*,
	indexed::*,
	indexed_fold::*,
	indexed_getter::*,
	indexed_lens::*,
	indexed_setter::*,
	indexed_traversal::*,
	iso::*,
	lens::*,
	market::*,
	prism::*,
	reverse::*,
	review::*,
	setter::*,
	shop::*,
	stall::*,
	tagged::*,
	traversal::*,
	zipping::*,
};
