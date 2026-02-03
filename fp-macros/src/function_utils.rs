use crate::{apply::ApplyInput, hm_ast::HMType};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use syn::{
	GenericArgument, GenericParam, PathArguments, ReturnType, Signature, TraitBound, Type,
	TypeParamBound, TypeTraitObject, WherePredicate,
};

#[derive(Debug, Deserialize)]
pub struct Config {
	#[serde(default)]
	pub brand_mappings: HashMap<String, String>,
	#[serde(default)]
	pub apply_macro_aliases: HashSet<String>,
	#[serde(default = "default_ignored_traits")]
	pub ignored_traits: HashSet<String>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			brand_mappings: HashMap::new(),
			apply_macro_aliases: HashSet::new(),
			ignored_traits: default_ignored_traits(),
		}
	}
}

fn default_ignored_traits() -> HashSet<String> {
	[
		"Clone",
		"Copy",
		"Debug",
		"Display",
		"PartialEq",
		"Eq",
		"PartialOrd",
		"Ord",
		"Hash",
		"Default",
		"Send",
		"Sync",
		"Sized",
		"Unpin",
	]
	.iter()
	.map(|s| s.to_string())
	.collect()
}

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
		n if config.apply_macro_aliases.contains(n) => TraitCategory::ApplyMacro,
		_ => TraitCategory::Other(name.to_string()),
	}
}

pub trait TypeVisitor {
	type Output;

	fn visit(
		&mut self,
		ty: &Type,
	) -> Self::Output {
		match ty {
			Type::Path(p) => self.visit_path(p),
			Type::Macro(m) => self.visit_macro(m),
			Type::Reference(r) => self.visit_reference(r),
			Type::ImplTrait(i) => self.visit_impl_trait(i),
			Type::TraitObject(t) => self.visit_trait_object(t),
			Type::BareFn(f) => self.visit_bare_fn(f),
			Type::Tuple(t) => self.visit_tuple(t),
			Type::Array(a) => self.visit_array(a),
			Type::Slice(s) => self.visit_slice(s),
			_ => self.visit_other(ty),
		}
	}

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) -> Self::Output;
	fn visit_macro(
		&mut self,
		type_macro: &syn::TypeMacro,
	) -> Self::Output;
	fn visit_reference(
		&mut self,
		type_ref: &syn::TypeReference,
	) -> Self::Output;
	fn visit_impl_trait(
		&mut self,
		impl_trait: &syn::TypeImplTrait,
	) -> Self::Output;
	fn visit_trait_object(
		&mut self,
		trait_obj: &syn::TypeTraitObject,
	) -> Self::Output;
	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) -> Self::Output;
	fn visit_tuple(
		&mut self,
		tuple: &syn::TypeTuple,
	) -> Self::Output;
	fn visit_array(
		&mut self,
		array: &syn::TypeArray,
	) -> Self::Output;
	fn visit_slice(
		&mut self,
		slice: &syn::TypeSlice,
	) -> Self::Output;
	fn visit_other(
		&mut self,
		ty: &syn::Type,
	) -> Self::Output;
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
	hm_signature: Option<Config>,
}

#[derive(Debug, Deserialize)]
struct CargoManifest {
	package: Option<PackageMetadata>,
}

#[derive(Debug, Deserialize)]
struct PackageMetadata {
	metadata: Option<CargoMetadata>,
}

pub fn load_config() -> Config {
	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
	let manifest_path = std::path::Path::new(&manifest_dir).join("Cargo.toml");

	if let Ok(content) = std::fs::read_to_string(manifest_path)
		&& let Ok(manifest) = toml::from_str::<CargoManifest>(&content)
		&& let Some(package) = manifest.package
		&& let Some(metadata) = package.metadata
		&& let Some(config) = metadata.hm_signature
	{
		return config;
	}
	Config::default()
}

pub fn analyze_generics(
	sig: &Signature,
	config: &Config,
) -> (HashSet<String>, HashMap<String, HMType>) {
	let mut fn_bounds = HashMap::new();
	let mut generic_names = HashSet::new();

	// Collect all generic type names
	for param in &sig.generics.params {
		if let GenericParam::Type(type_param) = param {
			generic_names.insert(type_param.ident.to_string());
		}
	}

	// Collect Fn bounds from generics
	for param in &sig.generics.params {
		if let GenericParam::Type(type_param) = param {
			let name = type_param.ident.to_string();
			for bound in &type_param.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound
					&& let Some(sig_ty) =
						get_fn_type(trait_bound, &fn_bounds, &generic_names, config)
				{
					fn_bounds.insert(name.clone(), sig_ty);
				}
			}
		}
	}

	// Collect Fn bounds from where clause
	if let Some(where_clause) = &sig.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(predicate_type) = predicate
				&& let Type::Path(type_path) = &predicate_type.bounded_ty
				&& type_path.path.segments.len() == 1
			{
				let name = type_path.path.segments[0].ident.to_string();
				for bound in &predicate_type.bounds {
					if let TypeParamBound::Trait(trait_bound) = bound
						&& let Some(sig_ty) =
							get_fn_type(trait_bound, &fn_bounds, &generic_names, config)
					{
						fn_bounds.insert(name.clone(), sig_ty);
					}
				}
			}
		}
	}

	(generic_names, fn_bounds)
}

pub fn get_fn_type(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> Option<HMType> {
	let name = trait_bound.path.segments.last().unwrap().ident.to_string();
	match classify_trait(&name, config) {
		TraitCategory::FnTrait => {
			Some(trait_bound_to_hm_arrow(trait_bound, fn_bounds, generic_names, config))
		}
		TraitCategory::FnBrand => Some(HMType::Variable("fn_brand_marker".to_string())),
		_ => None,
	}
}

pub fn trait_bound_to_hm_arrow(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	let segment = trait_bound.path.segments.last().unwrap();
	if let PathArguments::Parenthesized(args) = &segment.arguments {
		let inputs: Vec<HMType> =
			args.inputs.iter().map(|t| type_to_hm(t, fn_bounds, generic_names, config)).collect();
		let output = match &args.output {
			ReturnType::Default => HMType::Unit,
			ReturnType::Type(_, ty) => type_to_hm(ty, fn_bounds, generic_names, config),
		};

		let input_ty = if inputs.is_empty() {
			HMType::Unit
		} else if inputs.len() == 1 {
			inputs[0].clone()
		} else {
			HMType::Tuple(inputs)
		};

		HMType::Arrow(Box::new(input_ty), Box::new(output))
	} else {
		HMType::Variable("fn".to_string())
	}
}

pub fn type_to_hm(
	ty: &Type,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	let mut visitor = HMTypeBuilder { fn_bounds, generic_names, config };
	visitor.visit(ty)
}

struct HMTypeBuilder<'a> {
	fn_bounds: &'a HashMap<String, HMType>,
	generic_names: &'a HashSet<String>,
	config: &'a Config,
}

impl<'a> TypeVisitor for HMTypeBuilder<'a> {
	type Output = HMType;

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) -> Self::Output {
		if let Some(type_path_inner) = &type_path.qself {
			if type_path.path.segments.len() >= 2 {
				let trait_name = type_path.path.segments[0].ident.to_string();
				if let TraitCategory::FnBrand = classify_trait(&trait_name, self.config) {
					let last_segment = type_path.path.segments.last().unwrap();
					if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
						let mut type_args = Vec::new();
						for arg in &args.args {
							if let GenericArgument::Type(inner_ty) = arg {
								type_args.push(self.visit(inner_ty));
							}
						}

						if !type_args.is_empty() {
							let output = type_args.pop().unwrap();
							let input = if type_args.is_empty() {
								HMType::Unit
							} else if type_args.len() == 1 {
								type_args[0].clone()
							} else {
								HMType::Tuple(type_args)
							};
							return HMType::Arrow(Box::new(input), Box::new(output));
						}
					}
				}
			}

			let constructor_type = self.visit(&type_path_inner.ty);
			let last_segment = type_path.path.segments.last().unwrap();

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
			if let Some(segment) = type_path.path.segments.last()
				&& segment.ident == "PhantomData"
			{
				return HMType::Unit;
			}

			if type_path.path.segments.len() >= 2 {
				let first = &type_path.path.segments[0];
				let last = type_path.path.segments.last().unwrap();

				let mut constructor_name = first.ident.to_string();
				if self.generic_names.contains(&constructor_name) || constructor_name == "Self" {
					constructor_name = constructor_name.to_lowercase();
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
			let segment = type_path.path.segments.last().unwrap();
			let name = segment.ident.to_string();

			if (name == "Box" || name == "Arc" || name == "Rc")
				&& let PathArguments::AngleBracketed(args) = &segment.arguments
				&& let Some(GenericArgument::Type(inner_ty)) = args.args.first()
			{
				return self.visit(inner_ty);
			}

			if let Some(sig) = self.fn_bounds.get(&name) {
				if let HMType::Variable(v) = sig
					&& v == "fn_brand_marker"
				{
					return HMType::Variable(name.to_lowercase());
				}
				return sig.clone();
			}

			if self.generic_names.contains(&name) {
				return HMType::Variable(name.to_lowercase());
			}

			if name == "Self" {
				return HMType::Variable("self".to_string());
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
		if type_macro.mac.path.is_ident("Apply")
			&& let Ok(apply_input) = syn::parse2::<ApplyInput>(type_macro.mac.tokens.clone())
		{
			let constructor_type = self.visit(&apply_input.brand);
			let mut type_args = Vec::new();
			for arg in &apply_input.args.args {
				if let syn::GenericArgument::Type(inner_ty) = arg {
					type_args.push(self.visit(inner_ty));
				}
			}

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

	fn visit_trait_object(
		&mut self,
		trait_object: &syn::TypeTraitObject,
	) -> Self::Output {
		trait_object_to_hm(trait_object, self.fn_bounds, self.generic_names, self.config)
	}

	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) -> Self::Output {
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
			tuple.elems.iter().filter(|t| !is_phantom_data(t)).map(|t| self.visit(t)).collect();
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

pub fn trait_object_to_hm(
	trait_object: &TypeTraitObject,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	for bound in &trait_object.bounds {
		if let TypeParamBound::Trait(trait_bound) = bound {
			return trait_bound_to_hm_type(trait_bound, fn_bounds, generic_names, config);
		}
	}
	HMType::Variable("dyn".to_string())
}

pub fn trait_bound_to_hm_type(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	let segment = trait_bound.path.segments.last().unwrap();
	let name = segment.ident.to_string();

	if name == "Fn" || name == "FnMut" || name == "FnOnce" {
		return trait_bound_to_hm_arrow(trait_bound, fn_bounds, generic_names, config);
	}

	let name = if generic_names.contains(&name) || name == "Self" {
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
			return HMType::Constructor(name, arg_types);
		}
	}

	HMType::Variable(name)
}

pub fn format_brand_name(
	name: &str,
	config: &Config,
) -> String {
	if let Some(mapping) = config.brand_mappings.get(name) {
		return mapping.clone();
	}

	if let Some(stripped) = name.strip_suffix("Brand") {
		stripped.to_string()
	} else {
		name.to_string()
	}
}

pub fn is_phantom_data(ty: &Type) -> bool {
	match ty {
		Type::Path(type_path) => {
			if let Some(segment) = type_path.path.segments.last() {
				return segment.ident == "PhantomData";
			}
			false
		}
		Type::Reference(type_ref) => is_phantom_data(&type_ref.elem),
		_ => false,
	}
}

#[derive(Clone, Debug)]
pub enum LogicalParam {
	Explicit(syn::Pat),
	#[allow(dead_code)]
	Implicit(syn::Type),
}

pub fn get_logical_params(
	sig: &syn::Signature,
	config: &Config,
) -> Vec<LogicalParam> {
	let mut params = Vec::new();

	// 1. Explicit arguments
	for input in &sig.inputs {
		match input {
			syn::FnArg::Receiver(_) => continue, // Skip self
			syn::FnArg::Typed(pat_type) => {
				if !is_phantom_data(&pat_type.ty) {
					params.push(LogicalParam::Explicit((*pat_type.pat).clone()));
				}
			}
		}
	}

	// 2. Curried arguments from return type
	extract_curried_params(&sig.output, &mut params, config);

	params
}

fn extract_curried_params(
	output: &ReturnType,
	params: &mut Vec<LogicalParam>,
	config: &Config,
) {
	if let ReturnType::Type(_, ty) = output {
		extract_from_type(ty, params, config);
	}
}

fn extract_from_type(
	ty: &Type,
	params: &mut Vec<LogicalParam>,
	config: &Config,
) {
	let mut visitor = CurriedParamExtractor { params, config };
	visitor.visit(ty);
}

struct CurriedParamExtractor<'a> {
	params: &'a mut Vec<LogicalParam>,
	config: &'a Config,
}

impl<'a> TypeVisitor for CurriedParamExtractor<'a> {
	type Output = ();

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) -> Self::Output {
		if type_path.qself.is_some() && type_path.path.segments.len() >= 2 {
			let trait_name = type_path.path.segments[0].ident.to_string();
			if let TraitCategory::FnBrand = classify_trait(&trait_name, self.config) {
				let last_segment = type_path.path.segments.last().unwrap();
				if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
					let mut type_args: Vec<_> =
						args.args
							.iter()
							.filter_map(|arg| {
								if let GenericArgument::Type(t) = arg { Some(t) } else { None }
							})
							.collect();

					if !type_args.is_empty() {
						let output_type = type_args.pop().unwrap();
						for arg_ty in type_args {
							self.params.push(LogicalParam::Implicit((*arg_ty).clone()));
						}
						self.visit(output_type);
					}
				}
			}
		}
	}

	fn visit_macro(
		&mut self,
		type_macro: &syn::TypeMacro,
	) -> Self::Output {
		if type_macro.mac.path.is_ident("Apply")
			&& let Ok(apply_input) = syn::parse2::<ApplyInput>(type_macro.mac.tokens.clone())
		{
			// We could handle currying here if Apply! projects to a function.
			// But for now we just skip.
			let _ = apply_input;
		}
	}

	fn visit_reference(
		&mut self,
		_type_ref: &syn::TypeReference,
	) -> Self::Output {
		// Do nothing
	}

	fn visit_impl_trait(
		&mut self,
		impl_trait: &syn::TypeImplTrait,
	) -> Self::Output {
		for bound in &impl_trait.bounds {
			if let TypeParamBound::Trait(trait_bound) = bound {
				self.visit_trait_bound_helper(trait_bound);
			}
		}
	}

	fn visit_trait_object(
		&mut self,
		trait_obj: &syn::TypeTraitObject,
	) -> Self::Output {
		for bound in &trait_obj.bounds {
			if let TypeParamBound::Trait(trait_bound) = bound {
				self.visit_trait_bound_helper(trait_bound);
			}
		}
	}

	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) -> Self::Output {
		for input in &bare_fn.inputs {
			self.params.push(LogicalParam::Implicit(input.ty.clone()));
		}
		if let ReturnType::Type(_, ty) = &bare_fn.output {
			self.visit(ty);
		}
	}

	fn visit_tuple(
		&mut self,
		_tuple: &syn::TypeTuple,
	) -> Self::Output {
	}
	fn visit_array(
		&mut self,
		_array: &syn::TypeArray,
	) -> Self::Output {
	}
	fn visit_slice(
		&mut self,
		_slice: &syn::TypeSlice,
	) -> Self::Output {
	}
	fn visit_other(
		&mut self,
		_ty: &syn::Type,
	) -> Self::Output {
	}
}

impl<'a> CurriedParamExtractor<'a> {
	fn visit_trait_bound_helper(
		&mut self,
		trait_bound: &TraitBound,
	) {
		let segment = trait_bound.path.segments.last().unwrap();
		let name = segment.ident.to_string();

		if let TraitCategory::FnTrait = classify_trait(&name, self.config)
			&& let PathArguments::Parenthesized(args) = &segment.arguments
		{
			for input in &args.inputs {
				self.params.push(LogicalParam::Implicit(input.clone()));
			}
			if let ReturnType::Type(_, ty) = &args.output {
				self.visit(ty);
			}
		}
	}
}
