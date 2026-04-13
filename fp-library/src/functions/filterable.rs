#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			dispatch::filterable::{
				FilterDispatch,
				FilterMapDispatch,
				PartitionDispatch,
				PartitionMapDispatch,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- filter --

	/// Filters the values in a filterable context using a predicate, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::filter`](crate::functions::explicit::filter) with a turbofish.
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
		"The predicate to apply to each value.",
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
	/// let y = filter(|x: i32| x > 3, Some(5));
	/// assert_eq!(y, Some(5));
	/// ```
	pub fn filter<'a, FA, A: 'a + Clone, Marker>(
		f: impl FilterDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		FA: InferableBrand_cdc7cd43dac7585f, {
		f.dispatch(fa)
	}

	// -- filter_map --

	/// Filters and maps the values in a filterable context, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::filter_map`](crate::functions::explicit::filter_map) with a turbofish.
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
		"The function to apply to each value. Returns `Some(b)` to keep or `None` to discard.",
		"The filterable instance (owned or borrowed)."
	)]
	///
	#[document_returns("A new filterable instance containing only the kept values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let y = filter_map(|x: i32| if x > 3 { Some(x) } else { None }, Some(5));
	/// assert_eq!(y, Some(5));
	/// ```
	pub fn filter_map<'a, FA, A: 'a, B: 'a, Marker>(
		f: impl FilterMapDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: InferableBrand_cdc7cd43dac7585f, {
		f.dispatch(fa)
	}

	// -- partition --

	/// Partitions the values in a filterable context using a predicate, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::partition`](crate::functions::explicit::partition) with a turbofish.
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
		"The predicate to apply to each value.",
		"The filterable instance (owned or borrowed)."
	)]
	///
	#[document_returns("A tuple of two filterable instances split by the predicate.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let (no, yes) = partition(|x: i32| x > 3, Some(5));
	/// assert_eq!(yes, Some(5));
	/// assert_eq!(no, None);
	/// ```
	pub fn partition<'a, FA, A: 'a + Clone, Marker>(
		f: impl PartitionDispatch<'a, <FA as InferableBrand_cdc7cd43dac7585f>::Brand, A, FA, Marker>,
		fa: FA,
	) -> (
		Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	)
	where
		FA: InferableBrand_cdc7cd43dac7585f, {
		f.dispatch(fa)
	}

	// -- partition_map --

	/// Partitions the values using a function returning `Result`, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`InferableBrand`](crate::kinds::InferableBrand_cdc7cd43dac7585f). Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::partition_map`](crate::functions::explicit::partition_map) with a turbofish.
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
		"The function to apply to each value. Returns `Ok(o)` or `Err(e)`.",
		"The filterable instance (owned or borrowed)."
	)]
	///
	#[document_returns("A tuple of two filterable instances: `Err` values and `Ok` values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let (errs, oks) = partition_map(|x: i32| Ok::<i32, i32>(x * 2), Some(5));
	/// assert_eq!(errs, None);
	/// assert_eq!(oks, Some(10));
	/// ```
	pub fn partition_map<'a, FA, A: 'a, E: 'a, O: 'a, Marker>(
		f: impl PartitionMapDispatch<
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
		FA: InferableBrand_cdc7cd43dac7585f, {
		f.dispatch(fa)
	}
}

pub use inner::*;
