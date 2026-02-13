//! Generic and trait analysis subsystem.
//!
//! This module provides utilities for analyzing:
//! - Generic parameters (lifetimes, types, consts)
//! - Pattern detection (FnBrand, Apply!)
//! - Trait bounds and classifications

pub mod generics;
pub mod patterns;
pub mod traits;

pub use generics::{analyze_generics, get_all_parameters, get_type_parameters};
pub use patterns::{get_apply_macro_parameters, get_fn_brand_info};
pub use traits::{TraitCategory, classify_trait, format_brand_name};
