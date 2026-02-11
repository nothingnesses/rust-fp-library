//! Self and associated type resolution subsystem.
//!
//! This module handles:
//! - Context extraction from impl blocks
//! - Resolving Self references to concrete types
//! - Projection map management
//! - Resolution errors

pub mod context;
pub mod impl_key;
pub mod projection_key;
pub mod resolver;

pub use context::get_context;
pub use impl_key::ImplKey;
pub use projection_key::ProjectionKey;
