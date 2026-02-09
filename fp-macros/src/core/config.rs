//! Configuration access utilities.
//!
//! This module provides a convenient API for accessing the cached configuration,
//! re-exporting the existing configuration loading infrastructure.

pub use crate::config::types::{Config, UserConfig};
pub use crate::config::loading::{load_config, load_user_config};

/// Gets the cached configuration.
///
/// This is a convenience alias for `load_config()` that emphasizes the caching behavior.
/// The configuration is loaded from Cargo.toml only once per compilation and then cached.
///
/// # Examples
///
/// ```ignore
/// use fp_macros::core::get_config;
///
/// let config = get_config();
/// // Use config...
/// ```
#[inline]
pub fn get_config() -> Config {
    load_config()
}
