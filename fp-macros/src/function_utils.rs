use crate::apply::ApplyInput;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use syn::{
	GenericArgument, PathArguments, ReturnType, TraitBound, Type, TypeParamBound, TypeTraitObject,
};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
	pub brand_mappings: HashMap<String, String>,
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

pub fn get_fn_signature(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> Option<String> {
	let name = trait_bound.path.segments.last().unwrap().ident.to_string();
	if name == "Fn" || name == "FnMut" || name == "FnOnce" {
		Some(format_fn_trait(trait_bound, fn_bounds, generic_names, config))
	} else if name == "SendCloneableFn" || name == "CloneableFn" || name == "Function" {
		Some("fn_brand_marker".to_string())
	} else {
		None
	}
}

pub fn format_fn_trait(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> String {
	let segment = trait_bound.path.segments.last().unwrap();
	if let PathArguments::Parenthesized(args) = &segment.arguments {
		let inputs: Vec<String> =
			args.inputs.iter().map(|t| format_type(t, fn_bounds, generic_names, config)).collect();
		let output = match &args.output {
			ReturnType::Default => "()".to_string(),
			ReturnType::Type(_, ty) => format_type(ty, fn_bounds, generic_names, config),
		};

		let input_str =
			if inputs.len() == 1 { inputs[0].clone() } else { format!("({})", inputs.join(", ")) };

		format!("{} -> {}", input_str, output)
	} else {
		"fn".to_string()
	}
}

pub fn format_type(
	ty: &Type,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> String {
	match ty {
		Type::Path(type_path) => {
			if let Some(type_path_inner) = &type_path.qself {
				if type_path.path.segments.len() >= 2 {
					let trait_name = type_path.path.segments[0].ident.to_string();
					if trait_name == "SendCloneableFn"
						|| trait_name == "CloneableFn"
						|| trait_name == "Function"
					{
						let last_segment = type_path.path.segments.last().unwrap();
						if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
							let mut type_args = Vec::new();
							for arg in &args.args {
								if let GenericArgument::Type(inner_ty) = arg {
									type_args.push(format_type(
										inner_ty,
										fn_bounds,
										generic_names,
										config,
									));
								}
							}

							if !type_args.is_empty() {
								let output = type_args.pop().unwrap();
								let input = if type_args.is_empty() {
									"()".to_string()
								} else if type_args.len() == 1 {
									type_args[0].clone()
								} else {
									format!("({})", type_args.join(", "))
								};
								return format!("{} -> {}", input, output);
							}
						}
					}
				}

				let constructor =
					format_type(&type_path_inner.ty, fn_bounds, generic_names, config);
				let last_segment = type_path.path.segments.last().unwrap();

				if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
					let mut type_args = Vec::new();
					for arg in &args.args {
						if let GenericArgument::Type(inner_ty) = arg {
							type_args.push(format_type_arg(
								inner_ty,
								fn_bounds,
								generic_names,
								config,
							));
						}
					}
					if !type_args.is_empty() {
						return format!("{} {}", constructor, type_args.join(" "));
					}
				}
				return constructor;
			}

			if let Some(segment) = type_path.path.segments.last()
				&& segment.ident == "PhantomData"
			{
				return "()".to_string();
			}

			if type_path.path.segments.len() >= 2 {
				let first = &type_path.path.segments[0];
				let last = type_path.path.segments.last().unwrap();

				let mut constructor = first.ident.to_string();
				if generic_names.contains(&constructor) || constructor == "Self" {
					constructor = constructor.to_lowercase();
				} else {
					constructor = format_brand_name(&constructor, config);
				}

				if let PathArguments::AngleBracketed(args) = &last.arguments {
					let mut type_args = Vec::new();
					for arg in &args.args {
						if let GenericArgument::Type(inner_ty) = arg {
							type_args.push(format_type_arg(
								inner_ty,
								fn_bounds,
								generic_names,
								config,
							));
						}
					}
					if !type_args.is_empty() {
						return format!("{} {}", constructor, type_args.join(" "));
					}
				}
			}

			let segment = type_path.path.segments.last().unwrap();
			let name = segment.ident.to_string();

			if (name == "Box" || name == "Arc" || name == "Rc")
				&& let PathArguments::AngleBracketed(args) = &segment.arguments
				&& let Some(GenericArgument::Type(inner_ty)) = args.args.first()
			{
				return format_type(inner_ty, fn_bounds, generic_names, config);
			}

			if let Some(sig) = fn_bounds.get(&name) {
				if sig == "fn_brand_marker" {
					return name.to_lowercase();
				}
				return sig.clone();
			}

			if generic_names.contains(&name) {
				return name.to_lowercase();
			}

			if name == "Self" {
				return "self".to_string();
			}

			let name = format_brand_name(&name, config);

			match &segment.arguments {
				PathArguments::AngleBracketed(args) => {
					let mut type_args = Vec::new();
					for arg in &args.args {
						if let GenericArgument::Type(inner_ty) = arg {
							type_args.push(format_type_arg(
								inner_ty,
								fn_bounds,
								generic_names,
								config,
							));
						}
					}
					if type_args.is_empty() {
						name
					} else {
						format!("{} {}", name, type_args.join(" "))
					}
				}
				_ => name,
			}
		}
		Type::ImplTrait(impl_trait) => {
			for bound in &impl_trait.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound {
					return format_trait_bound_as_type(
						trait_bound,
						fn_bounds,
						generic_names,
						config,
					);
				}
			}
			"impl_trait".to_string()
		}
		Type::Reference(type_ref) => format_type(&type_ref.elem, fn_bounds, generic_names, config),
		Type::Tuple(tuple) => {
			let types: Vec<String> = tuple
				.elems
				.iter()
				.filter(|t| !is_phantom_data(t))
				.map(|t| format_type(t, fn_bounds, generic_names, config))
				.collect();
			if types.is_empty() {
				"()".to_string()
			} else if types.len() == 1 {
				types[0].clone()
			} else {
				format!("({})", types.join(", "))
			}
		}
		Type::Array(array) => {
			let inner = format_type(&array.elem, fn_bounds, generic_names, config);
			format!("[{}]", inner)
		}
		Type::Slice(slice) => {
			let inner = format_type(&slice.elem, fn_bounds, generic_names, config);
			format!("[{}]", inner)
		}
		Type::TraitObject(type_trait_object) => {
			format_trait_object(type_trait_object, fn_bounds, generic_names, config)
		}
		Type::BareFn(bare_fn) => {
			let inputs: Vec<String> = bare_fn
				.inputs
				.iter()
				.map(|arg| format_type(&arg.ty, fn_bounds, generic_names, config))
				.collect();
			let output = match &bare_fn.output {
				ReturnType::Default => "()".to_string(),
				ReturnType::Type(_, ty) => format_type(ty, fn_bounds, generic_names, config),
			};
			let input_str = if inputs.len() == 1 {
				inputs[0].clone()
			} else {
				format!("({})", inputs.join(", "))
			};
			format!("{} -> {}", input_str, output)
		}
		Type::Macro(type_macro) => {
			if type_macro.mac.path.is_ident("Apply")
				&& let Ok(apply_input) = syn::parse2::<ApplyInput>(type_macro.mac.tokens.clone())
			{
				let constructor = format_type(&apply_input.brand, fn_bounds, generic_names, config);
				let mut type_args = Vec::new();
				for arg in &apply_input.args.args {
					if let syn::GenericArgument::Type(inner_ty) = arg {
						type_args.push(format_type_arg(inner_ty, fn_bounds, generic_names, config));
					}
				}
				if !type_args.is_empty() {
					return format!("{} {}", constructor, type_args.join(" "));
				}
				return constructor;
			}
			"macro".to_string()
		}
		_ => "_".to_string(),
	}
}

pub fn format_type_arg(
	ty: &Type,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> String {
	let s = format_type(ty, fn_bounds, generic_names, config);

	if !s.contains(' ') {
		return s;
	}

	if s.starts_with('(') && s.ends_with(')') {
		let mut depth = 0;
		let mut fully_wrapped = true;
		for (i, c) in s.chars().enumerate() {
			if c == '(' {
				depth += 1;
			} else if c == ')' {
				depth -= 1;
				if depth == 0 && i < s.len() - 1 {
					fully_wrapped = false;
					break;
				}
			}
		}
		if fully_wrapped {
			return s;
		}
	}

	format!("({})", s)
}

pub fn format_trait_object(
	trait_object: &TypeTraitObject,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> String {
	for bound in &trait_object.bounds {
		if let TypeParamBound::Trait(trait_bound) = bound {
			return format_trait_bound_as_type(trait_bound, fn_bounds, generic_names, config);
		}
	}
	"dyn".to_string()
}

pub fn format_trait_bound_as_type(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> String {
	let segment = trait_bound.path.segments.last().unwrap();
	let name = segment.ident.to_string();

	if name == "Fn" || name == "FnMut" || name == "FnOnce" {
		return format_fn_trait(trait_bound, fn_bounds, generic_names, config);
	}

	let name = if generic_names.contains(&name) || name == "Self" {
		name.to_lowercase()
	} else {
		format_brand_name(&name, config)
	};

	if let PathArguments::AngleBracketed(args) = &segment.arguments {
		let mut arg_strs = Vec::new();
		for arg in &args.args {
			match arg {
				GenericArgument::Type(ty) => {
					arg_strs.push(format_type_arg(ty, fn_bounds, generic_names, config));
				}
				GenericArgument::AssocType(assoc) => {
					arg_strs.push(format_type_arg(&assoc.ty, fn_bounds, generic_names, config));
				}
				_ => {}
			}
		}
		if !arg_strs.is_empty() {
			return format!("{} {}", name, arg_strs.join(" "));
		}
	}

	name
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
	input_fn: &syn::ItemFn,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> Vec<LogicalParam> {
	let mut params = Vec::new();

	// 1. Explicit arguments
	for input in &input_fn.sig.inputs {
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
	extract_curried_params(&input_fn.sig.output, &mut params, fn_bounds, generic_names, config);

	params
}

fn extract_curried_params(
	output: &ReturnType,
	params: &mut Vec<LogicalParam>,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) {
	if let ReturnType::Type(_, ty) = output {
		extract_from_type(ty, params, fn_bounds, generic_names, config);
	}
}

fn extract_from_type(
	ty: &Type,
	params: &mut Vec<LogicalParam>,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) {
	match ty {
		Type::ImplTrait(impl_trait) => {
			for bound in &impl_trait.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound {
					extract_from_trait_bound(trait_bound, params, fn_bounds, generic_names, config);
				}
			}
		}
		Type::TraitObject(trait_obj) => {
			for bound in &trait_obj.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound {
					extract_from_trait_bound(trait_bound, params, fn_bounds, generic_names, config);
				}
			}
		}
		Type::Path(type_path) => {
			if type_path.qself.is_some() && type_path.path.segments.len() >= 2 {
				let trait_name = type_path.path.segments[0].ident.to_string();
				if trait_name == "SendCloneableFn"
					|| trait_name == "CloneableFn"
					|| trait_name == "Function"
				{
					let last_segment = type_path.path.segments.last().unwrap();
					if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
						let mut type_args: Vec<_> = args
							.args
							.iter()
							.filter_map(|arg| {
								if let GenericArgument::Type(t) = arg { Some(t) } else { None }
							})
							.collect();

						if !type_args.is_empty() {
							let output_type = type_args.pop().unwrap();
							for arg_ty in type_args {
								params.push(LogicalParam::Implicit((*arg_ty).clone()));
							}
							extract_from_type(
								output_type,
								params,
								fn_bounds,
								generic_names,
								config,
							);
						}
					}
				}
			}
		}
		Type::BareFn(bare_fn) => {
			for input in &bare_fn.inputs {
				params.push(LogicalParam::Implicit(input.ty.clone()));
			}
			if let ReturnType::Type(_, ty) = &bare_fn.output {
				extract_from_type(ty, params, fn_bounds, generic_names, config);
			}
		}
		Type::Macro(type_macro) => {
			if type_macro.mac.path.is_ident("Apply")
				&& let Ok(apply_input) = syn::parse2::<ApplyInput>(type_macro.mac.tokens.clone())
			{
				// We could handle currying here if Apply! projects to a function.
				// But for now we just skip.
				let _ = apply_input;
			}
		}
		_ => {}
	}
}

fn extract_from_trait_bound(
	trait_bound: &TraitBound,
	params: &mut Vec<LogicalParam>,
	fn_bounds: &HashMap<String, String>,
	generic_names: &HashSet<String>,
	config: &Config,
) {
	let segment = trait_bound.path.segments.last().unwrap();
	let name = segment.ident.to_string();

	if (name == "Fn" || name == "FnMut" || name == "FnOnce")
		&& let PathArguments::Parenthesized(args) = &segment.arguments
	{
		for input in &args.inputs {
			params.push(LogicalParam::Implicit(input.clone()));
		}
		if let ReturnType::Type(_, ty) = &args.output {
			extract_from_type(ty, params, fn_bounds, generic_names, config);
		}
	}
}
