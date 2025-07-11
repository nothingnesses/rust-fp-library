use crate::hkt::{Apply, Kind};

pub trait Functor {
	/// forall a b. Functor f => (a -> b) -> f a -> f b
	fn map<F, A, B>(f: &F) -> impl Fn(&Apply<Self, A>) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B>,
		F: Fn(&A) -> B,
		A: Clone;
}

/// forall a b. Functor f => (a -> b) -> f a -> f b
pub fn map<Brand, F, A, B>(f: &F) -> impl Fn(&Apply<Brand, A>) -> Apply<Brand, B>
where
	Brand: Kind<A> + Kind<B> + Functor,
	F: Fn(&A) -> B,
	A: Clone,
{
	|fa| Brand::map::<_, _, _>(f)(fa)
}
