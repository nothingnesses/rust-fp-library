// @todo Add default impls
use crate::{
	hkt::{Apply, Kind},
	typeclasses::Monoid,
};

pub trait Foldable {
	fn fold_left<F, A, B>(f: F) -> impl Fn(B) -> Box<dyn Fn(Apply<Self, (A,)>) -> B>
	where
		Self: Kind<(A,)>,
		F: Fn(B) -> Box<dyn Fn(A) -> B>;

	fn fold_map<F, A, M>(f: F) -> impl Fn(Apply<Self, (A,)>) -> Apply<M, ()>
	where
		M: Monoid,
		F: Fn(A) -> Apply<M, ()>,
		Self: Kind<(A,)>;

	fn fold_right<F, A, B>(f: F) -> impl Fn(B) -> Box<dyn Fn(Apply<Self, (A,)>) -> B>
	where
		Self: Kind<(A,)>,
		F: Fn(A) -> Box<dyn Fn(B) -> B>;
}

pub fn fold_left<'a, Brand, F, A, B>(
	f: F
) -> impl Fn(B) -> Box<dyn 'a + Fn(Apply<Brand, (A,)>) -> B>
where
	Brand: Kind<(A,)> + Foldable,
	F: Fn(B) -> Box<dyn Fn(A) -> B>,
	B: 'a,
	Apply<Brand, (A,)>: 'a,
{
	move |b| Box::new(Brand::fold_left(&f)(b))
}

pub fn fold_map<Brand, F, A, M>(f: F) -> impl Fn(Apply<Brand, (A,)>) -> Apply<M, ()>
where
	Brand: Kind<(A,)> + Foldable,
	M: Monoid,
	F: Fn(A) -> Apply<M, ()>,
{
	Brand::fold_map::<_, _, M>(f)
}

pub fn fold_right<'a, Brand, F, A, B>(
	f: F
) -> impl Fn(B) -> Box<dyn 'a + Fn(Apply<Brand, (A,)>) -> B>
where
	Brand: Kind<(A,)> + Foldable,
	F: Fn(A) -> Box<dyn Fn(B) -> B>,
	B: 'a,
	Apply<Brand, (A,)>: 'a,
{
	move |b| Box::new(Brand::fold_right(&f)(b))
}
