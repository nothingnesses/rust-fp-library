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

/// Infrastructure trait names that are not semantic type class constraints.
/// These are marker/capability traits that should be excluded from dispatch
/// analysis and HM signature constraints.
const INFRASTRUCTURE_TRAITS: &[&str] =
	&["Send", "Sync", "Clone", "Copy", "Debug", "Display", "Sized", "LiftFn", "SendLiftFn"];

#[derive(Debug, PartialEq)]
pub enum TraitCategory {
	/// Fn, FnMut, FnOnce.
	FnTrait,
	/// CloneableFn, SendCloneableFn, Function (function-wrapping brands).
	FnBrand,
	/// The Apply! macro or aliases.
	ApplyMacro,
	/// Kind_* or InferableBrand_* traits.
	Kind,
	/// *Dispatch traits (e.g., FunctorDispatch, ApplyDispatch).
	Dispatch,
	/// Infrastructure traits (Send, Sync, Clone, etc.) that are not
	/// semantic type class constraints.
	Infrastructure,
	/// A semantic type class or other user-defined trait.
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
		n if n.starts_with(markers::INFERABLE_BRAND_PREFIX) => TraitCategory::Kind,
		n if n.ends_with(markers::DISPATCH_SUFFIX) => TraitCategory::Dispatch,
		n if INFRASTRUCTURE_TRAITS.contains(&n) => TraitCategory::Infrastructure,
		_ => TraitCategory::Other(name.to_string()),
	}
}

/// Check if a trait name represents a semantic type class constraint
/// (as opposed to infrastructure like Fn, Kind, Send, Dispatch, etc.).
///
/// Uses only compile-time constant checks without requiring Config.
/// This works because Config-dependent categories (Apply! macro aliases)
/// never appear as trait bounds in dispatch contexts.
pub fn is_semantic_type_class(name: &str) -> bool {
	// Not a Fn trait
	if traits::FN_TRAITS.contains(&name) {
		return false;
	}
	// Not a FnBrand
	if brands::FN_BRANDS.contains(&name) {
		return false;
	}
	// Not a Kind or InferableBrand trait
	if name.starts_with(markers::KIND_PREFIX) || name.starts_with(markers::INFERABLE_BRAND_PREFIX) {
		return false;
	}
	// Not a dispatch trait
	if name.ends_with(markers::DISPATCH_SUFFIX) {
		return false;
	}
	// Not infrastructure
	if INFRASTRUCTURE_TRAITS.contains(&name) {
		return false;
	}
	true
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
