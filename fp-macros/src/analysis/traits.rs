//! Trait classification and analysis.
//!
//! Extracted from function_utils.rs

use crate::config::Config;

#[derive(Debug, PartialEq)]
pub enum TraitCategory {
	FnTrait,
	FnBrand,
	ApplyMacro,
	Other(String),
}

pub fn classify_trait(
	name: &str,
	config: &Config,
) -> TraitCategory {
	match name {
		"Fn" | "FnMut" | "FnOnce" => TraitCategory::FnTrait,
		"SendCloneableFn" | "CloneableFn" | "Function" => TraitCategory::FnBrand,
		"Apply" => TraitCategory::ApplyMacro,
		n if config.apply_macro_aliases().contains(n) => TraitCategory::ApplyMacro,
		_ => TraitCategory::Other(name.to_string()),
	}
}

pub fn format_brand_name(
	name: &str,
	config: &Config,
) -> String {
	if let Some(mapping) = config.brand_mappings().get(name) {
		return mapping.clone();
	}

	if let Some(stripped) = name.strip_suffix("Brand") {
		stripped.to_string()
	} else {
		name.to_string()
	}
}
