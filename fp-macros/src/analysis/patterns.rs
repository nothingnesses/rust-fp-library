//! Pattern detection for HM type conversion.
//!
//! This module handles:
//! - FnBrand pattern detection and extraction
//! - Apply! macro pattern detection and extraction

use {
	crate::{
		analysis::traits::{
			TraitCategory,
			classify_trait,
		},
		core::{
			config::Config,
			constants::macros,
		},
		hkt::ApplyInput,
	},
	syn::{
		GenericArgument,
		PathArguments,
	},
};

// ============================================================================
// FnBrand and Apply! Pattern Detection
// ============================================================================

/// Helper structure to hold the result of FnBrand detection from a TypePath.
///
/// FnBrands (like CloneableFn, SendCloneableFn, Function) encode function types
/// using associated type syntax. This structure contains the extracted type arguments.
pub struct FnBrandInfo {
	/// Type arguments extracted from the FnBrand (excluding the return type)
	pub inputs: Vec<syn::Type>,
	/// The return type (last type argument)
	pub output: syn::Type,
}

/// Attempts to extract FnBrand information from a TypePath with QSelf.
///
/// FnBrands use the pattern `<Brand as Trait>::Apply<Input1, Input2, ..., Output>`
/// where the last type argument is the return type and earlier arguments are inputs.
///
/// ### Returns
/// `Some(FnBrandInfo)` if this is a valid FnBrand pattern, `None` otherwise.
pub fn get_fn_brand_info(
	type_path: &syn::TypePath,
	config: &Config,
) -> Option<FnBrandInfo> {
	if let Some(_qself) = &type_path.qself
		&& type_path.path.segments.len() >= 2
	{
		// SAFETY: segments.len() >= 2 checked above
		#[expect(clippy::indexing_slicing, reason = "segments.len() >= 2 checked above")]
		let trait_name = type_path.path.segments[0].ident.to_string();
		if let TraitCategory::FnBrand = classify_trait(&trait_name, config) {
			let last_segment = type_path.path.segments.last()?;
			if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
				let mut type_args: Vec<_> = args
					.args
					.iter()
					.filter_map(|arg| {
						if let GenericArgument::Type(t) = arg { Some(t.clone()) } else { None }
					})
					.collect();

				if !type_args.is_empty() {
					let output = type_args.pop()?;
					return Some(FnBrandInfo {
						inputs: type_args,
						output,
					});
				}
			}
		}
	}
	None
}

/// Attempts to parse an Apply! macro invocation and extract its arguments.
///
/// ### Returns
/// `Some((brand, args))` if this is a valid Apply! macro, `None` otherwise.
pub fn get_apply_macro_parameters(
	type_macro: &syn::TypeMacro
) -> Option<(syn::Type, Vec<syn::Type>)> {
	if type_macro.mac.path.is_ident(macros::APPLY_MACRO)
		&& let Ok(apply_input) = syn::parse2::<ApplyInput>(type_macro.mac.tokens.clone())
	{
		let brand = apply_input.brand;
		let args: Vec<_> = apply_input
			.args
			.args
			.iter()
			.filter_map(|arg| {
				if let syn::GenericArgument::Type(t) = arg { Some(t.clone()) } else { None }
			})
			.collect();
		return Some((brand, args));
	}
	None
}
