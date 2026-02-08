//! HM type transformation visitor.
//!
//! This module contains the HMTypeBuilder, which implements the TypeVisitor trait
//! to transform Rust types into Hindley-Milner representations.

use crate::{
	analysis::traits::format_brand_name,
	common::last_path_segment,
	config::Config,
	common::errors::known_types,
	hm_conversion::{
		HMType, converter::{
			TypeVisitor, extract_smart_pointer_inner, is_phantom_data_path, is_smart_pointer, trait_bound_to_hm_type,
		},
		extract_apply_macro_info, extract_fn_brand_info,
	},
};
use std::collections::{HashMap, HashSet};
use syn::{GenericArgument, PathArguments, ReturnType, TypeParamBound};

/// Visitor that builds HM type representations from Rust types.
///
/// This is the main type transformation engine. It implements TypeVisitor
/// to traverse Rust type syntax trees and produce HMType representations.
pub struct HMTypeBuilder<'a> {
	pub fn_bounds: &'a HashMap<String, HMType>,
	pub generic_names: &'a HashSet<String>,
	pub config: &'a Config,
}

impl<'a> TypeVisitor for HMTypeBuilder<'a> {
	type Output = HMType;

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) -> Self::Output {
		// Check for FnBrand pattern using shared helper
		if let Some(fn_brand_info) = extract_fn_brand_info(type_path, self.config) {
			let input_hm_types: Vec<_> =
				fn_brand_info.inputs.iter().map(|ty| self.visit(ty)).collect();
			let output_hm = self.visit(&fn_brand_info.output);

			let input = if input_hm_types.is_empty() {
				HMType::Unit
			} else if input_hm_types.len() == 1 {
				input_hm_types[0].clone()
			} else {
				HMType::Tuple(input_hm_types)
			};
			return HMType::Arrow(Box::new(input), Box::new(output_hm));
		}

		if let Some(type_path_inner) = &type_path.qself {
			let constructor_type = self.visit(&type_path_inner.ty);
			let Some(last_segment) = last_path_segment(&type_path.path) else {
				// Defensive fallback for malformed qualified paths
				return HMType::Variable("unknown".to_string());
			};

			let mut args_list = Vec::new();

			if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
				for arg in &args.args {
					if let GenericArgument::Type(inner_ty) = arg {
						args_list.push(self.visit(inner_ty));
					}
				}
			}

			// Merge constructor and args
			match constructor_type {
				HMType::Variable(name) => {
					if args_list.is_empty() {
						HMType::Variable(name)
					} else {
						HMType::Constructor(name, args_list)
					}
				}
				HMType::Constructor(name, mut prev_args) => {
					prev_args.extend(args_list);
					HMType::Constructor(name, prev_args)
				}
				_ => {
					// Fallback: treat the constructor as a string variable if possible, or just fail/print
					// For now, convert to string
					let name = format!("{}", constructor_type);
					HMType::Constructor(name, args_list)
				}
			}
		} else {
			// No QSelf
			if is_phantom_data_path(type_path) {
				return HMType::Unit;
			}

			if type_path.path.segments.len() >= 2 {
				let first = &type_path.path.segments[0];
				let Some(last) = last_path_segment(&type_path.path) else {
					// Should be unreachable given the len() >= 2 check, but handle defensively
					return HMType::Variable("unknown".to_string());
				};

				let mut constructor_name = first.ident.to_string();
				if self.config.concrete_types.contains(&constructor_name) {
					// Preserve concrete types as-is (keep original case)
				} else if self.generic_names.contains(&constructor_name) {
					// Keep type parameters in original case (uppercase)
				} else if constructor_name == known_types::SELF {
					// Use self_type_name if available, otherwise keep as "Self"
					constructor_name = self
						.config
						.self_type_name
						.clone()
						.unwrap_or_else(|| known_types::SELF.to_string());
				} else {
					constructor_name = format_brand_name(&constructor_name, self.config);
				}

				if let PathArguments::AngleBracketed(args) = &last.arguments {
					let mut type_args = Vec::new();
					for arg in &args.args {
						if let GenericArgument::Type(inner_ty) = arg {
							type_args.push(self.visit(inner_ty));
						}
					}
					if !type_args.is_empty() {
						return HMType::Constructor(constructor_name, type_args);
					}
				}
				return HMType::Variable(constructor_name);
			}

			// Simple path or Single segment
			let Some(segment) = last_path_segment(&type_path.path) else {
				// Defensive fallback for empty paths (shouldn't happen with valid Rust)
				return HMType::Variable("unknown".to_string());
			};
			let name = segment.ident.to_string();

			if is_smart_pointer(&name)
				&& let Some(inner_ty) = extract_smart_pointer_inner(segment)
			{
				return self.visit(inner_ty);
			}

			if let Some(sig) = self.fn_bounds.get(&name) {
				if let HMType::Variable(v) = sig
					&& v == known_types::FN_BRAND_MARKER
				{
					// Keep type parameters in original case
					return HMType::Variable(name);
				}
				return sig.clone();
			}

			// Check if this is a concrete type that should be preserved
			if self.config.concrete_types.contains(&name) {
				// But still process generic arguments if present
				match &segment.arguments {
					PathArguments::AngleBracketed(args) => {
						let mut type_args = Vec::new();
						for arg in &args.args {
							if let GenericArgument::Type(inner_ty) = arg {
								type_args.push(self.visit(inner_ty));
							}
						}
						if type_args.is_empty() {
							return HMType::Variable(name);
						} else {
							return HMType::Constructor(name, type_args);
						}
					}
					_ => return HMType::Variable(name),
				}
			}

			// Keep type parameters in original case (uppercase)
			if self.generic_names.contains(&name) {
				return HMType::Variable(name);
			}

			// Handle Self with self_type_name if available
			if name == known_types::SELF {
				return HMType::Variable(
					self.config
						.self_type_name
						.clone()
						.unwrap_or_else(|| known_types::SELF.to_string()),
				);
			}

			let brand_name = format_brand_name(&name, self.config);

			match &segment.arguments {
				PathArguments::AngleBracketed(args) => {
					let mut type_args = Vec::new();
					for arg in &args.args {
						if let GenericArgument::Type(inner_ty) = arg {
							type_args.push(self.visit(inner_ty));
						}
					}
					if type_args.is_empty() {
						HMType::Variable(brand_name)
					} else {
						HMType::Constructor(brand_name, type_args)
					}
				}
				_ => HMType::Variable(brand_name),
			}
		}
	}

	fn visit_macro(
		&mut self,
		type_macro: &syn::TypeMacro,
	) -> Self::Output {
		// Check for Apply! macro using shared helper
		if let Some((brand, args)) = extract_apply_macro_info(type_macro) {
			let constructor_type = self.visit(&brand);
			let type_args: Vec<_> = args.iter().map(|ty| self.visit(ty)).collect();

			match constructor_type {
				HMType::Variable(name) => {
					if type_args.is_empty() {
						HMType::Variable(name)
					} else {
						HMType::Constructor(name, type_args)
					}
				}
				HMType::Constructor(name, mut prev_args) => {
					prev_args.extend(type_args);
					HMType::Constructor(name, prev_args)
				}
				_ => {
					let name = format!("{}", constructor_type);
					HMType::Constructor(name, type_args)
				}
			}
		} else {
			HMType::Variable("macro".to_string())
		}
	}

	fn visit_reference(
		&mut self,
		type_ref: &syn::TypeReference,
	) -> Self::Output {
		let inner = self.visit(&type_ref.elem);
		if type_ref.mutability.is_some() {
			HMType::MutableReference(Box::new(inner))
		} else {
			HMType::Reference(Box::new(inner))
		}
	}

	fn visit_trait_object(
		&mut self,
		trait_object: &syn::TypeTraitObject,
	) -> Self::Output {
		// Erase auto traits and lifetimes from trait objects
		let mut bounds = Vec::new();
		for bound in &trait_object.bounds {
			if let syn::TypeParamBound::Trait(trait_bound) = bound
				&& let Some(segment) = last_path_segment(&trait_bound.path)
			{
				let name = segment.ident.to_string();
				if !self.config.ignored_traits().contains(&name) {
					bounds.push(trait_bound_to_hm_type(
						trait_bound,
						self.fn_bounds,
						self.generic_names,
						self.config,
					));
				}
			}
			// If path is empty, skip this bound (defensive handling)
		}

		if bounds.is_empty() {
			HMType::TraitObject(Box::new(HMType::Variable("_".to_string())))
		} else {
			let inner = if bounds.len() == 1 { bounds[0].clone() } else { HMType::Tuple(bounds) };
			HMType::TraitObject(Box::new(inner))
		}
	}

	fn visit_impl_trait(
		&mut self,
		impl_trait: &syn::TypeImplTrait,
	) -> Self::Output {
		for bound in &impl_trait.bounds {
			if let TypeParamBound::Trait(trait_bound) = bound {
				return trait_bound_to_hm_type(
					trait_bound,
					self.fn_bounds,
					self.generic_names,
					self.config,
				);
			}
		}
		HMType::Variable("impl_trait".to_string())
	}

	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) -> Self::Output {
		// Erase unsafe and lifetimes from bare fns
		let inputs: Vec<HMType> = bare_fn.inputs.iter().map(|arg| self.visit(&arg.ty)).collect();
		let output = match &bare_fn.output {
			ReturnType::Default => HMType::Unit,
			ReturnType::Type(_, ty) => self.visit(ty),
		};
		let input_ty = if inputs.len() == 1 { inputs[0].clone() } else { HMType::Tuple(inputs) };
		HMType::Arrow(Box::new(input_ty), Box::new(output))
	}

	fn visit_tuple(
		&mut self,
		tuple: &syn::TypeTuple,
	) -> Self::Output {
		let types: Vec<HMType> =
			tuple.elems.iter().filter(|t| !crate::common::is_phantom_data(t)).map(|t| self.visit(t)).collect();
		if types.is_empty() {
			HMType::Unit
		} else if types.len() == 1 {
			types[0].clone()
		} else {
			HMType::Tuple(types)
		}
	}

	fn visit_array(
		&mut self,
		array: &syn::TypeArray,
	) -> Self::Output {
		let inner = self.visit(&array.elem);
		HMType::List(Box::new(inner))
	}

	fn visit_slice(
		&mut self,
		slice: &syn::TypeSlice,
	) -> Self::Output {
		let inner = self.visit(&slice.elem);
		HMType::List(Box::new(inner))
	}

	fn visit_other(
		&mut self,
		_ty: &syn::Type,
	) -> Self::Output {
		HMType::Variable("_".to_string())
	}
}
