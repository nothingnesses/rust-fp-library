//! Dispatch for [`FilterableWithIndex::partition_map_with_index`](crate::classes::FilterableWithIndex::partition_map_with_index) and
//! [`RefFilterableWithIndex::ref_partition_map_with_index`](crate::classes::RefFilterableWithIndex::ref_partition_map_with_index).
//!
//! Provides the [`PartitionMapWithIndexDispatch`] trait and a unified [`partition_map_with_index`] free function
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
//! // Owned: dispatches to FilterableWithIndex::partition_map_with_index
//! let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
//! 	|i, x: i32| if i < 2 { Ok(x) } else { Err(x) },
//! 	vec![10, 20, 30, 40],
//! );
//! assert_eq!(oks, vec![10, 20]);
//! assert_eq!(errs, vec![30, 40]);
//!
//! // By-ref: dispatches to RefFilterableWithIndex::ref_partition_map_with_index
//! let v = vec![10, 20, 30, 40];
//! let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
//! 	|i, x: &i32| if i < 2 { Ok(*x) } else { Err(*x) },
//! 	&v,
//! );
//! assert_eq!(oks, vec![10, 20]);
//! assert_eq!(errs, vec![30, 40]);
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				FilterableWithIndex,
				RefFilterableWithIndex,
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

	/// Trait that routes a partition_map_with_index operation to the appropriate type class method.
	///
	/// The `Marker` type parameter is an implementation detail resolved by
	/// the compiler from the closure's argument type; callers never specify
	/// it directly. The `FA` type parameter is inferred from the container
	/// argument: owned for Val dispatch, borrowed for Ref dispatch.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The error type produced by the partitioning function.",
		"The success type produced by the partitioning function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait PartitionMapWithIndexDispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		E: 'a,
		O: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched partition_map_with_index operation.
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains the `Err` values, the second contains the `Ok` values."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
		/// 	|i, x: i32| if i < 2 { Ok(x) } else { Err(x) },
		/// 	vec![10, 20, 30, 40],
		/// );
		/// assert_eq!(oks, vec![10, 20]);
		/// assert_eq!(errs, vec![30, 40]);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		);
	}

	// -- Val: Fn(Brand::Index, A) -> Result<O, E> -> FilterableWithIndex::partition_map_with_index --

	/// Routes `Fn(Brand::Index, A) -> Result<O, E>` closures to [`FilterableWithIndex::partition_map_with_index`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The error type produced by the partitioning function.",
		"The success type produced by the partitioning function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and owned values.")]
	impl<'a, Brand, A, E, O, F>
		PartitionMapWithIndexDispatch<
			'a,
			Brand,
			A,
			E,
			O,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: FilterableWithIndex,
		A: 'a,
		E: 'a,
		O: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, A) -> Result<O, E> + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains the `Err` values, the second contains the `Ok` values."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::*,
		/// };
		///
		/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
		/// 	|i, x: i32| if i < 2 { Ok(x) } else { Err(x) },
		/// 	vec![10, 20, 30, 40],
		/// );
		/// assert_eq!(oks, vec![10, 20]);
		/// assert_eq!(errs, vec![30, 40]);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::partition_map_with_index(self, fa)
		}
	}

	// -- Ref: Fn(Brand::Index, &A) -> Result<O, E> -> RefFilterableWithIndex::ref_partition_map_with_index --

	/// Routes `Fn(Brand::Index, &A) -> Result<O, E>` closures to [`RefFilterableWithIndex::ref_partition_map_with_index`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The error type produced by the partitioning function.",
		"The success type produced by the partitioning function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and references.")]
	impl<'a, 'b, Brand, A, E, O, F>
		PartitionMapWithIndexDispatch<
			'a,
			Brand,
			A,
			E,
			O,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFilterableWithIndex,
		A: 'a,
		E: 'a,
		O: 'a,
		Brand::Index: 'a,
		F: Fn(Brand::Index, &A) -> Result<O, E> + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the filterable instance.")]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains the `Err` values, the second contains the `Ok` values."
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
		/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
		/// 	|i, x: &i32| if i < 2 { Ok(*x) } else { Err(*x) },
		/// 	&v,
		/// );
		/// assert_eq!(oks, vec![10, 20]);
		/// assert_eq!(errs, vec![30, 40]);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::ref_partition_map_with_index(self, fa)
		}
	}

	// -- Unified free function --

	/// Partitions the values in a filterable context using a function with index that returns `Result`.
	///
	/// Dispatches to either [`FilterableWithIndex::partition_map_with_index`] or
	/// [`RefFilterableWithIndex::ref_partition_map_with_index`] based on the closure's argument type:
	///
	/// - If the closure takes owned values (`Fn(Index, A) -> Result<O, E>`) and the
	///   container is owned, dispatches to [`FilterableWithIndex::partition_map_with_index`].
	/// - If the closure takes references (`Fn(Index, &A) -> Result<O, E>`) and the
	///   container is borrowed (`&fa`), dispatches to
	///   [`RefFilterableWithIndex::ref_partition_map_with_index`].
	///
	/// The `Marker` and `FA` type parameters are inferred automatically by the
	/// compiler from the closure's argument type and the container argument.
	/// Callers write `partition_map_with_index::<Brand, _, _, _, _, _>(...)` and never need to
	/// specify `Marker` or `FA` explicitly.
	///
	/// The dispatch is resolved at compile time with no runtime cost.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The error type produced by the partitioning function.",
		"The success type produced by the partitioning function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each value and its index. Returns `Ok(o)` for the success partition or `Err(e)` for the error partition.",
		"The filterable instance (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns(
		"A tuple of two filterable instances: the first contains the `Err` values, the second contains the `Ok` values."
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
	/// // Owned: dispatches to FilterableWithIndex::partition_map_with_index
	/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
	/// 	|i, x: i32| if i < 2 { Ok(x) } else { Err(x) },
	/// 	vec![10, 20, 30, 40],
	/// );
	/// assert_eq!(oks, vec![10, 20]);
	/// assert_eq!(errs, vec![30, 40]);
	///
	/// // By-ref: dispatches to RefFilterableWithIndex::ref_partition_map_with_index
	/// let v = vec![10, 20, 30, 40];
	/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
	/// 	|i, x: &i32| if i < 2 { Ok(*x) } else { Err(*x) },
	/// 	&v,
	/// );
	/// assert_eq!(oks, vec![10, 20]);
	/// assert_eq!(errs, vec![30, 40]);
	/// ```
	pub fn partition_map_with_index<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a,
		E: 'a,
		O: 'a,
		FA,
		Marker,
	>(
		f: impl PartitionMapWithIndexDispatch<'a, Brand, A, E, O, FA, Marker>,
		fa: FA,
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	) {
		f.dispatch(fa)
	}
}

pub use inner::*;
