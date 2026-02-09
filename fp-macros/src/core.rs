//! Core infrastructure for fp-macros.
//!
//! This module provides centralized utilities and infrastructure used across
//! the macro crate, including error handling, configuration management,
//! and result type helpers.

pub mod config;
pub mod constants;
pub mod error_handling;
pub mod result;

// Re-export commonly used types
pub use error_handling::Error;
pub use result::{Result, ToCompileError};
