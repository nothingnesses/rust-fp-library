use crate::{
	Functions,
	hkt::{Apply, Kind},
};

pub trait Sequence<Brand: Kind<A>, A> {
	/// forall f a b. Sequence f => f (a -> b) -> f a -> f b
	fn sequence<F, B>(ff: Apply<Brand, F>) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
	where
		F: Fn(A) -> B + Copy,
		Brand: Kind<F> + Kind<B>;
}

impl Functions {
	/// forall f a b. Sequence f => f (a -> b) -> f a -> f b
	pub fn sequence<Brand, F, A, B>(
		ff: Apply<Brand, F>
	) -> impl Fn(Apply<Brand, A>) -> Apply<Brand, B>
	where
		Brand: Kind<F> + Kind<A> + Kind<B>,
		F: Fn(A) -> B + Copy,
		Apply<Brand, A>: Sequence<Brand, A>,
	{
		let f = <Apply<Brand, A>>::sequence::<F, B>(ff);
		move |fa| f(fa)
	}
}
