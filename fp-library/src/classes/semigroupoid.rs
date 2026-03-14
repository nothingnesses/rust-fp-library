//! Semigroupoids, representing objects and composable relationships (morphisms) between them.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::*,
//! 	functions::*,
//! };
//!
//! let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
//! let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
//! let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
//! assert_eq!(h(5), 12); // (5 + 1) * 2
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::kinds::*,
		fp_macros::*,
	};

	/// A type class for semigroupoids.
	///
	/// A `Semigroupoid` is a set of objects and composable relationships
	/// (morphisms) between them.
	///
	/// ### Hierarchy Unification
	///
	/// This trait inherits from [`Kind!(type Of<'a, A: 'a, B: 'a>: 'a;)`](crate::kinds::Kind_266801a817966495).
	/// This unification ensures that all profunctors and arrows share a
	/// consistent higher-kinded representation, and requires that the source and target
	/// types of a morphism outlive the morphism's application lifetime.
	///
	/// ### Laws
	///
	/// Semigroupoid instances must satisfy the associative law:
	/// * Associativity: `compose(p, compose(q, r)) = compose(compose(p, q), r)`.
	#[document_examples]
	///
	/// Associativity for [`RcFnBrand`](crate::brands::RcFnBrand):
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let h = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x - 3);
	///
	/// // Associativity: compose(f, compose(g, h)) = compose(compose(f, g), h)
	/// let left = semigroupoid_compose::<RcFnBrand, _, _, _>(
	/// 	f.clone(),
	/// 	semigroupoid_compose::<RcFnBrand, _, _, _>(g.clone(), h.clone()),
	/// );
	/// let right = semigroupoid_compose::<RcFnBrand, _, _, _>(
	/// 	semigroupoid_compose::<RcFnBrand, _, _, _>(f, g),
	/// 	h,
	/// );
	/// // Both sides produce the same result for any input
	/// assert_eq!(left(10), right(10));
	/// assert_eq!(left(0), right(0));
	/// ```
	#[kind(type Of<'a, A: 'a, B: 'a>: 'a;)]
	pub trait Semigroupoid {
		/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
		///
		/// This method composes two morphisms `f` and `g` to produce a new morphism that represents the application of `g` followed by `f`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the morphisms.",
			"The source type of the first morphism.",
			"The target type of the first morphism and the source type of the second morphism.",
			"The target type of the second morphism."
		)]
		///
		#[document_parameters(
			"The second morphism to apply (from C to D).",
			"The first morphism to apply (from B to C)."
		)]
		///
		#[document_returns("The composed morphism (from B to D).")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::*,
		/// 	functions::*,
		/// };
		///
		/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
		/// let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
		/// let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
		/// assert_eq!(h(5), 12); // (5 + 1) * 2
		/// ```
		fn compose<'a, B: 'a, C: 'a, D: 'a>(
			f: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, C, D>),
			g: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, D>);
	}

	/// Takes morphisms `f` and `g` and returns the morphism `f . g` (`f` composed with `g`).
	///
	/// Free function version that dispatches to [the type class' associated function][`Semigroupoid::compose`].
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the morphisms.",
		"The brand of the semigroupoid.",
		"The source type of the first morphism.",
		"The target type of the first morphism and the source type of the second morphism.",
		"The target type of the second morphism."
	)]
	///
	#[document_parameters(
		"The second morphism to apply (from C to D).",
		"The first morphism to apply (from B to C)."
	)]
	///
	#[document_returns("The composed morphism (from B to D).")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::*,
	/// 	functions::*,
	/// };
	///
	/// let f = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x * 2);
	/// let g = cloneable_fn_new::<RcFnBrand, _, _>(|x: i32| x + 1);
	/// let h = semigroupoid_compose::<RcFnBrand, _, _, _>(f, g);
	/// assert_eq!(h(5), 12); // (5 + 1) * 2
	/// ```
	pub fn compose<'a, Brand: Semigroupoid, B: 'a, C: 'a, D: 'a>(
		f: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, C, D>),
		g: Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, D>) {
		Brand::compose::<B, C, D>(f, g)
	}
}

pub use inner::*;
