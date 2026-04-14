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
pub mod kind_attr;
pub mod trait_kind;

// Only needed for tests
#[cfg(test)]
pub use canonicalizer::Canonicalizer;
pub use {
	apply::{
		ApplyInput,
		apply_worker,
		resolve_inferable_brand,
	},
	associated_type::AssociatedTypeBase,
	canonicalizer::{
		generate_inferable_brand_name,
		generate_name,
	},
	impl_kind::{
		ImplKindInput,
		impl_kind_worker,
	},
	input::{
		AssociatedType,
		AssociatedTypes,
	},
	kind_attr::kind_attr_worker,
	trait_kind::trait_kind_worker,
};
