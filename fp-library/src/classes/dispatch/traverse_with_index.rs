//! Dispatch for [`TraversableWithIndex::traverse_with_index`](crate::classes::TraversableWithIndex::traverse_with_index) and
//! [`RefTraversableWithIndex::ref_traverse_with_index`](crate::classes::RefTraversableWithIndex::ref_traverse_with_index).
//!
//! Provides the [`TraverseWithIndexDispatch`] trait and a unified [`traverse_with_index`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	classes::dispatch::traverse_with_index,
//! };
//!
//! // Owned: dispatches to TraversableWithIndex::traverse_with_index
//! let y = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
//! 	|_i, x: i32| Some(x * 2),
//! 	vec![1, 2, 3],
//! );
//! assert_eq!(y, Some(vec![2, 4, 6]));
//!
//! // By-ref: dispatches to RefTraversableWithIndex::ref_traverse_with_index
//! let v = vec![1, 2, 3];
//! let y = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
//! 	|_i, x: &i32| Some(*x * 2),
//! 	&v,
//! );
//! assert_eq!(y, Some(vec![2, 4, 6]));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Applicative,
				LiftFn,
				RefTraversableWithIndex,
				TraversableWithIndex,
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

	/// Trait that routes a traverse_with_index operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FTA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand for the computation.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait TraverseWithIndexDispatch<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		B: 'a,
		F: Kind_cdc7cd43dac7585f,
		FTA,
		Marker,
	> {
		/// Perform the dispatched traverse_with_index operation.
		#[document_signature]
		///
		#[document_parameters("The structure to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::dispatch::traverse_with_index,
		/// };
		///
		/// let result = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
		/// 	|_i, x: i32| Some(x * 2),
		/// 	vec![1, 2, 3],
		/// );
		/// assert_eq!(result, Some(vec![2, 4, 6]));
		/// ```
		fn dispatch(
			self,
			ta: FTA,
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>);
	}

	// -- Val: Fn(Brand::Index, A) -> F<B> -> TraversableWithIndex::traverse_with_index --

	/// Routes `Fn(Brand::Index, A) -> F::Of<B>` closures to [`TraversableWithIndex::traverse_with_index`].
	///
	/// The `FnBrand` parameter is unused by the Val path but is accepted for
	/// uniformity with the Ref path.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The cloneable function brand (unused by Val path).",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and owned values.")]
	impl<'a, FnBrand, Brand, A, B, F, Func>
		TraverseWithIndexDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			F,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for Func
	where
		Brand: TraversableWithIndex,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		F: Applicative,
		Func:
			Fn(Brand::Index, A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Brand::Index: 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The structure to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::dispatch::traverse_with_index,
		/// };
		///
		/// let result = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
		/// 	|_i, x: i32| Some(x * 2),
		/// 	vec![1, 2, 3],
		/// );
		/// assert_eq!(result, Some(vec![2, 4, 6]));
		/// ```
		fn dispatch(
			self,
			ta: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		{
			Brand::traverse_with_index::<A, B, F>(self, ta)
		}
	}

	// -- Ref: Fn(Brand::Index, &A) -> F<B> -> RefTraversableWithIndex::ref_traverse_with_index --

	/// Routes `Fn(Brand::Index, &A) -> F::Of<B>` closures to [`RefTraversableWithIndex::ref_traverse_with_index`].
	///
	/// The `FnBrand` parameter is accepted for uniformity with [`TraverseDispatch`](crate::classes::dispatch::TraverseDispatch),
	/// but is not passed through to the underlying
	/// [`ref_traverse_with_index`](RefTraversableWithIndex::ref_traverse_with_index) call.
	///
	/// The container must be passed by reference (`&ta`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The cloneable function brand.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and references.")]
	impl<'a, 'b, FnBrand, Brand, A, B, F, Func>
		TraverseWithIndexDispatch<
			'a,
			FnBrand,
			Brand,
			A,
			B,
			F,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for Func
	where
		Brand: RefTraversableWithIndex,
		FnBrand: LiftFn + 'a,
		A: 'a + Clone,
		B: 'a + Clone,
		F: Applicative,
		Func:
			Fn(Brand::Index, &A) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) + 'a,
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>): Clone,
		Brand::Index: 'a,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the structure to traverse.")]
		///
		#[document_returns("The combined result in the applicative context.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::dispatch::traverse_with_index,
		/// };
		///
		/// let v = vec![1, 2, 3];
		/// let result = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
		/// 	|_i, x: &i32| Some(*x * 2),
		/// 	&v,
		/// );
		/// assert_eq!(result, Some(vec![2, 4, 6]));
		/// ```
		fn dispatch(
			self,
			ta: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
		{
			Brand::ref_traverse_with_index::<A, B, F>(self, ta)
		}
	}

	// -- Unified free function --

	/// Traverses a structure with an indexed effectful function, combining the results.
	///
	/// Dispatches to either [`TraversableWithIndex::traverse_with_index`] or
	/// [`RefTraversableWithIndex::ref_traverse_with_index`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(Index, A) -> F::Of<B>`) and the
	///   container is owned, dispatches to [`TraversableWithIndex::traverse_with_index`].
	///   The `FnBrand` parameter is unused but must be specified for uniformity.
	/// - If the closure takes references (`Fn(Index, &A) -> F::Of<B>`) and the
	///   container is borrowed (`&ta`), dispatches to
	///   [`RefTraversableWithIndex::ref_traverse_with_index`]. The `FnBrand`
	///   parameter is accepted for uniformity but is not passed through.
	///
	/// The `Marker` and `FTA` type parameters are inferred automatically by
	/// the compiler from the closure's argument type and the container
	/// argument. Callers write
	/// `traverse_with_index::<FnBrand, Brand, _, _, F, _, _>(...)` and never need to
	/// specify `Marker` or `FTA` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use.",
		"The brand of the traversable structure.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The indexed function to apply to each element, returning a value in an applicative context.",
		"The traversable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The structure wrapped in the applicative context.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	classes::dispatch::traverse_with_index,
	/// };
	///
	/// // Owned: dispatches to TraversableWithIndex::traverse_with_index
	/// let y = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
	/// 	|_i, x: i32| Some(x * 2),
	/// 	vec![1, 2, 3],
	/// );
	/// assert_eq!(y, Some(vec![2, 4, 6]));
	///
	/// // By-ref: dispatches to RefTraversableWithIndex::ref_traverse_with_index
	/// let v = vec![1, 2, 3];
	/// let y = traverse_with_index::<RcFnBrand, VecBrand, _, _, OptionBrand, _, _>(
	/// 	|_i, x: &i32| Some(*x * 2),
	/// 	&v,
	/// );
	/// assert_eq!(y, Some(vec![2, 4, 6]));
	/// ```
	pub fn traverse_with_index<
		'a,
		FnBrand,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		B: 'a,
		F: Kind_cdc7cd43dac7585f,
		FTA,
		Marker,
	>(
		func: impl TraverseWithIndexDispatch<'a, FnBrand, Brand, A, B, F, FTA, Marker>,
		ta: FTA,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	{
		func.dispatch(ta)
	}
}

pub use inner::*;
