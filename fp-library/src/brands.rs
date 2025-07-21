//! Higher-kinded representation of types.

use crate::hkt::{Apply, Kind};

pub use super::types::{
	option::OptionBrand,
	result::{ResultBrand, ResultWithErrBrand, ResultWithOkBrand},
	solo::SoloBrand,
};

/// Contains functions to convert between the concrete type and the
/// corresponding instantiation of [`Apply`](../hkt/apply/type.Apply.html).
pub trait Brand<Concrete, Parameters>: Kind<Parameters> {
	fn inject(a: Concrete) -> Self::Output;
	fn project(a: Self::Output) -> Concrete;
}

pub trait Brand1<Concrete, A>
where
	Self: Kind<(A,)>,
{
	fn inject(a: Concrete) -> Apply<Self, (A,)>;
	fn project(a: Apply<Self, (A,)>) -> Concrete;
}

pub trait Brand2<Concrete, A, B>
where
	Self: Kind<(A, B)>,
{
	fn inject(a: Concrete) -> Apply<Self, (A, B)>;
	fn project(a: Apply<Self, (A, B)>) -> Concrete;
}

pub trait Brand3<Concrete, A, B, C>
where
	Self: Kind<(A, B, C)>,
{
	fn inject(a: Concrete) -> Apply<Self, (A, B, C)>;
	fn project(a: Apply<Self, (A, B, C)>) -> Concrete;
}

pub trait Brand4<Concrete, A, B, C, D>
where
	Self: Kind<(A, B, C, D)>,
{
	fn inject(a: Concrete) -> Apply<Self, (A, B, C, D)>;
	fn project(a: Apply<Self, (A, B, C, D)>) -> Concrete;
}

impl<Me, Concrete, A> Brand<Concrete, (A,)> for Me
where
	Me: Kind<(A,)> + Brand1<Concrete, A>,
{
	fn inject(a: Concrete) -> Apply<Self, (A,)> {
		<Me as Brand1<Concrete, A>>::inject(a)
	}

	fn project(a: Apply<Self, (A,)>) -> Concrete {
		<Me as Brand1<Concrete, A>>::project(a)
	}
}

impl<Me, Concrete, A, B> Brand<Concrete, (A, B)> for Me
where
	Me: Kind<(A, B)> + Brand2<Concrete, A, B>,
{
	fn inject(a: Concrete) -> Apply<Self, (A, B)> {
		<Me as Brand2<Concrete, A, B>>::inject(a)
	}

	fn project(a: Apply<Self, (A, B)>) -> Concrete {
		<Me as Brand2<Concrete, A, B>>::project(a)
	}
}

impl<Me, Concrete, A, B, C> Brand<Concrete, (A, B, C)> for Me
where
	Me: Kind<(A, B, C)> + Brand3<Concrete, A, B, C>,
{
	fn inject(a: Concrete) -> Apply<Self, (A, B, C)> {
		<Me as Brand3<Concrete, A, B, C>>::inject(a)
	}

	fn project(a: Apply<Self, (A, B, C)>) -> Concrete {
		<Me as Brand3<Concrete, A, B, C>>::project(a)
	}
}

impl<Me, Concrete, A, B, C, D> Brand<Concrete, (A, B, C, D)> for Me
where
	Me: Kind<(A, B, C, D)> + Brand4<Concrete, A, B, C, D>,
{
	fn inject(a: Concrete) -> Apply<Self, (A, B, C, D)> {
		<Me as Brand4<Concrete, A, B, C, D>>::inject(a)
	}

	fn project(a: Apply<Self, (A, B, C, D)>) -> Concrete {
		<Me as Brand4<Concrete, A, B, C, D>>::project(a)
	}
}
