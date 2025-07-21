//! Traits representing type-level application.

use crate::hkt::{Apply1, Apply2, Apply3, Apply4};

/// Unifies the specialised `Kind` traits. Represents all kinds.
/// `Parameters` should be a tuple containing the types parameters.
/// `Output` represents the reified, concrete type.
pub trait Kind<Parameters> {
	type Output;
}
/// \* -> *
pub trait Kind1<A> {
	type Output;
}
/// \* -> * -> *
pub trait Kind2<A, B> {
	type Output;
}
/// \* -> * -> * -> *
pub trait Kind3<A, B, C> {
	type Output;
}
/// \* -> * -> * -> * -> *
pub trait Kind4<A, B, C, D> {
	type Output;
}

impl<Brand, A> Kind<(A,)> for Brand
where
	Brand: Kind1<A>,
{
	type Output = Apply1<Brand, A>;
}
impl<Brand, A, B> Kind<(A, B)> for Brand
where
	Brand: Kind2<A, B>,
{
	type Output = Apply2<Brand, A, B>;
}
impl<Brand, A, B, C> Kind<(A, B, C)> for Brand
where
	Brand: Kind3<A, B, C>,
{
	type Output = Apply3<Brand, A, B, C>;
}
impl<Brand, A, B, C, D> Kind<(A, B, C, D)> for Brand
where
	Brand: Kind4<A, B, C, D>,
{
	type Output = Apply4<Brand, A, B, C, D>;
}
