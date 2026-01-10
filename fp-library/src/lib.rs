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
