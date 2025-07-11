use crate::{
	hkt::{Apply, Inject, Kind, Project},
	typeclasses::Functor,
};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Maybe<A> {
	Just(A),
	#[default]
	Nothing,
}

pub struct MaybeBrand;

impl<A> Kind<A> for MaybeBrand {
	type Output = Maybe<A>;
}

impl<A> From<Maybe<A>> for Option<A> {
	fn from(value: Maybe<A>) -> Self {
		match value {
			Maybe::Just(a) => Some(a),
			Maybe::Nothing => None,
		}
	}
}

impl<A> From<Option<A>> for Maybe<A> {
	fn from(value: Option<A>) -> Self {
		match value {
			Some(a) => Self::Just(a),
			None => Self::Nothing,
		}
	}
}

impl<A> Inject<MaybeBrand> for Maybe<A> {
	type A = A;
	fn inject(self) -> Apply<MaybeBrand, A> {
		self
	}
}

impl<A> Project<MaybeBrand, A> for <MaybeBrand as Kind<A>>::Output {
	type Concrete = Maybe<A>;
	fn project(self) -> Self::Concrete {
		self
	}
}

impl<A> Functor<MaybeBrand, A> for Maybe<A> {
	fn map<F, B>(f: F) -> impl Fn(Apply<MaybeBrand, A>) -> Apply<MaybeBrand, B>
	where
		F: Fn(A) -> B + Copy,
		MaybeBrand: Kind<B>,
	{
		move |fa| Maybe::from(Option::from(fa).map(f)).inject()
	}
}
