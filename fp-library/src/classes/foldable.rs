use super::monoid::Monoid;
use crate::{
	Apply,
	brands::RcFnBrand,
	classes::{clonable_fn::ClonableFn, semigroup::Semigroup},
	kinds::*,
	types::Endofunction,
};
use std::rc::Rc;

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
	/// # Type Signature
	///
	/// `forall a b. Foldable t => ((a, b) -> b, b, t a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to each element and the accumulator.
	/// * `init`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// # Returns
	///
	/// The final accumulator value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_right(|a, b| a + b, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_right<'a, A: 'a + Clone, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
			lifetimes: ('a),
			types: (A)
		),
	) -> B
	where
		F: Fn(A, B) -> B + 'a,
	{
		let f = Rc::new(f);
		let m = Self::fold_map(
			move |a: A| {
				let f = f.clone();
				Endofunction::<RcFnBrand, B>::new(<RcFnBrand as ClonableFn>::new(move |b| {
					f(a.clone(), b)
				}))
			},
			fa,
		);
		m.0(init)
	}

	/// Folds the structure by applying a function from left to right.
	///
	/// # Type Signature
	///
	/// `forall a b. Foldable t => ((b, a) -> b, b, t a) -> b`
	///
	/// # Parameters
	///
	/// * `f`: The function to apply to the accumulator and each element.
	/// * `init`: The initial value of the accumulator.
	/// * `fa`: The structure to fold.
	///
	/// # Returns
	///
	/// The final accumulator value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_left(|b, a| b + a, 10, x);
	/// assert_eq!(y, 15);
	/// ```
	fn fold_left<'a, A: 'a + Clone, B: 'a, F>(
		f: F,
		init: B,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
			lifetimes: ('a),
			types: (A)
		),
	) -> B
	where
		F: Fn(B, A) -> B + 'a,
	{
		let f = Rc::new(f);
		let m = Self::fold_right(
			move |a: A, k: Endofunction<'a, RcFnBrand, B>| {
				let f = f.clone();
				// k is the "rest" of the computation.
				// We want to perform "current" (f(b, a)) then "rest".
				// Endofunction composition is f . g (f after g).
				// So we want k . current.
				// append(k, current).
				let current =
					Endofunction::<RcFnBrand, B>::new(<RcFnBrand as ClonableFn>::new(move |b| {
						f(b, a.clone())
					}));
				Semigroup::append(k, current)
			},
			Endofunction::<RcFnBrand, B>::empty(),
			fa,
		);
		m.0(init)
	}

	/// Maps values to a monoid and combines them.
	///
	/// # Type Signature
	///
	/// `forall a m. (Foldable t, Monoid m) => ((a) -> m, t a) -> m`
	///
	/// # Parameters
	///
	/// * `f`: The function to map each element to a monoid.
	/// * `fa`: The structure to fold.
	///
	/// # Returns
	///
	/// The combined monoid value.
	///
	/// # Examples
	///
	/// ```
	/// use fp_library::classes::foldable::Foldable;
	/// use fp_library::brands::OptionBrand;
	/// use fp_library::types::string; // Import Monoid impl for String
	///
	/// let x = Some(5);
	/// let y = OptionBrand::fold_map(|a: i32| a.to_string(), x);
	/// assert_eq!(y, "5".to_string());
	/// ```
	fn fold_map<'a, A: 'a + Clone, M, F>(
		f: F,
		fa: Apply!(
			brand: Self,
			signature: ('a, A: 'a) -> 'a,
			lifetimes: ('a),
			types: (A)
		),
	) -> M
	where
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
	{
		Self::fold_right(move |a, m| M::append(f(a), m), M::empty(), fa)
	}
}

/// Folds the structure by applying a function from right to left.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_right`].
///
/// # Type Signature
///
/// `forall a b. Foldable t => ((a, b) -> b, b, t a) -> b`
///
/// # Parameters
///
/// * `f`: The function to apply to each element and the accumulator.
/// * `init`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// # Returns
///
/// The final accumulator value.
///
/// # Examples
///
/// ```
/// use fp_library::classes::foldable::fold_right;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = fold_right::<OptionBrand, _, _, _>(|a, b| a + b, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_right<'a, Brand: Foldable, A: 'a + Clone, B: 'a, F>(
	f: F,
	init: B,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
		lifetimes: ('a),
		types: (A)
	),
) -> B
where
	F: Fn(A, B) -> B + 'a,
{
	Brand::fold_right(f, init, fa)
}

/// Folds the structure by applying a function from left to right.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_left`].
///
/// # Type Signature
///
/// `forall a b. Foldable t => ((b, a) -> b, b, t a) -> b`
///
/// # Parameters
///
/// * `f`: The function to apply to the accumulator and each element.
/// * `init`: The initial value of the accumulator.
/// * `fa`: The structure to fold.
///
/// # Returns
///
/// The final accumulator value.
///
/// # Examples
///
/// ```
/// use fp_library::classes::foldable::fold_left;
/// use fp_library::brands::OptionBrand;
///
/// let x = Some(5);
/// let y = fold_left::<OptionBrand, _, _, _>(|b, a| b + a, 10, x);
/// assert_eq!(y, 15);
/// ```
pub fn fold_left<'a, Brand: Foldable, A: 'a + Clone, B: 'a, F>(
	f: F,
	init: B,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
		lifetimes: ('a),
		types: (A)
	),
) -> B
where
	F: Fn(B, A) -> B + 'a,
{
	Brand::fold_left(f, init, fa)
}

/// Maps values to a monoid and combines them.
///
/// Free function version that dispatches to [the type class' associated function][`Foldable::fold_map`].
///
/// # Type Signature
///
/// `forall a m. (Foldable t, Monoid m) => ((a) -> m, t a) -> m`
///
/// # Parameters
///
/// * `f`: The function to map each element to a monoid.
/// * `fa`: The structure to fold.
///
/// # Returns
///
/// The combined monoid value.
///
/// # Examples
///
/// ```
/// use fp_library::classes::foldable::fold_map;
/// use fp_library::brands::OptionBrand;
/// use fp_library::types::string; // Import Monoid impl for String
///
/// let x = Some(5);
/// let y = fold_map::<OptionBrand, _, _, _>(|a: i32| a.to_string(), x);
/// assert_eq!(y, "5".to_string());
/// ```
pub fn fold_map<'a, Brand: Foldable, A: 'a + Clone, M, F>(
	f: F,
	fa: Apply!(
		brand: Brand,
		signature: ('a, A: 'a) -> 'a,
		lifetimes: ('a),
		types: (A)
	),
) -> M
where
	M: Monoid + 'a,
	F: Fn(A) -> M + 'a,
{
	Brand::fold_map(f, fa)
}
