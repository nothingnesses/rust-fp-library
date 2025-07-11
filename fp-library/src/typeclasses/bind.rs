use crate::hkt::{Apply, Kind};

pub trait Bind {
	/// forall a b. Bind m => m a -> (a -> m b) -> m b
	fn bind<F, A, B>(ma: &Apply<Self, A>) -> impl Fn(&F) -> Apply<Self, B>
	where
		Self: Kind<A> + Kind<B> + Sized,
		F: Fn(&A) -> Apply<Self, B>,
		Apply<Self, B>: Clone;
}

/// forall m a b. Bind m => m a -> (a → m b) → m b
pub fn bind<Brand, F, A, B>(ma: &Apply<Brand, A>) -> impl Fn(&F) -> Apply<Brand, B>
where
	Brand: Kind<A> + Kind<B> + Bind,
	F: Fn(&A) -> Apply<Brand, B>,
	Apply<Brand, B>: Clone,
{
	|f| Brand::bind::<F, A, B>(ma)(f)
}
