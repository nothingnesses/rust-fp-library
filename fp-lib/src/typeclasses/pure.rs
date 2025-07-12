use crate::{
	Functions,
	hkt::{Apply, Apply2, Kind, Kind2},
};

pub trait Pure<Brand: Kind<A>, A> {
	/// forall f a. Pure f => a -> f a
	fn pure(a: A) -> Apply<Brand, A>
	where
		Brand: Kind<A>;
}

pub trait Pure2<Brand: Kind2<A, B>, A, B> {
	/// forall f a. Pure f => a -> f a
	fn pure(b: B) -> Apply2<Brand, A, B>
	where
		Brand: Kind2<A, B>;
}

impl Functions {
	/// forall f a. Pure f => a -> f a
	pub fn pure<Brand, A>(a: A) -> Apply<Brand, A>
	where
		Brand: Kind<A>,
		Apply<Brand, A>: Pure<Brand, A>,
	{
		<Apply<Brand, A>>::pure(a)
	}
}
