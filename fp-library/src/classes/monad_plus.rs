//! Monads with a choice operator and identity element, combining [`Monad`](crate::classes::Monad) and [`Alternative`](crate::classes::Alternative).
//!
//! A `MonadPlus` is a type that supports both monadic sequencing
//! ([`Monad`](crate::classes::Monad)) and choice with an identity element
//! ([`Alternative`](crate::classes::Alternative)). It has no members of its own; it specifies that
//! the type constructor has both `Monad` and `Alternative` instances.
//!
//! This module is a port of PureScript's
//! [`Control.MonadPlus`](https://pursuit.purescript.org/packages/purescript-control/docs/Control.MonadPlus).
//!
//! # Hierarchy
//!
//! ```text
//! Functor
//!   |
//!   +-- Pointed + Semiapplicative --> Applicative
//!   |
//!   +-- Applicative + Semimonad ----> Monad
//!   |
//!   +-- Alt + (identity element) ---> Plus
//!   |
//!   +-- Applicative + Plus ---------> Alternative
//!   |
//!   +-- Monad + Alternative --------> MonadPlus
//! ```

#[fp_macros::document_module(no_validation)]
mod inner {
	use crate::classes::*;

	/// A type class for monads that also support choice, combining [`Monad`]
	/// and [`Alternative`].
	///
	/// `class (Monad m, Alternative m) => MonadPlus m`
	///
	/// `MonadPlus` has no members of its own. It specifies that the type
	/// constructor supports both monadic sequencing (`bind`, `pure`) and
	/// choice with an identity element (`alt`, `empty`).
	///
	/// # Laws
	///
	/// A lawful `MonadPlus` must satisfy:
	///
	/// - **Distributivity:** binding over a choice distributes:
	///
	///   ```text
	///   bind(alt(x, y), f) == alt(bind(x, f), bind(y, f))
	///   ```
	///
	/// The following property also holds for any `Monad + Alternative`:
	///
	/// - **Left zero:** binding on the identity element yields the
	///   identity element:
	///
	///   ```text
	///   bind(empty(), f) == empty()
	///   ```
	///
	/// # Examples
	///
	/// Distributivity law for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// };
	///
	/// let x: Vec<i32> = vec![1, 2];
	/// let y: Vec<i32> = vec![3, 4];
	/// let f = |n: i32| vec![n * 10, n * 100];
	///
	/// assert_eq!(
	/// 	bind::<VecBrand, _, _, _, _>(alt::<VecBrand, _, _, _>(x.clone(), y.clone()), f),
	/// 	alt::<VecBrand, _, _, _>(
	/// 		bind::<VecBrand, _, _, _, _>(x, f),
	/// 		bind::<VecBrand, _, _, _, _>(y, f)
	/// 	),
	/// );
	/// ```
	///
	/// Left-zero law for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::{
	/// 		explicit::bind,
	/// 		*,
	/// 	},
	/// };
	///
	/// let f = |n: i32| vec![n * 2];
	///
	/// assert_eq!(
	/// 	bind::<VecBrand, _, _, _, _>(plus_empty::<VecBrand, i32>(), f),
	/// 	plus_empty::<VecBrand, i32>(),
	/// );
	/// ```
	///
	/// # Implementors
	///
	/// The blanket implementation applies to every brand with both [`Monad`]
	/// and [`Alternative`]. The following brands are known to be *lawful*
	/// (satisfying distributivity and left zero):
	///
	/// - [`VecBrand`](crate::brands::VecBrand)
	/// - [`CatListBrand`](crate::brands::CatListBrand)
	///
	/// Note that [`OptionBrand`](crate::brands::OptionBrand) acquires the
	/// trait via the blanket impl but does *not* satisfy the distributivity
	/// law, because `alt` for `Option` picks the first `Some` and discards
	/// the second branch entirely.
	pub trait MonadPlus: Monad + Alternative {}

	/// Blanket implementation of [`MonadPlus`].
	#[document_type_parameters("The brand type.")]
	impl<Brand> MonadPlus for Brand where Brand: Monad + Alternative {}
}

pub use inner::*;

#[cfg(test)]
mod tests {
	use {
		crate::{
			brands::*,
			functions::{
				explicit::{
					alt,
					bind,
				},
				*,
			},
			types::cat_list::CatList,
		},
		quickcheck_macros::quickcheck,
	};

	// -- Distributivity: bind(alt(x, y), f) == alt(bind(x, f), bind(y, f)) --

	/// Tests the distributivity law for MonadPlus with VecBrand.
	#[quickcheck]
	fn distributivity_vec(
		x: Vec<i32>,
		y: Vec<i32>,
	) -> bool {
		let f = |n: i32| {
			if n > 0 { vec![n.wrapping_mul(2)] } else { vec![] }
		};
		bind::<VecBrand, _, _, _, _>(alt::<VecBrand, _, _, _>(x.clone(), y.clone()), f)
			== alt::<VecBrand, _, _, _>(
				bind::<VecBrand, _, _, _, _>(x, f),
				bind::<VecBrand, _, _, _, _>(y, f),
			)
	}

	/// Tests the distributivity law for MonadPlus with CatListBrand.
	#[quickcheck]
	fn distributivity_cat_list(
		xv: Vec<i32>,
		yv: Vec<i32>,
	) -> bool {
		let x: CatList<i32> = xv.into_iter().collect();
		let y: CatList<i32> = yv.into_iter().collect();
		let f = |n: i32| -> CatList<i32> {
			if n > 0 { CatList::singleton(n.wrapping_mul(2)) } else { CatList::empty() }
		};
		bind::<CatListBrand, _, _, _, _>(alt::<CatListBrand, _, _, _>(x.clone(), y.clone()), f)
			== alt::<CatListBrand, _, _, _>(
				bind::<CatListBrand, _, _, _, _>(x, f),
				bind::<CatListBrand, _, _, _, _>(y, f),
			)
	}

	// -- Left zero: bind(empty(), f) == empty() --

	/// Tests the left-zero law for MonadPlus with OptionBrand.
	#[test]
	fn left_zero_option() {
		let f = |n: i32| if n > 0 { Some(n * 2) } else { None };
		assert_eq!(
			bind::<OptionBrand, _, _, _, _>(plus_empty::<OptionBrand, i32>(), f),
			plus_empty::<OptionBrand, i32>(),
		);
	}

	/// Tests the left-zero law for MonadPlus with VecBrand.
	#[test]
	fn left_zero_vec() {
		let f = |n: i32| if n > 0 { vec![n * 2] } else { vec![] };
		assert_eq!(
			bind::<VecBrand, _, _, _, _>(plus_empty::<VecBrand, i32>(), f),
			plus_empty::<VecBrand, i32>(),
		);
	}

	/// Tests the left-zero law for MonadPlus with CatListBrand.
	#[test]
	fn left_zero_cat_list() {
		let f = |n: i32| -> CatList<i32> {
			if n > 0 { CatList::singleton(n * 2) } else { CatList::empty() }
		};
		assert_eq!(
			bind::<CatListBrand, _, _, _, _>(plus_empty::<CatListBrand, i32>(), f),
			plus_empty::<CatListBrand, i32>(),
		);
	}
}
