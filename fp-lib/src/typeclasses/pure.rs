use crate::{
	Functions,
	hkt::{Apply, Apply2, Kind, Kind2},
};

pub trait Pure {
	/// forall f a. Pure f => a -> f a
	fn pure<A>(a: A) -> Apply<Self, A>
	where
		Self: Kind<A>;
}

pub trait Pure2 {
	/// forall f a. Pure f => a -> f a
	fn pure<A, B>(b: B) -> Apply2<Self, A, B>
	where
		Self: Kind2<A, B>;
}

impl Functions {
	/// forall f a. Pure f => a -> f a
	pub fn pure<Brand, A>(a: A) -> Apply<Brand, A>
	where
		Brand: Kind<A> + Pure,
	{
		Brand::pure(a)
	}
}
