//! Dispatch for [`Filterable::filter_map`](crate::classes::Filterable::filter_map) and
//! [`RefFilterable::ref_filter_map`](crate::classes::RefFilterable::ref_filter_map).
//!
//! Provides the [`FilterMapDispatch`] trait and a unified [`filter_map`] free function
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
//! // Owned: dispatches to Filterable::filter_map
//! let y = filter_map_explicit::<OptionBrand, _, _, _, _>(
//! 	|x: i32| if x > 3 { Some(x) } else { None },
//! 	Some(5),
//! );
//! assert_eq!(y, Some(5));
//!
//! // By-ref: dispatches to RefFilterable::ref_filter_map
//! let y = filter_map_explicit::<OptionBrand, _, _, _, _>(
//! 	|x: &i32| if *x > 3 { Some(*x) } else { None },
//! 	&Some(5),
//! );
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
		"Dispatch marker type, inferred automatically. Either [`Val`](crate::classes::dispatch::Val) or [`Ref`](crate::classes::dispatch::Ref)."
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
		/// 	functions::*,
		/// };
		///
		/// let result = filter_map_explicit::<OptionBrand, _, _, _, _>(
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
		/// 	functions::*,
		/// };
		///
		/// let result = filter_map_explicit::<OptionBrand, _, _, _, _>(
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
		/// 	functions::*,
		/// };
		///
		/// let result = filter_map_explicit::<OptionBrand, _, _, _, _>(
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

	// -- Unified free function --

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
	/// Callers write `filter_map_explicit::<Brand, _, _, _, _>(...)` and never need to
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
	/// 	functions::*,
	/// };
	///
	/// // Owned: dispatches to Filterable::filter_map
	/// let y = filter_map_explicit::<OptionBrand, _, _, _, _>(
	/// 	|x: i32| if x > 3 { Some(x) } else { None },
	/// 	Some(5),
	/// );
	/// assert_eq!(y, Some(5));
	///
	/// // By-ref: dispatches to RefFilterable::ref_filter_map
	/// let y = filter_map_explicit::<OptionBrand, _, _, _, _>(
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
}

pub use inner::*;
