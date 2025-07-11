//! Type-level projection.

use super::*;

/// \* -> *
pub trait Project<Brand: Kind<A>, A> {
	type Concrete;
	fn project(self) -> Self::Concrete;
}

/// \* -> * -> *
pub trait Project2<Brand: Kind2<A, B>, A, B> {
	type Concrete;
	fn project(self) -> Self::Concrete;
}

/// \* -> * -> * -> *
pub trait Project3<Brand: Kind3<A, B, C>, A, B, C> {
	type Concrete;
	fn project(self) -> Self::Concrete;
}

/// \* -> * -> * -> * -> *
pub trait Project4<Brand: Kind4<A, B, C, D>, A, B, C, D> {
	type Concrete;
	fn project(self) -> Self::Concrete;
}
