use crate::{
	hkt::{Apply2, Inject2, Kind2, Project2},
	typeclasses::{Functor2, Pure2},
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

impl<A, B> Inject2<EitherBrand> for Either<A, B> {
	type A = A;
	type B = B;
	fn inject(self) -> Apply2<EitherBrand, A, B> {
		self
	}
}

impl<A, B> Project2<EitherBrand, A, B> for <EitherBrand as Kind2<A, B>>::Output {
	type Concrete = Either<A, B>;
	fn project(self) -> Self::Concrete {
		self
	}
}

impl<A, B> Functor2<EitherBrand, A, B> for Either<A, B> {
	fn map<F, C>(f: F) -> impl Fn(Apply2<EitherBrand, A, B>) -> Apply2<EitherBrand, A, C>
	where
		F: Fn(B) -> C + Copy,
		EitherBrand: Kind2<A, C>,
	{
		move |fa| Either::from(Result::from(fa).map(f)).inject()
	}
}

impl<A, B> Pure2<EitherBrand, A, B> for Either<A, B> {
	fn pure(b: B) -> Apply2<EitherBrand, A, B>
	where
		EitherBrand: Kind2<A, B>,
	{
		Either::Right(b).inject()
	}
}
