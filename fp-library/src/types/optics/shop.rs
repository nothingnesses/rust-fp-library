//! The `Shop` profunctor, used for lenses.
//!
//! `Shop<A, B, S, T>` wraps a getter `S -> A` and a setter `S -> B -> T`.

use {
	crate::{
		Apply,
		classes::{
			Profunctor,
			Strong,
		},
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Shop` profunctor.
pub struct Shop<'a, A, B, S, T> {
	/// Getter function.
	pub get: Box<dyn Fn(S) -> A + 'a>,
	/// Setter function.
	pub set: Box<dyn Fn(S, B) -> T + 'a>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, A, B, S, T> Shop<'a, A, B, S, T> {
	/// Creates a new `Shop` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::Shop;
	///
	/// let shop = Shop::new(|s: (i32, i32)| s.0, |s: (i32, i32), b: i32| (b, s.1));
	/// assert_eq!((shop.get)((10, 20)), 10);
	/// assert_eq!((shop.set)((10, 20), 30), (30, 20));
	/// ```
	pub fn new(
		get: impl Fn(S) -> A + 'a,
		set: impl Fn(S, B) -> T + 'a,
	) -> Self {
		Shop {
			get: Box::new(get),
			set: Box::new(set),
			_phantom: PhantomData,
		}
	}
}

/// Brand for the `Shop` profunctor.
pub struct ShopBrand<A, B>(PhantomData<(A, B)>);

impl_kind! {
	impl<A: 'static, B: 'static> for ShopBrand<A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Shop<'a, A, B, S, T>;
	}
}

impl<A: 'static, B: 'static> Profunctor for ShopBrand<A, B> {
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
		Shop::new(move |s| get((*st)(s)), move |s, b| (*uv_2)(set((*st_2)(s), b)))
	}
}

impl<A: 'static, B: 'static> Strong for ShopBrand<A, B> {
	fn first<'a, S: 'a, T: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
		let get = pab.get;
		let set = pab.set;
		Shop::new(move |(s, _)| get(s), move |(s, c), b| (set(s, b), c))
	}
}
