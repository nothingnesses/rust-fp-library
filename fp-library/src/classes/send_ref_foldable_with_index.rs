//! Thread-safe by-reference variant of [`FoldableWithIndex`](crate::classes::FoldableWithIndex).
//!
//! **User story:** "I want to fold over a thread-safe memoized value by reference, with access to the index."
//!
//! All three methods (`send_ref_fold_map_with_index`, `send_ref_fold_right_with_index`,
//! `send_ref_fold_left_with_index`) have default implementations in terms of each other,
//! so implementors only need to provide one.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::send_ref_foldable_with_index::SendRefFoldableWithIndex,
//! 	types::*,
//! };
//!
//! let lazy = ArcLazy::new(|| 42);
//! let result =
//! 	<LazyBrand<ArcLazyConfig> as SendRefFoldableWithIndex>::send_ref_fold_map_with_index::<
//! 		ArcFnBrand,
//! 		_,
//! 		_,
//! 	>(|_, x: &i32| x.to_string(), &lazy);
//! assert_eq!(result, "42");
//! ```

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::{
				send_clone_fn::SendLiftFn,
				*,
			},
			kinds::*,
			types::{
				Dual,
				SendEndofunction,
			},
		},
		fp_macros::*,
	};

	/// Thread-safe by-reference folding with index over a structure.
	///
	/// Similar to [`RefFoldableWithIndex`], but closures and elements must be `Send + Sync`.
	///
	/// All three methods (`send_ref_fold_map_with_index`, `send_ref_fold_right_with_index`,
	/// `send_ref_fold_left_with_index`) have default implementations in terms of each other,
	/// so implementors only need to provide one.
	#[kind(type Of<'a, A: 'a>: 'a;)]
	pub trait SendRefFoldableWithIndex: SendRefFoldable + WithIndex {
		/// Maps each element to a monoid by reference with index (thread-safe).
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
		/// 	classes::send_ref_foldable_with_index::SendRefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 42);
		/// let result =
		/// 	<LazyBrand<ArcLazyConfig> as SendRefFoldableWithIndex>::send_ref_fold_map_with_index::<
		/// 		ArcFnBrand,
		/// 		_,
		/// 		_,
		/// 	>(|_, x: &i32| x.to_string(), &lazy);
		/// assert_eq!(result, "42");
		/// ```
		fn send_ref_fold_map_with_index<
			'a,
			FnBrand,
			A: Send + Sync + 'a + Clone,
			R: Monoid + Send + Sync + 'a,
		>(
			f: impl Fn(Self::Index, &A) -> R + Send + Sync + 'a,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> R
		where
			FnBrand: SendLiftFn + 'a,
			Self::Index: Send + Sync + 'a, {
			Self::send_ref_fold_right_with_index::<FnBrand, A, R>(
				move |i, a: &A, acc| Semigroup::append(f(i, a), acc),
				Monoid::empty(),
				fa,
			)
		}

		/// Folds the structure from the right by reference with index (thread-safe).
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
		/// 	classes::send_ref_foldable_with_index::SendRefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 10);
		/// let result =
		/// 	<LazyBrand<ArcLazyConfig> as SendRefFoldableWithIndex>::send_ref_fold_right_with_index::<
		/// 		ArcFnBrand,
		/// 		_,
		/// 		_,
		/// 	>(|_, x: &i32, acc: i32| acc + *x, 0, &lazy);
		/// assert_eq!(result, 10);
		/// ```
		fn send_ref_fold_right_with_index<
			'a,
			FnBrand,
			A: Send + Sync + 'a + Clone,
			B: Send + Sync + 'a,
		>(
			func: impl Fn(Self::Index, &A, B) -> B + Send + Sync + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: SendLiftFn + 'a,
			Self::Index: Send + Sync + 'a, {
			let f =
				<FnBrand as SendLiftFn>::new(move |(i, a, b): (Self::Index, A, B)| func(i, &a, b));
			let m = Self::send_ref_fold_map_with_index::<FnBrand, A, SendEndofunction<FnBrand, B>>(
				move |i, a: &A| {
					let a = a.clone();
					let f = f.clone();
					SendEndofunction::<FnBrand, B>::new(<FnBrand as SendLiftFn>::new(move |b| {
						let a = a.clone();
						let i = i.clone();
						f((i, a, b))
					}))
				},
				fa,
			);
			m.0(initial)
		}

		/// Folds the structure from the left by reference with index (thread-safe).
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
		/// 	classes::send_ref_foldable_with_index::SendRefFoldableWithIndex,
		/// 	types::*,
		/// };
		///
		/// let lazy = ArcLazy::new(|| 10);
		/// let result =
		/// 	<LazyBrand<ArcLazyConfig> as SendRefFoldableWithIndex>::send_ref_fold_left_with_index::<
		/// 		ArcFnBrand,
		/// 		_,
		/// 		_,
		/// 	>(|acc: i32, _, x: &i32| acc + *x, 0, &lazy);
		/// assert_eq!(result, 10);
		/// ```
		fn send_ref_fold_left_with_index<
			'a,
			FnBrand,
			A: Send + Sync + 'a + Clone,
			B: Send + Sync + 'a,
		>(
			func: impl Fn(B, Self::Index, &A) -> B + Send + Sync + 'a,
			initial: B,
			fa: &Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B
		where
			FnBrand: SendLiftFn + 'a,
			Self::Index: Send + Sync + 'a, {
			let f =
				<FnBrand as SendLiftFn>::new(move |(b, i, a): (B, Self::Index, A)| func(b, i, &a));
			let m = Self::send_ref_fold_map_with_index::<
				FnBrand,
				A,
				Dual<SendEndofunction<FnBrand, B>>,
			>(
				move |i, a: &A| {
					let a = a.clone();
					let f = f.clone();
					Dual(SendEndofunction::<FnBrand, B>::new(<FnBrand as SendLiftFn>::new(
						move |b| {
							let a = a.clone();
							let i = i.clone();
							f((b, i, a))
						},
					)))
				},
				fa,
			);
			(m.0).0(initial)
		}
	}

	/// Maps each element to a monoid by reference with its index (thread-safe).
	///
	/// Free function version that dispatches to [the type class' associated function][`SendRefFoldableWithIndex::send_ref_fold_map_with_index`].
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
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// let lazy = ArcLazy::new(|| 42);
	/// let result = send_ref_fold_map_with_index::<ArcFnBrand, LazyBrand<ArcLazyConfig>, _, _>(
	/// 	|_, x: &i32| x.to_string(),
	/// 	&lazy,
	/// );
	/// assert_eq!(result, "42");
	/// ```
	pub fn send_ref_fold_map_with_index<
		'a,
		FnBrand: SendLiftFn + 'a,
		Brand: SendRefFoldableWithIndex,
		A: Send + Sync + 'a + Clone,
		R: Monoid + Send + Sync + 'a,
	>(
		f: impl Fn(Brand::Index, &A) -> R + Send + Sync + 'a,
		fa: &Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> R
	where
		Brand::Index: Send + Sync + 'a, {
		Brand::send_ref_fold_map_with_index::<FnBrand, A, R>(f, fa)
	}
}

pub use inner::*;
