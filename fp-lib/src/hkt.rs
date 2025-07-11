//! Higher-kinded types using type-level defunctionalisation based on [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf).

pub mod apply;
pub mod inject;
pub mod project;

pub use self::apply::*;
pub use self::inject::*;
pub use self::project::*;
