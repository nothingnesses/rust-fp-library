//! The `Market` profunctor, used for prisms.
//!
//! `Market<A, B, S, T>` wraps a preview function `S -> Result<A, T>` and a review function `B -> T`.

use {
	crate::{
		Apply,
		classes::{
			Choice,
			Profunctor,
		},
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Market` profunctor.
pub struct Market<'a, A, B, S, T> {
	/// Preview function: tries to extract the focus.
	pub preview: Box<dyn Fn(S) -> Result<A, T> + 'a>,
	/// Review function: constructs the structure.
	pub review: Box<dyn Fn(B) -> T + 'a>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, A, B, S, T> Market<'a, A, B, S, T> {
	/// Creates a new `Market` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::Market;
	///
	/// let market = Market::new(
	/// 	|s: String| s.parse::<i32>().map_err(|_| "error".to_string()),
	/// 	|n: i32| n.to_string(),
	/// );
	/// assert_eq!((market.preview)("123".to_string()), Ok(123));
	/// assert_eq!((market.review)(456), "456".to_string());
	/// ```
	pub fn new(
		preview: impl Fn(S) -> Result<A, T> + 'a,
		review: impl Fn(B) -> T + 'a,
	) -> Self {
		Market {
			preview: Box::new(preview),
			review: Box::new(review),
			_phantom: PhantomData,
		}
	}
}

/// Brand for the `Market` profunctor.
pub struct MarketBrand<A, B>(PhantomData<(A, B)>);

impl_kind! {
	impl<A: 'static, B: 'static> for MarketBrand<A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Market<'a, A, B, S, T>;
	}
}

impl<A: 'static, B: 'static> Profunctor for MarketBrand<A, B> {
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
		let uv = std::rc::Rc::new(uv);
		let uv_2 = uv.clone();
		Market::new(move |s| preview(st(s)).map_err(|u| (*uv)(u)), move |b| (*uv_2)(review(b)))
	}
}

impl<A: 'static, B: 'static> Choice for MarketBrand<A, B> {
	fn left<'a, S: 'a, T: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, S>, Result<C, T>>)
	{
		let preview = pab.preview;
		let review = pab.review;
		Market::new(
			move |r| match r {
				Ok(c) => Err(Ok(c)),
				Err(s) => preview(s).map_err(Err),
			},
			move |b| Err(review(b)),
		)
	}

	fn right<'a, S: 'a, T: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<S, C>, Result<T, C>>)
	{
		let preview = pab.preview;
		let review = pab.review;
		Market::new(
			move |r| match r {
				Ok(s) => preview(s).map_err(Ok),
				Err(c) => Err(Err(c)),
			},
			move |b| Ok(review(b)),
		)
	}
}
