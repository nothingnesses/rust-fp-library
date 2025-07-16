use crate::{
	hkt::{Apply, Kind},
	typeclasses::{Empty, Functor, Pure, Sequence},
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

impl Functor for MaybeBrand {
	fn map<F, A, B>(f: F) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
	{
		move |fa| Maybe::from(Option::from(fa).map(f))
	}
}

impl Pure for MaybeBrand {
	fn pure<A>(a: A) -> Apply<Self, A>
	{
		Maybe::Just(a)
	}
}

impl Empty for MaybeBrand {
	fn empty<A>() -> Apply<Self, A> {
		Maybe::Nothing
	}
}

impl Sequence for MaybeBrand {
	fn sequence<F, A, B>(ff: Apply<Self, F>) -> impl Fn(Apply<Self, A>) -> Apply<Self, B>
	where
		F: Fn(A) -> B + Copy,
	{
		move |fa| match (ff, fa) {
			(Maybe::Just(f), Maybe::Just(a)) => Maybe::Just(f(a)),
			_ => Maybe::Nothing,
		}
	}
}
