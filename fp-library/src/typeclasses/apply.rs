use crate::hkt::{Apply as App, Kind};

pub trait Apply {
	/// forall a b. Apply f => f (a -> b) -> f a -> f b
	fn apply<F, A, B>(ff: App<Self, (F,)>) -> impl Fn(App<Self, (A,)>) -> App<Self, (B,)>
	where
		Self: Kind<(F,)> + Kind<(A,)> + Kind<(B,)>,
		F: Fn(A) -> B,
		App<Self, (F,)>: Clone;
}

/// forall a b. Apply f => f (a -> b) -> f a -> f b
pub fn apply<Brand, F, A, B>(ff: App<Brand, (F,)>) -> impl Fn(App<Brand, (A,)>) -> App<Brand, (B,)>
where
	Brand: Kind<(F,)> + Kind<(A,)> + Kind<(B,)> + Apply,
	F: Fn(A) -> B,
	App<Brand, (F,)>: Clone,
{
	move |fa| Brand::apply::<F, _, _>(ff.to_owned())(fa)
}
