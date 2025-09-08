//! Simulates higher-kinded types using type-level defunctionalisation based on Yallop
//! and White's [Lightweight higher-kinded polymorphism](https://www.cl.cam.ac.uk/~jdy22/papers/lightweight-higher-kinded-polymorphism.pdf).
//!
//! [`Kind` traits][crate::hkt::kinds] represent the arity of a kind.
//! These traits are implemented by [`Brand` types][crate::brands],
//! which represent higher-kinded (unapplied/partially-applied) forms
//! (type constructors) of [types][crate::types].

pub mod apply;
pub mod kinds;

pub use self::apply::*;
pub use self::kinds::*;
