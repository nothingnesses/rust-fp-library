//! Dispatch for [`FilterableWithIndex::filter_map_with_index`](crate::classes::FilterableWithIndex::filter_map_with_index) and
//! [`RefFilterableWithIndex::ref_filter_map_with_index`](crate::classes::RefFilterableWithIndex::ref_filter_map_with_index).
//!
//! Provides the [`FilterMapWithIndexDispatch`] trait and a unified [`filter_map_with_index`] free function
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
//! // Owned: dispatches to FilterableWithIndex::filter_map_with_index
//! let y = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
//! 	|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
//! 	vec![10, 20, 30, 40],
//! );
//! assert_eq!(y, vec![20, 60]);
//!
//! // By-ref: dispatches to RefFilterableWithIndex::ref_filter_map_with_index
//! let v = vec![10, 20, 30, 40];
//! let y = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
//! 	|i, x: &i32| if i % 2 == 0 { Some(*x * 2) } else { None },
//! 	&v,
//! );
//! assert_eq!(y, vec![20, 60]);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				FilterableWithIndex,
				RefFilterableWithIndex,
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

	/// Trait that routes a filter_map_with_index operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The type of the result(s) of applying the function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FilterMapWithIndexDispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		B: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched filter_map_with_index operation.
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A new filterable instance containing only the values for which the function returned `Some`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
		/// 	|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
		/// 	vec![10, 20, 30, 40],
		/// );
		/// assert_eq!(result, vec![20, 60]);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: Fn(Brand::Index, A) -> Option<B> -> FilterableWithIndex::filter_map_with_index --

	/// Routes `Fn(Brand::Index, A) -> Option<B>` closures to [`FilterableWithIndex::filter_map_with_index`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and owned values.")]
	impl<'a, Brand, A, B, F>
		FilterMapWithIndexDispatch<
			'a,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: FilterableWithIndex,
		A: 'a,
		B: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, A) -> Option<B> + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A new filterable instance containing only the values for which the function returned `Some`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let result = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
		/// 	|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
		/// 	vec![10, 20, 30, 40],
		/// );
		/// assert_eq!(result, vec![20, 60]);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::filter_map_with_index(self, fa)
		}
	}

	// -- Ref: Fn(Brand::Index, &A) -> Option<B> -> RefFilterableWithIndex::ref_filter_map_with_index --

	/// Routes `Fn(Brand::Index, &A) -> Option<B>` closures to [`RefFilterableWithIndex::ref_filter_map_with_index`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and references.")]
	impl<'a, 'b, Brand, A, B, F>
		FilterMapWithIndexDispatch<
			'a,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFilterableWithIndex,
		A: 'a,
		B: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, &A) -> Option<B> + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the filterable instance.")]
		///
		#[document_returns(
			"A new filterable instance containing only the values for which the function returned `Some`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40];
		/// let result = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
		/// 	|i, x: &i32| if i % 2 == 0 { Some(*x * 2) } else { None },
		/// 	&v,
		/// );
		/// assert_eq!(result, vec![20, 60]);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_filter_map_with_index(self, fa)
		}
	}

	// -- Unified free function --

	/// Filters and maps the values in a filterable context with index.
	///
	/// Dispatches to either [`FilterableWithIndex::filter_map_with_index`] or
	/// [`RefFilterableWithIndex::ref_filter_map_with_index`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(Index, A) -> Option<B>`) and the
	///   container is owned, dispatches to [`FilterableWithIndex::filter_map_with_index`].
	/// - If the closure takes references (`Fn(Index, &A) -> Option<B>`) and the
	///   container is borrowed (`&fa`), dispatches to
	///   [`RefFilterableWithIndex::ref_filter_map_with_index`].
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by the
	/// compiler from the closure's argument type and the container argument.
	/// Callers write `filter_map_with_index_explicit::<Brand, _, _, _, _>(...)` and never need to
	/// specify `Marker` or `FA` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The type of the result(s) of applying the function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each value and its index. Returns `Some(b)` to keep the value or `None` to discard it.",
		"The filterable instance (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns(
		"A new filterable instance containing only the values for which the function returned `Some`."
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
	/// // Owned: dispatches to FilterableWithIndex::filter_map_with_index
	/// let y = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
	/// 	|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
	/// 	vec![10, 20, 30, 40],
	/// );
	/// assert_eq!(y, vec![20, 60]);
	///
	/// // By-ref: dispatches to RefFilterableWithIndex::ref_filter_map_with_index
	/// let v = vec![10, 20, 30, 40];
	/// let y = filter_map_with_index_explicit::<VecBrand, _, _, _, _>(
	/// 	|i, x: &i32| if i % 2 == 0 { Some(*x * 2) } else { None },
	/// 	&v,
	/// );
	/// assert_eq!(y, vec![20, 60]);
	/// ```
	pub fn filter_map_with_index<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		B: 'a,
		FA,
		Marker,
	>(
		f: impl FilterMapWithIndexDispatch<'a, Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		f.dispatch(fa)
	}
}

pub use inner::*;
