//! Foldable type class.
//!
//! This module defines the [`Foldable`] trait, which represents data structures that can be folded to a single value.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{brands::*, functions::*};
//!
//! let x = Some(5);
//! let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
//! assert_eq!(y, 15);
//! ```

use super::monoid::Monoid;
use crate::{
	Apply,
	classes::{clonable_fn::ClonableFn, semigroup::Semigroup},
	kinds::*,
	types::Endofunction,
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
pub trait Foldable: Kind_cdc7cd43dac7585f {
	/// Folds the structure by applying a function from right to left.
	///
	/// This method performs a right-associative fold of the structure.
	///
	/// ### Type Signature
	///
	/// `forall f b a. Foldable f => ((a, b) -> b, b, f a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `B`: The type of the accumulator.
	/// * `A`: The type of the elements in the structure.
	/// * `Func`: The type of the folding function.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to each element and the accumulator.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_right<'a, FnBrand, B: 'a, A: 'a + Clone, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(A, B) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		let f = <FnBrand as ClonableFn>::new(move |(a, b)| func(a, b));
		let m = Self::fold_map::<FnBrand, Endofunction<FnBrand, B>, A, _>(
			move |a: A| {
				let f = f.clone();
				Endofunction::<FnBrand, B>::new(<FnBrand as ClonableFn>::new(move |b| {
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
	///
	/// ### Type Signature
	///
	/// `forall f b a. Foldable f => ((b, a) -> b, b, f a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `B`: The type of the accumulator.
	/// * `A`: The type of the elements in the structure.
	/// * `Func`: The type of the folding function.
	///
	/// ### Parameters
	///
	/// * `func`: The function to apply to the accumulator and each element.
	/// * `initial`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = fold_left::<RcFnBrand, OptionBrand, _, _, _>(|b, a| b + a, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_left<'a, FnBrand, B: 'a, A: 'a + Clone, Func>(
		func: Func,
		initial: B,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B
	where
		Func: Fn(B, A) -> B + 'a,
		FnBrand: ClonableFn + 'a,
	{
		let f = <FnBrand as ClonableFn>::new(move |(b, a)| func(b, a));
		let m = Self::fold_right::<FnBrand, Endofunction<FnBrand, B>, A, _>(
			move |a: A, k: Endofunction<'a, FnBrand, B>| {
				let f = f.clone();
				// k is the "rest" of the computation.
				// We want to perform "current" (f(b, a)) then "rest".
				// Endofunction composition is f . g (f after g).
				// So we want k . current.
				// append(k, current).
				let current =
					Endofunction::<FnBrand, B>::new(<FnBrand as ClonableFn>::new(move |b| {
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
	///
	/// ### Type Signature
	///
	/// `forall f m a. (Foldable f, Monoid m) => ((a) -> m, f a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `FnBrand`: The brand of the clonable function to use.
	/// * `M`: The type of the monoid.
	/// * `A`: The type of the elements in the structure.
	/// * `Func`: The type of the mapping function.
	///
	/// ### Parameters
	///
	/// * `func`: The function to map each element to a monoid.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{brands::*, functions::*};
	///
	/// let x = Some(5);
	/// let y = fold_map::<RcFnBrand, OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
	/// assert_eq!(y, "5".to_string());
	/// ```
	fn fold_map<'a, FnBrand, M, A: 'a + Clone, Func>(
		func: Func,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M
	where
		M: Monoid + 'a,
		Func: Fn(A) -> M + 'a,
		FnBrand: ClonableFn + 'a,
	{
		Self::fold_right::<FnBrand, M, A, _>(move |a, m| M::append(func(a), m), M::empty(), fa)
	}
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_right`].
///
/// ### Type Signature
///
/// `forall f b a. Foldable f => ((a, b) -> b, b, f a) -> b`
///
/// ### Type Parameters
///
/// * `FnBrand`: The brand of the clonable function to use.
/// * `Brand`: The brand of the foldable structure.
/// * `B`: The type of the accumulator.
/// * `A`: The type of the elements in the structure.
/// * `Func`: The type of the folding function.
///
/// ### Parameters
///
/// * `func`: The function to apply to each element and the accumulator.
/// * `initial`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The final accumulator value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_right<'a, FnBrand, Brand: Foldable, B: 'a, A: 'a + Clone, Func>(
	func: Func,
	initial: B,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> B
where
	Func: Fn(A, B) -> B + 'a,
	FnBrand: ClonableFn + 'a,
{
	Brand::fold_right::<FnBrand, B, A, Func>(func, initial, fa)
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_left`].
///
/// ### Type Signature
///
/// `forall f b a. Foldable f => ((b, a) -> b, b, f a) -> b`
///
/// ### Type Parameters
///
/// * `FnBrand`: The brand of the clonable function to use.
/// * `Brand`: The brand of the foldable structure.
/// * `B`: The type of the accumulator.
/// * `A`: The type of the elements in the structure.
/// * `Func`: The type of the folding function.
///
/// ### Parameters
///
/// * `func`: The function to apply to the accumulator and each element.
/// * `initial`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The final accumulator value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = fold_left::<RcFnBrand, OptionBrand, _, _, _>(|b, a| b + a, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_left<'a, FnBrand, Brand: Foldable, B: 'a, A: 'a + Clone, Func>(
	func: Func,
	initial: B,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> B
where
	Func: Fn(B, A) -> B + 'a,
	FnBrand: ClonableFn + 'a,
{
	Brand::fold_left::<FnBrand, B, A, Func>(func, initial, fa)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_map`].
///
/// ### Type Signature
///
/// `forall f m a. (Foldable f, Monoid m) => ((a) -> m, f a) -> m`
///
/// ### Type Parameters
///
/// * `FnBrand`: The brand of the clonable function to use.
/// * `Brand`: The brand of the foldable structure.
/// * `M`: The type of the monoid.
/// * `A`: The type of the elements in the structure.
/// * `Func`: The type of the mapping function.
///
/// ### Parameters
///
/// * `func`: The function to map each element to a monoid.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The combined monoid value.
///
/// ### Examples
///
/// ```
/// use fp_library::{brands::*, functions::*};
///
/// let x = Some(5);
/// let y = fold_map::<RcFnBrand, OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
/// assert_eq!(y, "5".to_string());
/// ```
pub fn fold_map<'a, FnBrand, Brand: Foldable, M, A: 'a + Clone, Func>(
	func: Func,
	fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
) -> M
where
	M: Monoid + 'a,
	Func: Fn(A) -> M + 'a,
	FnBrand: ClonableFn + 'a,
{
	Brand::fold_map::<FnBrand, M, A, Func>(func, fa)
}
