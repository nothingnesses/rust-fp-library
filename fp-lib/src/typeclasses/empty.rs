use crate::{
	Functions,
	hkt::{Apply, Kind},
};

pub trait Empty<Brand: Kind<A>, A> {
	/// forall f a. Empty f => () -> f a
	fn empty() -> Apply<Brand, A>;
}

impl Functions {
	/// forall f a. Empty f => () -> f a
	pub fn empty<Brand, A>() -> Apply<Brand, A>
	where
		Brand: Kind<A>,
		Apply<Brand, A>: Empty<Brand, A>,
	{
		<Apply<Brand, A>>::empty()
	}
}
