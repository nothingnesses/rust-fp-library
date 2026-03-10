//! The `Market` profunctor, used for prisms.
//!
//! `Market<A, B, S, T>` wraps a preview function `S -> Result<A, T>` and a review function `B -> T`.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::optics::*,
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
		fp_macros::*,
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
	pub struct Market<'a, FunctionBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
		/// Preview function: tries to extract the focus.
		pub preview: <FunctionBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
		/// Review function: constructs the structure.
		pub review: <FunctionBrand as CloneableFn>::Of<'a, B, T>,
	}

	#[document_type_parameters(
		"The lifetime of the functions.",
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function.",
		"The source type of the structure.",
		"The target type of the structure."
	)]
	impl<'a, FunctionBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a>
		Market<'a, FunctionBrand, A, B, S, T>
	{
		/// Creates a new `Market` instance.
		#[document_signature]
		///
		#[document_parameters("The preview function.", "The review function.")]
		///
		#[document_returns("A new instance of the type.")]
		///
		#[document_examples]
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
			preview: <FunctionBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
			review: <FunctionBrand as CloneableFn>::Of<'a, B, T>,
		) -> Self {
			Market {
				preview,
				review,
			}
		}
	}

	impl_kind! {
		impl<FunctionBrand: CloneableFn + 'static, A: 'static, B: 'static> for MarketBrand<FunctionBrand, A, B> {
			#[document_default]
			type Of<'a, S: 'a, T: 'a>: 'a = Market<'a, FunctionBrand, A, B, S, T>;
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function."
	)]
	impl<FunctionBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor
		for MarketBrand<FunctionBrand, A, B>
	{
		/// Maps functions over the input and output of the `Market` profunctor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the functions.",
			"The source type of the new structure.",
			"The target type of the new structure.",
			"The source type of the original structure.",
			"The target type of the original structure."
		)]
		///
		#[document_parameters(
			"The function to apply to the input.",
			"The function to apply to the output.",
			"The market instance to transform."
		)]
		///
		#[document_returns("A transformed `Market` instance.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// // Market is usually used internally by Prism optics
		/// let market = Market::<RcFnBrand, i32, i32, String, String>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: String| {
		/// 		s.parse::<i32>().map_err(|_| "error".to_string())
		/// 	}),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|n: i32| n.to_string()),
		/// );
		/// let transformed = <MarketBrand<RcFnBrand, i32, i32> as Profunctor>::dimap(
		/// 	|s: String| s,
		/// 	|s: String| s,
		/// 	market,
		/// );
		/// assert_eq!((transformed.preview)("123".to_string()), Ok(123));
		/// ```
		fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a>(
			st: impl Fn(S) -> T + 'a,
			uv: impl Fn(U) -> V + 'a,
			puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>) {
			let preview = puv.preview;
			let review = puv.review;
			let st = <FunctionBrand as CloneableFn>::new(st);
			let uv = <FunctionBrand as CloneableFn>::new(uv);
			let uv_2 = uv.clone();
			Market::new(
				<FunctionBrand as CloneableFn>::new(move |s: S| {
					(*preview)((*st)(s)).map_err(|u| (*uv)(u))
				}),
				<FunctionBrand as CloneableFn>::new(move |b: B| (*uv_2)((*review)(b))),
			)
		}
	}

	#[document_type_parameters(
		"The cloneable function brand.",
		"The type of the value produced by the preview function.",
		"The type of the value consumed by the review function."
	)]
	impl<FunctionBrand: CloneableFn + 'static, A: 'static, B: 'static> Choice
		for MarketBrand<FunctionBrand, A, B>
	{
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
		#[document_returns("A transformed `Market` instance that operates on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let market = Market::<RcFnBrand, i32, i32, i32, i32>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s| Ok(s)),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|b| b),
		/// );
		/// let left_market = <MarketBrand<RcFnBrand, i32, i32> as Choice>::left::<i32, i32, i32>(market);
		/// assert_eq!((left_market.preview)(Err(42)), Ok(42));
		/// ```
		fn left<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, S>, Result<C, T>>)
		{
			let preview = pab.preview;
			let review = pab.review;
			Market::new(
				<FunctionBrand as CloneableFn>::new(move |r: Result<C, S>| match r {
					Ok(c) => Err(Ok(c)),
					Err(s) => (*preview)(s).map_err(Err),
				}),
				<FunctionBrand as CloneableFn>::new(move |b: B| Err((*review)(b))),
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
		#[document_returns("A transformed `Market` instance that operates on `Result` types.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		optics::*,
		/// 		*,
		/// 	},
		/// 	classes::{
		/// 		cloneable_fn::new as cloneable_fn_new,
		/// 		optics::*,
		/// 		profunctor::*,
		/// 	},
		/// 	types::optics::*,
		/// };
		///
		/// let market = Market::<RcFnBrand, i32, i32, i32, i32>::new(
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s| Ok(s)),
		/// 	cloneable_fn_new::<RcFnBrand, _, _>(|b| b),
		/// );
		/// let right_market = <MarketBrand<RcFnBrand, i32, i32> as Choice>::right::<i32, i32, i32>(market);
		/// assert_eq!((right_market.preview)(Ok(42)), Ok(42));
		/// ```
		fn right<'a, S: 'a, T: 'a, C: 'a>(
			pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<S, C>, Result<T, C>>)
		{
			let preview = pab.preview;
			let review = pab.review;
			Market::new(
				<FunctionBrand as CloneableFn>::new(move |r: Result<S, C>| match r {
					Ok(s) => (*preview)(s).map_err(Ok),
					Err(c) => Err(Err(c)),
				}),
				<FunctionBrand as CloneableFn>::new(move |b: B| Ok((*review)(b))),
			)
		}
	}
}
pub use inner::*;
