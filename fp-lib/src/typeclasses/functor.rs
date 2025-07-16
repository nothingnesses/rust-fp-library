use crate::{
	Functions,
	hkt::{Apply, Apply2, Kind, Kind2},
};

pub trait Functor {
	/// forall f a b. Functor f => (a -> b) -> f a -> f b
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
		Self: Kind<A> + Kind<B>;
}

pub trait Functor2 {
	/// forall f a b. Functor f => (a -> b) -> f a -> f b
	fn map<F, A, B, C>(f: F) -> impl Fn(Apply2<Self, A, B>) -> Apply2<Self, A, C>
	where
		F: Fn(B) -> C + Copy,
		Self: Kind2<A, B> + Kind2<A, C>;
}

impl Functions {
	/// forall f a b. Functor f => (a -> b) -> f a -> f b
	pub fn map<Brand, F, A, B>(f: F) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
	where
		F: Fn(A) -> B + Copy,
		Brand: Kind<A> + Kind<B> + Functor,
	{
		move |fa| Brand::map::<F, A, B>(f)(fa)
	}
}
