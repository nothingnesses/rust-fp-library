//! Convenience type aliases for the [kind traits](../kinds/index.html).

use crate::hkt::kinds::*;

/// \* -> *
pub type Apply<Brand, A> = <Brand as Kind<A>>::Output;
/// \* -> * -> *
pub type Apply2<Brand, A, B> = <Brand as Kind2<A, B>>::Output;
/// \* -> * -> * -> *
pub type Apply3<Brand, A, B, C> = <Brand as Kind3<A, B, C>>::Output;
/// \* -> * -> * -> * -> *
pub type Apply4<Brand, A, B, C, D> = <Brand as Kind4<A, B, C, D>>::Output;
