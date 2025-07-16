use crate::{
	Functions,
	hkt::{Apply, Kind},
};

pub trait Empty {
	/// forall f a. Empty f => () -> f a
	fn empty<A>() -> Apply<Self, A>
	where
		Self: Kind<A>;
}

impl Functions {
	/// forall f a. Empty f => () -> f a
	pub fn empty<Brand, A>() -> Apply<Brand, A>
	where
		Brand: Kind<A> + Empty,
	{
		Brand::empty()
	}
}
