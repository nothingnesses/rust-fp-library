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
//! 	functions::explicit::*,
//! 	types::*,
//! };
//!
//! // By-value fold_right (Vec)
//! let result = fold_right::<RcFnBrand, VecBrand, _, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
//! assert_eq!(result, 6);
//!
//! // By-ref fold_right (Lazy, closure receives &A)
//! let lazy = RcLazy::new(|| 10);
//! let result =
//! 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32, b| *a + b, 5, &lazy);
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
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- FoldRightDispatch --

	/// Trait that routes a fold_right operation to the appropriate type class method.
	///
	/// `Fn(A, B) -> B` resolves to [`Val`], `Fn(&A, B) -> B` resolves to [`Ref`].
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
	pub trait FoldRightDispatch<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>
	{
		/// Perform the dispatched fold_right operation.
		#[document_signature]
		#[document_parameters("The initial accumulator value.", "The structure to fold.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = fold_right::<RcFnBrand, VecBrand, _, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: FA,
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
	impl<'a, FnBrand, Brand, A, B, F>
		FoldRightDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
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
		/// 	functions::explicit::*,
		/// };
		/// let result = fold_right::<RcFnBrand, VecBrand, _, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
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
		FoldRightDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFoldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		F: Fn(&A, B) -> B + 'a,
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32, b| *a + b, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::ref_fold_right::<FnBrand, A, B>(self, initial, fa)
		}
	}

	// -- FoldLeftDispatch --

	/// Trait that routes a fold_left operation to the appropriate type class method.
	///
	/// `Fn(B, A) -> B` resolves to [`Val`], `Fn(B, &A) -> B` resolves to [`Ref`].
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
	pub trait FoldLeftDispatch<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>
	{
		/// Perform the dispatched fold_left operation.
		#[document_signature]
		#[document_parameters("The initial accumulator value.", "The structure to fold.")]
		#[document_returns("The final accumulator value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = fold_left::<RcFnBrand, VecBrand, _, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: FA,
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
	impl<'a, FnBrand, Brand, A, B, F>
		FoldLeftDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
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
		/// 	functions::explicit::*,
		/// };
		/// let result = fold_left::<RcFnBrand, VecBrand, _, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
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
		FoldLeftDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFoldable,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a,
		F: Fn(B, &A) -> B + 'a,
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
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_left::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|b, a: &i32| b + *a, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		fn dispatch(
			self,
			initial: B,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> B {
			Brand::ref_fold_left::<FnBrand, A, B>(self, initial, fa)
		}
	}

	// -- FoldMapDispatch --

	/// Trait that routes a fold_map operation to the appropriate type class method.
	///
	/// `Fn(A) -> M` resolves to [`Val`], `Fn(&A) -> M` resolves to [`Ref`].
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
	pub trait FoldMapDispatch<'a, FnBrand, Brand: Kind_cdc7cd43dac7585f, A: 'a, M, FA, Marker> {
		/// Perform the dispatched fold_map operation.
		#[document_signature]
		#[document_parameters("The structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		/// let result = fold_map::<RcFnBrand, VecBrand, _, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		/// ```
		fn dispatch(
			self,
			fa: FA,
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
	impl<'a, FnBrand, Brand, A, M, F>
		FoldMapDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			M,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
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
		/// 	functions::explicit::*,
		/// };
		/// let result = fold_map::<RcFnBrand, VecBrand, _, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
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
		FoldMapDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			M,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFoldable,
		FnBrand: LiftFn + 'a,
		A: Clone + 'a,
		M: Monoid + 'a,
		F: Fn(&A) -> M + 'a,
	{
		#[document_signature]
		#[document_parameters("A reference to the structure to fold.")]
		#[document_returns("The combined monoid value.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// 	types::*,
		/// };
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32| a.to_string(), &lazy);
		/// assert_eq!(result, "10");
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> M {
			Brand::ref_fold_map::<FnBrand, A, M>(self, fa)
		}
	}

	// -- Inference wrappers --

	/// Folds a structure from the right, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::fold_right`](crate::functions::explicit::fold_right) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The type of the accumulator.",
		"The brand, inferred via InferableBrand from FA and the element type."
	)]
	///
	#[document_parameters(
		"The folding function.",
		"The initial accumulator value.",
		"The structure to fold (owned for Val, borrowed for Ref)."
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
	/// let result = fold_right::<RcFnBrand, _, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
	/// assert_eq!(result, 6);
	/// ```
	pub fn fold_right<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Brand>(
		func: impl FoldRightDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		initial: B,
		fa: FA,
	) -> B
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		func.dispatch(initial, fa)
	}

	/// Folds a structure from the left, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::fold_left`](crate::functions::explicit::fold_left) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The type of the accumulator.",
		"The brand, inferred via InferableBrand from FA and the element type."
	)]
	///
	#[document_parameters(
		"The folding function.",
		"The initial accumulator value.",
		"The structure to fold (owned for Val, borrowed for Ref)."
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
	/// let result = fold_left::<RcFnBrand, _, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
	/// assert_eq!(result, 6);
	/// ```
	pub fn fold_left<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Brand>(
		func: impl FoldLeftDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		initial: B,
		fa: FA,
	) -> B
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		func.dispatch(initial, fa)
	}

	/// Maps values to a monoid and combines them, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::fold_map`](crate::functions::explicit::fold_map) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The monoid type.",
		"The brand, inferred via InferableBrand from FA and the element type."
	)]
	///
	#[document_parameters(
		"The mapping function.",
		"The structure to fold (owned for Val, borrowed for Ref)."
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
	/// let result = fold_map::<RcFnBrand, _, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
	/// assert_eq!(result, "123");
	/// ```
	pub fn fold_map<'a, FnBrand, FA, A: 'a, M: Monoid + 'a, Brand>(
		func: impl FoldMapDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			M,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> M
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		func.dispatch(fa)
	}

	// -- Explicit dispatch free functions --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Folds a structure from the right.
		///
		/// Dispatches to [`Foldable::fold_right`] or [`RefFoldable::ref_fold_right`]
		/// based on whether the closure takes `A` or `&A`.
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
			"The folding function.",
			"The initial accumulator value.",
			"The structure to fold (owned for Val, borrowed for Ref)."
		)]
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
		/// // By-value
		/// let result = fold_right::<RcFnBrand, VecBrand, _, _, _, _>(|a, b| a + b, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		///
		/// // By-ref
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_right::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32, b| *a + b, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		pub fn fold_right<
			'a,
			FnBrand,
			Brand: Kind_cdc7cd43dac7585f,
			A: 'a + Clone,
			B: 'a,
			FA,
			Marker,
		>(
			func: impl FoldRightDispatch<'a, FnBrand, Brand, A, B, FA, Marker>,
			initial: B,
			fa: FA,
		) -> B {
			func.dispatch(initial, fa)
		}

		/// Folds a structure from the left.
		///
		/// Dispatches to [`Foldable::fold_left`] or [`RefFoldable::ref_fold_left`]
		/// based on whether the closure takes `A` or `&A`.
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
			"The folding function.",
			"The initial accumulator value.",
			"The structure to fold (owned for Val, borrowed for Ref)."
		)]
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
		/// // By-value
		/// let result = fold_left::<RcFnBrand, VecBrand, _, _, _, _>(|b, a| b + a, 0, vec![1, 2, 3]);
		/// assert_eq!(result, 6);
		///
		/// // By-ref
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_left::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|b, a: &i32| b + *a, 5, &lazy);
		/// assert_eq!(result, 15);
		/// ```
		pub fn fold_left<
			'a,
			FnBrand,
			Brand: Kind_cdc7cd43dac7585f,
			A: 'a + Clone,
			B: 'a,
			FA,
			Marker,
		>(
			func: impl FoldLeftDispatch<'a, FnBrand, Brand, A, B, FA, Marker>,
			initial: B,
			fa: FA,
		) -> B {
			func.dispatch(initial, fa)
		}

		/// Maps values to a monoid and combines them.
		///
		/// Dispatches to [`Foldable::fold_map`] or [`RefFoldable::ref_fold_map`]
		/// based on whether the closure takes `A` or `&A`.
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
			"The mapping function.",
			"The structure to fold (owned for Val, borrowed for Ref)."
		)]
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
		/// // By-value
		/// let result = fold_map::<RcFnBrand, VecBrand, _, _, _, _>(|a: i32| a.to_string(), vec![1, 2, 3]);
		/// assert_eq!(result, "123");
		///
		/// // By-ref
		/// let lazy = RcLazy::new(|| 10);
		/// let result =
		/// 	fold_map::<RcFnBrand, LazyBrand<RcLazyConfig>, _, _, _, _>(|a: &i32| a.to_string(), &lazy);
		/// assert_eq!(result, "10");
		/// ```
		pub fn fold_map<
			'a,
			FnBrand,
			Brand: Kind_cdc7cd43dac7585f,
			A: 'a,
			M: Monoid + 'a,
			FA,
			Marker,
		>(
			func: impl FoldMapDispatch<'a, FnBrand, Brand, A, M, FA, Marker>,
			fa: FA,
		) -> M {
			func.dispatch(fa)
		}
	}
}

pub use inner::*;
