//! Traits representing type-level application.

/// Unifies the specialised `Kind` traits. Represents all kinds.
///
/// `Parameters` should be a tuple containing the types parameters.
/// `Output` represents the reified, concrete type.
pub trait Kind<Parameters> {
	type Output;
}

pub use crate::macros::generate_kind::{Kind1, Kind2, Kind3, Kind4};
