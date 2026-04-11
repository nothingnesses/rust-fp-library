//! Dispatch for [`FoldableWithIndex::fold_map_with_index`](crate::classes::FoldableWithIndex::fold_map_with_index) and
//! [`RefFoldableWithIndex::ref_fold_map_with_index`](crate::classes::RefFoldableWithIndex::ref_fold_map_with_index).
//!
//! Provides the [`FoldMapWithIndexDispatch`] trait and a unified [`fold_map_with_index`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! 	types::*,
//! };
//!
//! // By-value fold_map_with_index (Vec)
//! let result = fold_map_with_index_explicit::<RcFnBrand, VecBrand, _, _, _, _>(
//! 	|i, x: i32| format!("{i}:{x}"),
//! 	vec![10, 20, 30],
//! );
//! assert_eq!(result, "0:101:202:30");
//!
//! // By-ref fold_map_with_index (Lazy, closure receives &A)
//! let lazy = RcLazy::new(|| 42);
//! let result = fold_map_with_index_explicit::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
//! 	|_, x: &i32| x.to_string(),
//! 	&lazy,
//! );
//! assert_eq!(result, "42");
//! ```

#[fp_macros::document_module]

pub(crate) mod inner {
	use {
		crate::{
			classes::{
				FoldableWithIndex,
				LiftFn,
				Monoid,
				RefFoldableWithIndex,
				WithIndex,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- FoldMapWithIndexDispatch --

	/// Trait that routes a fold_map_with_index operation to the appropriate type class method.
	///
	/// `Fn(Brand::Index, A) -> M` resolves to [`Val`], `Fn(Brand::Index, &A) -> M` resolves to [`Ref`].
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The monoid type.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FoldMapWithIndexDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		M,
		FA,
		Marker,
	> {
		/// Perform the dispatched fold_map_with_index operation.
		#[document_signature]
		#[document_parameters("The structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = fold_map_with_index_explicit::<RcFnBrand, VecBrand, _, _, _, _>(
		/// 	|i, x: i32| format!("{i}:{x}"),
		/// 	vec![10, 20, 30],
		/// );
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> M;
	}

	/// Routes `Fn(Brand::Index, A) -> M` closures to [`FoldableWithIndex::fold_map_with_index`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The monoid type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, M, F>
		FoldMapWithIndexDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			M,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: FoldableWithIndex,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		M: Monoid + 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, A) -> M + 'a,
	{
		#[document_signature]
		#[document_parameters("The structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = fold_map_with_index_explicit::<RcFnBrand, VecBrand, _, _, _, _>(
		/// 	|i, x: i32| format!("{i}:{x}"),
		/// 	vec![10, 20, 30],
		/// );
		/// assert_eq!(result, "0:101:202:30");
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			Brand::fold_map_with_index::<FnBrand, A, M>(self, fa)
		}
	}

	/// Routes `Fn(Brand::Index, &A) -> M` closures to [`RefFoldableWithIndex::ref_fold_map_with_index`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The monoid type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, 'b, FnBrand, Brand, A, M, F>
		FoldMapWithIndexDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			M,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFoldableWithIndex,
		FnBrand: LiftFn + 'a,
		A: Clone + 'a,
		M: Monoid + 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, &A) -> M + 'a,
	{
		#[document_signature]
		#[document_parameters("A reference to the structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 42);
		/// let result = fold_map_with_index_explicit::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
		/// 	|_, x: &i32| x.to_string(),
		/// 	&lazy,
		/// );
		/// assert_eq!(result, "42");
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			Brand::ref_fold_map_with_index::<FnBrand, A, M>(self, fa)
		}
	}

	/// Maps values with their index to a monoid and combines them.
	///
	/// Dispatches to [`FoldableWithIndex::fold_map_with_index`] or
	/// [`RefFoldableWithIndex::ref_fold_map_with_index`] based on whether the
	/// closure takes `A` or `&A`.
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by the
	/// compiler from the closure's argument type and the container argument.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The monoid type.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters(
		"The mapping function that receives an index and element.",
		"The structure to fold (owned for Val, borrowed for Ref)."
	)]
	#[document_returns("The combined monoid value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// 	types::*,
	/// };
	///
	/// // By-value
	/// let result = fold_map_with_index_explicit::<RcFnBrand, VecBrand, _, _, _, _>(
	/// 	|i, x: i32| format!("{i}:{x}"),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "0:101:202:30");
	///
	/// // By-ref
	/// let lazy = RcLazy::new(|| 42);
	/// let result = fold_map_with_index_explicit::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
	/// 	|_, x: &i32| x.to_string(),
	/// 	&lazy,
	/// );
	/// assert_eq!(result, "42");
	/// ```
	pub fn fold_map_with_index<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		M: Monoid + 'a,
		FA,
		Marker,
	>(
		func: impl FoldMapWithIndexDispatch<'a, FnBrand, Brand, A, M, FA, Marker>,
		fa: FA,
	) -> M {
		func.dispatch(fa)
	}
}

pub use inner::*;
