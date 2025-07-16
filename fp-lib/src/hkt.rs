//! Higher-kinded types using type-level defunctionalisation based on [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf).

//! Type-level application.

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

/// \* -> *
pub type Apply<Brand, A> = <Brand as Kind<A>>::Output;
/// \* -> * -> *
pub type Apply2<Brand, A, B> = <Brand as Kind2<A, B>>::Output;
/// \* -> * -> * -> *
pub type Apply3<Brand, A, B, C> = <Brand as Kind3<A, B, C>>::Output;
/// \* -> * -> * -> * -> *
pub type Apply4<Brand, A, B, C, D> = <Brand as Kind4<A, B, C, D>>::Output;
