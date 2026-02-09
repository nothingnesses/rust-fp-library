//! Generic and trait analysis subsystem.
//!
//! This module provides utilities for analyzing:
//! - Generic parameters (lifetimes, types, consts)
//! - Trait bounds and classifications

pub mod generics;
pub mod traits;

pub use generics::{GenericAnalyzer, analyze_generics};
pub use traits::{TraitCategory, classify_trait, format_brand_name};
