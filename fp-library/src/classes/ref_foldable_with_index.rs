//! By-reference variant of [`FoldableWithIndex`](crate::classes::FoldableWithIndex).
//!
//! **User story:** "I want to fold over a memoized value by reference, with access to the index."
//!
//! All three methods (`ref_fold_map_with_index`, `ref_fold_right_with_index`,
//! `ref_fold_left_with_index`) have default implementations in terms of each other,
//! so implementors only need to provide one.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::ref_foldable_with_index::RefFoldableWithIndex,
//! 	types::*,
//! };
//!
//! let lazy = RcLazy::new(|| 42);
//! let result = <LazyBrand<RcLazyConfig> as RefFoldableWithIndex>::ref_fold_map_with_index::<
//! 	RcFnBrand,
//! 	_,
//! 	_,
//! >(|_, x: &i32| x.to_string(), &lazy);
//! assert_eq!(result, "42");
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::*,
			kinds::*,
			types::{
				Dual,
				Endofunction,
			},
		},
		fp_macros::*,
	};

	/// By-reference folding with index over a structure.
	///
	/// Similar to [`FoldableWithIndex`], but the closure receives `&A` instead of `A`.
	/// This is the honest interface for memoized types like [`Lazy`](crate::types::Lazy)
	/// that internally hold a cached `&A`.
	///
	/// All three methods (`ref_fold_map_with_index`, `ref_fold_right_with_index`,
	/// `ref_fold_left_with_index`) have default implementations in terms of each other,
	/// so implementors only need to provide one.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait RefFoldableWithIndex: RefFoldable + WithIndex {
		/// Maps each element of the structure to a monoid by reference,
		/// providing the index, and combines the results.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The monoid type."
		)]
		#[document_parameters(
			"The function to apply to each element's index and reference.",
			"The structure to fold over."
		)]
		#[document_returns("The combined result.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_foldable_with_index::RefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 42);
		/// let result = <LazyBrand<RcLazyConfig> as RefFoldableWithIndex>::ref_fold_map_with_index::<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|_, x: &i32| x.to_string(), &lazy);
		/// assert_eq!(result, "42");
		/// ```
		fn ref_fold_map_with_index<'a, FnBrand, A: 'a + Clone, R: Monoid + 'a>(
			f: impl Fn(Self::Index, &A) -> R + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: LiftFn + 'a,
			Self::Index: 'a, {
			Self::ref_fold_right_with_index::<FnBrand, A, R>(
				move |i, a: &A, acc| Semigroup::append(f(i, a), acc),
				Monoid::empty(),
				fa,
			)
		}

		/// Folds the structure from the right by reference, providing the index.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The type of the accumulator."
		)]
		#[document_parameters(
			"The function to apply to each element's index, reference, and accumulator.",
			"The initial value of the accumulator.",
			"The structure to fold over."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_foldable_with_index::RefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 10);
		/// let result = <LazyBrand<RcLazyConfig> as RefFoldableWithIndex>::ref_fold_right_with_index::<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|_, x: &i32, acc: i32| acc + *x, 0, &lazy);
		/// assert_eq!(result, 10);
		/// ```
		fn ref_fold_right_with_index<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(Self::Index, &A, B) -> B + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a,
			Self::Index: 'a, {
			let f = <FnBrand as LiftFn>::new(move |(i, a, b): (Self::Index, A, B)| func(i, &a, b));
			let m = Self::ref_fold_map_with_index::<FnBrand, A, Endofunction<FnBrand, B>>(
				move |i, a: &A| {
					let a = a.clone();
					let f = f.clone();
					Endofunction::<FnBrand, B>::new(<FnBrand as LiftFn>::new(move |b| {
						let a = a.clone();
						let i = i.clone();
						f((i, a, b))
					}))
				},
				fa,
			);
			m.0(initial)
		}

		/// Folds the structure from the left by reference, providing the index.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of the values.",
			"The brand of the cloneable function to use.",
			"The type of the elements.",
			"The type of the accumulator."
		)]
		#[document_parameters(
			"The function to apply to the accumulator, each element's index, and reference.",
			"The initial value of the accumulator.",
			"The structure to fold over."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::ref_foldable_with_index::RefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = RcLazy::new(|| 10);
		/// let result = <LazyBrand<RcLazyConfig> as RefFoldableWithIndex>::ref_fold_left_with_index::<
		/// 	RcFnBrand,
		/// 	_,
		/// 	_,
		/// >(|_, acc: i32, x: &i32| acc + *x, 0, &lazy);
		/// assert_eq!(result, 10);
		/// ```
		fn ref_fold_left_with_index<'a, FnBrand, A: 'a + Clone, B: 'a>(
			func: impl Fn(Self::Index, B, &A) -> B + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: LiftFn + 'a,
			Self::Index: 'a, {
			let f = <FnBrand as LiftFn>::new(move |(i, b, a): (Self::Index, B, A)| func(i, b, &a));
			let m = Self::ref_fold_map_with_index::<FnBrand, A, Dual<Endofunction<FnBrand, B>>>(
				move |i, a: &A| {
					let a = a.clone();
					let f = f.clone();
					Dual(Endofunction::<FnBrand, B>::new(<FnBrand as LiftFn>::new(move |b| {
						let a = a.clone();
						let i = i.clone();
						f((i, b, a))
					})))
				},
				fa,
			);
			(m.0).0(initial)
		}
	}

	/// Maps each element to a monoid by reference with its index and combines the results.
	///
	/// Free function version that dispatches to [the type class' associated function][`RefFoldableWithIndex::ref_fold_map_with_index`].
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the structure.",
		"The type of the elements.",
		"The monoid type."
	)]
	#[document_parameters(
		"The function to apply to each element's index and reference.",
		"The structure to fold over."
	)]
	#[document_returns("The combined result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::explicit::*,
	/// 	types::*,
	/// };
	///
	/// let lazy = RcLazy::new(|| 42);
	/// let result = fold_map_with_index::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
	/// 	|_, x: &i32| x.to_string(),
	/// 	&lazy,
	/// );
	/// assert_eq!(result, "42");
	/// ```
	pub fn ref_fold_map_with_index<
		'a,
		FnBrand: LiftFn + 'a,
		Brand: RefFoldableWithIndex,
		A: 'a + Clone,
		R: Monoid + 'a,
	>(
		f: impl Fn(Brand::Index, &A) -> R + 'a,
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> R
	where
		Brand::Index: 'a, {
		Brand::ref_fold_map_with_index::<FnBrand, A, R>(f, fa)
	}
}

pub use inner::*;
