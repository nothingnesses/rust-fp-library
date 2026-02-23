//! The `Const` functor, which ignores its second type parameter.

use {
	crate::{
		Apply,
		classes::{
			apply_first::ApplyFirst,
			apply_second::ApplySecond,
			cloneable_fn::CloneableFn,
			functor::Functor,
			lift::Lift,
			monoid::Monoid,
			pointed::Pointed,
			semiapplicative::Semiapplicative,
			semigroup::Semigroup,
		},
		impl_kind,
		kinds::*,
	},
	fp_macros::document_type_parameters,
	std::marker::PhantomData,
};

/// The `Const` functor.
///
/// `Const<R, A>` stores a value of type `R` and ignores the type `A`.
#[document_type_parameters("The lifetime of the values.", "The stored type.", "The ignored type.")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Const<'a, R, A>(pub R, pub PhantomData<&'a A>);

impl<'a, R, A> Const<'a, R, A> {
	/// Creates a new `Const` instance.
	pub fn new(r: R) -> Self {
		Const(r, PhantomData)
	}
}

/// Brand for the `Const` functor.
pub struct ConstBrand<R>(PhantomData<R>);

impl_kind! {
	impl<R: 'static> for ConstBrand<R> {
		type Of<'a, A: 'a>: 'a = Const<'a, R, A>;
	}
}

impl<R: 'static> Functor for ConstBrand<R> {
	fn map<'a, A: 'a, B: 'a, F>(
		_f: F,
		fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>)
	where
		F: Fn(A) -> B + 'a, {
		Const::new(fa.0)
	}
}

impl<R: 'static + Semigroup> Lift for ConstBrand<R> {
	fn lift2<'a, A, B, C, Func>(
		_func: Func,
		fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, C>)
	where
		Func: Fn(A, B) -> C + 'a,
		A: Clone + 'a,
		B: Clone + 'a,
		C: 'a, {
		Const::new(R::append(fa.0, fb.0))
	}
}

impl<R: 'static + Semigroup> Semiapplicative for ConstBrand<R> {
	fn apply<'a, FnBrand: 'a + CloneableFn, A: 'a + Clone, B: 'a>(
		ff: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, <FnBrand as CloneableFn>::Of<'a, A, B>>),
		fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>) {
		Const::new(R::append(ff.0, fa.0))
	}
}

impl<R: 'static + Semigroup> ApplyFirst for ConstBrand<R> {
	fn apply_first<'a, A: 'a, B: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>) {
		Const::new(R::append(fa.0, fb.0))
	}
}

impl<R: 'static + Semigroup> ApplySecond for ConstBrand<R> {
	fn apply_second<'a, A: 'a, B: 'a>(
		fa: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>),
		fb: Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>),
	) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, B>) {
		Const::new(R::append(fa.0, fb.0))
	}
}

impl<R: 'static + Monoid> Pointed for ConstBrand<R> {
	fn pure<'a, A: 'a>(_a: A) -> Apply!(<Self as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>) {
		Const::new(R::empty())
	}
}
