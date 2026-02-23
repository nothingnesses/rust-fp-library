//! Type-to-HM conversion logic.
//!
//! This module contains the main conversion logic for transforming
//! Rust types into Hindley-Milner representations.

use {
	crate::{
		analysis::traits::format_brand_name,
		core::{
			config::Config,
			constants::{traits, types},
		},
		hm::{HmAst, ast_builder::HmAstBuilder},
		support::{TypeVisitor, last_path_segment},
	},
	std::collections::{HashMap, HashSet},
	syn::{GenericArgument, PathArguments, ReturnType, TraitBound, Type},
};

// ============================================================================
// Main Conversion Entry Point
// ============================================================================

/// Convert a Rust type to its Hindley-Milner representation.
///
/// This is the main entry point for type conversion. It creates an HMTypeBuilder
/// visitor and uses it to transform the type.
pub fn type_to_hm(
	ty: &Type,
	fn_bounds: &HashMap<String, HmAst>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HmAst {
	let mut visitor = HmAstBuilder { fn_bounds, generic_names, config };
	visitor.visit(ty)
}

// ============================================================================
// Helper Functions for Type Conversion
// ============================================================================

/// Helper function to check if a type path is PhantomData
pub(crate) fn is_phantom_data_path(type_path: &syn::TypePath) -> bool {
	if let Some(segment) = type_path.path.segments.last() {
		segment.ident == types::PHANTOM_DATA
	} else {
		false
	}
}

/// Helper function to check if a type name is a smart pointer (Box, Arc, Rc)
pub(crate) fn is_smart_pointer(name: &str) -> bool {
	types::SMART_POINTERS.contains(&name)
}

/// Helper function to extract the inner type from a smart pointer if present
pub(crate) fn get_smart_pointer_inner(segment: &syn::PathSegment) -> Option<&syn::Type> {
	if let PathArguments::AngleBracketed(args) = &segment.arguments
		&& let Some(GenericArgument::Type(inner_ty)) = args.args.first()
	{
		return Some(inner_ty);
	}
	None
}

/// Convert a trait bound to an HM type representation.
///
/// This is used for processing trait bounds in impl trait and trait object types.
pub fn trait_bound_to_hm_type(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, HmAst>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HmAst {
	let Some(segment) = last_path_segment(&trait_bound.path) else {
		// Defensive fallback for malformed trait bounds
		return HmAst::Variable("trait".to_string());
	};
	let name = segment.ident.to_string();

	if traits::FN_TRAITS.contains(&name.as_str()) {
		return trait_bound_to_hm_arrow(trait_bound, fn_bounds, generic_names, config);
	}

	let name = if generic_names.contains(&name) || name == types::SELF {
		name.to_lowercase()
	} else {
		format_brand_name(&name, config)
	};

	if let PathArguments::AngleBracketed(args) = &segment.arguments {
		let mut arg_types = Vec::new();
		for arg in &args.args {
			match arg {
				GenericArgument::Type(ty) => {
					arg_types.push(type_to_hm(ty, fn_bounds, generic_names, config));
				}
				GenericArgument::AssocType(assoc) => {
					arg_types.push(type_to_hm(&assoc.ty, fn_bounds, generic_names, config));
				}
				_ => {}
			}
		}
		if !arg_types.is_empty() {
			return HmAst::Constructor(name, arg_types);
		}
	}

	HmAst::Variable(name)
}

/// Convert a trait bound with parenthesized syntax (Fn/FnMut/FnOnce) to an arrow type.
pub fn trait_bound_to_hm_arrow(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, HmAst>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HmAst {
	let Some(segment) = last_path_segment(&trait_bound.path) else {
		// Defensive fallback for malformed input (should never occur with valid Rust)
		return HmAst::Variable("fn".to_string());
	};
	if let PathArguments::Parenthesized(args) = &segment.arguments {
		// Erase HRTB lifetimes from trait bound
		let _ = &trait_bound.lifetimes;

		let inputs: Vec<HmAst> =
			args.inputs.iter().map(|t| type_to_hm(t, fn_bounds, generic_names, config)).collect();
		let output = match &args.output {
			ReturnType::Default => HmAst::Unit,
			ReturnType::Type(_, ty) => type_to_hm(ty, fn_bounds, generic_names, config),
		};

		let input_ty = if inputs.is_empty() {
			HmAst::Unit
		} else if inputs.len() == 1 {
			inputs[0].clone()
		} else {
			HmAst::Tuple(inputs)
		};

		HmAst::Arrow(Box::new(input_ty), Box::new(output))
	} else {
		HmAst::Variable("fn".to_string())
	}
}
