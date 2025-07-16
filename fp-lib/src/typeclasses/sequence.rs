use crate::{
	Functions,
	hkt::{Apply, Apply2, Kind, Kind2},
};

pub trait Sequence {
	/// forall f a b. Sequence f => f (a -> b) -> f a -> f b
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
		Self: Kind<F> + Kind<A> + Kind<B>;
}

pub trait Sequence2 {
	/// forall f a b. Sequence f => f (a -> b) -> f a -> f b
	fn sequence<F, A, B, C>(
		ff: Apply2<Self, A, F>
	) -> impl Fn(Apply2<Self, A, B>) -> Apply2<Self, A, C>
	where
		F: Fn(B) -> C + Copy,
		Self: Kind2<A, F> + Kind2<A, B> + Kind2<A, C>,
		A: Clone;
}

impl Functions {
	/// forall f a b. Sequence f => f (a -> b) -> f a -> f b
	pub fn sequence<Brand, F, A, B>(
		ff: Apply<Brand, F>
	) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
	where
		Brand: Kind<F> + Kind<A> + Kind<B> + Sequence,
		F: Fn(A) -> B + Copy,
		A: Clone,
	{
		let f = Brand::sequence::<F, A, B>(ff);
		move |fa| f(fa)
	}
}
