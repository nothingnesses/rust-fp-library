//! Convenience type aliases for the [kind traits][crate::hkt::kinds].

use crate::hkt::kinds::*;

/// Unifies the specialised [`Apply`][crate::hkt::apply] aliases.
///
/// `Brand` should be the type representing the higher-kinded form of another type.
/// `Parameters` should be a tuple containing the types parameters.
pub type Apply<Brand, Parameters> = <Brand as Kind<Parameters>>::Output;
/// Alias for [types][crate::types] of kind `* -> *`.
pub type Apply1<Brand, A> = <Brand as Kind1<A>>::Output;
/// Alias for [types][crate::types] of kind `* -> * -> *`.
pub type Apply2<Brand, A, B> = <Brand as Kind2<A, B>>::Output;
/// Alias for [types][crate::types] of kind `* -> * -> * -> *`.
pub type Apply3<Brand, A, B, C> = <Brand as Kind3<A, B, C>>::Output;
/// Alias for [types][crate::types] of kind `* -> * -> * -> * -> *`.
pub type Apply4<Brand, A, B, C, D> = <Brand as Kind4<A, B, C, D>>::Output;
