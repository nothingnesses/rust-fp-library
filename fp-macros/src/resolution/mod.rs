//! Self and associated type resolution subsystem.
//!
//! This module handles:
//! - Context extraction from impl blocks
//! - Resolving Self references to concrete types
//! - Projection map management
//! - Resolution errors

pub mod context;
pub mod errors;
pub mod projection_key;
pub mod resolver;

pub use context::extract_context;
pub use errors::ErrorCollector;
pub use projection_key::ProjectionKey;
