//! The `Exchange` profunctor, used for isomorphisms.
//!
//! `Exchange<A, B, S, T>` wraps a forward function `S -> A` and a backward function `B -> T`.

use {
	crate::{
		Apply,
		classes::Profunctor,
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Exchange` profunctor.
pub struct Exchange<'a, A, B, S, T> {
	/// Forward function.
	pub get: Box<dyn Fn(S) -> A + 'a>,
	/// Backward function.
	pub set: Box<dyn Fn(B) -> T + 'a>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, A, B, S, T> Exchange<'a, A, B, S, T> {
	/// Creates a new `Exchange` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::Exchange;
	///
	/// let exchange = Exchange::new(|s: String| s.len(), |n: usize| n.to_string());
	/// assert_eq!((exchange.get)("hello".to_string()), 5);
	/// assert_eq!((exchange.set)(10), "10".to_string());
	/// ```
	pub fn new(
		get: impl Fn(S) -> A + 'a,
		set: impl Fn(B) -> T + 'a,
	) -> Self {
		Exchange {
			get: Box::new(get),
			set: Box::new(set),
			_phantom: PhantomData,
		}
	}
}

/// Brand for the `Exchange` profunctor.
pub struct ExchangeBrand<A, B>(PhantomData<(A, B)>);

impl_kind! {
	impl<A: 'static, B: 'static> for ExchangeBrand<A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Exchange<'a, A, B, S, T>;
	}
}

impl<A: 'static, B: 'static> Profunctor for ExchangeBrand<A, B> {
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
		Exchange::new(move |s| get(st(s)), move |b| uv(set(b)))
	}
}
