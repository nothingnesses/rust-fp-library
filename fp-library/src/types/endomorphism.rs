use crate::{
	aliases::ClonableFn,
	functions::{compose, identity},
	hkt::{Apply, Brand, Brand0, Kind0},
	typeclasses::{Monoid, Semigroup},
};
use std::{marker::PhantomData, sync::Arc};

/// Endomorphism monoid.
#[derive(Clone)]
pub struct Endomorphism<'a, A>(pub Arc<dyn 'a + Fn(A) -> A>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<'a, A>(A, PhantomData<&'a A>);

impl<'a, A> Kind0 for EndomorphismBrand<'a, A> {
	type Output = Endomorphism<'a, A>;
}

impl<'a, A> Brand0<Endomorphism<'a, A>> for EndomorphismBrand<'a, A> {
	fn inject(a: Endomorphism<'a, A>) -> Apply<Self, ()> {
		a
	}
	fn project(a: Apply<Self, ()>) -> Endomorphism<'a, A> {
		a
	}
}

impl<'a, A> Semigroup<'a> for EndomorphismBrand<'a, A> {
	fn append(a: Apply<Self, ()>) -> ClonableFn<'a, Apply<Self, ()>, Apply<Self, ()>> {
		let a = <Self as Brand<_, _>>::project(a).0;
		Arc::new(move |b| Endomorphism(compose(a.clone())(<Self as Brand<_, _>>::project(b).0)))
	}
}

impl<'a, A> Monoid<'a> for EndomorphismBrand<'a, A> {
	fn empty() -> Apply<Self, ()> {
		Endomorphism(Arc::new(identity))
	}
}
