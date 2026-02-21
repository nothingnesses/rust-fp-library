//! The `Stall` profunctor, used for affine traversals.
//!
//! `Stall<A, B, S, T>` wraps a preview function `S -> Result<A, T>` and a setter function `S -> B -> T`.

use {
	crate::{
		Apply,
		classes::{
			Choice,
			Profunctor,
			Strong,
		},
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Stall` profunctor.
pub struct Stall<'a, A, B, S, T> {
	/// Preview function: tries to extract the focus.
	pub get: Box<dyn Fn(S) -> Result<A, T> + 'a>,
	/// Setter function.
	pub set: Box<dyn Fn(S, B) -> T + 'a>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, A, B, S, T> Stall<'a, A, B, S, T> {
	/// Creates a new `Stall` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::Stall;
	///
	/// let stall = Stall::new(|s: (i32, i32)| Ok(s.0), |s: (i32, i32), b: i32| (b, s.1));
	/// assert_eq!((stall.get)((10, 20)), Ok(10));
	/// assert_eq!((stall.set)((10, 20), 30), (30, 20));
	/// ```
	pub fn new(
		get: impl Fn(S) -> Result<A, T> + 'a,
		set: impl Fn(S, B) -> T + 'a,
	) -> Self {
		Stall {
			get: Box::new(get),
			set: Box::new(set),
			_phantom: PhantomData,
		}
	}
}

/// Brand for the `Stall` profunctor.
pub struct StallBrand<A, B>(PhantomData<(A, B)>);

impl_kind! {
	impl<A: 'static, B: 'static> for StallBrand<A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Stall<'a, A, B, S, T>;
	}
}

impl<A: 'static, B: 'static> Profunctor for StallBrand<A, B> {
	fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a, FuncST, FuncUV>(
		st: FuncST,
		uv: FuncUV,
		puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>)
	where
		FuncST: Fn(S) -> T + 'a,
		FuncUV: Fn(U) -> V + 'a, {
		let get = puv.get;
		let set = puv.set;
		let st = std::rc::Rc::new(st);
		let uv = std::rc::Rc::new(uv);
		let st_2 = st.clone();
		let uv_2 = uv.clone();
		Stall::new(
			move |s| get((*st)(s)).map_err(|u| (*uv)(u)),
			move |s, b| (*uv_2)(set((*st_2)(s), b)),
		)
	}
}

impl<A: 'static, B: 'static> Strong for StallBrand<A, B> {
	fn first<'a, S: 'a, T: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
		let get = pab.get;
		let set = pab.set;
		Stall::new(move |(s, c)| get(s).map_err(|t| (t, c)), move |(s, c), b| (set(s, b), c))
	}
}

impl<A: 'static, B: 'static> Choice for StallBrand<A, B> {
	fn left<'a, S: 'a, T: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, S>, Result<C, T>>)
	{
		let get = pab.get;
		let set = pab.set;
		Stall::new(
			move |r| match r {
				Err(s) => get(s).map_err(Err),
				Ok(c) => Err(Ok(c)),
			},
			move |r, b| match r {
				Err(s) => Err(set(s, b)),
				Ok(c) => Ok(c),
			},
		)
	}

	fn right<'a, S: 'a, T: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<S, C>, Result<T, C>>)
	{
		let get = pab.get;
		let set = pab.set;
		Stall::new(
			move |r| match r {
				Ok(s) => get(s).map_err(Ok),
				Err(c) => Err(Err(c)),
			},
			move |r, b| match r {
				Ok(s) => Ok(set(s, b)),
				Err(c) => Err(c),
			},
		)
	}
}
