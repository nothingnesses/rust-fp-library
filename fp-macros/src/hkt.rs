//! Higher-Kinded Type (HKT) functionality.
//!
//! This module provides macros for working with Higher-Kinded Types in Rust:
//! - `Kind!` - Generate Kind trait names
//! - `trait_kind!` - Define Kind traits
//! - `impl_kind!` - Implement Kind traits for brands
//! - `Apply!` - Apply brands to type arguments

pub mod apply;
pub mod associated_type;
pub mod canonicalizer;
pub mod impl_kind;
pub mod input;
pub mod trait_kind;

pub use apply::{ApplyInput, apply_worker};
pub use associated_type::AssociatedTypeBase;
pub use canonicalizer::generate_name;
pub use impl_kind::{ImplKindInput, impl_kind_worker};
pub use input::{AssociatedType, AssociatedTypes};
pub use trait_kind::trait_kind_worker;

// Only needed for tests
#[cfg(test)]
pub use canonicalizer::Canonicalizer;
