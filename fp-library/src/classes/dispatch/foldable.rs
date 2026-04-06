//! Dispatch for [`Foldable::fold_right`](crate::classes::Foldable::fold_right),
//! [`fold_left`](crate::classes::Foldable::fold_left),
//! [`fold_map`](crate::classes::Foldable::fold_map), and their by-reference
//! counterparts in [`RefFoldable`](crate::classes::RefFoldable).
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
//! // By-value fold_right (Vec)
//! let result = fold_right::<RcFnBrand, VecBrand, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
//! assert_eq!(result, 6);
//!
//! // By-ref fold_right (Lazy, closure receives &A)
//! let lazy = RcLazy::new(|| 10);
//! let result =
//! 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _>(|a: &i32, b| *a + b, 5, lazy);
//! assert_eq!(result, 15);
//! ```

#[fp_macros::document_module]

pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Foldable,
				LiftFn,
				Monoid,
				RefFoldable,
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- FoldRightDispatch --

	/// Trait that routes a fold_right operation to the appropriate type class method.
	///
	/// `Fn(A, B) -> B` resolves to [`Val`], `Fn(&A, B) -> B` resolves to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FoldRightDispatch<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Perform the dispatched fold_right operation.
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
		/// let result = fold_right::<RcFnBrand, VecBrand, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B;
	}

	/// Routes `Fn(A, B) -> B` closures to [`Foldable::fold_right`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The accumulator type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, B, F> FoldRightDispatch<'a, FnBrand, Brand, A, B, Val> for F
	where
		Brand: Foldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		F: Fn(A, B) -> B + 'a,
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
		/// let result = fold_right::<RcFnBrand, VecBrand, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::fold_right::<FnBrand, A, B>(self, initial, fa)
		}
	}

	/// Routes `Fn(&A, B) -> B` closures to [`RefFoldable::ref_fold_right`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The accumulator type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, B, F> FoldRightDispatch<'a, FnBrand, Brand, A, B, Ref> for F
	where
		Brand: RefFoldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		F: Fn(&A, B) -> B + 'a,
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
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _>(|a: &i32, b| *a + b, 5, lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::ref_fold_right::<FnBrand, A, B>(self, initial, fa)
		}
	}

	/// Folds a structure from the right.
	///
	/// Dispatches to [`Foldable::fold_right`] or [`RefFoldable::ref_fold_right`]
	/// based on whether the closure takes `A` or `&A`.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters(
		"The folding function.",
		"The initial accumulator value.",
		"The structure to fold."
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
	/// let result = fold_right::<RcFnBrand, VecBrand, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
	/// assert_eq!(result, 6);
	///
	/// // By-ref
	/// let lazy = RcLazy::new(|| 10);
	/// let result =
	/// 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _>(|a: &i32, b| *a + b, 5, lazy);
	/// assert_eq!(result, 15);
	/// ```
	pub fn fold_right<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, B: 'a, Marker>(
		func: impl FoldRightDispatch<'a, FnBrand, Brand, A, B, Marker>,
		initial: B,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B {
		func.dispatch(initial, fa)
	}

	// -- FoldLeftDispatch --

	/// Trait that routes a fold_left operation to the appropriate type class method.
	///
	/// `Fn(B, A) -> B` resolves to [`Val`], `Fn(B, &A) -> B` resolves to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FoldLeftDispatch<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker> {
		/// Perform the dispatched fold_left operation.
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
		/// let result = fold_left::<RcFnBrand, VecBrand, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B;
	}

	/// Routes `Fn(B, A) -> B` closures to [`Foldable::fold_left`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The accumulator type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, B, F> FoldLeftDispatch<'a, FnBrand, Brand, A, B, Val> for F
	where
		Brand: Foldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		F: Fn(B, A) -> B + 'a,
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
		/// let result = fold_left::<RcFnBrand, VecBrand, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::fold_left::<FnBrand, A, B>(self, initial, fa)
		}
	}

	/// Routes `Fn(B, &A) -> B` closures to [`RefFoldable::ref_fold_left`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The accumulator type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, B, F> FoldLeftDispatch<'a, FnBrand, Brand, A, B, Ref> for F
	where
		Brand: RefFoldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		F: Fn(B, &A) -> B + 'a,
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
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_left::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _>(|b, a: &i32| b + *a, 5, lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::ref_fold_left::<FnBrand, A, B>(self, initial, fa)
		}
	}

	/// Folds a structure from the left.
	///
	/// Dispatches to [`Foldable::fold_left`] or [`RefFoldable::ref_fold_left`]
	/// based on whether the closure takes `A` or `&A`.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters(
		"The folding function.",
		"The initial accumulator value.",
		"The structure to fold."
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
	/// let result = fold_left::<RcFnBrand, VecBrand, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
	/// assert_eq!(result, 6);
	///
	/// // By-ref
	/// let lazy = RcLazy::new(|| 10);
	/// let result =
	/// 	fold_left::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _>(|b, a: &i32| b + *a, 5, lazy);
	/// assert_eq!(result, 15);
	/// ```
	pub fn fold_left<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, B: 'a, Marker>(
		func: impl FoldLeftDispatch<'a, FnBrand, Brand, A, B, Marker>,
		initial: B,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> B {
		func.dispatch(initial, fa)
	}

	// -- FoldMapDispatch --

	/// Trait that routes a fold_map operation to the appropriate type class method.
	///
	/// `Fn(A) -> M` resolves to [`Val`], `Fn(&A) -> M` resolves to [`Ref`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The monoid type.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FoldMapDispatch<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a, M, Marker> {
		/// Perform the dispatched fold_map operation.
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
		/// let result = fold_map::<RcFnBrand, VecBrand, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M;
	}

	/// Routes `Fn(A) -> M` closures to [`Foldable::fold_map`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The monoid type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, M, F> FoldMapDispatch<'a, FnBrand, Brand, A, M, Val> for F
	where
		Brand: Foldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		M: Monoid + 'a,
		F: Fn(A) -> M + 'a,
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
		/// let result = fold_map::<RcFnBrand, VecBrand, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			Brand::fold_map::<FnBrand, A, M>(self, fa)
		}
	}

	/// Routes `Fn(&A) -> M` closures to [`RefFoldable::ref_fold_map`].
	#[document_type_parameters(
		"The lifetime.",
		"The cloneable function brand.",
		"The foldable brand.",
		"The element type.",
		"The monoid type.",
		"The closure type."
	)]
	#[document_parameters("The closure.")]
	impl<'a, FnBrand, Brand, A, M, F> FoldMapDispatch<'a, FnBrand, Brand, A, M, Ref> for F
	where
		Brand: RefFoldable,
		FnBrand: LiftFn + 'a,
		A: Clone + 'a,
		M: Monoid + 'a,
		F: Fn(&A) -> M + 'a,
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
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _>(|a: &i32| a.to_string(), lazy);
		/// assert_eq!(result, "10");
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			Brand::ref_fold_map::<FnBrand, A, M>(self, fa)
		}
	}

	/// Maps values to a monoid and combines them.
	///
	/// Dispatches to [`Foldable::fold_map`] or [`RefFoldable::ref_fold_map`]
	/// based on whether the closure takes `A` or `&A`.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the foldable structure.",
		"The type of the elements.",
		"The monoid type.",
		"Dispatch marker type, inferred automatically."
	)]
	#[document_parameters("The mapping function.", "The structure to fold.")]
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
	/// let result = fold_map::<RcFnBrand, VecBrand, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
	/// assert_eq!(result, "123");
	///
	/// // By-ref
	/// let lazy = RcLazy::new(|| 10);
	/// let result =
	/// 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _>(|a: &i32| a.to_string(), lazy);
	/// assert_eq!(result, "10");
	/// ```
	pub fn fold_map<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a, M: Monoid + 'a, Marker>(
		func: impl FoldMapDispatch<'a, FnBrand, Brand, A, M, Marker>,
		fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> M {
		func.dispatch(fa)
	}
}

pub use inner::*;
