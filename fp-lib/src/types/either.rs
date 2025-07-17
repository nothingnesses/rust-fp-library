use crate::{
	hkt::{Apply2, Kind, Kind2},
	typeclasses::{Functor2, Pure2, Sequence2},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Either<A, B> {
	Left(A),
	Right(B),
}

pub struct EitherBrand;

impl<A, B> Kind2<A, B> for EitherBrand {
	type Output = Either<A, B>;
}

impl<A, B> From<Either<A, B>> for Result<B, A> {
	fn from(value: Either<A, B>) -> Self {
		match value {
			Either::Left(a) => Err(a),
			Either::Right(a) => Ok(a),
		}
	}
}

impl<A, B> From<Result<B, A>> for Either<A, B> {
	fn from(value: Result<B, A>) -> Self {
		match value {
			Ok(a) => Either::Right(a),
			Err(a) => Either::Left(a),
		}
	}
}

impl Functor2 for EitherBrand {
	fn map<F, A, B, C>(f: F) -> impl Fn(Apply2<Self, A, B>) -> Apply2<Self, A, C>
	where
		F: Fn(B) -> C + Copy,
	{
		move |fa| Either::from(Result::from(fa).map(f))
	}
}

impl Pure2 for EitherBrand {
	fn pure<A, B>(b: B) -> Apply2<Self, A, B> {
		Either::Right(b)
	}
}

impl Sequence2 for EitherBrand {
	fn sequence<F, A, B, C>(
		ff: Apply2<Self, A, F>
	) -> impl Fn(Apply2<Self, A, B>) -> Apply2<Self, A, C>
	where
		F: Fn(B) -> C + Copy,
		A: Clone
	{
		move |fa| match (ff.clone(), fa) {
			(Either::Left(e), _) => Either::Left(e),
			(Either::Right(f), fa) => EitherBrand::map(f)(fa),
		}
	}
}
