//! Dispatch for [`Filterable::partition`](crate::classes::Filterable::partition) and
//! [`RefFilterable::ref_partition`](crate::classes::RefFilterable::ref_partition).
//!
//! Provides the [`PartitionDispatch`] trait and a unified [`partition`] free function
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
//! // Owned: dispatches to Filterable::partition
//! let (no, yes) = partition_explicit::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
//! assert_eq!(yes, Some(5));
//! assert_eq!(no, None);
//!
//! // By-ref: dispatches to RefFilterable::ref_partition
//! let (no, yes) = partition_explicit::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
//! assert_eq!(yes, Some(5));
//! assert_eq!(no, None);
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
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
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
		/// 	functions::*,
		/// };
		///
		/// let (no, yes) = partition_explicit::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
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
		/// 	functions::*,
		/// };
		///
		/// let (no, yes) = partition_explicit::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
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
		/// 	functions::*,
		/// };
		///
		/// let (no, yes) = partition_explicit::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
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

	// -- Unified free function --

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
	/// Callers write `partition_explicit::<Brand, _, _, _>(...)` and never need to
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
	/// 	functions::*,
	/// };
	///
	/// // Owned: dispatches to Filterable::partition
	/// let (no, yes) = partition_explicit::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
	/// assert_eq!(yes, Some(5));
	/// assert_eq!(no, None);
	///
	/// // By-ref: dispatches to RefFilterable::ref_partition
	/// let (no, yes) = partition_explicit::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
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
}

pub use inner::*;
