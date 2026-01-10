//! A functional programming library for Rust featuring your favourite higher-kinded types and type classes.
//!
//! # Module Structure
//!
//! * `v2`: The new, zero-cost, uncurried API. This is the recommended API for new code.
//! * `classes`, `types`, `functions`: The legacy v1 API (deprecated).
//! * `brands`, `hkt`, `macros`: Shared infrastructure used by both APIs.

pub mod brands;
pub mod hkt;
pub mod macros;

#[cfg(feature = "v1")]
#[deprecated(since = "0.0.21", note = "Use fp_library::v2::classes instead")]
pub mod classes;

#[cfg(feature = "v1")]
#[deprecated(since = "0.0.21", note = "Use fp_library::v2::functions instead")]
pub mod functions;

#[cfg(feature = "v1")]
#[deprecated(since = "0.0.21", note = "Use fp_library::v2::types instead")]
pub mod types;

#[cfg(feature = "v2")]
pub mod v2;
