//! Semigroupoid type class.
//!
//! This module defines the [`Semigroupoid`] trait, which represents a set of objects and composable relationships between them.

use crate::{Apply, kinds::*};

/// A type class for semigroupoids.
///
/// A `Semigroupoid` is a set of objects and composable relationships
/// (morphisms) between them.
///
/// ### Laws
///
/// Semigroupoid instances must satisfy the associative law:
/// * Associativity: `compose(p, compose(q, r)) = compose(compose(p, q), r)`.
pub trait Semigroupoid: Kind_fcf9d56b89a0b8b9 {
	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// This method composes two morphisms `f` and `g` to produce a new morphism that represents the application of `g` followed by `f`.
	///
	/// ### Type Signature
	///
	/// `forall b c d. Semigroupoid a => (a c d, a b c) -> a b d`
	///
	/// ### Type Parameters
	///
	/// * `B`: The source type of the first morphism.
	/// * `C`: The target type of the first morphism and the source type of the second morphism.
	/// * `D`: The target type of the second morphism.
	///
	/// ### Parameters
	///
	/// * `f`: The second morphism to apply (from C to D).
	/// * `g`: The first morphism to apply (from B to C).
	///
	/// ### Returns
	///
	/// The composed morphism (from B to D).
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::semigroupoid::Semigroupoid;
	/// use fp_library::brands::RcFnBrand;
	/// use fp_library::classes::clonable_fn::ClonableFn;
	///
	/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
	/// let g = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
	/// let h = RcFnBrand::compose(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	fn compose<'a, B: 'a, C: 'a, D: 'a>(
		f: Apply!(
			brand: Self,
			signature: ('a, C, D),
		),
		g: Apply!(
			brand: Self,
			signature: ('a, B, C),
		),
	) -> Apply!(
		brand: Self,
		signature: ('a, B, D),
	);
}

/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
///
/// Free function version that dispatches to [the type class' associated function][`Semigroupoid::compose`].
///
/// ### Type Signature
///
/// `forall b c d. Semigroupoid a => (a c d, a b c) -> a b d`
///
/// ### Type Parameters
///
/// * `Brand`: The brand of the semigroupoid.
/// * `B`: The source type of the first morphism.
/// * `C`: The target type of the first morphism and the source type of the second morphism.
/// * `D`: The target type of the second morphism.
///
/// ### Parameters
///
/// * `f`: The second morphism to apply (from C to D).
/// * `g`: The first morphism to apply (from B to C).
///
/// ### Returns
///
/// The composed morphism (from B to D).
///
/// ### Examples
///
/// ```
/// use fp_library::classes::semigroupoid::compose;
/// use fp_library::brands::RcFnBrand;
/// use fp_library::classes::clonable_fn::ClonableFn;
///
/// let f = <RcFnBrand as ClonableFn>::new(|x: i32| x * 2);
/// let g = <RcFnBrand as ClonableFn>::new(|x: i32| x + 1);
/// let h = compose::<RcFnBrand, _, _, _>(f, g);
/// assert_eq!(h(5), 12); // (5 + 1) * 2
/// ```
pub fn compose<'a, Brand: Semigroupoid, B: 'a, C: 'a, D: 'a>(
	f: Apply!(
		brand: Brand,
		signature: ('a, C, D),
	),
	g: Apply!(
		brand: Brand,
		signature: ('a, B, C),
	),
) -> Apply!(
	brand: Brand,
	signature: ('a, B, D),
) {
	Brand::compose(f, g)
}
