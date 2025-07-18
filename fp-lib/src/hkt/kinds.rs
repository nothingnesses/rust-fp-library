//! Traits representing type-level application.

/// \* -> *
pub trait Kind<A> {
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
