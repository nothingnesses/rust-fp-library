//! Dispatch for [`FunctorWithIndex::map_with_index`](crate::classes::FunctorWithIndex::map_with_index) and
//! [`RefFunctorWithIndex::ref_map_with_index`](crate::classes::RefFunctorWithIndex::ref_map_with_index).
//!
//! Provides the [`MapWithIndexDispatch`] trait and a unified [`map_with_index`] free function
//! that routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::*,
//! };
//!
//! // Owned: dispatches to FunctorWithIndex::map_with_index
//! let y =
//! 	map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: i32| x + i as i32, vec![10, 20, 30]);
//! assert_eq!(y, vec![10, 21, 32]);
//!
//! // By-ref: dispatches to RefFunctorWithIndex::ref_map_with_index
//! let v = vec![10, 20, 30];
//! let y = map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: &i32| *x + i as i32, &v);
//! assert_eq!(y, vec![10, 21, 32]);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				FunctorWithIndex,
				RefFunctorWithIndex,
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

	/// Trait that routes a map_with_index operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait MapWithIndexDispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		B: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched map_with_index operation.
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function with index."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result =
		/// 	map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: i32| x + i as i32, vec![10, 20, 30]);
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: Fn(Brand::Index, A) -> B -> FunctorWithIndex::map_with_index --

	/// Routes `Fn(Brand::Index, A) -> B` closures to [`FunctorWithIndex::map_with_index`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and owned values.")]
	impl<'a, Brand, A, B, F>
		MapWithIndexDispatch<
			'a,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: FunctorWithIndex,
		A: 'a,
		B: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The functor instance containing the value(s).")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function with index."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result =
		/// 	map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: i32| x + i as i32, vec![10, 20, 30]);
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::map_with_index(self, fa)
		}
	}

	// -- Ref: Fn(Brand::Index, &A) -> B -> RefFunctorWithIndex::ref_map_with_index --

	/// Routes `Fn(Brand::Index, &A) -> B` closures to [`RefFunctorWithIndex::ref_map_with_index`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and references.")]
	impl<'a, 'b, Brand, A, B, F>
		MapWithIndexDispatch<
			'a,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFunctorWithIndex,
		A: 'a,
		B: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, &A) -> B + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the functor instance.")]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function with index."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30];
		/// let result = map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: &i32| *x + i as i32, &v);
		/// assert_eq!(result, vec![10, 21, 32]);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_map_with_index(self, fa)
		}
	}

	// -- Unified free function --

	/// Maps a function with index over the values in a functor context.
	///
	/// Dispatches to either [`FunctorWithIndex::map_with_index`] or
	/// [`RefFunctorWithIndex::ref_map_with_index`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(Index, A) -> B`) and the container is
	///   owned, dispatches to [`FunctorWithIndex::map_with_index`].
	/// - If the closure takes references (`Fn(Index, &A) -> B`) and the container is
	///   borrowed (`&fa`), dispatches to [`RefFunctorWithIndex::ref_map_with_index`].
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by the
	/// compiler from the closure's argument type and the container argument.
	/// Callers write `map_with_index_explicit::<Brand, _, _, _, _>(...)` and never need to
	/// specify `Marker` or `FA` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the functor.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each value and its index.",
		"The functor instance (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns(
		"A new functor instance containing the result(s) of applying the function with index."
	)]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // Owned: dispatches to FunctorWithIndex::map_with_index
	/// let y =
	/// 	map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: i32| x + i as i32, vec![10, 20, 30]);
	/// assert_eq!(y, vec![10, 21, 32]);
	///
	/// // By-ref: dispatches to RefFunctorWithIndex::ref_map_with_index
	/// let v = vec![10, 20, 30];
	/// let y = map_with_index_explicit::<VecBrand, _, _, _, _>(|i, x: &i32| *x + i as i32, &v);
	/// assert_eq!(y, vec![10, 21, 32]);
	/// ```
	pub fn map_with_index<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		B: 'a,
		FA,
		Marker,
	>(
		f: impl MapWithIndexDispatch<'a, Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch(fa)
	}
}

pub use inner::*;
