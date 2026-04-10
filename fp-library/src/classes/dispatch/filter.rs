//! Dispatch for [`Filterable::filter`](crate::classes::Filterable::filter) and
//! [`RefFilterable::ref_filter`](crate::classes::RefFilterable::ref_filter).
//!
//! Provides the [`FilterDispatch`] trait and a unified [`filter`] free function
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
//! // Owned: dispatches to Filterable::filter
//! let y = filter::<OptionBrand, _, _, _>(|x: i32| x > 3, Some(5));
//! assert_eq!(y, Some(5));
//!
//! // By-ref: dispatches to RefFilterable::ref_filter
//! let y = filter::<OptionBrand, _, _, _>(|x: &i32| *x > 3, &Some(5));
//! assert_eq!(y, Some(5));
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
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
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
		/// 	functions::*,
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
		/// 	functions::*,
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
		/// 	functions::*,
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

	// -- Unified free function --

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
	/// 	functions::*,
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
}

pub use inner::*;
