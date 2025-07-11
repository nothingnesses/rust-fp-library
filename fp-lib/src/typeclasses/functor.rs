use crate::{
	Functions,
	hkt::{Apply, Apply2, Kind, Kind2},
};

pub trait Functor<Brand: Kind<A>, A> {
	/// forall f a b. Functor f => (a -> b) -> f a -> f b
	fn map<F, B>(f: F) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
	where
		F: Fn(A) -> B + Copy,
		Brand: Kind<B>;
}

pub trait Functor2<Brand: Kind2<A, B>, A, B> {
	/// forall f a b. Functor f => (a -> b) -> f a -> f b
	fn map<F, C>(f: F) -> impl Fn(Apply2<Brand, A, B>) -> Apply2<Brand, A, C>
	where
		F: Fn(B) -> C + Copy,
		Brand: Kind2<A, C>;
}

impl Functions {
	/// forall f a b. Functor f => (a -> b) -> f a -> f b
	pub fn map<Brand, F, A, B>(f: F) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
	where
		F: Fn(A) -> B + Copy,
		Brand: Kind<A> + Kind<B>,
		Apply<Brand, A>: Functor<Brand, A>,
	{
		move |fa| <Apply<Brand, A> as Functor<_, _>>::map::<F, B>(f)(fa)
	}
}
