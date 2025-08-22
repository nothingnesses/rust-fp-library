use crate::{
	aliases::ArcFn,
	functions::{compose, identity},
	hkt::{Apply0, Kind0},
	typeclasses::{Monoid, Semigroup},
};
use std::{marker::PhantomData, sync::Arc};

/// Endomorphism monoid.
#[derive(Clone)]
pub struct Endomorphism<'a, A>(pub ArcFn<'a, A, A>);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndomorphismBrand<'a, A>(&'a A, PhantomData<&'a A>);

impl<'a, A> Kind0 for EndomorphismBrand<'a, A> {
	type Output = Endomorphism<'a, A>;
}

impl<'a, A> Semigroup<'a> for EndomorphismBrand<'a, A> {
	fn append(a: Apply0<Self>) -> ArcFn<'a, Apply0<Self>, Apply0<Self>> {
		Arc::new(move |b| Endomorphism(compose(a.0.clone())(b.0)))
	}
}

impl<'a, A> Monoid<'a> for EndomorphismBrand<'a, A> {
	fn empty() -> Apply0<Self> {
		Endomorphism(Arc::new(identity))
	}
}
