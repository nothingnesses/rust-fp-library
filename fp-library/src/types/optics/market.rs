//! The `Market` profunctor, used for prisms.
//!
//! `Market<A, B, S, T>` wraps a preview function `S -> Result<A, T>` and a review function `B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				CloneableFn,
				profunctor::{
					Choice,
					Profunctor,
				},
			},
			impl_kind,
			kinds::*,
		},
		fp_macros::{
			document_parameters,
			document_type_parameters,
		},
		std::marker::PhantomData,
	};

	/// The `Market` profunctor.
	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	pub struct Market<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Preview function: tries to extract the focus.
		pub preview: <FnBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
		/// Review function: constructs the structure.
		pub review: <FnBrand as CloneableFn>::Of<'a, B, T>,
		pub(crate) _phantom: PhantomData<(A, B)>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Market<'a, FnBrand, A, B, S, T> {
		/// Creates a new `Market` instance.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The review function.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::RcFnBrand,
		/// 	classes::cloneable_fn::new as cloneable_fn_new,
		/// 	types::optics::Market,
		/// };
		///
		/// let market = Market::<RcFnBrand, i32, i32, String, String>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: String| {
		/// 		s.parse::<i32>().map_err(|_| "error".to_string())
		/// 	}),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|n: i32| n.to_string()),
		/// );
		/// assert_eq!((market.preview)("123".to_string()), Ok(123));
		/// assert_eq!((market.review)(456), "456".to_string());
		/// ```
		pub fn new(
			preview: <FnBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
			review: <FnBrand as CloneableFn>::Of<'a, B, T>,
		) -> Self {
			Market {
				preview,
				review,
				_phantom: PhantomData,
			}
		}
	}

	/// Brand for the `Market` profunctor.
	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function."
	)]
	pub struct MarketBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

	impl_kind! {
		impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for MarketBrand<FnBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Market<'a, FnBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor
		for MarketBrand<FnBrand, A, B>
	{
		/// Maps functions over the input and output of the `Market` profunctor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the new structure.",
			"The target type of the new structure.",
			"The source type of the original structure.",
			"The target type of the original structure.",
			"The type of the function to apply to the input.",
			"The type of the function to apply to the output."
		)]
		///
		#[document_parameters(
			"The function to apply to the input.",
			"The function to apply to the output.",
			"The market instance to transform."
		)]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Market is usually used internally by Prism optics
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a, FuncST, FuncUV>(
			st: FuncST,
			uv: FuncUV,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>)
		where
			FuncST: Fn(S) -> T + 'a,
			FuncUV: Fn(U) -> V + 'a, {
			let preview = puv.preview;
			let review = puv.review;
			let st = <FnBrand as CloneableFn>::new(st);
			let uv = <FnBrand as CloneableFn>::new(uv);
			let uv_2 = uv.clone();
			Market::new(
				<FnBrand as CloneableFn>::new(move |s: S| {
					(*preview)((*st)(s)).map_err(|u| (*uv)(u))
				}),
				<FnBrand as CloneableFn>::new(move |b: B| (*uv_2)((*review)(b))),
			)
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function."
	)]
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Choice for MarketBrand<FnBrand, A, B> {
		/// Lifts the `Market` profunctor to operate on the left component of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The market instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Market is usually used internally by Prism optics
		/// ```
		fn left<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, S>, Result<C, T>>)
		{
			let preview = pab.preview;
			let review = pab.review;
			Market::new(
				<FnBrand as CloneableFn>::new(move |r: Result<C, S>| match r {
					Ok(c) => Err(Ok(c)),
					Err(s) => (*preview)(s).map_err(Err),
				}),
				<FnBrand as CloneableFn>::new(move |b: B| Err((*review)(b))),
			)
		}

		/// Lifts the `Market` profunctor to operate on the right component of a `Result`.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the structure.",
			"The target type of the structure.",
			"The type of the other component."
		)]
		///
		#[document_parameters("The market instance to transform.")]
		///
		/// ### Examples
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	classes::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Market is usually used internally by Prism optics
		/// ```
		fn right<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<S, C>, Result<T, C>>)
		{
			let preview = pab.preview;
			let review = pab.review;
			Market::new(
				<FnBrand as CloneableFn>::new(move |r: Result<S, C>| match r {
					Ok(s) => (*preview)(s).map_err(Ok),
					Err(c) => Err(Err(c)),
				}),
				<FnBrand as CloneableFn>::new(move |b: B| Ok((*review)(b))),
			)
		}
	}
}
pub use inner::*;
