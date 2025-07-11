//! Type-level injection.

use super::*;

/// \* -> *
pub trait Inject<Brand> {
	type A;
	fn inject(self) -> Apply<Brand, Self::A>
	where
		Brand: Kind<Self::A>;
}
/// \* -> * -> *
pub trait Inject2<Brand> {
	type A;
	type B;
	fn inject(self) -> Apply2<Brand, Self::A, Self::B>
	where
		Brand: Kind2<Self::A, Self::B>;
}
/// \* -> * -> * -> *
pub trait Inject3<Brand> {
	type A;
	type B;
	type C;
	fn inject(self) -> Apply3<Brand, Self::A, Self::B, Self::C>
	where
		Brand: Kind3<Self::A, Self::B, Self::C>;
}
/// \* -> * -> * -> * -> *
pub trait Inject4<Brand> {
	type A;
	type B;
	type C;
	type D;
	fn inject(self) -> Apply4<Brand, Self::A, Self::B, Self::C, Self::D>
	where
		Brand: Kind4<Self::A, Self::B, Self::C, Self::D>;
}
