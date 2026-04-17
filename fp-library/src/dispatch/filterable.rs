//! Dispatch for filterable operations:
//! [`Filterable`](crate::classes::Filterable) and
//! [`RefFilterable`](crate::classes::RefFilterable).
//!
//! Provides the following dispatch traits and unified free functions:
//!
//! - [`FilterMapDispatch`] + [`explicit::filter_map`]
//! - [`FilterDispatch`] + [`explicit::filter`]
//! - [`PartitionDispatch`] + [`explicit::partition`]
//! - [`PartitionMapDispatch`] + [`explicit::partition_map`]
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
//! // filter_map: Owned
//! let y =
//! 	filter_map::<OptionBrand, _, _, _, _>(|x: i32| if x > 3 { Some(x) } else { None }, Some(5));
//! assert_eq!(y, Some(5));
//!
//! // filter: Owned
//! let y = filter::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
//! assert_eq!(y, Some(5));
//!
//! // partition: Owned
//! let (no, yes) = partition::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
//! assert_eq!(yes, Some(5));
//! assert_eq!(no, None);
//!
//! // partition_map: Owned
//! let (errs, oks) =
//! 	partition_map::<OptionBrand, _, _, _, _, _>(|x: i32| Ok::<i32, i32>(x * 2), Some(5));
//! assert_eq!(errs, None);
//! assert_eq!(oks, Some(10));
//! ```

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			classes::{
				Filterable,
				RefFilterable,
			},
			dispatch::{
				Ref,
				Val,
			},
			kinds::*,
		},
		fp_macros::*,
	};

	// -- FilterMapDispatch --

	/// Trait that routes a filter_map operation to the appropriate type class method.
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
	pub trait FilterMapDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker> {
		/// Perform the dispatched filter_map operation.
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
		/// let result = filter_map::<OptionBrand, _, _, _, _>(
		/// 	|x: i32| if x > 3 { Some(x * 2) } else { None },
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
	}

	// -- Val: Fn(A) -> Option<B> -> Filterable::filter_map --

	/// Routes `Fn(A) -> Option<B>` closures to [`Filterable::filter_map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The type of the result(s) of applying the function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, B, F>
		FilterMapDispatch<
			'a,
			Brand,
			A,
			B,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: Filterable,
		A: 'a,
		B: 'a,
		F: Fn(A) -> Option<B> + 'a,
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
		/// let result = filter_map::<OptionBrand, _, _, _, _>(
		/// 	|x: i32| if x > 3 { Some(x * 2) } else { None },
		/// 	Some(5),
		/// );
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::filter_map(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> Option<B> -> RefFilterable::ref_filter_map --

	/// Routes `Fn(&A) -> Option<B>` closures to [`RefFilterable::ref_filter_map`].
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
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, B, F>
		FilterMapDispatch<
			'a,
			Brand,
			A,
			B,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFilterable,
		A: 'a,
		B: 'a,
		F: Fn(&A) -> Option<B> + 'a,
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
		/// let result = filter_map::<OptionBrand, _, _, _, _>(
		/// 	|x: &i32| if *x > 3 { Some(*x * 2) } else { None },
		/// 	&Some(5),
		/// );
		/// assert_eq!(result, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Brand::ref_filter_map(self, fa)
		}
	}

	// -- FilterDispatch --

	/// Trait that routes a filter operation to the appropriate type class method.
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
	pub trait FilterDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, FA, Marker> {
		/// Perform the dispatched filter operation.
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
		/// let result = filter::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
	}

	// -- Val: Fn(A) -> bool -> Filterable::filter --

	/// Routes `Fn(A) -> bool` closures to [`Filterable::filter`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, F>
		FilterDispatch<
			'a,
			Brand,
			A,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: Filterable,
		A: 'a + Clone,
		F: Fn(A) -> bool + 'a,
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
		/// let result = filter::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::filter(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> bool -> RefFilterable::ref_filter --

	/// Routes `Fn(&A) -> bool` closures to [`RefFilterable::ref_filter`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, F>
		FilterDispatch<
			'a,
			Brand,
			A,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFilterable,
		A: 'a + Clone,
		F: Fn(&A) -> bool + 'a,
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
		/// let result = filter::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
		/// assert_eq!(result, Some(5));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			Brand::ref_filter(self, fa)
		}
	}

	// -- PartitionDispatch --

	/// Trait that routes a partition operation to the appropriate type class method.
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
	pub trait PartitionDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, FA, Marker> {
		/// Perform the dispatched partition operation.
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains elements satisfying the predicate, the second contains the rest."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let (no, yes) = partition::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		/// assert_eq!(yes, Some(5));
		/// assert_eq!(no, None);
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		);
	}

	// -- Val: Fn(A) -> bool -> Filterable::partition --

	/// Routes `Fn(A) -> bool` closures to [`Filterable::partition`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, F>
		PartitionDispatch<
			'a,
			Brand,
			A,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: Filterable,
		A: 'a + Clone,
		F: Fn(A) -> bool + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("The filterable instance containing the value(s).")]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains elements satisfying the predicate, the second contains the rest."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let (no, yes) = partition::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		/// assert_eq!(yes, Some(5));
		/// assert_eq!(no, None);
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			Brand::partition(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> bool -> RefFilterable::ref_partition --

	/// Routes `Fn(&A) -> bool` closures to [`RefFilterable::ref_partition`].
	///
	/// The container must be passed by reference (`&fa`).
	#[document_type_parameters(
		"The lifetime of the values.",
		"The borrow lifetime.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, F>
		PartitionDispatch<
			'a,
			Brand,
			A,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFilterable,
		A: 'a + Clone,
		F: Fn(&A) -> bool + 'a,
	{
		#[document_signature]
		///
		#[document_parameters("A reference to the filterable instance.")]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains elements satisfying the predicate, the second contains the rest."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	functions::explicit::*,
		/// };
		///
		/// let (no, yes) = partition::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
		/// assert_eq!(yes, Some(5));
		/// assert_eq!(no, None);
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			Brand::ref_partition(self, fa)
		}
	}

	// -- PartitionMapDispatch --

	/// Trait that routes a partition_map operation to the appropriate type class method.
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
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::dispatch::Val) or [`Ref`](crate::dispatch::Ref)."
	)]
	#[document_parameters("The closure implementing this dispatch.")]
	pub trait PartitionMapDispatch<
		'a,
		Brand: Kind_cdc7cd43dac7585f,
		A: 'a,
		E: 'a,
		O: 'a,
		FA,
		Marker,
	> {
		/// Perform the dispatched partition_map operation.
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
		/// let (errs, oks) =
		/// 	partition_map::<OptionBrand, _, _, _, _, _>(|x: i32| Ok::<i32, i32>(x * 2), Some(5));
		/// assert_eq!(errs, None);
		/// assert_eq!(oks, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: FA,
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		);
	}

	// -- Val: Fn(A) -> Result<O, E> -> Filterable::partition_map --

	/// Routes `Fn(A) -> Result<O, E>` closures to [`Filterable::partition_map`].
	#[document_type_parameters(
		"The lifetime of the values.",
		"The brand of the filterable.",
		"The type of the value(s) inside the filterable.",
		"The error type produced by the partitioning function.",
		"The success type produced by the partitioning function.",
		"The closure type."
	)]
	#[document_parameters("The closure that takes owned values.")]
	impl<'a, Brand, A, E, O, F>
		PartitionMapDispatch<
			'a,
			Brand,
			A,
			E,
			O,
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Val,
		> for F
	where
		Brand: Filterable,
		A: 'a,
		E: 'a,
		O: 'a,
		F: Fn(A) -> Result<O, E> + 'a,
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
		/// let (errs, oks) =
		/// 	partition_map::<OptionBrand, _, _, _, _, _>(|x: i32| Ok::<i32, i32>(x * 2), Some(5));
		/// assert_eq!(errs, None);
		/// assert_eq!(oks, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::partition_map(self, fa)
		}
	}

	// -- Ref: Fn(&A) -> Result<O, E> -> RefFilterable::ref_partition_map --

	/// Routes `Fn(&A) -> Result<O, E>` closures to [`RefFilterable::ref_partition_map`].
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
	#[document_parameters("The closure that takes references.")]
	impl<'a, 'b, Brand, A, E, O, F>
		PartitionMapDispatch<
			'a,
			Brand,
			A,
			E,
			O,
			&'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Ref,
		> for F
	where
		Brand: RefFilterable,
		A: 'a,
		E: 'a,
		O: 'a,
		F: Fn(&A) -> Result<O, E> + 'a,
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
		/// let (errs, oks) =
		/// 	partition_map::<OptionBrand, _, _, _, _, _>(|x: &i32| Ok::<i32, i32>(*x * 2), &Some(5));
		/// assert_eq!(errs, None);
		/// assert_eq!(oks, Some(10));
		/// ```
		fn dispatch(
			self,
			fa: &'b Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
		) {
			Brand::ref_partition_map(self, fa)
		}
	}

	// -- Inference wrappers --

	/// Filters the values in a filterable context using a predicate, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `Slot` trait. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::filter`](crate::functions::explicit::filter) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"The brand, inferred via Slot from FA and the element type."
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
	pub fn filter<'a, FA, A: 'a + Clone, Brand>(
		f: impl FilterDispatch<'a, Brand, A, FA, <FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	/// Filters and maps the values in a filterable context, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `Slot` trait. Both owned and borrowed containers are supported.
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
		"The brand, inferred via Slot from FA and the element type."
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
	pub fn filter_map<'a, FA, A: 'a, B: 'a, Brand>(
		f: impl FilterMapDispatch<
			'a,
			Brand,
			A,
			B,
			FA,
			<FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	/// Partitions the values in a filterable context using a predicate, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `Slot` trait. Both owned and borrowed containers are supported.
	///
	/// For types with multiple brands, use
	/// [`explicit::partition`](crate::functions::explicit::partition) with a turbofish.
	#[document_signature]
	///
	#[document_type_parameters(
		"The lifetime of the values.",
		"The container type (owned or borrowed). Brand is inferred from this.",
		"The type of the value(s) inside the filterable.",
		"The brand, inferred via Slot from FA and the element type."
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
	pub fn partition<'a, FA, A: 'a + Clone, Brand>(
		f: impl PartitionDispatch<'a, Brand, A, FA, <FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker>,
		fa: FA,
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	/// Partitions the values using a function returning `Result`, inferring the brand
	/// from the container type.
	///
	/// The `Brand` type parameter is inferred from the concrete type of `fa`
	/// via the `Slot` trait. Both owned and borrowed containers are supported.
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
		"The brand, inferred via Slot from FA and the element type."
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
	pub fn partition_map<'a, FA, A: 'a, E: 'a, O: 'a, Brand>(
		f: impl PartitionMapDispatch<
			'a,
			Brand,
			A,
			E,
			O,
			FA,
			<FA as Slot_cdc7cd43dac7585f<'a, Brand, A>>::Marker,
		>,
		fa: FA,
	) -> (
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>),
		Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, O>),
	)
	where
		Brand: Kind_cdc7cd43dac7585f,
		FA: Slot_cdc7cd43dac7585f<'a, Brand, A>, {
		f.dispatch(fa)
	}

	// -- Explicit dispatch free functions --

	/// Explicit dispatch functions requiring a Brand turbofish.
	///
	/// For most use cases, prefer the inference-enabled wrappers from
	/// [`functions`](crate::functions).
	pub mod explicit {
		use super::*;

		/// Filters and maps the values in a filterable context.
		///
		/// Dispatches to either [`Filterable::filter_map`] or
		/// [`RefFilterable::ref_filter_map`] based on the closure's argument type:
		///
		/// - If the closure takes owned values (`Fn(A) -> Option<B>`) and the
		///   container is owned, dispatches to [`Filterable::filter_map`].
		/// - If the closure takes references (`Fn(&A) -> Option<B>`) and the
		///   container is borrowed (`&fa`), dispatches to
		///   [`RefFilterable::ref_filter_map`].
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler from the closure's argument type and the container argument.
		/// Callers write `filter_map::<Brand, _, _, _, _>(...)` and never need to
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
			"The function to apply to each value. Returns `Some(b)` to keep the value or `None` to discard it.",
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
		/// // Owned: dispatches to Filterable::filter_map
		/// let y =
		/// 	filter_map::<OptionBrand, _, _, _, _>(|x: i32| if x > 3 { Some(x) } else { None }, Some(5));
		/// assert_eq!(y, Some(5));
		///
		/// // By-ref: dispatches to RefFilterable::ref_filter_map
		/// let y = filter_map::<OptionBrand, _, _, _, _>(
		/// 	|x: &i32| if *x > 3 { Some(*x) } else { None },
		/// 	&Some(5),
		/// );
		/// assert_eq!(y, Some(5));
		/// ```
		pub fn filter_map<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker>(
			f: impl FilterMapDispatch<'a, Brand, A, B, FA, Marker>,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			f.dispatch(fa)
		}

		/// Filters the values in a filterable context using a predicate.
		///
		/// Dispatches to either [`Filterable::filter`] or
		/// [`RefFilterable::ref_filter`] based on the closure's argument type:
		///
		/// - If the closure takes owned values (`Fn(A) -> bool`) and the
		///   container is owned, dispatches to [`Filterable::filter`].
		/// - If the closure takes references (`Fn(&A) -> bool`) and the
		///   container is borrowed (`&fa`), dispatches to
		///   [`RefFilterable::ref_filter`].
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler from the closure's argument type and the container argument.
		/// Callers write `filter::<Brand, _, _, _>(...)` and never need to
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
			"The predicate to apply to each value. Returns `true` to keep the value or `false` to discard it.",
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
		/// // Owned: dispatches to Filterable::filter
		/// let y = filter::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		/// assert_eq!(y, Some(5));
		///
		/// // By-ref: dispatches to RefFilterable::ref_filter
		/// let y = filter::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
		/// assert_eq!(y, Some(5));
		/// ```
		pub fn filter<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, FA, Marker>(
			f: impl FilterDispatch<'a, Brand, A, FA, Marker>,
			fa: FA,
		) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>) {
			f.dispatch(fa)
		}

		/// Partitions the values in a filterable context using a predicate.
		///
		/// Dispatches to either [`Filterable::partition`] or
		/// [`RefFilterable::ref_partition`] based on the closure's argument type:
		///
		/// - If the closure takes owned values (`Fn(A) -> bool`) and the
		///   container is owned, dispatches to [`Filterable::partition`].
		/// - If the closure takes references (`Fn(&A) -> bool`) and the
		///   container is borrowed (`&fa`), dispatches to
		///   [`RefFilterable::ref_partition`].
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler from the closure's argument type and the container argument.
		/// Callers write `partition::<Brand, _, _, _>(...)` and never need to
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
			"The predicate to apply to each value. Returns `true` for the first partition or `false` for the second.",
			"The filterable instance (owned for Val, borrowed for Ref)."
		)]
		///
		#[document_returns(
			"A tuple of two filterable instances: the first contains elements satisfying the predicate, the second contains the rest."
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
		/// // Owned: dispatches to Filterable::partition
		/// let (no, yes) = partition::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
		/// assert_eq!(yes, Some(5));
		/// assert_eq!(no, None);
		///
		/// // By-ref: dispatches to RefFilterable::ref_partition
		/// let (no, yes) = partition::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
		/// assert_eq!(yes, Some(5));
		/// assert_eq!(no, None);
		/// ```
		pub fn partition<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a + Clone, FA, Marker>(
			f: impl PartitionDispatch<'a, Brand, A, FA, Marker>,
			fa: FA,
		) -> (
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) {
			f.dispatch(fa)
		}

		/// Partitions the values in a filterable context using a function that returns `Result`.
		///
		/// Dispatches to either [`Filterable::partition_map`] or
		/// [`RefFilterable::ref_partition_map`] based on the closure's argument type:
		///
		/// - If the closure takes owned values (`Fn(A) -> Result<O, E>`) and the
		///   container is owned, dispatches to [`Filterable::partition_map`].
		/// - If the closure takes references (`Fn(&A) -> Result<O, E>`) and the
		///   container is borrowed (`&fa`), dispatches to
		///   [`RefFilterable::ref_partition_map`].
		///
		/// The `Marker` and `FA` type parameters are inferred automatically by the
		/// compiler from the closure's argument type and the container argument.
		/// Callers write `partition_map::<Brand, _, _, _, _, _>(...)` and never need to
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
			"The function to apply to each value. Returns `Ok(o)` for the success partition or `Err(e)` for the error partition.",
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
		/// // Owned: dispatches to Filterable::partition_map
		/// let (errs, oks) =
		/// 	partition_map::<OptionBrand, _, _, _, _, _>(|x: i32| Ok::<i32, i32>(x * 2), Some(5));
		/// assert_eq!(errs, None);
		/// assert_eq!(oks, Some(10));
		///
		/// // By-ref: dispatches to RefFilterable::ref_partition_map
		/// let (errs, oks) =
		/// 	partition_map::<OptionBrand, _, _, _, _, _>(|x: &i32| Ok::<i32, i32>(*x * 2), &Some(5));
		/// assert_eq!(errs, None);
		/// assert_eq!(oks, Some(10));
		/// ```
		pub fn partition_map<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, E: 'a, O: 'a, FA, Marker>(
			f: impl PartitionMapDispatch<'a, Brand, A, E, O, FA, Marker>,
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
