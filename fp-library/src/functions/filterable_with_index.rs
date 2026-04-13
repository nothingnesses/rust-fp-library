#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			classes::WithIndex,
			dispatch::filterable_with_index::{
				FilterMapWithIndexDispatch,
				FilterWithIndexDispatch,
				PartitionMapWithIndexDispatch,
				PartitionWithIndexDispatch,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- filter_with_index --

	/// Filters the values using a predicate with index, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::filter_with_index`](crate::functions::explicit::filter_with_index) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"Dispatch marker type, inferred automatically."
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
	pub fn filter_with_index<'a, FA, A: 'a + Clone, Marker>(
		f: impl FilterWithIndexDispatch<
			'a,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			A,
			FA,
			Marker,
		>,
		fa: FA,
	) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		FA: InferableBrand_cdc7cd43dac7585f,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
		f.dispatch(fa)
	}

	// -- filter_map_with_index --

	/// Filters and maps the values with index, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
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
		"Dispatch marker type, inferred automatically."
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
	pub fn filter_map_with_index<'a, FA, A: 'a, B: 'a, Marker>(
		f: impl FilterMapWithIndexDispatch<
			'a,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			A,
			B,
			FA,
			Marker,
		>,
		fa: FA,
	) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: InferableBrand_cdc7cd43dac7585f,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
		f.dispatch(fa)
	}

	// -- partition_with_index --

	/// Partitions the values using a predicate with index, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::partition_with_index`](crate::functions::explicit::partition_with_index) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"Dispatch marker type, inferred automatically."
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
	pub fn partition_with_index<'a, FA, A: 'a + Clone, Marker>(
		f: impl PartitionWithIndexDispatch<
			'a,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			A,
			FA,
			Marker,
		>,
		fa: FA,
	) -> (
		Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	)
	where
		FA: InferableBrand_cdc7cd43dac7585f,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
		f.dispatch(fa)
	}

	// -- partition_map_with_index --

	/// Partitions the values using a function with index returning `Result`,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
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
		"Dispatch marker type, inferred automatically."
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
	pub fn partition_map_with_index<'a, FA, A: 'a, E: 'a, O: 'a, Marker>(
		f: impl PartitionMapWithIndexDispatch<
			'a,
			<FA as InferableBrand_cdc7cd43dac7585f>::Brand,
			A,
			E,
			O,
			FA,
			Marker,
		>,
		fa: FA,
	) -> (
		Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	)
	where
		FA: InferableBrand_cdc7cd43dac7585f,
		<FA as InferableBrand_cdc7cd43dac7585f>::Brand: WithIndex, {
		f.dispatch(fa)
	}
}

pub use inner::*;
