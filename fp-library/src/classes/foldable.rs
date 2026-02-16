//! Data structures that can be folded into a single value from the left or right.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! let x = Some(5);
//! let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
//! assert_eq!(y, 15);
//! ```

use {
	super::monoid::Monoid,
	crate::{
		Apply,
		classes::{cloneable_fn::CloneableFn, semigroup::Semigroup},
		kinds::*,
		types::Endofunction,
	},
	fp_macros::{document_parameters, document_signature, document_type_parameters},
};

/// A type class for structures that can be folded to a single value.
///
/// A `Foldable` represents a structure that can be folded over to combine its elements
/// into a single result.
///
/// ### Minimal Implementation
///
/// A minimal implementation of `Foldable` requires implementing either [`Foldable::fold_right`] or [`Foldable::fold_map`].
///
/// *   If [`Foldable::fold_right`] is implemented, [`Foldable::fold_map`] and [`Foldable::fold_left`] are derived from it.
/// *   If [`Foldable::fold_map`] is implemented, [`Foldable::fold_right`] is derived from it, and [`Foldable::fold_left`] is derived from the derived [`Foldable::fold_right`].
///
/// Note that [`Foldable::fold_left`] is not sufficient on its own because the default implementations of [`Foldable::fold_right`] and [`Foldable::fold_map`] do not depend on it.
pub trait Foldable: Kind_ad6c20556a82a1f0 {
	/// Folds the structure by applying a function from right to left.
	///
	/// This method performs a right-associative fold of the structure.
	#[document_signature]
	///
	#[document_type_parameters(
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	#[document_parameters(
		"The function to apply to each element and the accumulator.",
		"The initial value of the accumulator.",
		"The structure to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_right<FnBrand, A: Clone, B, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
	) -> B
	where
		Func: Fn(A, B) -> B,
		FnBrand: CloneableFn,
	{
		let f = <FnBrand as CloneableFn>::new(move |(a, b)| func(a, b));
		let m = Self::fold_map::<FnBrand, A, Endofunction<FnBrand, B>, _>(
			move |a: A| {
				let f = f.clone();
				Endofunction::<FnBrand, B>::new(<FnBrand as CloneableFn>::new(move |b| {
					f((a.clone(), b))
				}))
			},
			fa,
		);
		m.0(initial)
	}

	/// Folds the structure by applying a function from left to right.
	///
	/// This method performs a left-associative fold of the structure.
	#[document_signature]
	///
	#[document_type_parameters(
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the accumulator.",
		"The type of the folding function."
	)]
	///
	#[document_parameters(
		"The function to apply to the accumulator and each element.",
		"The initial value of the accumulator.",
		"The structure to fold."
	)]
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let x = Some(5);
	/// let y = fold_left::<RcFnBrand, OptionBrand, _, _, _>(|b, a| b + a, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_left<FnBrand, A: Clone, B, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
	) -> B
	where
		Func: Fn(B, A) -> B,
		FnBrand: CloneableFn,
	{
		let f = <FnBrand as CloneableFn>::new(move |(b, a)| func(b, a));
		let m = Self::fold_right::<FnBrand, A, Endofunction<FnBrand, B>, _>(
			move |a: A, k: Endofunction<FnBrand, B>| {
				let f = f.clone();
				// k is the "rest" of the computation.
				// We want to perform "current" (f(b, a)) then "rest".
				// Endofunction composition is f . g (f after g).
				// So we want k . current.
				// append(k, current).
				let current =
					Endofunction::<FnBrand, B>::new(<FnBrand as CloneableFn>::new(move |b| {
						f((b, a.clone()))
					}));
				Semigroup::append(k, current)
			},
			Endofunction::<FnBrand, B>::empty(),
			fa,
		);
		m.0(initial)
	}

	/// Maps values to a monoid and combines them.
	///
	/// This method maps each element of the structure to a monoid and then combines the results using the monoid's `append` operation.
	#[document_signature]
	///
	#[document_type_parameters(
		"The brand of the cloneable function to use.",
		"The type of the elements in the structure.",
		"The type of the monoid.",
		"The type of the mapping function."
	)]
	///
	#[document_parameters(
		"The function to map each element to a monoid.",
		"The structure to fold."
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
	/// let x = Some(5);
	/// let y = fold_map::<RcFnBrand, OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
	/// assert_eq!(y, "5".to_string());
	/// ```
	fn fold_map<FnBrand, A: Clone, M, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<T>; )>::Of<A>),
	) -> M
	where
		M: Monoid,
		Func: Fn(A) -> M,
		FnBrand: CloneableFn,
	{
		Self::fold_right::<FnBrand, A, M, _>(move |a, m| M::append(func(a), m), M::empty(), fa)
	}
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_right`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the cloneable function to use.",
	"The brand of the foldable structure.",
	"The type of the elements in the structure.",
	"The type of the accumulator.",
	"The type of the folding function."
)]
///
#[document_parameters(
	"The function to apply to each element and the accumulator.",
	"The initial value of the accumulator.",
	"The structure to fold."
)]
///
/// ### Returns
///
/// The final accumulator value.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Some(5);
/// let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_right<FnBrand, Brand: Foldable, A: Clone, B, Func>(
	func: Func,
	initial: B,
	fa: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
) -> B
where
	Func: Fn(A, B) -> B,
	FnBrand: CloneableFn,
{
	Brand::fold_right::<FnBrand, A, B, Func>(func, initial, fa)
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_left`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the cloneable function to use.",
	"The brand of the foldable structure.",
	"The type of the elements in the structure.",
	"The type of the accumulator.",
	"The type of the folding function."
)]
///
#[document_parameters(
	"The function to apply to the accumulator and each element.",
	"The initial value of the accumulator.",
	"The structure to fold."
)]
///
/// ### Returns
///
/// The final accumulator value.
///
/// ### Examples
///
/// ```
/// use fp_library::{
/// 	brands::*,
/// 	functions::*,
/// };
///
/// let x = Some(5);
/// let y = fold_left::<RcFnBrand, OptionBrand, _, _, _>(|b, a| b + a, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_left<FnBrand, Brand: Foldable, A: Clone, B, Func>(
	func: Func,
	initial: B,
	fa: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
) -> B
where
	Func: Fn(B, A) -> B,
	FnBrand: CloneableFn,
{
	Brand::fold_left::<FnBrand, A, B, Func>(func, initial, fa)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_map`].
#[document_signature]
///
#[document_type_parameters(
	"The brand of the cloneable function to use.",
	"The brand of the foldable structure.",
	"The type of the elements in the structure.",
	"The type of the monoid.",
	"The type of the mapping function."
)]
///
#[document_parameters("The function to map each element to a monoid.", "The structure to fold.")]
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
/// let x = Some(5);
/// let y = fold_map::<RcFnBrand, OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
/// assert_eq!(y, "5".to_string());
/// ```
pub fn fold_map<FnBrand, Brand: Foldable, A: Clone, M, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<T>; )>::Of<A>),
) -> M
where
	M: Monoid,
	Func: Fn(A) -> M,
	FnBrand: CloneableFn,
{
	Brand::fold_map::<FnBrand, A, M, Func>(func, fa)
}
