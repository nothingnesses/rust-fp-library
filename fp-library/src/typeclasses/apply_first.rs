use crate::hkt::{Apply, Kind};

pub trait ApplyFirst {
	/// forall a b. ApplyFirst f => f a -> f b -> f a
	fn apply_first<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
		A: Clone,
		B: Clone;
}

/// forall a b. ApplyFirst f => f a -> f b -> f a
pub fn apply_first<Brand, A, B>(
	fa: Apply<Brand, (A,)>
) -> impl Fn(Apply<Brand, (B,)>) -> Apply<Brand, (A,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + ApplyFirst,
	Apply<Brand, (A,)>: Clone,
	A: Clone,
	B: Clone,
{
	Brand::apply_first::<A, B>(fa)
}
