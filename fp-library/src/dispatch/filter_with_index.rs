//! Dispatch for [`FilterableWithIndex::filter_with_index`](crate::classes::FilterableWithIndex::filter_with_index) and
//! [`RefFilterableWithIndex::ref_filter_with_index`](crate::classes::RefFilterableWithIndex::ref_filter_with_index).
//!
//! Provides the [`FilterWithIndexDispatch`] trait and a unified [`filter_with_index`] free function
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
//! // Owned: dispatches to FilterableWithIndex::filter_with_index
//! let y =
//! 	filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
//! assert_eq!(y, vec![10, 20]);
//!
//! // By-ref: dispatches to RefFilterableWithIndex::ref_filter_with_index
//! let v = vec![10, 20, 30, 40];
//! let y = filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
//! assert_eq!(y, vec![10, 20]);
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

	/// Trait that routes a filter_with_index operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait FilterWithIndexDispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a + Clone,
		FA,
		Marker,
	> {
		/// Perform the dispatched filter_with_index operation.
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A new filterable instance containing only the values for which the predicate returned `true`."
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
		/// 	filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(result, vec![10, 20]);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	// -- Val: Fn(Brand::Index, A) -> bool -> FilterableWithIndex::filter_with_index --

	/// Routes `Fn(Brand::Index, A) -> bool` closures to [`FilterableWithIndex::filter_with_index`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and owned values.")]
	impl<'a, Brand, A, F>
		FilterWithIndexDispatch<
			'a,
			Brand,
			A,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: FilterableWithIndex,
		A: 'a + Clone,
		Brand::Index: 'a,
		F: Fn(Brand::Index, A) -> bool + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A new filterable instance containing only the values for which the predicate returned `true`."
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
		/// 	filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(result, vec![10, 20]);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::filter_with_index(self, fa)
		}
	}

	// -- Ref: Fn(Brand::Index, &A) -> bool -> RefFilterableWithIndex::ref_filter_with_index --

	/// Routes `Fn(Brand::Index, &A) -> bool` closures to [`RefFilterableWithIndex::ref_filter_with_index`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and references.")]
	impl<'a, 'b, Brand, A, F>
		FilterWithIndexDispatch<
			'a,
			Brand,
			A,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFilterableWithIndex,
		A: 'a + Clone,
		Brand::Index: 'a,
		F: Fn(Brand::Index, &A) -> bool + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the filterable instance.")]
		///
		#[document_returns(
			"A new filterable instance containing only the values for which the predicate returned `true`."
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
		/// let result = filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
		/// assert_eq!(result, vec![10, 20]);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_filter_with_index(self, fa)
		}
	}

	// -- Unified free function --

	/// Filters the values in a filterable context using a predicate with index.
	///
	/// Dispatches to either [`FilterableWithIndex::filter_with_index`] or
	/// [`RefFilterableWithIndex::ref_filter_with_index`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(Index, A) -> bool`) and the
	///   container is owned, dispatches to [`FilterableWithIndex::filter_with_index`].
	/// - If the closure takes references (`Fn(Index, &A) -> bool`) and the
	///   container is borrowed (`&fa`), dispatches to
	///   [`RefFilterableWithIndex::ref_filter_with_index`].
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by the
	/// compiler from the closure's argument type and the container argument.
	/// Callers write `filter_with_index_explicit::<Brand, _, _, _>(...)` and never need to
	/// specify `Marker` or `FA` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The predicate to apply to each value and its index. Returns `true` to keep the value or `false` to discard it.",
		"The filterable instance (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns(
		"A new filterable instance containing only the values for which the predicate returned `true`."
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
	/// // Owned: dispatches to FilterableWithIndex::filter_with_index
	/// let y =
	/// 	filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
	/// assert_eq!(y, vec![10, 20]);
	///
	/// // By-ref: dispatches to RefFilterableWithIndex::ref_filter_with_index
	/// let v = vec![10, 20, 30, 40];
	/// let y = filter_with_index_explicit::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
	/// assert_eq!(y, vec![10, 20]);
	/// ```
	pub fn filter_with_index<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a + Clone,
		FA,
		Marker,
	>(
		f: impl FilterWithIndexDispatch<'a, Brand, A, FA, Marker>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
		f.dispatch(fa)
	}
}

pub use inner::*;
