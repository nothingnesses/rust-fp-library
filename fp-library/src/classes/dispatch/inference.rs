//! Brand-inference wrappers for dispatch functions.
//!
//! Each function in this module wraps a dispatch function, using
//! [`DefaultBrand`](crate::classes::default_brand::DefaultBrand) to infer the Brand type parameter from the
//! container's concrete type. This eliminates the need for turbofish
//! on the most common free functions.
//!
//! ### Examples
//!
//! ```
//! use fp_library::functions::*;
//!
//! // No turbofish needed: Brand inferred from Option<i32>
//! let y = map(|x: i32| x * 2, Some(5));
//! assert_eq!(y, Some(10));
//!
//! // Ref dispatch also works: Brand inferred from &Vec<i32>
//! let v = vec![1, 2, 3];
//! let y = map(|x: &i32| *x * 2, &v);
//! assert_eq!(y, vec![2, 4, 6]);
//! ```

pub(crate) mod inner {
	use {
		crate::{
			classes::{
				WithIndex,
				default_brand::DefaultBrand,
				dispatch::{
					filter::FilterDispatch,
					filter_map_with_index::FilterMapWithIndexDispatch,
					filter_with_index::FilterWithIndexDispatch,
					filterable::FilterMapDispatch,
					functor::FunctorDispatch,
					lift::{
						Lift2Dispatch,
						Lift3Dispatch,
						Lift4Dispatch,
						Lift5Dispatch,
					},
					map_with_index::MapWithIndexDispatch,
					partition::PartitionDispatch,
					partition_map::PartitionMapDispatch,
					partition_map_with_index::PartitionMapWithIndexDispatch,
					partition_with_index::PartitionWithIndexDispatch,
					semimonad::BindDispatch,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- map --

	/// Maps a function over a functor, inferring the brand from the container type.
	///
	/// This is the primary API for mapping. The `Brand` type parameter is
	/// inferred from the concrete type of `fa` via [`DefaultBrand`]. Both
	/// owned and borrowed containers are supported:
	///
	/// - Owned: `map(|x: i32| x + 1, Some(5))` infers `OptionBrand`.
	/// - Borrowed: `map(|x: &i32| *x + 1, &Some(5))` infers `OptionBrand`
	///   via the blanket `impl DefaultBrand for &T`.
	///
	/// For types with multiple brands (e.g., `Result`), use
	/// [`map_explicit`](crate::functions::map_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to the value(s).",
		"The functor instance (owned or borrowed)."
	)]
	///
	#[document_returns("A new functor instance containing the result(s) of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// // Brand inferred from Option<i32>
	/// assert_eq!(map(|x: i32| x * 2, Some(5)), Some(10));
	///
	/// // Brand inferred from &Vec<i32> via blanket impl
	/// let v = vec![1, 2, 3];
	/// assert_eq!(map(|x: &i32| *x + 10, &v), vec![11, 12, 13]);
	/// ```
	pub fn map<'a, FA, A: 'a, B: 'a, Marker>(
		f: impl FunctorDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: DefaultBrand, {
		f.dispatch(fa)
	}

	// -- bind --

	/// Sequences a monadic computation, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ma`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`bind_explicit`](crate::functions::bind_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value inside the monad.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The monadic value (owned for Val, borrowed for Ref).",
		"The function to apply to the value."
	)]
	///
	#[document_returns("The result of sequencing the computation.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let result = bind(Some(5), |x: i32| Some(x * 2));
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn bind<'a, FA, A: 'a, B: 'a, Marker>(
		ma: FA,
		f: impl BindDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: DefaultBrand, {
		f.dispatch(ma)
	}

	// -- bind_flipped --

	/// Sequences a monadic computation (flipped argument order), inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ma`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`bind_flipped_explicit`](crate::functions::bind_flipped_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The input element type.",
		"The output element type.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element.",
		"The monadic value (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The result of binding the function over the value.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let result = bind_flipped(|x: i32| Some(x * 2), Some(5));
	/// assert_eq!(result, Some(10));
	/// ```
	pub fn bind_flipped<'a, FA, A: 'a, B: 'a, Marker>(
		f: impl BindDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		ma: FA,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: DefaultBrand, {
		f.dispatch(ma)
	}

	// -- filter_map --

	/// Filters and maps the values in a filterable context, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`filter_map_explicit`](crate::functions::filter_map_explicit) with a turbofish.
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
		f: impl FilterMapDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: DefaultBrand, {
		f.dispatch(fa)
	}

	// -- filter --

	/// Filters the values in a filterable context using a predicate, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`filter_explicit`](crate::functions::filter_explicit()) with a turbofish.
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
		f: impl FilterDispatch<'a, <FA as DefaultBrand>::Brand, A, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		FA: DefaultBrand, {
		f.dispatch(fa)
	}

	// -- partition --

	/// Partitions the values in a filterable context using a predicate, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`partition_explicit`](crate::functions::partition_explicit()) with a turbofish.
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
		f: impl PartitionDispatch<'a, <FA as DefaultBrand>::Brand, A, FA, Marker>,
		fa: FA,
	) -> (
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	)
	where
		FA: DefaultBrand, {
		f.dispatch(fa)
	}

	// -- partition_map --

	/// Partitions the values using a function returning `Result`, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`partition_map_explicit`](crate::functions::partition_map_explicit()) with a turbofish.
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
		f: impl PartitionMapDispatch<'a, <FA as DefaultBrand>::Brand, A, E, O, FA, Marker>,
		fa: FA,
	) -> (
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	)
	where
		FA: DefaultBrand, {
		f.dispatch(fa)
	}

	// -- lift2 --

	/// Lifts a binary function into a functor context, inferring the brand
	/// from the first container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. The dispatch trait constrains `fb` to the same brand.
	///
	/// For types with multiple brands, use
	/// [`lift2_explicit`](crate::functions::lift2_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The first container type. Brand is inferred from this.",
		"The second container type.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to lift.",
		"The first context (owned or borrowed).",
		"The second context (owned or borrowed)."
	)]
	///
	#[document_returns("A new context containing the result of applying the function.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let z = lift2(|a, b| a + b, Some(1), Some(2));
	/// assert_eq!(z, Some(3));
	/// ```
	pub fn lift2<'a, FA, FB, A: 'a, B: 'a, C: 'a, Marker>(
		f: impl Lift2Dispatch<'a, <FA as DefaultBrand>::Brand, A, B, C, FA, FB, Marker>,
		fa: FA,
		fb: FB,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, C>)
	where
		FA: DefaultBrand, {
		f.dispatch(fa, fb)
	}

	// -- lift3 --

	/// Lifts a ternary function into a functor context, inferring the brand
	/// from the first container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. The dispatch trait constrains all other containers
	/// to the same brand.
	///
	/// For types with multiple brands, use
	/// [`lift3_explicit`](crate::functions::lift3_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The first container type. Brand is inferred from this.",
		"The second container type.",
		"The third container type.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the third value.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to lift.",
		"First context (owned or borrowed).",
		"Second context (owned or borrowed).",
		"Third context (owned or borrowed)."
	)]
	///
	#[document_returns("A new context containing the result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let r = lift3(|a, b, c| a + b + c, Some(1), Some(2), Some(3));
	/// assert_eq!(r, Some(6));
	/// ```
	pub fn lift3<'a, FA, FB, FC, A: 'a, B: 'a, C: 'a, D: 'a, Marker>(
		f: impl Lift3Dispatch<'a, <FA as DefaultBrand>::Brand, A, B, C, D, FA, FB, FC, Marker>,
		fa: FA,
		fb: FB,
		fc: FC,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, D>)
	where
		FA: DefaultBrand, {
		f.dispatch(fa, fb, fc)
	}

	// -- lift4 --

	/// Lifts a quaternary function into a functor context, inferring the brand
	/// from the first container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. The dispatch trait constrains all other containers
	/// to the same brand.
	///
	/// For types with multiple brands, use
	/// [`lift4_explicit`](crate::functions::lift4_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The first container type. Brand is inferred from this.",
		"The second container type.",
		"The third container type.",
		"The fourth container type.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the third value.",
		"The type of the fourth value.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to lift.",
		"First context (owned or borrowed).",
		"Second context (owned or borrowed).",
		"Third context (owned or borrowed).",
		"Fourth context (owned or borrowed)."
	)]
	///
	#[document_returns("A new context containing the result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let r = lift4(|a, b, c, d| a + b + c + d, Some(1), Some(2), Some(3), Some(4));
	/// assert_eq!(r, Some(10));
	/// ```
	pub fn lift4<'a, FA, FB, FC, FD, A: 'a, B: 'a, C: 'a, D: 'a, E: 'a, Marker>(
		f: impl Lift4Dispatch<'a, <FA as DefaultBrand>::Brand, A, B, C, D, E, FA, FB, FC, FD, Marker>,
		fa: FA,
		fb: FB,
		fc: FC,
		fd: FD,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>)
	where
		FA: DefaultBrand, {
		f.dispatch(fa, fb, fc, fd)
	}

	// -- lift5 --

	/// Lifts a quinary function into a functor context, inferring the brand
	/// from the first container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. The dispatch trait constrains all other containers
	/// to the same brand.
	///
	/// For types with multiple brands, use
	/// [`lift5_explicit`](crate::functions::lift5_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The first container type. Brand is inferred from this.",
		"The second container type.",
		"The third container type.",
		"The fourth container type.",
		"The fifth container type.",
		"The type of the first value.",
		"The type of the second value.",
		"The type of the third value.",
		"The type of the fourth value.",
		"The type of the fifth value.",
		"The type of the result.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to lift.",
		"1st context (owned or borrowed).",
		"2nd context (owned or borrowed).",
		"3rd context (owned or borrowed).",
		"4th context (owned or borrowed).",
		"5th context (owned or borrowed)."
	)]
	///
	#[document_returns("A new context containing the result.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let r = lift5(|a, b, c, d, e| a + b + c + d + e, Some(1), Some(2), Some(3), Some(4), Some(5));
	/// assert_eq!(r, Some(15));
	/// ```
	pub fn lift5<'a, FA, FB, FC, FD, FE, A: 'a, B: 'a, C: 'a, D: 'a, E: 'a, G: 'a, Marker>(
		f: impl Lift5Dispatch<
			'a,
			<FA as DefaultBrand>::Brand,
			A,
			B,
			C,
			D,
			E,
			G,
			FA,
			FB,
			FC,
			FD,
			FE,
			Marker,
		>,
		fa: FA,
		fb: FB,
		fc: FC,
		fd: FD,
		fe: FE,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, G>)
	where
		FA: DefaultBrand, {
		f.dispatch(fa, fb, fc, fd, fe)
	}

	// -- map_with_index --

	/// Maps a function with index over a functor, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`map_with_index_explicit`](crate::functions::map_with_index_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the functor.",
		"The type of the result(s) of applying the function.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each value and its index.",
		"The functor instance (owned or borrowed)."
	)]
	///
	#[document_returns("A new functor instance containing the results.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let y = map_with_index(|i, x: i32| x + i as i32, vec![10, 20, 30]);
	/// assert_eq!(y, vec![10, 21, 32]);
	/// ```
	pub fn map_with_index<'a, FA, A: 'a, B: 'a, Marker>(
		f: impl MapWithIndexDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		f.dispatch(fa)
	}

	// -- filter_with_index --

	/// Filters the values using a predicate with index, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`filter_with_index_explicit`](crate::functions::filter_with_index_explicit()) with a turbofish.
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
		f: impl FilterWithIndexDispatch<'a, <FA as DefaultBrand>::Brand, A, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		f.dispatch(fa)
	}

	// -- filter_map_with_index --

	/// Filters and maps the values with index, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`filter_map_with_index_explicit`](crate::functions::filter_map_with_index_explicit()) with a turbofish.
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
		f: impl FilterMapWithIndexDispatch<'a, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		fa: FA,
	) -> Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		f.dispatch(fa)
	}

	// -- partition_with_index --

	/// Partitions the values using a predicate with index, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`partition_with_index_explicit`](crate::functions::partition_with_index_explicit()) with a turbofish.
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
		f: impl PartitionWithIndexDispatch<'a, <FA as DefaultBrand>::Brand, A, FA, Marker>,
		fa: FA,
	) -> (
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	)
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		f.dispatch(fa)
	}

	// -- partition_map_with_index --

	/// Partitions the values using a function with index returning `Result`,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`partition_map_with_index_explicit`](crate::functions::partition_map_with_index_explicit()) with a turbofish.
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
		f: impl PartitionMapWithIndexDispatch<'a, <FA as DefaultBrand>::Brand, A, E, O, FA, Marker>,
		fa: FA,
	) -> (
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	)
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		f.dispatch(fa)
	}
}

pub use inner::*;
