//! Convenience type aliases for the [kind traits][crate::hkt::kinds].

use crate::hkt::Kind;

/// Unifies the specialised [`Apply`][crate::hkt::apply] aliases.
///
/// `Brand` should be the type representing the higher-kinded form of another type.
/// `Parameters` should be a tuple containing the types parameters.
pub type Apply<Brand, Parameters> = <Brand as Kind<Parameters>>::Output;

pub use crate::macros::hkt::{Apply1, Apply2, Apply3, Apply4};
