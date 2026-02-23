//! The `Market` profunctor, used for prisms.
//!
//! `Market<A, B, S, T>` wraps a preview function `S -> Result<A, T>` and a review function `B -> T`.

use {
	crate::{
		Apply,
		classes::{
			Choice,
			CloneableFn,
			Profunctor,
		},
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Market` profunctor.
pub struct Market<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
	/// Preview function: tries to extract the focus.
	pub preview: <FnBrand as CloneableFn>::Of<'a, S, Result<A, T>>,
	/// Review function: constructs the structure.
	pub review: <FnBrand as CloneableFn>::Of<'a, B, T>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Market<'a, FnBrand, A, B, S, T> {
	/// Creates a new `Market` instance.
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
	/// let market = Market::<RcFnBrand, i32, String, String, String>::new(
	/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: String| s.parse::<i32>().map_err(|_| "error".to_string())),
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
pub struct MarketBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

impl_kind! {
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for MarketBrand<FnBrand, A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Market<'a, FnBrand, A, B, S, T>;
	}
}

impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor for MarketBrand<FnBrand, A, B> {
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
			<FnBrand as CloneableFn>::new(move |s: S| (*preview)((*st)(s)).map_err(|u| (*uv)(u))),
			<FnBrand as CloneableFn>::new(move |b: B| (*uv_2)((*review)(b)))
		)
	}
}

impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Choice for MarketBrand<FnBrand, A, B> {
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
