//! Configuration subsystem for the macro system.
//!
//! This module handles:
//! - Loading configuration from Cargo.toml
//! - Configuration types and validation
//! - Runtime configuration state

pub mod loading;
pub mod types;

pub use loading::load_config;
pub use types::Config;
