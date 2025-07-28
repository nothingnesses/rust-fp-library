use crate::hkt::{Apply, Kind};

pub trait ApplySecond {
	/// forall f a b. ApplySecond f => f a -> f b -> f b
	fn apply_second<A, B>(fa: Apply<Self, (A,)>) -> impl Fn(Apply<Self, (B,)>) -> Apply<Self, (B,)>
	where
		Self: Kind<(A,)> + Kind<(B,)>,
		Apply<Self, (A,)>: Clone,
		B: Clone;
}

/// forall f a b. ApplySecond f => f a -> f b -> f b
pub fn apply_second<Brand, A, B>(
	fa: Apply<Brand, (A,)>
) -> impl Fn(Apply<Brand, (B,)>) -> Apply<Brand, (B,)>
where
	Brand: Kind<(A,)> + Kind<(B,)> + ApplySecond,
	Apply<Brand, (A,)>: Clone,
	B: Clone,
{
	Brand::apply_second::<A, B>(fa)
}
