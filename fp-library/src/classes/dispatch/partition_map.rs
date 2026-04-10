//! Dispatch for [`Filterable::partition_map`](crate::classes::Filterable::partition_map) and
//! [`RefFilterable::ref_partition_map`](crate::classes::RefFilterable::ref_partition_map).
//!
//! Provides the [`PartitionMapDispatch`] trait and a unified [`partition_map`] free function
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
//! // Owned: dispatches to Filterable::partition_map
//! let (errs, oks) =
//! 	partition_map::<OptionBrand, _, _, _, _, _>(|x: i32| Ok::<i32, i32>(x * 2), Some(5));
//! assert_eq!(errs, None);
//! assert_eq!(oks, Some(10));
//!
//! // By-ref: dispatches to RefFilterable::ref_partition_map
//! let (errs, oks) =
//! 	partition_map::<OptionBrand, _, _, _, _, _>(|x: &i32| Ok::<i32, i32>(*x * 2), &Some(5));
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
				dispatch::{
					Ref,
					Val,
				},
			},
			kinds::*,
		},
		fp_macros::*,
	};

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
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
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
		/// 	functions::*,
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
		/// 	functions::*,
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
		/// 	functions::*,
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

	// -- Unified free function --

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
	/// 	functions::*,
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

pub use inner::*;
