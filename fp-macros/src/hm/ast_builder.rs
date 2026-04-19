//! HM type transformation visitor.
//!
//! This module contains the HmAstBuilder, which implements the TypeVisitor trait
//! to transform Rust types into Hindley-Milner representations.

use {
	crate::{
		analysis::{
			get_apply_macro_parameters,
			get_fn_brand_info,
			traits::format_brand_name,
		},
		core::{
			config::Config,
			constants::{
				markers,
				types,
			},
		},
		hm::{
			HmAst,
			converter::{
				get_smart_pointer_inner,
				is_phantom_data_path,
				is_smart_pointer,
				trait_bound_to_hm_type,
			},
		},
		support::{
			TypeVisitor,
			last_path_segment,
		},
	},
	std::collections::{
		HashMap,
		HashSet,
	},
	syn::{
		GenericArgument,
		PathArguments,
		ReturnType,
		TypeParamBound,
	},
};

/// Visitor that builds HM type representations from Rust types.
///
/// This is the main type transformation engine. It implements TypeVisitor
/// to traverse Rust type syntax trees and produce HmAst representations.
pub struct HmAstBuilder<'a> {
	pub fn_bounds: &'a HashMap<String, HmAst>,
	pub generic_names: &'a HashSet<String>,
	pub config: &'a Config,
}

impl HmAstBuilder<'_> {
	/// Extract type arguments from angle-bracketed generic arguments,
	/// converting each to its HM representation.
	fn visit_type_args(
		&mut self,
		args: &syn::AngleBracketedGenericArguments,
	) -> Vec<HmAst> {
		args.args
			.iter()
			.filter_map(|arg| {
				if let GenericArgument::Type(inner_ty) = arg {
					Some(self.visit(inner_ty))
				} else {
					None
				}
			})
			.collect()
	}

	/// Extract the HM representation from a list of trait bounds (as found
	/// in `impl Trait` and `dyn Trait` types).
	///
	/// Filters out ignored traits (Send, Sync, etc.) and converts the
	/// remaining bounds via `trait_bound_to_hm_type`. Returns the single
	/// bound directly, multiple bounds as a Tuple, or a placeholder Variable
	/// if no meaningful bounds remain.
	fn extract_trait_bound_hm(
		&mut self,
		bounds: &syn::punctuated::Punctuated<TypeParamBound, syn::token::Plus>,
	) -> HmAst {
		let mut hm_bounds = Vec::new();
		for bound in bounds {
			if let TypeParamBound::Trait(trait_bound) = bound
				&& let Some(segment) = last_path_segment(&trait_bound.path)
			{
				let name = segment.ident.to_string();
				if !self.config.ignored_traits().contains(&name) {
					hm_bounds.push(trait_bound_to_hm_type(
						trait_bound,
						self.fn_bounds,
						self.generic_names,
						self.config,
					));
				}
			}
		}
		match hm_bounds.len() {
			0 => HmAst::Variable("_".to_string()),
			1 => hm_bounds.into_iter().next().unwrap_or_else(|| HmAst::Variable("_".to_string())),
			_ => HmAst::Tuple(hm_bounds),
		}
	}

	/// Build a Variable or Constructor node depending on whether type arguments
	/// are present. Used for concrete types and brand-named types.
	fn variable_or_constructor(
		&mut self,
		name: String,
		arguments: &PathArguments,
	) -> HmAst {
		if let PathArguments::AngleBracketed(args) = arguments {
			let type_args = self.visit_type_args(args);
			if !type_args.is_empty() {
				return HmAst::Constructor(name, type_args);
			}
		}
		HmAst::Variable(name)
	}
}

impl<'a> TypeVisitor for HmAstBuilder<'a> {
	type Output = HmAst;

	fn default_output(&self) -> Self::Output {
		HmAst::Unit
	}

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) -> Self::Output {
		// Check for FnBrand pattern using shared helper
		if let Some(fn_brand_info) = get_fn_brand_info(type_path, self.config) {
			let input_hm_types: Vec<_> =
				fn_brand_info.inputs.iter().map(|ty| self.visit(ty)).collect();
			let output_hm = self.visit(&fn_brand_info.output);

			let input = if input_hm_types.is_empty() {
				HmAst::Unit
			} else if input_hm_types.len() == 1 {
				// SAFETY: length checked to be 1 above
				#[expect(clippy::indexing_slicing, reason = "Length checked above")]
				input_hm_types[0].clone()
			} else {
				HmAst::Tuple(input_hm_types)
			};
			return HmAst::Arrow(Box::new(input), Box::new(output_hm));
		}

		if let Some(type_path_inner) = &type_path.qself {
			let constructor_type = self.visit(&type_path_inner.ty);
			let Some(last_segment) = last_path_segment(&type_path.path) else {
				// Defensive fallback for malformed qualified paths
				return HmAst::Variable("unknown".to_string());
			};

			let args_list = if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
				self.visit_type_args(args)
			} else {
				Vec::new()
			};

			// Merge constructor and args
			match constructor_type {
				HmAst::Variable(name) =>
					if args_list.is_empty() {
						HmAst::Variable(name)
					} else {
						HmAst::Constructor(name, args_list)
					},
				HmAst::Constructor(name, mut prev_args) => {
					prev_args.extend(args_list);
					HmAst::Constructor(name, prev_args)
				}
				_ => {
					// Fallback: treat the constructor as a string variable if possible, or just fail/print
					// For now, convert to string
					let name = format!("{constructor_type}");
					HmAst::Constructor(name, args_list)
				}
			}
		} else {
			// No QSelf
			if is_phantom_data_path(type_path) {
				return HmAst::Unit;
			}

			if type_path.path.segments.len() >= 2 {
				// SAFETY: segments.len() >= 2 checked above
				#[expect(clippy::indexing_slicing, reason = "Length checked above")]
				let first = &type_path.path.segments[0];
				let Some(last) = last_path_segment(&type_path.path) else {
					// Should be unreachable given the len() >= 2 check, but handle defensively
					return HmAst::Variable("unknown".to_string());
				};

				// Detect associated type access: Brand::Index where first segment
				// is a known generic type parameter. Return just the associated
				// type name (e.g., "Index") instead of the brand name.
				let first_name = first.ident.to_string();
				if type_path.path.segments.len() == 2
					&& self.generic_names.contains(&first_name)
					&& matches!(last.arguments, PathArguments::None)
				{
					return HmAst::Variable(last.ident.to_string());
				}

				let mut constructor_name = first_name;
				if self.config.concrete_types.contains(&constructor_name) {
					// Preserve concrete types as-is (keep original case)
				} else if self.generic_names.contains(&constructor_name) {
					// Keep type parameters in original case (uppercase)
				} else if constructor_name == types::SELF {
					// Use self_type_name if available, otherwise keep as "Self"
					constructor_name = self
						.config
						.self_type_name
						.clone()
						.unwrap_or_else(|| types::SELF.to_string());
				} else {
					constructor_name = format_brand_name(&constructor_name, self.config);
				}

				if let PathArguments::AngleBracketed(args) = &last.arguments {
					let type_args = self.visit_type_args(args);
					if !type_args.is_empty() {
						return HmAst::Constructor(constructor_name, type_args);
					}
				}
				return HmAst::Variable(constructor_name);
			}

			// Simple path or Single segment
			let Some(segment) = last_path_segment(&type_path.path) else {
				// Defensive fallback for empty paths (shouldn't happen with valid Rust)
				return HmAst::Variable("unknown".to_string());
			};
			let name = segment.ident.to_string();

			if is_smart_pointer(&name)
				&& let Some(inner_ty) = get_smart_pointer_inner(segment)
			{
				return self.visit(inner_ty);
			}

			if let Some(sig) = self.fn_bounds.get(&name) {
				if let HmAst::Variable(v) = sig
					&& v == markers::FN_BRAND_MARKER
				{
					// Keep type parameters in original case
					return HmAst::Variable(name);
				}
				return sig.clone();
			}

			// Check if this is a concrete type that should be preserved
			if self.config.concrete_types.contains(&name) {
				// But still process generic arguments if present
				return self.variable_or_constructor(name, &segment.arguments);
			}

			// Keep type parameters in original case (uppercase)
			if self.generic_names.contains(&name) {
				return HmAst::Variable(name);
			}

			// Handle Self with self_type_name if available
			if name == types::SELF {
				return HmAst::Variable(
					self.config.self_type_name.clone().unwrap_or_else(|| types::SELF.to_string()),
				);
			}

			let brand_name = format_brand_name(&name, self.config);
			self.variable_or_constructor(brand_name, &segment.arguments)
		}
	}

	fn visit_macro(
		&mut self,
		type_macro: &syn::TypeMacro,
	) -> Self::Output {
		// Check for Apply! macro using shared helper
		if let Some((brand, args)) = get_apply_macro_parameters(type_macro) {
			let constructor_type = self.visit(&brand);
			let type_args: Vec<_> = args.iter().map(|ty| self.visit(ty)).collect();

			match constructor_type {
				HmAst::Variable(name) =>
					if type_args.is_empty() {
						HmAst::Variable(name)
					} else {
						HmAst::Constructor(name, type_args)
					},
				HmAst::Constructor(name, mut prev_args) => {
					prev_args.extend(type_args);
					HmAst::Constructor(name, prev_args)
				}
				_ => {
					let name = format!("{constructor_type}");
					HmAst::Constructor(name, type_args)
				}
			}
		} else {
			HmAst::Variable("macro".to_string())
		}
	}

	fn visit_reference(
		&mut self,
		type_ref: &syn::TypeReference,
	) -> Self::Output {
		let inner = self.visit(&type_ref.elem);
		if type_ref.mutability.is_some() {
			HmAst::MutableReference(Box::new(inner))
		} else {
			HmAst::Reference(Box::new(inner))
		}
	}

	fn visit_trait_object(
		&mut self,
		trait_object: &syn::TypeTraitObject,
	) -> Self::Output {
		// HM signatures erase dyn, same as impl: both represent the abstract
		// type (e.g., Iterator String), not Rust's dispatch strategy.
		self.extract_trait_bound_hm(&trait_object.bounds)
	}

	fn visit_impl_trait(
		&mut self,
		impl_trait: &syn::TypeImplTrait,
	) -> Self::Output {
		self.extract_trait_bound_hm(&impl_trait.bounds)
	}

	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) -> Self::Output {
		// Erase unsafe and lifetimes from bare fns
		let inputs: Vec<HmAst> = bare_fn.inputs.iter().map(|arg| self.visit(&arg.ty)).collect();
		let output = match &bare_fn.output {
			ReturnType::Default => HmAst::Unit,
			ReturnType::Type(_, ty) => self.visit(ty),
		};
		// SAFETY: inputs.len() == 1 checked in condition
		#[expect(clippy::indexing_slicing, reason = "Length checked above")]
		let input_ty = if inputs.len() == 1 { inputs[0].clone() } else { HmAst::Tuple(inputs) };
		HmAst::Arrow(Box::new(input_ty), Box::new(output))
	}

	fn visit_tuple(
		&mut self,
		tuple: &syn::TypeTuple,
	) -> Self::Output {
		let types: Vec<HmAst> = tuple
			.elems
			.iter()
			.filter(|t| !crate::support::is_phantom_data(t))
			.map(|t| self.visit(t))
			.collect();
		if types.is_empty() {
			HmAst::Unit
		} else if types.len() == 1 {
			// SAFETY: types.len() == 1 checked above
			#[expect(clippy::indexing_slicing, reason = "Length checked above")]
			types[0].clone()
		} else {
			HmAst::Tuple(types)
		}
	}

	fn visit_array(
		&mut self,
		array: &syn::TypeArray,
	) -> Self::Output {
		let inner = self.visit(&array.elem);
		HmAst::List(Box::new(inner))
	}

	fn visit_slice(
		&mut self,
		slice: &syn::TypeSlice,
	) -> Self::Output {
		let inner = self.visit(&slice.elem);
		HmAst::List(Box::new(inner))
	}

	fn visit_other(
		&mut self,
		_ty: &syn::Type,
	) -> Self::Output {
		HmAst::Variable("_".to_string())
	}
}
