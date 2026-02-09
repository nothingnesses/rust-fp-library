//! Core infrastructure for fp-macros.
//!
//! This module provides centralized utilities and infrastructure used across
//! the macro crate, including attribute filtering, configuration management,
//! and common validation logic.

pub mod attributes;
pub mod config;

pub use attributes::DocAttributeFilter;
pub use config::get_config;
