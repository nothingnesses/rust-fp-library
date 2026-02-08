//! Shared utilities used across the macro system.
//!
//! This module provides common infrastructure including:
//! - Error handling
//! - Attribute utilities
//! - Syntax helpers

pub mod attributes;
pub mod errors;
pub mod syntax;

pub use syntax::{LogicalParam, get_logical_params, is_phantom_data, last_path_segment};
