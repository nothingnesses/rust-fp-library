//! Higher-Kinded Type (HKT) functionality.
//!
//! This module provides macros for working with Higher-Kinded Types in Rust:
//! - `Kind!` - Generate Kind trait names
//! - `def_kind!` - Define Kind traits
//! - `impl_kind!` - Implement Kind traits for brands
//! - `Apply!` - Apply brands to type arguments

pub mod apply;
pub mod canonicalizer;
pub mod impl_kind;
pub mod input;
pub mod kind;
pub mod associated_type;

pub use apply::{ApplyInput, apply_worker};
pub use canonicalizer::generate_name;
pub use impl_kind::{ImplKindInput, impl_kind_worker};
pub use input::{AssociatedType, AssociatedTypes};
pub use kind::def_kind_worker;
pub use associated_type::AssociatedTypeBase;

// Only needed for tests
#[cfg(test)]
pub use canonicalizer::Canonicalizer;
