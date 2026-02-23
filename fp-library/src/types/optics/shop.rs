//! The `Shop` profunctor, used for lenses.
//!
//! `Shop<A, B, S, T>` wraps a getter `S -> A` and a setter `S -> B -> T`.

use {
	crate::{
		Apply,
		classes::{
			CloneableFn,
			Profunctor,
			Strong,
		},
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Shop` profunctor.
pub struct Shop<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
	/// Getter function.
	pub get: <FnBrand as CloneableFn>::Of<'a, S, A>,
	/// Setter function.
	pub set: <FnBrand as CloneableFn>::Of<'a, (S, B), T>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Shop<'a, FnBrand, A, B, S, T> {
	/// Creates a new `Shop` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcFnBrand,
	/// 	classes::cloneable_fn::new as cloneable_fn_new,
	/// 	types::optics::Shop,
	/// };
	///
	/// let shop = Shop::<RcFnBrand, i32, i32, (i32, i32), (i32, i32)>::new(
	/// 	cloneable_fn_new::<RcFnBrand, _, _>(|s: (i32, i32)| s.0),
	/// 	cloneable_fn_new::<RcFnBrand, _, _>(|(s, b): ((i32, i32), i32)| (b, s.1))
	/// );
	/// assert_eq!((shop.get)((10, 20)), 10);
	/// assert_eq!((shop.set)(((10, 20), 30)), (30, 20));
	/// ```
	pub fn new(
		get: <FnBrand as CloneableFn>::Of<'a, S, A>,
		set: <FnBrand as CloneableFn>::Of<'a, (S, B), T>,
	) -> Self {
		Shop {
			get,
			set,
			_phantom: PhantomData,
		}
	}
}

pub struct ShopBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

impl_kind! {
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for ShopBrand<FnBrand, A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Shop<'a, FnBrand, A, B, S, T>;
	}
}

impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor for ShopBrand<FnBrand, A, B> {
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
		let st = <FnBrand as CloneableFn>::new(st);
		let uv = <FnBrand as CloneableFn>::new(uv);
		let st_2 = st.clone();
		let uv_2 = uv.clone();
		Shop::new(
			<FnBrand as CloneableFn>::new(move |s: S| (*get)((*st)(s))),
			<FnBrand as CloneableFn>::new(move |(s, b): (S, B)| (*uv_2)((*set)(((*st_2)(s), b))))
		)
	}
}

impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Strong for ShopBrand<FnBrand, A, B> {
	fn first<'a, S: 'a, T: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (S, C), (T, C)>) {
		let get = pab.get;
		let set = pab.set;
		Shop::new(
			<FnBrand as CloneableFn>::new(move |(s, _): (S, C)| (*get)(s)),
			<FnBrand as CloneableFn>::new(move |((s, c), b): ((S, C), B)| ((*set)((s, b)), c))
		)
	}
}
