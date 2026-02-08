//! Self and associated type resolution subsystem.
//!
//! This module handles:
//! - Context extraction from impl blocks
//! - Resolving Self references to concrete types
//! - Projection map management
//! - Resolution errors

pub mod context;
pub mod resolver;
pub mod errors;

pub use context::extract_context;
pub use errors::ErrorCollector;
