//! The `Forget` profunctor, used for folds and getters.
//!
//! `Forget<P, R, A, B>` wraps a function `A -> R`, ignoring the `B` parameter.

use {
	crate::{
		Apply,
		brands::FnBrand,
		classes::{
			Choice,
			CloneableFn,
			Profunctor,
			Strong,
			UnsizedCoercible,
			monoid::Monoid,
			wander::Wander,
		},
		impl_kind,
		kinds::*,
	},
	fp_macros::document_type_parameters,
	std::marker::PhantomData,
};

/// The `Forget` profunctor.
///
/// `Forget<P, R, A, B>` is a profunctor that ignores its second type argument `B`
/// and instead stores a function from `A` to `R`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The pointer brand.",
	"The return type of the function.",
	"The input type of the function.",
	"The ignored type."
)]
pub struct Forget<'a, P, R, A, B>(
	pub Apply!(<FnBrand<P> as Kind!( type Of<'b, U: 'b, V: 'b>: 'b; )>::Of<'a, A, R>),
	PhantomData<B>,
)
where
	P: UnsizedCoercible,
	R: 'a,
	A: 'a;

impl<'a, P, R, A, B> Forget<'a, P, R, A, B>
where
	P: UnsizedCoercible,
	R: 'a,
	A: 'a,
{
	/// Creates a new `Forget` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::{
	/// 	brands::RcBrand,
	/// 	types::optics::Forget,
	/// };
	///
	/// let forget = Forget::<RcBrand, i32, String, i32>::new(|s: String| s.len() as i32);
	/// // Access via the underlying function wrapper, which implements Deref
	/// assert_eq!((forget.0)("hello".to_string()), 5);
	/// ```
	pub fn new(f: impl Fn(A) -> R + 'a) -> Self {
		Forget(<FnBrand<P> as CloneableFn>::new(f), PhantomData)
	}
}

impl<'a, P, R, A, B> Clone for Forget<'a, P, R, A, B>
where
	P: UnsizedCoercible,
	R: 'a,
	A: 'a,
{
	fn clone(&self) -> Self {
		Forget(self.0.clone(), PhantomData)
	}
}

/// Brand for the `Forget` profunctor.
pub struct ForgetBrand<P, R>(PhantomData<(P, R)>);

impl_kind! {
	impl<P: UnsizedCoercible + 'static, R: 'static> for ForgetBrand<P, R> {
		type Of<'a, A: 'a, B: 'a>: 'a = Forget<'a, P, R, A, B>;
	}
}

impl<P: UnsizedCoercible + 'static, R: 'static> Profunctor for ForgetBrand<P, R> {
	fn dimap<'a, A: 'a, B: 'a, C: 'a, D: 'a, FuncAB, FuncCD>(
		ab: FuncAB,
		_cd: FuncCD,
		pbc: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, B, C>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, D>)
	where
		FuncAB: Fn(A) -> B + 'a,
		FuncCD: Fn(C) -> D + 'a, {
		Forget::new(move |a| (pbc.0)(ab(a)))
	}
}

impl<P: UnsizedCoercible + 'static, R: 'static> Strong for ForgetBrand<P, R> {
	fn first<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
		Forget::new(move |(a, _)| (pab.0)(a))
	}
}

impl<P: UnsizedCoercible + 'static, R: 'static + Monoid> Wander for ForgetBrand<P, R> {
	fn wander<'a, S: 'a, T: 'a, A: 'a, B: 'a, TFunc>(
		traversal: TFunc,
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, S, T>)
	where
		TFunc: crate::classes::wander::TraversalFunc<'a, S, T, A, B> + 'a, {
		use crate::types::const_val::ConstBrand;
		Forget::new(move |s| {
			let pab = pab.clone();
			(traversal.apply::<ConstBrand<R>>(
				Box::new(move |a| crate::types::const_val::Const::new((pab.0)(a))),
				s,
			))
			.0
		})
	}
}

impl<P: UnsizedCoercible + 'static, R: 'static + Monoid> Choice for ForgetBrand<P, R> {
	fn left<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<C, A>, Result<C, B>>)
	{
		Forget::new(move |r| match r {
			Err(a) => (pab.0)(a),
			Ok(_) => R::empty(),
		})
	}

	fn right<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, Result<A, C>, Result<B, C>>)
	{
		Forget::new(move |r| match r {
			Ok(a) => (pab.0)(a),
			Err(_) => R::empty(),
		})
	}
}
