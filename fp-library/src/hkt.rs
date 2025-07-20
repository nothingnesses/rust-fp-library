//! Higher-kinded types using type-level defunctionalisation based on Yallop
//! and White's [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf).

pub mod apply;
pub mod kinds;

pub use self::apply::*;
pub use self::kinds::*;
