//! Dispatch for filterable-with-index operations:
//! [`FilterableWithIndex`](crate::classes::FilterableWithIndex) and
//! [`RefFilterableWithIndex`](crate::classes::RefFilterableWithIndex).
//!
//! Provides the following dispatch traits and unified free functions:
//!
//! - [`FilterWithIndexDispatch`] + [`explicit::filter_with_index`]
//! - [`FilterMapWithIndexDispatch`] + [`explicit::filter_map_with_index`]
//! - [`PartitionWithIndexDispatch`] + [`explicit::partition_with_index`]
//! - [`PartitionMapWithIndexDispatch`] + [`explicit::partition_map_with_index`]
//!
//! Each routes to the appropriate trait method based on the closure's argument
//! type.
//!
//! ### Examples
//!
//! ```
//! use fp_library::{
//! 	brands::*,
//! 	functions::explicit::*,
//! };
//!
//! // filter_with_index
//! let y = filter_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
//! assert_eq!(y, vec![10, 20]);
//!
//! // filter_map_with_index
//! let y = filter_map_with_index::<VecBrand, _, _, _, _>(
//! 	|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
//! 	vec![10, 20, 30, 40],
//! );
//! assert_eq!(y, vec![20, 60]);
//!
//! // partition_with_index
//! let (not_satisfied, satisfied) =
//! 	partition_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
//! assert_eq!(satisfied, vec![10, 20]);
//! assert_eq!(not_satisfied, vec![30, 40]);
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

	// -- FilterWithIndexDispatch --

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
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = filter_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(result, vec![10, 20]);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

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
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = filter_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(result, vec![10, 20]);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::filter_with_index(self, fa)
		}
	}

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
		/// 	functions::explicit::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40];
		/// let result = filter_with_index::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
		/// assert_eq!(result, vec![10, 20]);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_filter_with_index(self, fa)
		}
	}

	// -- FilterMapWithIndexDispatch --

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
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = filter_map_with_index::<VecBrand, _, _, _, _>(
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let result = filter_map_with_index::<VecBrand, _, _, _, _>(
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40];
		/// let result = filter_map_with_index::<VecBrand, _, _, _, _>(
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

	// -- PartitionWithIndexDispatch --

	/// Trait that routes a partition_with_index operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait PartitionWithIndexDispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		A: 'a + Clone,
		FA,
		Marker,
	> {
		/// Perform the dispatched partition_with_index operation.
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains elements not satisfying the predicate, the second contains those that do."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let (not_satisfied, satisfied) =
		/// 	partition_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(satisfied, vec![10, 20]);
		/// assert_eq!(not_satisfied, vec![30, 40]);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		);
	}

	/// Routes `Fn(Brand::Index, A) -> bool` closures to [`FilterableWithIndex::partition_with_index`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes an index and owned values.")]
	impl<'a, Brand, A, F>
		PartitionWithIndexDispatch<
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
			"A tuple of two filterable instances: the first contains elements not satisfying the predicate, the second contains those that do."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let (not_satisfied, satisfied) =
		/// 	partition_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(satisfied, vec![10, 20]);
		/// assert_eq!(not_satisfied, vec![30, 40]);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			Brand::partition_with_index(self, fa)
		}
	}

	/// Routes `Fn(Brand::Index, &A) -> bool` closures to [`RefFilterableWithIndex::ref_partition_with_index`].
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
		PartitionWithIndexDispatch<
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
			"A tuple of two filterable instances: the first contains elements not satisfying the predicate, the second contains those that do."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let v = vec![10, 20, 30, 40];
		/// let (not_satisfied, satisfied) =
		/// 	partition_with_index::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
		/// assert_eq!(satisfied, vec![10, 20]);
		/// assert_eq!(not_satisfied, vec![30, 40]);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			Brand::ref_partition_with_index(self, fa)
		}
	}

	// -- PartitionMapWithIndexDispatch --

	/// Trait that routes a partition_map_with_index operation to the appropriate type class method.
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The error type produced by the partitioning function.",
		"The success type produced by the partitioning function.",
		"The container type (owned or borrowed), inferred from the argument.",
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
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
		/// 	functions::explicit::*,
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
		/// 	functions::explicit::*,
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
		/// 	functions::explicit::*,
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

	// -- Inference wrappers --

	/// Filters the values using a predicate with index, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::filter_with_index`](crate::functions::explicit::filter_with_index) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"The brand, inferred via InferableBrand from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The predicate to apply to each value and its index.",
		"The filterable instance (owned or borrowed)."
	)]
	///
	#[document_returns(
		"A new filterable instance containing only the values satisfying the predicate."
	)]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let y = filter_with_index(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
	/// assert_eq!(y, vec![10, 20]);
	/// ```
	pub fn filter_with_index<'a, FA, A: 'a + Clone, Brand>(
		f: impl FilterWithIndexDispatch<
			'a,
			Brand,
			A,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	/// Filters and maps the values with index, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::filter_map_with_index`](crate::functions::explicit::filter_map_with_index) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"The type of the result(s) of applying the function.",
		"The brand, inferred via InferableBrand from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The function to apply to each value and its index. Returns `Some(b)` to keep or `None` to discard.",
		"The filterable instance (owned or borrowed)."
	)]
	///
	#[document_returns("A new filterable instance containing only the kept values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let y = filter_map_with_index(
	/// 	|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
	/// 	vec![10, 20, 30, 40],
	/// );
	/// assert_eq!(y, vec![20, 60]);
	/// ```
	pub fn filter_map_with_index<'a, FA, A: 'a, B: 'a, Brand>(
		f: impl FilterMapWithIndexDispatch<
			'a,
			Brand,
			A,
			B,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	/// Partitions the values using a predicate with index, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::partition_with_index`](crate::functions::explicit::partition_with_index) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"The brand, inferred via InferableBrand from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The predicate to apply to each value and its index.",
		"The filterable instance (owned or borrowed)."
	)]
	///
	#[document_returns("A tuple of two filterable instances split by the predicate.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let (not_sat, sat) = partition_with_index(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
	/// assert_eq!(sat, vec![10, 20]);
	/// assert_eq!(not_sat, vec![30, 40]);
	/// ```
	pub fn partition_with_index<'a, FA, A: 'a + Clone, Brand>(
		f: impl PartitionWithIndexDispatch<
			'a,
			Brand,
			A,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	)
	where
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	/// Partitions the values using a function with index returning `Result`,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `InferableBrand` trait. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::partition_map_with_index`](crate::functions::explicit::partition_map_with_index) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"The error type produced by the partitioning function.",
		"The success type produced by the partitioning function.",
		"The brand, inferred via InferableBrand from FA and the closure's input type."
	)]
	///
	#[document_parameters(
		"The function to apply to each value and its index. Returns `Ok(o)` or `Err(e)`.",
		"The filterable instance (owned or borrowed)."
	)]
	///
	#[document_returns("A tuple of two filterable instances: `Err` values and `Ok` values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let (errs, oks) = partition_map_with_index(
	/// 	|i, x: i32| if i < 2 { Ok(x) } else { Err(x) },
	/// 	vec![10, 20, 30, 40],
	/// );
	/// assert_eq!(oks, vec![10, 20]);
	/// assert_eq!(errs, vec![30, 40]);
	/// ```
	pub fn partition_map_with_index<'a, FA, A: 'a, E: 'a, O: 'a, Brand>(
		f: impl PartitionMapWithIndexDispatch<
			'a,
			Brand,
			A,
			E,
			O,
			FA,
			<FA as InferableBrand_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	)
	where
		Brand: Kind_cdc7cd43dac7585f + WithIndex,
		FA: InferableBrand_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	// -- Explicit dispatch free functions --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Filters the values in a filterable context using a predicate with index.
		///
		/// Dispatches to either [`FilterableWithIndex::filter_with_index`] or
		/// [`RefFilterableWithIndex::ref_filter_with_index`] based on the closure's argument type.
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
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned
		/// let y = filter_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(y, vec![10, 20]);
		///
		/// // By-ref
		/// let v = vec![10, 20, 30, 40];
		/// let y = filter_with_index::<VecBrand, _, _, _>(|i, _x: &i32| i < 2, &v);
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

		/// Filters and maps the values in a filterable context with index.
		///
		/// Dispatches to either [`FilterableWithIndex::filter_map_with_index`] or
		/// [`RefFilterableWithIndex::ref_filter_map_with_index`] based on the closure's argument type.
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
		/// 	functions::explicit::*,
		/// };
		///
		/// // Owned
		/// let y = filter_map_with_index::<VecBrand, _, _, _, _>(
		/// 	|i, x: i32| if i % 2 == 0 { Some(x * 2) } else { None },
		/// 	vec![10, 20, 30, 40],
		/// );
		/// assert_eq!(y, vec![20, 60]);
		///
		/// // By-ref
		/// let v = vec![10, 20, 30, 40];
		/// let y = filter_map_with_index::<VecBrand, _, _, _, _>(
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

		/// Partitions the values in a filterable context using a predicate with index.
		///
		/// Dispatches to either [`FilterableWithIndex::partition_with_index`] or
		/// [`RefFilterableWithIndex::ref_partition_with_index`] based on the closure's argument type.
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
			"The predicate to apply to each value and its index. Returns `true` for the first partition or `false` for the second.",
			"The filterable instance (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains elements not satisfying the predicate, the second contains those that do."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let (not_satisfied, satisfied) =
		/// 	partition_with_index::<VecBrand, _, _, _>(|i, _x: i32| i < 2, vec![10, 20, 30, 40]);
		/// assert_eq!(satisfied, vec![10, 20]);
		/// assert_eq!(not_satisfied, vec![30, 40]);
		/// ```
		pub fn partition_with_index<
			'a,
			Brand: Kind_cdc7cd43dac7585f + WithIndex,
			A: 'a + Clone,
			FA,
			Marker,
		>(
			f: impl PartitionWithIndexDispatch<'a, Brand, A, FA, Marker>,
			fa: FA,
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			f.dispatch(fa)
		}

		/// Partitions the values in a filterable context using a function with index that returns `Result`.
		///
		/// Dispatches to either [`FilterableWithIndex::partition_map_with_index`] or
		/// [`RefFilterableWithIndex::ref_partition_map_with_index`] based on the closure's argument type.
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
		/// 	functions::explicit::*,
		/// };
		///
		/// let (errs, oks) = partition_map_with_index::<VecBrand, _, _, _, _, _>(
		/// 	|i, x: i32| if i < 2 { Ok(x) } else { Err(x) },
		/// 	vec![10, 20, 30, 40],
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
}

pub use inner::*;
