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
//! let y = fold_right_explicit::<RcFnBrand, OptionBrand, _, _, _, _>(|a, b| a + b, 10, x);
//! assert_eq!(y, 15);
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
			types::*,
		},
		fp_macros::*,
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
	///
	/// ### Laws
	///
	/// `Foldable` instances must be internally consistent:
	/// * fold_map/fold_right consistency: `fold_map(f, fa) = fold_right(|a, m| append(f(a), m), empty(), fa)`.
	#[document_examples]
	///
	/// Foldable laws for [`Vec`]:
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let xs = vec![1, 2, 3];
	/// let f = |a: i32| a.to_string();
	///
	/// // fold_map/fold_right consistency:
	/// // fold_map(f, fa) = fold_right(|a, m| append(f(a), m), empty(), fa)
	/// assert_eq!(
	/// 	fold_map_explicit::<RcFnBrand, VecBrand, _, _, _, _>(f, xs.clone()),
	/// 	fold_right_explicit::<RcFnBrand, VecBrand, _, _, _, _>(
	/// 		|a: i32, m: String| append(f(a), m),
	/// 		empty::<String>(),
	/// 		xs,
	/// 	),
	/// );
	/// ```
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait Foldable {
		/// Folds the structure by applying a function from right to left.
		///
		/// This method performs a right-associative fold of the structure.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to each element and the accumulator.",
			"The initial value of the accumulator.",
			"The structure to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = fold_right_explicit::<RcFnBrand, OptionBrand, _, _, _, _>(|a, b| a + b, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(A, B) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a, {
			let f = <FnBrand as LiftFn>::new(move |(a, b)| func(a, b));
			let m = Self::fold_map::<FnBrand, A, Endofunction<FnBrand, B>>(
				move |a: A| {
					let f = f.clone();
					Endofunction::<FnBrand, B>::new(<FnBrand as LiftFn>::new(move |b| {
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
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the accumulator."
		)]
		///
		#[document_parameters(
			"The function to apply to the accumulator and each element.",
			"The initial value of the accumulator.",
			"The structure to fold."
		)]
		///
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = fold_left_explicit::<RcFnBrand, OptionBrand, _, _, _, _>(|b, a| b + a, 10, x);
		/// assert_eq!(y, 15);
		/// ```
		fn fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, A) -> B + 'a,
			initial: B,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a, {
			let f = <FnBrand as LiftFn>::new(move |(b, a)| func(b, a));
			let m = Self::fold_right::<FnBrand, A, Endofunction<FnBrand, B>>(
				move |a: A, k: Endofunction<'a, FnBrand, B>| {
					let f = f.clone();
					// k is the "rest" of the computation.
					// We want to perform "current" (f(b, a)) then "rest".
					// Endofunction composition is f . g (f after g).
					// So we want k . current.
					// append(k, current).
					let current =
						Endofunction::<FnBrand, B>::new(<FnBrand as LiftFn>::new(move |b| {
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
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The type of the monoid."
		)]
		///
		#[document_parameters(
			"The function to map each element to a monoid.",
			"The structure to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let x = Some(5);
		/// let y = fold_map_explicit::<RcFnBrand, OptionBrand, _, _, _, _>(|a: i32| a.to_string(), x);
		/// assert_eq!(y, "5".to_string());
		/// ```
		fn fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(A) -> M + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			M: Monoid + 'a,
			FnBrand: LiftFn + 'a, {
			Self::fold_right::<FnBrand, A, M>(move |a, m| M::append(func(a), m), M::empty(), fa)
		}
	}
}

pub use inner::*;
