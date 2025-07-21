use crate::hkt::{Apply, Kind};

pub trait Pure {
	/// forall a. Pure f => a -> f a
	fn pure<A>(a: A) -> Apply<Self, (A,)>
	where
		Self: Kind<(A,)>;
}

/// forall a. Pure f => a -> f a
pub fn pure<Brand, A>(a: A) -> Apply<Brand, (A,)>
where
	Brand: Kind<(A,)> + Pure,
{
	Brand::pure(a)
}
