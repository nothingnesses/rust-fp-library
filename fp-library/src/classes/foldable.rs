//! Foldable type class.
//!
//! This module defines the [`Foldable`] trait, which represents data structures that can be folded to a single value.

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
/// # Minimal Implementation
///
/// A minimal implementation of `Foldable` requires implementing either [`Foldable::fold_right`] or [`Foldable::fold_map`].
///
/// *   If [`Foldable::fold_right`] is implemented, [`Foldable::fold_map`] and [`Foldable::fold_left`] are derived from it.
/// *   If [`Foldable::fold_map`] is implemented, [`Foldable::fold_right`] is derived from it, and [`Foldable::fold_left`] is derived from the derived [`Foldable::fold_right`].
///
/// Note that [`Foldable::fold_left`] is not sufficient on its own because the default implementations of [`Foldable::fold_right`] and [`Foldable::fold_map`] do not depend on it.
pub trait Foldable: Kind_c3c3610c70409ee6 {
	/// Folds the structure by applying a function from right to left.
	///
	/// This method performs a right-associative fold of the structure.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable t => ((a, b) -> b, b, t a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `ClonableFnBrand`: The brand of the clonable function to use.
	/// * `A`: The type of the elements in the structure.
	/// * `B`: The type of the accumulator.
	/// * `F`: The type of the folding function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to each element and the accumulator.
	/// * `init`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_right::<RcFnBrand, _, _, _>(|a, b| a + b, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_right<'a, ClonableFnBrand, A: 'a + Clone, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
		ClonableFnBrand: ClonableFn + 'a,
	{
		let f = <ClonableFnBrand as ClonableFn>::new(move |(a, b)| f(a, b));
		let m = Self::fold_map::<ClonableFnBrand, A, Endofunction<ClonableFnBrand, B>, _>(
			move |a: A| {
				let f = f.clone();
				Endofunction::<ClonableFnBrand, B>::new(<ClonableFnBrand as ClonableFn>::new(
					move |b| f((a.clone(), b)),
				))
			},
			fa,
		);
		m.0(init)
	}

	/// Folds the structure by applying a function from left to right.
	///
	/// This method performs a left-associative fold of the structure.
	///
	/// ### Type Signature
	///
	/// `forall a b. Foldable t => ((b, a) -> b, b, t a) -> b`
	///
	/// ### Type Parameters
	///
	/// * `ClonableFnBrand`: The brand of the clonable function to use.
	/// * `A`: The type of the elements in the structure.
	/// * `B`: The type of the accumulator.
	/// * `F`: The type of the folding function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to apply to the accumulator and each element.
	/// * `init`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The final accumulator value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_left::<RcFnBrand, _, _, _>(|b, a| b + a, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_left<'a, ClonableFnBrand, A: 'a + Clone, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
		ClonableFnBrand: ClonableFn + 'a,
	{
		let f = <ClonableFnBrand as ClonableFn>::new(move |(b, a)| f(b, a));
		let m = Self::fold_right::<ClonableFnBrand, A, Endofunction<ClonableFnBrand, B>, _>(
			move |a: A, k: Endofunction<'a, ClonableFnBrand, B>| {
				let f = f.clone();
				// k is the "rest" of the computation.
				// We want to perform "current" (f(b, a)) then "rest".
				// Endofunction composition is f . g (f after g).
				// So we want k . current.
				// append(k, current).
				let current = Endofunction::<ClonableFnBrand, B>::new(
					<ClonableFnBrand as ClonableFn>::new(move |b| f((b, a.clone()))),
				);
				Semigroup::append(k, current)
			},
			Endofunction::<ClonableFnBrand, B>::empty(),
			fa,
		);
		m.0(init)
	}

	/// Maps values to a monoid and combines them.
	///
	/// This method maps each element of the structure to a monoid and then combines the results using the monoid's `append` operation.
	///
	/// ### Type Signature
	///
	/// `forall a m. (Foldable t, Monoid m) => ((a) -> m, t a) -> m`
	///
	/// ### Type Parameters
	///
	/// * `ClonableFnBrand`: The brand of the clonable function to use.
	/// * `A`: The type of the elements in the structure.
	/// * `M`: The type of the monoid.
	/// * `F`: The type of the mapping function.
	///
	/// ### Parameters
	///
	/// * `f`: The function to map each element to a monoid.
	/// * `fa`: The structure to fold.
	///
	/// ### Returns
	///
	/// The combined monoid value.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::string; // Import Monoid impl for String
	/// use fp_library::brands::RcFnBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_map::<RcFnBrand, _, _, _>(|a: i32| a.to_string(), x);
	/// assert_eq!(y, "5".to_string());
	/// ```
	fn fold_map<'a, ClonableFnBrand, A: 'a + Clone, M, F>(
		f: F,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
		),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
		ClonableFnBrand: ClonableFn + 'a,
	{
		Self::fold_right::<ClonableFnBrand, A, M, _>(move |a, m| M::append(f(a), m), M::empty(), fa)
	}
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_right`].
///
/// ### Type Signature
///
/// `forall a b. Foldable t => ((a, b) -> b, b, t a) -> b`
///
/// ### Type Parameters
///
/// * `ClonableFnBrand`: The brand of the clonable function to use.
/// * `Brand`: The brand of the foldable structure.
/// * `A`: The type of the elements in the structure.
/// * `B`: The type of the accumulator.
/// * `F`: The type of the folding function.
///
/// ### Parameters
///
/// * `f`: The function to apply to each element and the accumulator.
/// * `init`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The final accumulator value.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::foldable::fold_right;
/// use fp_library::brands::OptionBrand;
/// use fp_library::brands::RcFnBrand;
///
/// let x = Some(5);
/// let y = fold_right::<RcFnBrand, OptionBrand, _, _, _>(|a, b| a + b, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_right<'a, ClonableFnBrand, Brand: Foldable, A: 'a + Clone, B: 'a, F>(
	f: F,
	init: B,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
	),
) -> B
where
	F: Fn(A, B) -> B + 'a,
	ClonableFnBrand: ClonableFn + 'a,
{
	Brand::fold_right::<ClonableFnBrand, A, B, F>(f, init, fa)
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_left`].
///
/// ### Type Signature
///
/// `forall a b. Foldable t => ((b, a) -> b, b, t a) -> b`
///
/// ### Type Parameters
///
/// * `ClonableFnBrand`: The brand of the clonable function to use.
/// * `Brand`: The brand of the foldable structure.
/// * `A`: The type of the elements in the structure.
/// * `B`: The type of the accumulator.
/// * `F`: The type of the folding function.
///
/// ### Parameters
///
/// * `f`: The function to apply to the accumulator and each element.
/// * `init`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The final accumulator value.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::foldable::fold_left;
/// use fp_library::brands::OptionBrand;
/// use fp_library::brands::RcFnBrand;
///
/// let x = Some(5);
/// let y = fold_left::<RcFnBrand, OptionBrand, _, _, _>(|b, a| b + a, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_left<'a, ClonableFnBrand, Brand: Foldable, A: 'a + Clone, B: 'a, F>(
	f: F,
	init: B,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
	),
) -> B
where
	F: Fn(B, A) -> B + 'a,
	ClonableFnBrand: ClonableFn + 'a,
{
	Brand::fold_left::<ClonableFnBrand, A, B, F>(f, init, fa)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_map`].
///
/// ### Type Signature
///
/// `forall a m. (Foldable t, Monoid m) => ((a) -> m, t a) -> m`
///
/// ### Type Parameters
///
/// * `ClonableFnBrand`: The brand of the clonable function to use.
/// * `Brand`: The brand of the foldable structure.
/// * `A`: The type of the elements in the structure.
/// * `M`: The type of the monoid.
/// * `F`: The type of the mapping function.
///
/// ### Parameters
///
/// * `f`: The function to map each element to a monoid.
/// * `fa`: The structure to fold.
///
/// ### Returns
///
/// The combined monoid value.
///
/// ### Examples
///
/// ```
/// use fp_library::classes::foldable::fold_map;
/// use fp_library::brands::OptionBrand;
/// use fp_library::types::string; // Import Monoid impl for String
/// use fp_library::brands::RcFnBrand;
///
/// let x = Some(5);
/// let y = fold_map::<RcFnBrand, OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
/// assert_eq!(y, "5".to_string());
/// ```
pub fn fold_map<'a, ClonableFnBrand, Brand: Foldable, A: 'a + Clone, M, F>(
	f: F,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
	),
) -> M
where
	M: Monoid + 'a,
	F: Fn(A) -> M + 'a,
	ClonableFnBrand: ClonableFn + 'a,
{
	Brand::fold_map::<ClonableFnBrand, A, M, F>(f, fa)
}
