use crate::hkt::{Apply as App, Kind};

pub trait Apply {
	/// forall f a b. Apply f => f (a -> b) -> f a -> f b
	fn apply<F, A, B>(ff: App<Self, (F,)>) -> impl Fn(App<Self, (A,)>) -> App<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		App<Self, (F,)>: Clone,
		F: Fn(A) -> B,
		A: Clone;
}

/// forall f a b. Apply f => f (a -> b) -> f a -> f b
pub fn apply<Brand, F, A, B>(ff: App<Brand, (F,)>) -> impl Fn(App<Brand, (A,)>) -> App<Brand, (B,)>
where
	Brand: Kind<(F,)> + Kind<(A,)> + Kind<(B,)> + Apply,
	App<Brand, (F,)>: Clone,
	F: Fn(A) -> B,
	A: Clone,
{
	move |fa| Brand::apply::<F, _, _>(ff.to_owned())(fa)
}
