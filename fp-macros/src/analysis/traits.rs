//! Trait classification and analysis.

use {
	crate::{
		core::{
			config::Config,
			constants::{
				brands,
				macros,
				markers,
				traits,
			},
		},
		hm::HmAst,
		support::last_path_segment,
	},
	std::collections::{
		HashMap,
		HashSet,
	},
};

#[derive(Debug, PartialEq)]
pub enum TraitCategory {
	FnTrait,
	FnBrand,
	ApplyMacro,
	Kind,
	Other(String),
}

pub fn classify_trait(
	name: &str,
	config: &Config,
) -> TraitCategory {
	match name {
		n if traits::FN_TRAITS.contains(&n) => TraitCategory::FnTrait,
		n if brands::FN_BRANDS.contains(&n) => TraitCategory::FnBrand,
		macros::APPLY_MACRO => TraitCategory::ApplyMacro,
		n if config.apply_macro_aliases().contains(n) => TraitCategory::ApplyMacro,
		n if n.starts_with(markers::KIND_PREFIX) => TraitCategory::Kind,
		n if n.starts_with(markers::SLOT_PREFIX) => TraitCategory::Kind,
		_ => TraitCategory::Other(name.to_string()),
	}
}

/// Extract the HM type from a trait bound if it represents a function type.
///
/// Returns Some(HMType) if the trait bound is a function trait (Fn, FnMut, FnOnce, or FnBrand),
/// None otherwise.
pub fn get_fn_type_from_bound(
	trait_bound: &syn::TraitBound,
	fn_bounds: &HashMap<String, HmAst>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> Option<HmAst> {
	let segment = last_path_segment(&trait_bound.path)?;
	let name = segment.ident.to_string();
	match classify_trait(&name, config) {
		TraitCategory::FnTrait => Some(crate::hm::converter::trait_bound_to_hm_arrow(
			trait_bound,
			fn_bounds,
			generic_names,
			config,
		)),
		TraitCategory::FnBrand => Some(HmAst::Variable(markers::FN_BRAND_MARKER.to_string())),
		_ => None,
	}
}

pub fn format_brand_name(
	name: &str,
	config: &Config,
) -> String {
	if let Some(mapping) = config.brand_mappings().get(name) {
		return mapping.clone();
	}

	if let Some(stripped) = name.strip_suffix(markers::BRAND_SUFFIX) {
		if stripped.is_empty() {
			// "Brand" alone should not be stripped to ""
			name.to_string()
		} else {
			stripped.to_string()
		}
	} else {
		name.to_string()
	}
}
