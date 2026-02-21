//! The `Grating` profunctor, used for grates.
//!
//! `Grating<A, B, S, T>` wraps a function `((S -> A) -> B) -> T`.

use {
	crate::{
		Apply,
		classes::{
			Closed,
			Profunctor,
		},
		impl_kind,
		kinds::*,
	},
	std::marker::PhantomData,
};

/// The `Grating` profunctor.
pub struct Grating<'a, A, B, S, T> {
	/// Grating function.
	pub run: Box<dyn Fn(Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>) -> T + 'a>,
	pub(crate) _phantom: PhantomData<(A, B)>,
}

impl<'a, A, B, S, T> Grating<'a, A, B, S, T> {
	/// Creates a new `Grating` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::Grating;
	///
	/// let grating = Grating::<i32, i32, (i32, i32), i32>::new(|f| {
	/// 	f(Box::new(|(x, _)| x)) + f(Box::new(|(_, y)| y))
	/// });
	/// ```
	pub fn new(run: impl Fn(Box<dyn Fn(Box<dyn Fn(S) -> A + 'a>) -> B + 'a>) -> T + 'a) -> Self {
		Grating {
			run: Box::new(run),
			_phantom: PhantomData,
		}
	}
}

/// Brand for the `Grating` profunctor.
pub struct GratingBrand<A, B>(PhantomData<(A, B)>);

impl_kind! {
	impl<A: 'static, B: 'static> for GratingBrand<A, B> {
		type Of<'a, S: 'a, T: 'a>: 'a = Grating<'a, A, B, S, T>;
	}
}

impl<A: 'static, B: 'static> Profunctor for GratingBrand<A, B> {
	fn dimap<'a, S: 'a, T: 'a, U: 'a, V: 'a, FuncST, FuncUV>(
		st: FuncST,
		uv: FuncUV,
		puv: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, T, U>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, V>)
	where
		FuncST: Fn(S) -> T + 'a,
		FuncUV: Fn(U) -> V + 'a, {
		let run = puv.run;
		let st = std::rc::Rc::new(st);
		let uv = std::rc::Rc::new(uv);
		Grating::new(move |f| {
			let st = st.clone();
			let uv = uv.clone();
			uv(run(Box::new(move |g| {
				let st = st.clone();
				f(Box::new(move |s| g(st(s))))
			})))
		})
	}
}

impl<A: 'static, B: 'static> Closed for GratingBrand<A, B> {
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
