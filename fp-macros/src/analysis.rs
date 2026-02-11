//! Generic and trait analysis subsystem.
//!
//! This module provides utilities for analyzing:
//! - Generic parameters (lifetimes, types, consts)
//! - Pattern detection (FnBrand, Apply!)
//! - Trait bounds and classifications

pub mod generics;
pub mod patterns;
pub mod traits;

pub use generics::{analyze_generics, extract_all_params, extract_type_params};
pub use patterns::{extract_apply_macro_info, extract_fn_brand_info};
pub use traits::{TraitCategory, classify_trait, format_brand_name};
