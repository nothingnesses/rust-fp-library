use {
	crate::{
		analysis::{
			get_apply_macro_parameters,
			patterns::get_fn_brand_info,
			traits::{
				TraitCategory,
				classify_trait,
			},
		},
		core::{
			config::Config,
			constants::types,
		},
		support::type_visitor::TypeVisitor,
	},
	syn::{
		PathArguments,
		ReturnType,
		TraitBound,
		Type,
		TypeParamBound,
	},
};

/// Represents a parameter in a function signature, either explicit or implicit.
#[derive(Clone, Debug)]
pub enum Parameter {
	/// A parameter that appears explicitly in the function signature (e.g., `x: i32`)
	Explicit(syn::Pat),
	/// A parameter that is implicit from trait bounds or other context (e.g., from Fn trait bounds)
	///
	/// This variant is constructed during curried parameter extraction and matched in
	/// documentation generation to represent implicit parameters as `_` in signatures.
	///
	/// Note: The `syn::Type` field is currently not accessed during matching, but is preserved
	/// for potential future use in generating more detailed documentation. This causes a
	/// compiler warning about unused fields, which is expected and acceptable.
	#[allow(dead_code)]
	Implicit(syn::Type),
}

/// Check if a type is PhantomData.
pub fn is_phantom_data(ty: &Type) -> bool {
	match ty {
		Type::Path(type_path) => {
			if let Some(segment) = type_path.path.segments.last() {
				return segment.ident == types::PHANTOM_DATA;
			}
			false
		}
		Type::Reference(type_ref) => is_phantom_data(&type_ref.elem),
		_ => false,
	}
}

/// Extract all logical parameters from a function signature.
///
/// This includes both explicit parameters and implicit curried parameters from the return type.
pub fn get_parameters(
	sig: &syn::Signature,
	config: &Config,
) -> Vec<Parameter> {
	let mut params = Vec::new();

	// 1. Explicit arguments
	for input in &sig.inputs {
		match input {
			syn::FnArg::Receiver(_) => continue, // Skip self
			syn::FnArg::Typed(pat_type) =>
				if !is_phantom_data(&pat_type.ty) {
					params.push(Parameter::Explicit((*pat_type.pat).clone()));
				},
		}
	}

	// 2. Curried arguments from return type
	get_curried_parameters(&sig.output, &mut params, config);

	params
}

fn get_curried_parameters(
	output: &ReturnType,
	params: &mut Vec<Parameter>,
	config: &Config,
) {
	if let ReturnType::Type(_, ty) = output {
		get_parameters_from_type(ty, params, config);
	}
}

fn get_parameters_from_type(
	ty: &Type,
	params: &mut Vec<Parameter>,
	config: &Config,
) {
	let mut visitor = CurriedParametersExtractor {
		params,
		config,
	};
	visitor.visit(ty);
}

/// Get the last segment of a path.
pub fn last_path_segment(path: &syn::Path) -> Option<&syn::PathSegment> {
	path.segments.last()
}

/// A type visitor that extracts curried parameters from types.
///
/// This visitor traverses type structures looking for function-like patterns
/// and extracts their parameters as implicit `LogicalParam` entries.
struct CurriedParametersExtractor<'a> {
	params: &'a mut Vec<Parameter>,
	config: &'a Config,
}

impl<'a> TypeVisitor for CurriedParametersExtractor<'a> {
	type Output = ();

	fn default_output(&self) -> Self::Output {}

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) {
		// Check for FnBrand pattern using shared helper
		if let Some(fn_brand_info) = get_fn_brand_info(type_path, self.config) {
			// Add all input types as implicit parameters
			for input_ty in fn_brand_info.inputs {
				self.params.push(Parameter::Implicit(input_ty));
			}
			// Recursively visit the output type for nested currying
			self.visit(&fn_brand_info.output);
		}
	}

	fn visit_macro(
		&mut self,
		type_macro: &syn::TypeMacro,
	) {
		// Apply! macro support is handled by extracting its info, but we don't
		// currently extract curried parameters from Apply! projections.
		// This could be enhanced in the future if needed.
		let _ = get_apply_macro_parameters(type_macro);
	}

	fn visit_impl_trait(
		&mut self,
		impl_trait: &syn::TypeImplTrait,
	) {
		for bound in &impl_trait.bounds {
			if let TypeParamBound::Trait(trait_bound) = bound {
				self.visit_trait_bound_helper(trait_bound);
			}
		}
	}

	fn visit_trait_object(
		&mut self,
		trait_obj: &syn::TypeTraitObject,
	) {
		for bound in &trait_obj.bounds {
			if let TypeParamBound::Trait(trait_bound) = bound {
				self.visit_trait_bound_helper(trait_bound);
			}
		}
	}

	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) {
		for input in &bare_fn.inputs {
			self.params.push(Parameter::Implicit(input.ty.clone()));
		}
		if let ReturnType::Type(_, ty) = &bare_fn.output {
			self.visit(ty);
		}
	}
}

impl<'a> CurriedParametersExtractor<'a> {
	fn visit_trait_bound_helper(
		&mut self,
		trait_bound: &TraitBound,
	) {
		let Some(segment) = last_path_segment(&trait_bound.path) else {
			return; // Skip malformed trait bounds
		};
		let name = segment.ident.to_string();

		if let TraitCategory::FnTrait = classify_trait(&name, self.config)
			&& let PathArguments::Parenthesized(args) = &segment.arguments
		{
			for input in &args.inputs {
				self.params.push(Parameter::Implicit(input.clone()));
			}
			if let ReturnType::Type(_, ty) = &args.output {
				self.visit(ty);
			}
		}
	}
}
