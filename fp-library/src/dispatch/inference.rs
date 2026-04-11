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
				Monoid,
				WithIndex,
				default_brand::DefaultBrand,
			},
			dispatch::{
				alt::AltDispatch,
				apply_first::ApplyFirstDispatch,
				apply_second::ApplySecondDispatch,
				compact::CompactDispatch,
				filter::FilterDispatch,
				filter_map_with_index::FilterMapWithIndexDispatch,
				filter_with_index::FilterWithIndexDispatch,
				filterable::FilterMapDispatch,
				fold_left_with_index::FoldLeftWithIndexDispatch,
				fold_map_with_index::FoldMapWithIndexDispatch,
				fold_right_with_index::FoldRightWithIndexDispatch,
				foldable::{
					FoldLeftDispatch,
					FoldMapDispatch,
					FoldRightDispatch,
				},
				functor::FunctorDispatch,
				join::JoinDispatch,
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
				separate::SeparateDispatch,
				traversable::TraverseDispatch,
				traverse_with_index::TraverseWithIndexDispatch,
				wilt::WiltDispatch,
				wither::WitherDispatch,
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

	// -- Tier 2: partial inference (Brand inferred, FnBrand and/or F/M explicit) --

	// -- fold_right --

	/// Folds a structure from the right, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`fold_right_explicit`](crate::functions::fold_right_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
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
	pub fn fold_right<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
		func: impl FoldRightDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		initial: B,
		fa: FA,
	) -> B
	where
		FA: DefaultBrand, {
		func.dispatch(initial, fa)
	}

	// -- fold_left --

	/// Folds a structure from the left, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`fold_left_explicit`](crate::functions::fold_left_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
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
	pub fn fold_left<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
		func: impl FoldLeftDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		initial: B,
		fa: FA,
	) -> B
	where
		FA: DefaultBrand, {
		func.dispatch(initial, fa)
	}

	// -- fold_map --

	/// Maps values to a monoid and combines them, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`fold_map_explicit`](crate::functions::fold_map_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The monoid type.",
		"Dispatch marker type, inferred automatically."
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
	pub fn fold_map<'a, FnBrand, FA, A: 'a, M: Monoid + 'a, Marker>(
		func: impl FoldMapDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, A, M, FA, Marker>,
		fa: FA,
	) -> M
	where
		FA: DefaultBrand, {
		func.dispatch(fa)
	}

	// -- traverse --

	/// Traverses a structure, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via [`DefaultBrand`]. `FnBrand` and `F` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`traverse_explicit`](crate::functions::traverse_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand (must be specified explicitly).",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a value in an applicative context.",
		"The traversable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let y = traverse::<RcFnBrand, _, _, _, OptionBrand, _>(|x: i32| Some(x * 2), Some(5));
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn traverse<'a, FnBrand, FA, A: 'a, B: 'a, F: Kind_cdc7cd43dac7585f, Marker>(
		func: impl TraverseDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, A, B, F, FA, Marker>,
		ta: FA,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		FA: DefaultBrand, {
		func.dispatch(ta)
	}

	// -- fold_map_with_index --

	/// Maps values with their index to a monoid and combines them, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`fold_map_with_index_explicit`](crate::functions::fold_map_with_index_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The monoid type.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The mapping function that receives an index and element.",
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
	/// let result = fold_map_with_index::<RcFnBrand, _, _, _, _>(
	/// 	|i, x: i32| format!("{i}:{x}"),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "0:101:202:30");
	/// ```
	pub fn fold_map_with_index<'a, FnBrand, FA, A: 'a, M: Monoid + 'a, Marker>(
		func: impl FoldMapWithIndexDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, A, M, FA, Marker>,
		fa: FA,
	) -> M
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		func.dispatch(fa)
	}

	// -- fold_right_with_index --

	/// Folds a structure from the right with index, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`fold_right_with_index_explicit`](crate::functions::fold_right_with_index_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The folding function that receives an index, element, and accumulator.",
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
	/// let result = fold_right_with_index::<RcFnBrand, _, _, _, _>(
	/// 	|i, x: i32, acc: String| format!("{acc}{i}:{x},"),
	/// 	String::new(),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "2:30,1:20,0:10,");
	/// ```
	pub fn fold_right_with_index<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
		func: impl FoldRightWithIndexDispatch<
			'a,
			FnBrand,
			<FA as DefaultBrand>::Brand,
			A,
			B,
			FA,
			Marker,
		>,
		initial: B,
		fa: FA,
	) -> B
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		func.dispatch(initial, fa)
	}

	// -- fold_left_with_index --

	/// Folds a structure from the left with index, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. `FnBrand` must still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`fold_left_with_index_explicit`](crate::functions::fold_left_with_index_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements.",
		"The type of the accumulator.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The folding function that receives an index, accumulator, and element.",
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
	/// let result = fold_left_with_index::<RcFnBrand, _, _, _, _>(
	/// 	|i, acc: String, x: i32| format!("{acc}{i}:{x},"),
	/// 	String::new(),
	/// 	vec![10, 20, 30],
	/// );
	/// assert_eq!(result, "0:10,1:20,2:30,");
	/// ```
	pub fn fold_left_with_index<'a, FnBrand, FA, A: 'a + Clone, B: 'a, Marker>(
		func: impl FoldLeftWithIndexDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, A, B, FA, Marker>,
		initial: B,
		fa: FA,
	) -> B
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		func.dispatch(initial, fa)
	}

	// -- traverse_with_index --

	/// Traverses a structure with an indexed effectful function, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via [`DefaultBrand`]. `FnBrand` and `F` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`traverse_with_index_explicit`](crate::functions::traverse_with_index_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"The applicative functor brand (must be specified explicitly).",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The indexed function to apply to each element, returning a value in an applicative context.",
		"The traversable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let y = traverse_with_index::<RcFnBrand, _, _, _, OptionBrand, _>(
	/// 	|_i, x: i32| Some(x * 2),
	/// 	vec![1, 2, 3],
	/// );
	/// assert_eq!(y, Some(vec![2, 4, 6]));
	/// ```
	pub fn traverse_with_index<'a, FnBrand, FA, A: 'a, B: 'a, F: Kind_cdc7cd43dac7585f, Marker>(
		func: impl TraverseWithIndexDispatch<
			'a,
			FnBrand,
			<FA as DefaultBrand>::Brand,
			A,
			B,
			F,
			FA,
			Marker,
		>,
		ta: FA,
	) -> Apply!(<F as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)>)
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: WithIndex, {
		func.dispatch(ta)
	}

	// -- wilt --

	/// Partitions a structure based on a function returning a Result in an applicative context,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via [`DefaultBrand`]. `FnBrand` and `M` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`wilt_explicit`](crate::functions::wilt_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The applicative functor brand (must be specified explicitly).",
		"The type of the elements in the input structure.",
		"The error type.",
		"The success type.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning a Result in an applicative context.",
		"The witherable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The partitioned structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let y = wilt::<RcFnBrand, _, OptionBrand, _, _, _, _>(
	/// 	|a: i32| Some(if a > 2 { Ok(a) } else { Err(a) }),
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some((None, Some(5))));
	/// ```
	pub fn wilt<'a, FnBrand, FA, M: Kind_cdc7cd43dac7585f, A: 'a, E: 'a, O: 'a, Marker>(
		func: impl WiltDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, M, A, E, O, FA, Marker>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		(
			Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		),
	>)
	where
		FA: DefaultBrand, {
		func.dispatch(ta)
	}

	// -- wither --

	/// Maps a function over a data structure and filters out None results in an applicative context,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `ta`
	/// via [`DefaultBrand`]. `FnBrand` and `M` (the applicative brand) must
	/// still be specified explicitly.
	/// Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`wither_explicit`](crate::functions::wither_explicit()) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the cloneable function to use (must be specified explicitly).",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The applicative functor brand (must be specified explicitly).",
		"The type of the elements in the input structure.",
		"The type of the elements in the output structure.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The function to apply to each element, returning an Option in an applicative context.",
		"The witherable structure (owned for Val, borrowed for Ref)."
	)]
	///
	#[document_returns("The filtered structure wrapped in the applicative context.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// let y = wither::<RcFnBrand, _, OptionBrand, _, _, _>(
	/// 	|a: i32| Some(if a > 2 { Some(a * 2) } else { None }),
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some(Some(10)));
	/// ```
	pub fn wither<'a, FnBrand, FA, M: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, Marker>(
		func: impl WitherDispatch<'a, FnBrand, <FA as DefaultBrand>::Brand, M, A, B, FA, Marker>,
		ta: FA,
	) -> Apply!(<M as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
		'a,
		Apply!(<<FA as DefaultBrand>::Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>),
	>)
	where
		FA: DefaultBrand, {
		func.dispatch(ta)
	}

	// -- Tier 3: closureless dispatch (container-type-driven) --

	// -- alt --

	/// Combines two values in a context, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa1`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`alt_explicit`](crate::functions::alt_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the functor.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The first container (owned or borrowed).",
		"The second container (same ownership as the first)."
	)]
	///
	#[document_returns("A new container from the combination of both inputs.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(alt(None, Some(5)), Some(5));
	///
	/// let x = vec![1, 2];
	/// let y = vec![3, 4];
	/// assert_eq!(alt(&x, &y), vec![1, 2, 3, 4]);
	/// ```
	pub fn alt<'a, FA, A: 'a + Clone, Marker>(
		fa1: FA,
		fa2: FA,
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
	where
		FA: DefaultBrand + AltDispatch<'a, <FA as DefaultBrand>::Brand, A, Marker>, {
		fa1.dispatch_alt(fa2)
	}

	// -- compact --

	/// Removes `None` values from a container of `Option`s, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`compact_explicit`](crate::functions::compact_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the `Option` wrappers.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The container of `Option` values (owned or borrowed).")]
	///
	#[document_returns("A new container with `None` values removed and `Some` values unwrapped.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(compact(vec![Some(1), None, Some(3)]), vec![1, 3]);
	///
	/// let v = vec![Some(1), None, Some(3)];
	/// assert_eq!(compact(&v), vec![1, 3]);
	/// ```
	pub fn compact<'a, FA, A: 'a, Marker>(
		fa: FA
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
	where
		FA: DefaultBrand + CompactDispatch<'a, <FA as DefaultBrand>::Brand, A, Marker>, {
		fa.dispatch_compact()
	}

	// -- separate --

	/// Separates a container of `Result` values into two containers, inferring
	/// the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`separate_explicit`](crate::functions::separate_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The error type inside the `Result` wrappers.",
		"The success type inside the `Result` wrappers.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The container of `Result` values (owned or borrowed).")]
	///
	#[document_returns("A tuple of two containers: `Err` values and `Ok` values.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// let (errs, oks) = separate(vec![Ok(1), Err(2), Ok(3)]);
	/// assert_eq!(oks, vec![1, 3]);
	/// assert_eq!(errs, vec![2]);
	/// ```
	pub fn separate<'a, FA, E: 'a, O: 'a, Marker>(
		fa: FA
	) -> (
		<<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, E>,
		<<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, O>,
	)
	where
		FA: DefaultBrand + SeparateDispatch<'a, <FA as DefaultBrand>::Brand, E, O, Marker>, {
		fa.dispatch_separate()
	}

	// -- join --

	/// Removes one layer of monadic nesting, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `mma`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`join_explicit`](crate::functions::join_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the inner layer.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters("The nested monadic value (owned or borrowed).")]
	///
	#[document_returns("A container with one layer of nesting removed.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(join(Some(Some(5))), Some(5));
	///
	/// let x = Some(Some(5));
	/// assert_eq!(join(&x), Some(5));
	/// ```
	pub fn join<'a, FA, A: 'a, Marker>(
		mma: FA
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
	where
		FA: DefaultBrand + JoinDispatch<'a, <FA as DefaultBrand>::Brand, A, Marker>, {
		mma.dispatch_join()
	}

	// -- apply_first --

	/// Sequences two applicative actions, keeping the result of the first,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`apply_first_explicit`](crate::functions::apply_first_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The first container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The first container (its values are preserved).",
		"The second container (its values are discarded)."
	)]
	///
	#[document_returns("A container preserving the values from the first input.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(apply_first(Some(5), Some(10)), Some(5));
	///
	/// let a = Some(5);
	/// let b = Some(10);
	/// assert_eq!(apply_first(&a, &b), Some(5));
	/// ```
	pub fn apply_first<'a, FA, A: 'a, B: 'a, Marker>(
		fa: FA,
		fb: <FA as ApplyFirstDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>>::FB,
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>
	where
		FA: DefaultBrand + ApplyFirstDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>, {
		fa.dispatch_apply_first(fb)
	}

	// -- apply_second --

	/// Sequences two applicative actions, keeping the result of the second,
	/// inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`apply_second_explicit`](crate::functions::apply_second_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The first container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the first container.",
		"The type of the value(s) inside the second container.",
		"Dispatch marker type, inferred automatically."
	)]
	///
	#[document_parameters(
		"The first container (its values are discarded).",
		"The second container (its values are preserved)."
	)]
	///
	#[document_returns("A container preserving the values from the second input.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::functions::*;
	///
	/// assert_eq!(apply_second(Some(5), Some(10)), Some(10));
	///
	/// let a = Some(5);
	/// let b = Some(10);
	/// assert_eq!(apply_second(&a, &b), Some(10));
	/// ```
	pub fn apply_second<'a, FA, A: 'a, B: 'a, Marker>(
		fa: FA,
		fb: <FA as ApplySecondDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>>::FB,
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
	where
		FA: DefaultBrand + ApplySecondDispatch<'a, <FA as DefaultBrand>::Brand, A, B, Marker>, {
		fa.dispatch_apply_second(fb)
	}

	// -- contramap --

	/// Contravariantly maps a function over a value, inferring the brand from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via [`DefaultBrand`]. Only owned containers are supported; there is no
	/// `RefContravariant` trait because the Ref pattern is about closures
	/// receiving element references (`&A`), but `contramap`'s closure
	/// produces elements (`Fn(B) -> A`), not consumes them. The
	/// directionality is reversed compared to [`Functor`](crate::classes::Functor),
	/// so the `&A` convention does not apply.
	///
	/// For types with multiple brands, use
	/// [`contramap_explicit`](crate::functions::contramap_explicit) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type. Brand is inferred from this.",
		"The type of the value(s) inside the contravariant functor.",
		"The type that the new contravariant functor accepts."
	)]
	///
	#[document_parameters(
		"The function mapping the new input type to the original input type.",
		"The contravariant functor instance."
	)]
	///
	#[document_returns("A new contravariant functor that accepts values of type `B`.")]
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	functions::*,
	/// };
	///
	/// // contramap requires DefaultBrand on the container type.
	/// // Most profunctor-based types do not implement DefaultBrand,
	/// // so use contramap_explicit for those.
	/// assert!(true);
	/// ```
	pub fn contramap<'a, FA, A: 'a, B: 'a>(
		f: impl Fn(B) -> A + 'a,
		fa: FA,
	) -> <<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, B>
	where
		FA: DefaultBrand,
		<FA as DefaultBrand>::Brand: crate::classes::Contravariant,
		FA: Into<<<FA as DefaultBrand>::Brand as Kind_cdc7cd43dac7585f>::Of<'a, A>>, {
		<<FA as DefaultBrand>::Brand as crate::classes::Contravariant>::contramap(f, fa.into())
	}
}

pub use inner::*;
