use crate::hkt::{Apply, Kind};

pub trait Bind {
	/// forall a b. Bind m => m a -> (a -> m b) -> m b
	fn bind<F, A, B>(ma: Apply<Self, (A,)>) -> impl Fn(F) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)> + Sized,
		Apply<Self, (A,)>: Clone,
		F: Fn(A) -> Apply<Self, (B,)>;
}

/// forall a b. Bind m => m a -> (a → m b) → m b
pub fn bind<Brand, F, A, B>(ma: Apply<Brand, (A,)>) -> impl Fn(F) -> Apply<Brand, (B,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + Bind,
	Apply<Brand, (A,)>: Clone,
	F: Fn(A) -> Apply<Brand, (B,)>,
{
	move |f| Brand::bind::<F, A, B>(ma.to_owned())(f)
}
