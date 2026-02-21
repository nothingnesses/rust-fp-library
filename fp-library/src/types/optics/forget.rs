//! The `Forget` profunctor, used for folds and getters.
//!
//! `Forget<R, A, B>` wraps a function `A -> R`, ignoring the `B` parameter.

use {
	crate::{
		Apply,
		classes::{
			Choice,
			Profunctor,
			Strong,
			monoid::Monoid,
		},
		impl_kind,
		kinds::*,
	},
	fp_macros::document_type_parameters,
	std::marker::PhantomData,
};

/// The `Forget` profunctor.
///
/// `Forget<R, A, B>` is a profunctor that ignores its second type argument `B`
/// and instead stores a function from `A` to `R`.
#[document_type_parameters(
	"The lifetime of the values.",
	"The return type of the function.",
	"The input type of the function.",
	"The ignored type."
)]
pub struct Forget<'a, R, A, B>(pub Box<dyn Fn(A) -> R + 'a>, PhantomData<B>);

impl<'a, R, A, B> Forget<'a, R, A, B> {
	/// Creates a new `Forget` instance.
	///
	/// ### Examples
	///
	/// ```
	/// use fp_library::types::optics::Forget;
	///
	/// let forget = Forget::<i32, String, i32>::new(|s: String| s.len() as i32);
	/// assert_eq!((forget.0)("hello".to_string()), 5);
	/// ```
	pub fn new(f: impl Fn(A) -> R + 'a) -> Self {
		Forget(Box::new(f), PhantomData)
	}
}

impl<'a, R, A, B> Clone for Forget<'a, R, A, B>
where
	R: 'a,
	A: 'a,
{
	fn clone(&self) -> Self {
		// This is tricky because Box<dyn Fn> is not Clone.
		// In a real implementation, we'd use Rc or similar.
		// Since this is for optics, and optics are often evaluated immediately,
		// we might need to change this to use a pointer brand like FnBrand.
		panic!(
			"Forget cannot be cloned directly. Use a pointer-wrapped version if cloning is needed."
		)
	}
}

/// Brand for the `Forget` profunctor.
pub struct ForgetBrand<R>(PhantomData<R>);

impl_kind! {
	impl<R: 'static> for ForgetBrand<R> {
		type Of<'a, A: 'a, B: 'a>: 'a = Forget<'a, R, A, B>;
	}
}

impl<R: 'static> Profunctor for ForgetBrand<R> {
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

impl<R: 'static> Strong for ForgetBrand<R> {
	fn first<'a, A: 'a, B: 'a, C: 'a>(
		pab: Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, A, B>)
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a, U: 'a>: 'a; )>::Of<'a, (A, C), (B, C)>) {
		Forget::new(move |(a, _)| (pab.0)(a))
	}
}

impl<R: 'static + Monoid> Choice for ForgetBrand<R> {
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
