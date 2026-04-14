//! By-reference variant of [`Foldable`](crate::classes::Foldable).
//!
//! **User story:** "I want to fold over a memoized value without consuming it."
//!
//! This trait is for types like [`Lazy`](crate::types::Lazy) where the container
//! holds a cached value accessible by reference. The closures receive `&A` instead
//! of `A`, avoiding unnecessary cloning.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! 	types::*,
//! };
//!
//! let lazy = RcLazy::new(|| 10);
//! let result =
//! 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32| a.to_string(), &lazy);
//! assert_eq!(result, "10");
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
			types::Endofunction,
		},
		fp_macros::*,
	};

	/// By-reference folding over a structure.
	///
	/// Similar to [`Foldable`], but closures receive `&A` instead of `A`.
	/// This is the honest interface for memoized types like [`Lazy`](crate::types::Lazy)
	/// that internally hold a cached `&A` and would otherwise force a clone
	/// to satisfy the by-value `Foldable` signature.
	///
	/// All three methods (`ref_fold_map`, `ref_fold_right`, `ref_fold_left`)
	/// have default implementations in terms of each other, so implementors
	/// only need to provide one.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFoldable {
		/// Maps values to a monoid by reference and combines them.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the elements.",
			"The brand of the cloneable function to use.",
			"The type of the elements in the structure.",
			"The monoid type."
		)]
		///
		#[document_parameters(
			"The function to map each element reference to a monoid.",
			"The structure to fold."
		)]
		///
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 5);
		/// let result =
		/// 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32| a.to_string(), &lazy);
		/// assert_eq!(result, "5");
		/// ```
		fn ref_fold_map<'a, FnBrand, A: 'a + Clone, M>(
			func: impl Fn(&A) -> M + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M
		where
			FnBrand: LiftFn + 'a,
			M: Monoid + 'a, {
			Self::ref_fold_right::<FnBrand, A, M>(
				move |a: &A, acc| Semigroup::append(func(a), acc),
				Monoid::empty(),
				fa,
			)
		}

		/// Folds the structure from the right by reference.
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
			"The function to apply to each element reference and the accumulator.",
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32, b| *a + b, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn ref_fold_right<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(&A, B) -> B + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a, {
			let f = <FnBrand as LiftFn>::new(move |(a, b): (A, B)| func(&a, b));
			let m = Self::ref_fold_map::<FnBrand, A, Endofunction<FnBrand, B>>(
				move |a: &A| {
					let a = a.clone();
					let f = f.clone();
					Endofunction::<FnBrand, B>::new(<FnBrand as LiftFn>::new(move |b| {
						f((a.clone(), b))
					}))
				},
				fa,
			);
			m.0(initial)
		}

		/// Folds the structure from the left by reference.
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
			"The function to apply to the accumulator and each element reference.",
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_left::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|b, a: &i32| b + *a, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn ref_fold_left<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(B, &A) -> B + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a, {
			let f = <FnBrand as LiftFn>::new(move |(b, a): (B, A)| func(b, &a));
			let m = Self::ref_fold_right::<FnBrand, A, Endofunction<FnBrand, B>>(
				move |a: &A, k: Endofunction<'a, FnBrand, B>| {
					let a = a.clone();
					let f = f.clone();
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
	}
}

pub use inner::*;
