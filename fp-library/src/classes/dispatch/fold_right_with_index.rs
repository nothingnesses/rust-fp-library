//! Dispatch for [`FoldableWithIndex::fold_right_with_index`](crate::classes::FoldableWithIndex::fold_right_with_index) and
//! [`RefFoldableWithIndex::ref_fold_right_with_index`](crate::classes::RefFoldableWithIndex::ref_fold_right_with_index).
//!
//! Provides the [`FoldRightWithIndexDispatch`] trait and a unified [`fold_right_with_index`] free function
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
//! // By-value fold_right_with_index (Vec)
//! let result = fold_right_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
//! 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
//! 	String::new(),
//! 	vec![10, 20, 30],
//! );
//! assert_eq!(result, "2:30,1:20,0:10,");
//!
//! // By-ref fold_right_with_index (Lazy, closure receives &A)
//! let lazy = RcLazy::new(|| 10);
//! let result = fold_right_with_index::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
//! 	|_, x: &i32, acc| acc + *x,
//! 	0,
//! 	&lazy,
//! );
//! assert_eq!(result, 10);
//! ```

#[fp_macros::document_module]

pub(crate) mod inner {
	use {
		crate::{
			classes::{
				FoldableWithIndex,
				LiftFn,
				RefFoldableWithIndex,
				WithIndex,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- FoldRightWithIndexDispatch --

	/// Trait that routes a fold_right_with_index operation to the appropriate type class method.
	///
	/// `Fn(Brand::Index, A, B) -> B` resolves to [`Val`], `Fn(Brand::Index, &A, B) -> B` resolves to [`Ref`].
	/// The `FA` type parameter is inferred from the container argument: owned
	/// for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The type of the accumulator.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FoldRightWithIndexDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		B: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched fold_right_with_index operation.
		#[document_signature]
		#[document_parameters("The initial accumulator value.", "The structure to fold.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = fold_right_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
		/// 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
		/// 	String::new(),
		/// 	vec![10, 20, 30],
		/// );
		/// assert_eq!(result, "2:30,1:20,0:10,");
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: FA,
		) -> B;
	}

	/// Routes `Fn(Brand::Index, A, B) -> B` closures to [`FoldableWithIndex::fold_right_with_index`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The accumulator type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, B, F>
		FoldRightWithIndexDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: FoldableWithIndex,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, A, B) -> B + 'a,
	{
		#[document_signature]
		#[document_parameters("The initial accumulator value.", "The structure to fold.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		/// let result = fold_right_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
		/// 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
		/// 	String::new(),
		/// 	vec![10, 20, 30],
		/// );
		/// assert_eq!(result, "2:30,1:20,0:10,");
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::fold_right_with_index::<FnBrand, A, B>(self, initial, fa)
		}
	}

	/// Routes `Fn(Brand::Index, &A, B) -> B` closures to [`RefFoldableWithIndex::ref_fold_right_with_index`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The accumulator type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, 'b, FnBrand, Brand, A, B, F>
		FoldRightWithIndexDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFoldableWithIndex,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, &A, B) -> B + 'a,
	{
		#[document_signature]
		#[document_parameters(
			"The initial accumulator value.",
			"A reference to the structure to fold."
		)]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 10);
		/// let result = fold_right_with_index::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
		/// 	|_, x: &i32, acc| acc + *x,
		/// 	0,
		/// 	&lazy,
		/// );
		/// assert_eq!(result, 10);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::ref_fold_right_with_index::<FnBrand, A, B>(self, initial, fa)
		}
	}

	/// Folds a structure from the right with index.
	///
	/// Dispatches to [`FoldableWithIndex::fold_right_with_index`] or
	/// [`RefFoldableWithIndex::ref_fold_right_with_index`] based on whether
	/// the closure takes `A` or `&A`.
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by the
	/// compiler from the closure's argument type and the container argument.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The type of the accumulator.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters(
		"The folding function that receives an index, element, and accumulator.",
		"The initial accumulator value.",
		"The structure to fold (owned for Val, borrowed for Ref)."
	)]
	#[document_returns("The final accumulator value.")]
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
	/// let result = fold_right_with_index::<RcFnBrand, VecBrand, _, _, _, _>(
	/// 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
	/// 	String::new(),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "2:30,1:20,0:10,");
	///
	/// // By-ref
	/// let lazy = RcLazy::new(|| 10);
	/// let result = fold_right_with_index::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(
	/// 	|_, x: &i32, acc| acc + *x,
	/// 	0,
	/// 	&lazy,
	/// );
	/// assert_eq!(result, 10);
	/// ```
	pub fn fold_right_with_index<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a + Clone,
		B: 'a,
		FA,
		Marker,
	>(
		func: impl FoldRightWithIndexDispatch<'a, FnBrand, Brand, A, B, FA, Marker>,
		initial: B,
		fa: FA,
	) -> B {
		func.dispatch(initial, fa)
	}
}

pub use inner::*;
