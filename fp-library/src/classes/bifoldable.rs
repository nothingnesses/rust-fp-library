//! Data structures with two type arguments that can be folded into a single value.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x: Result<i32, i32> = Ok(5);
//! let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
//! 	|e: i32| e.to_string(),
//! 	|s: i32| s.to_string(),
//! 	x,
//! );
//! assert_eq!(y, "5".to_string());
//! ```

use {
	crate::{
		Apply,
		classes::{
			cloneable_fn::CloneableFn,
			monoid::Monoid,
			semigroup::Semigroup,
		},
		kinds::*,
		types::Endofunction,
	},
	fp_macros::{
		document_parameters,
		document_signature,
		document_type_parameters,
	},
};

/// A type class for data structures with two type arguments that can be folded.
///
/// A `Bifoldable` represents a container with two type parameters, where elements
/// of either type can be folded into a single result. A fold requires two step
/// functions, one for each type argument.
///
/// ### Minimal Implementation
///
/// A minimal implementation requires either [`Bifoldable::bi_fold_map`] or
/// [`Bifoldable::bi_fold_right`] to be defined directly:
///
/// * If [`Bifoldable::bi_fold_right`] is implemented, [`Bifoldable::bi_fold_map`]
///   and [`Bifoldable::bi_fold_left`] are derived from it.
/// * If [`Bifoldable::bi_fold_map`] is implemented, [`Bifoldable::bi_fold_right`]
///   is derived via `Endofunction`, and [`Bifoldable::bi_fold_left`] is derived
///   from the derived [`Bifoldable::bi_fold_right`].
///
/// Note: defining both defaults creates a circular dependency and will not terminate.
///
/// ### Laws
///
/// `Bifoldable` instances must be consistent with `Bifunctor` when one is also
/// defined, following the general principle that the structure of the fold mirrors
/// the structure of the map.
pub trait Bifoldable: Kind_266801a817966495 {
	/// Folds the bifoldable structure from right to left using two step functions.
	///
	/// This method performs a right-associative fold, dispatching to `f` for
	/// elements of the first type and `g` for elements of the second type.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The type of the accumulator.",
		"The type of the first step function.",
		"The type of the second step function."
	)]
	///
	#[document_parameters(
		"The step function for first-position elements.",
		"The step function for second-position elements.",
		"The initial accumulator value.",
		"The bifoldable structure to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value after folding all elements.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Err(3);
	/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	|e, acc| acc - e,
	/// 	|s, acc| acc + s,
	/// 	10,
	/// 	x,
	/// );
	/// assert_eq!(y, 7);
	/// ```
	fn bi_fold_right<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a, FA, FB>(
		f: FA,
		g: FB,
		z: C,
		p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> C
	where
		FA: Fn(A, C) -> C + 'a,
		FB: Fn(B, C) -> C + 'a, {
		let f = <FnBrand as CloneableFn>::new(move |(a, c)| f(a, c));
		let g = <FnBrand as CloneableFn>::new(move |(b, c)| g(b, c));
		let endo = Self::bi_fold_map::<FnBrand, A, B, Endofunction<'a, FnBrand, C>, _, _>(
			move |a: A| {
				let f = f.clone();
				Endofunction::<FnBrand, C>::new(<FnBrand as CloneableFn>::new(move |c| {
					f((a.clone(), c))
				}))
			},
			move |b: B| {
				let g = g.clone();
				Endofunction::<FnBrand, C>::new(<FnBrand as CloneableFn>::new(move |c| {
					g((b.clone(), c))
				}))
			},
			p,
		);
		endo.0(z)
	}

	/// Folds the bifoldable structure from left to right using two step functions.
	///
	/// This method performs a left-associative fold, dispatching to `f` for
	/// elements of the first type and `g` for elements of the second type.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The type of the accumulator.",
		"The type of the first step function.",
		"The type of the second step function."
	)]
	///
	#[document_parameters(
		"The step function for first-position elements.",
		"The step function for second-position elements.",
		"The initial accumulator value.",
		"The bifoldable structure to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value after folding all elements.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_fold_left::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	|acc, e| acc - e,
	/// 	|acc, s| acc + s,
	/// 	10,
	/// 	x,
	/// );
	/// assert_eq!(y, 15);
	/// ```
	fn bi_fold_left<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, C: 'a, FA, FB>(
		f: FA,
		g: FB,
		z: C,
		p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> C
	where
		FA: Fn(C, A) -> C + 'a,
		FB: Fn(C, B) -> C + 'a, {
		let f = <FnBrand as CloneableFn>::new(move |(c, a)| f(c, a));
		let g = <FnBrand as CloneableFn>::new(move |(c, b)| g(c, b));
		let endo = Self::bi_fold_right::<FnBrand, A, B, Endofunction<'a, FnBrand, C>, _, _>(
			move |a: A, k: Endofunction<'a, FnBrand, C>| {
				let f = f.clone();
				let current =
					Endofunction::<FnBrand, C>::new(<FnBrand as CloneableFn>::new(move |c| {
						f((c, a.clone()))
					}));
				Semigroup::append(k, current)
			},
			move |b: B, k: Endofunction<'a, FnBrand, C>| {
				let g = g.clone();
				let current =
					Endofunction::<FnBrand, C>::new(<FnBrand as CloneableFn>::new(move |c| {
						g((c, b.clone()))
					}));
				Semigroup::append(k, current)
			},
			Endofunction::<FnBrand, C>::empty(),
			p,
		);
		endo.0(z)
	}

	/// Maps elements of both types to a monoid and combines the results.
	///
	/// This method maps each element of the first type using `f` and each element
	/// of the second type using `g`, then combines all results using the monoid's
	/// `append` operation.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The type of the first-position elements.",
		"The type of the second-position elements.",
		"The monoid type to fold into.",
		"The type of the first mapping function.",
		"The type of the second mapping function."
	)]
	///
	#[document_parameters(
		"The function mapping first-position elements to the monoid.",
		"The function mapping second-position elements to the monoid.",
		"The bifoldable structure to fold."
	)]
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x: Result<i32, i32> = Ok(5);
	/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
	/// 	|e: i32| e.to_string(),
	/// 	|s: i32| s.to_string(),
	/// 	x,
	/// );
	/// assert_eq!(y, "5".to_string());
	/// ```
	fn bi_fold_map<'a, FnBrand: CloneableFn + 'a, A: 'a + Clone, B: 'a + Clone, M, FA, FB>(
		f: FA,
		g: FB,
		p: Apply!(<Self as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
	) -> M
	where
		M: Monoid + 'a,
		FA: Fn(A) -> M + 'a,
		FB: Fn(B) -> M + 'a, {
		Self::bi_fold_right::<FnBrand, A, B, M, _, _>(
			move |a, m| M::append(f(a), m),
			move |b, m| M::append(g(b), m),
			M::empty(),
			p,
		)
	}
}

/// Folds the bifoldable structure from right to left using two step functions.
///
/// Free function version that dispatches to [the type class' associated function][`Bifoldable::bi_fold_right`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cloneable function to use.",
	"The brand of the bifoldable structure.",
	"The type of the first-position elements.",
	"The type of the second-position elements.",
	"The type of the accumulator.",
	"The type of the first step function.",
	"The type of the second step function."
)]
///
#[document_parameters(
	"The step function for first-position elements.",
	"The step function for second-position elements.",
	"The initial accumulator value.",
	"The bifoldable structure to fold."
)]
///
/// ### Returns
///
/// The final accumulator value after folding all elements.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x: Result<i32, i32> = Err(3);
/// let y = bi_fold_right::<RcFnBrand, ResultBrand, _, _, _, _, _>(
/// 	|e, acc| acc - e,
/// 	|s, acc| acc + s,
/// 	10,
/// 	x,
/// );
/// assert_eq!(y, 7);
/// ```
pub fn bi_fold_right<
	'a,
	FnBrand: CloneableFn + 'a,
	Brand: Bifoldable,
	A: 'a + Clone,
	B: 'a + Clone,
	C: 'a,
	FA,
	FB,
>(
	f: FA,
	g: FB,
	z: C,
	p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
) -> C
where
	FA: Fn(A, C) -> C + 'a,
	FB: Fn(B, C) -> C + 'a, {
	Brand::bi_fold_right::<FnBrand, A, B, C, FA, FB>(f, g, z, p)
}

/// Folds the bifoldable structure from left to right using two step functions.
///
/// Free function version that dispatches to [the type class' associated function][`Bifoldable::bi_fold_left`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cloneable function to use.",
	"The brand of the bifoldable structure.",
	"The type of the first-position elements.",
	"The type of the second-position elements.",
	"The type of the accumulator.",
	"The type of the first step function.",
	"The type of the second step function."
)]
///
#[document_parameters(
	"The step function for first-position elements.",
	"The step function for second-position elements.",
	"The initial accumulator value.",
	"The bifoldable structure to fold."
)]
///
/// ### Returns
///
/// The final accumulator value after folding all elements.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x: Result<i32, i32> = Ok(5);
/// let y = bi_fold_left::<RcFnBrand, ResultBrand, _, _, _, _, _>(
/// 	|acc, e| acc - e,
/// 	|acc, s| acc + s,
/// 	10,
/// 	x,
/// );
/// assert_eq!(y, 15);
/// ```
pub fn bi_fold_left<
	'a,
	FnBrand: CloneableFn + 'a,
	Brand: Bifoldable,
	A: 'a + Clone,
	B: 'a + Clone,
	C: 'a,
	FA,
	FB,
>(
	f: FA,
	g: FB,
	z: C,
	p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
) -> C
where
	FA: Fn(C, A) -> C + 'a,
	FB: Fn(C, B) -> C + 'a, {
	Brand::bi_fold_left::<FnBrand, A, B, C, FA, FB>(f, g, z, p)
}

/// Maps elements of both types to a monoid and combines the results.
///
/// Free function version that dispatches to [the type class' associated function][`Bifoldable::bi_fold_map`].
#[document_signature]
///
#[document_type_parameters(
	"The lifetime of the values.",
	"The brand of the cloneable function to use.",
	"The brand of the bifoldable structure.",
	"The type of the first-position elements.",
	"The type of the second-position elements.",
	"The monoid type to fold into.",
	"The type of the first mapping function.",
	"The type of the second mapping function."
)]
///
#[document_parameters(
	"The function mapping first-position elements to the monoid.",
	"The function mapping second-position elements to the monoid.",
	"The bifoldable structure to fold."
)]
///
/// ### Returns
///
/// The combined monoid value.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x: Result<i32, i32> = Ok(5);
/// let y = bi_fold_map::<RcFnBrand, ResultBrand, _, _, _, _, _>(
/// 	|e: i32| e.to_string(),
/// 	|s: i32| s.to_string(),
/// 	x,
/// );
/// assert_eq!(y, "5".to_string());
/// ```
pub fn bi_fold_map<
	'a,
	FnBrand: CloneableFn + 'a,
	Brand: Bifoldable,
	A: 'a + Clone,
	B: 'a + Clone,
	M,
	FA,
	FB,
>(
	f: FA,
	g: FB,
	p: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, B>),
) -> M
where
	M: Monoid + 'a,
	FA: Fn(A) -> M + 'a,
	FB: Fn(B) -> M + 'a, {
	Brand::bi_fold_map::<FnBrand, A, B, M, FA, FB>(f, g, p)
}
