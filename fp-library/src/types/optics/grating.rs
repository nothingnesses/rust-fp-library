//! The `Grating` profunctor, used for grates.
//!
//! `Grating<A, B, S, T>` wraps a function `((S -> A) -> B) -> T`.

use {
	crate::{
		Apply,
		classes::{
			CloneableFn,
			Closed,
			Profunctor,
		},
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Grating` profunctor.
pub struct Grating<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> {
	/// Grating function.
	pub run: <FnBrand as CloneableFn>::Of<'a, Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>, T>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, FnBrand: CloneableFn, A: 'a, B: 'a, S: 'a, T: 'a> Grating<'a, FnBrand, A, B, S, T> {
	/// Creates a new `Grating` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcFnBrand,
	/// 	classes::cloneable_fn::new as cloneable_fn_new,
	/// 	types::optics::Grating,
	/// };
	///
	/// let grating = Grating::<RcFnBrand, i32, i32, (i32, i32), i32>::new(cloneable_fn_new::<RcFnBrand, _, _>(|f| {
	/// 	f(Box::new(|(x, _)| x)) + f(Box::new(|(_, y)| y))
	/// }));
	/// ```
	pub fn new(run: <FnBrand as CloneableFn>::Of<'a, Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>, T>) -> Self {
		Grating {
			run,
			_phantom: PhantomData,
		}
	}
}

/// Brand for the `Grating` profunctor.
pub struct GratingBrand<FnBrand, A, B>(PhantomData<(FnBrand, A, B)>);

impl_kind! {
	impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> for GratingBrand<FnBrand, A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Grating<'a, FnBrand, A, B, S, T>;
	}
}

impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Profunctor for GratingBrand<FnBrand, A, B> {
	fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a, FuncST, FuncUV>(
		st: FuncST,
		uv: FuncUV,
		puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>)
	where
		FuncST: Fn(S) -> T + 'a,
		FuncUV: Fn(U) -> V + 'a, {
		let run = puv.run;
		let st = <FnBrand as CloneableFn>::new(st);
		let uv = <FnBrand as CloneableFn>::new(uv);
		Grating::<FnBrand, A, B, S, V>::new(<FnBrand as CloneableFn>::new(move |f: Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>| {
			let st = st.clone();
			let uv = uv.clone();
			(*uv)((*run)(Box::new(move |g| {
				let st = st.clone();
				f(Box::new(move |s| g((*st)(s))))
			})))
		}))
	}
}

impl<FnBrand: CloneableFn + 'static, A: 'static, B: 'static> Closed for GratingBrand<FnBrand, A, B> {
	fn closed<'a, X: 'a, S: 'a, T: 'a>(
		_pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Box<dyn Fn(X) -> S + 'a>, Box<dyn Fn(X) -> T + 'a>>)
	{
		// This is currently unimplemented because the profunctor encoding of Grate
		// requires cloning the input X to support structure reconstruction,
		// which cannot be expressed within the current trait constraints.
		panic!(
			"Grating::closed is not yet implemented for all X. Please use concrete Grate types instead."
		)
	}
}
