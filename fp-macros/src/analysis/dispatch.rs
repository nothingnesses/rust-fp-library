//! Dispatch trait analysis for HM signature generation.
//!
//! Extracts semantic information (type class constraints, arrow types) from
//! dispatch trait impl blocks. This information is used by `#[document_module]`
//! to generate correct HM signatures for inference wrapper functions.

use {
	crate::core::constants::markers,
	std::collections::HashMap,
	syn::{
		GenericArgument,
		Item,
		PathArguments,
		ReturnType,
		Type,
		WherePredicate,
	},
};

// -- Data structures --

/// Information extracted from a dispatch trait's Val impl block.
#[derive(Debug, Clone)]
pub struct DispatchTraitInfo {
	/// The dispatch trait name (e.g., "FunctorDispatch").
	#[expect(dead_code, reason = "Stored for diagnostics")]
	pub trait_name: String,
	/// The Brand type parameter name (e.g., "Brand").
	pub brand_param: String,
	/// The Kind trait name from the Brand param's bound (e.g., "Kind_cdc7cd43dac7585f").
	/// None if the Kind hash was not found directly on the Brand parameter.
	pub kind_trait_name: Option<String>,
	/// The primary semantic type class constraint (e.g., "Functor").
	/// Extracted from `Brand: Functor` in the Val impl's where clause.
	pub semantic_constraint: Option<String>,
	/// Secondary constraints on other type params that should appear
	/// in the HM signature (e.g., ("M", "Applicative") for traverse).
	pub secondary_constraints: Vec<(String, String)>,
	/// Arrow type extracted from the Fn bound, or None for closureless.
	pub arrow_type: Option<DispatchArrow>,
	/// Whether the closure is a tuple (bimap, bi_fold, etc.).
	pub tuple_closure: bool,
	/// Return type structure of the dispatch method.
	pub return_structure: ReturnStructure,
	/// Container param mapping: maps function type params to their element
	/// types. E.g., for lift2: [("FA", "A"), ("FB", "B")]. Derived from
	/// the dispatch trait's generic parameter positions.
	pub container_params: Vec<(String, Vec<String>)>,
}

/// Describes the HM return type structure of a dispatch trait.
#[derive(Debug, Clone)]
pub enum ReturnStructure {
	/// Returns a simple type variable (e.g., `B` for fold_right, `M` for fold_map).
	Plain(String),
	/// Returns the brand applied to type args (e.g., `F B` for map, `F B D` for bimap).
	Applied(Vec<String>),
	/// Returns a nested application (e.g., `G (F B)` for traverse, `M (F B)` for wither).
	Nested { outer_param: String, inner_args: Vec<String> },
}

/// Arrow type information extracted from a dispatch impl's Fn bound.
#[derive(Debug, Clone)]
pub struct DispatchArrow {
	/// The Fn bound's input parameter representations.
	pub inputs: Vec<DispatchArrowParam>,
	/// The Fn bound's output type, classified structurally.
	pub output: ArrowOutput,
}

/// Structured representation of an arrow's output type.
#[derive(Debug, Clone)]
pub enum ArrowOutput {
	/// A plain type variable (e.g., `B`, `bool`, `Option B`).
	Plain(String),
	/// The Brand applied to type args (e.g., `Apply!(<Brand as Kind>::Of<B>)` -> `Brand B`).
	BrandApplied(Vec<String>),
	/// Another type param applied to type args (e.g., `Apply!(<F as Kind>::Of<B>)` -> `F B`).
	OtherApplied { brand: String, args: Vec<String> },
}

/// A single parameter in a dispatch arrow type.
#[derive(Debug, Clone)]
pub enum DispatchArrowParam {
	/// A simple type parameter (e.g., "A").
	TypeParam(String),
	/// An associated type on the Brand (e.g., Brand::Index -> "Index").
	AssociatedType { assoc_name: String },
}

// -- Constants --

/// Trait names that are infrastructure, not semantic type class constraints.
const INFRASTRUCTURE_TRAITS: &[&str] =
	&["Send", "Sync", "Clone", "Copy", "Debug", "Display", "Sized", "LiftFn", "SendLiftFn"];

// -- Analysis entry point --

/// Analyze all dispatch traits in a module's items.
///
/// Scans for traits ending with "Dispatch", finds their Val impl blocks,
/// and extracts semantic constraints and arrow types.
pub fn analyze_dispatch_traits(items: &[Item]) -> HashMap<String, DispatchTraitInfo> {
	let mut result = HashMap::new();

	// Find all dispatch trait names
	let dispatch_trait_names: Vec<String> = items
		.iter()
		.filter_map(|item| {
			if let Item::Trait(item_trait) = item {
				let name = item_trait.ident.to_string();
				if name.ends_with(markers::DISPATCH_SUFFIX) {
					return Some(name);
				}
			}
			None
		})
		.collect();

	// For each dispatch trait, find its Val impl and trait definition, then extract info
	for trait_name in &dispatch_trait_names {
		if let Some(val_impl) = find_val_impl(items, trait_name) {
			let trait_def = items.iter().find_map(|item| {
				if let Item::Trait(item_trait) = item
					&& item_trait.ident == trait_name.as_str()
				{
					return Some(item_trait);
				}
				None
			});
			let info = extract_dispatch_info(trait_name, val_impl, trait_def);
			result.insert(trait_name.clone(), info);
		}
	}

	result
}

// -- Impl block finding --

/// Find the Val impl block for a given dispatch trait.
///
/// Scans all trait type arguments for "Val" by name, not by position.
fn find_val_impl<'a>(
	items: &'a [Item],
	trait_name: &str,
) -> Option<&'a syn::ItemImpl> {
	items.iter().find_map(|item| {
		if let Item::Impl(item_impl) = item {
			// Check if this impl is for the right trait
			if let Some((_, trait_path, _)) = &item_impl.trait_ {
				let impl_trait_name =
					trait_path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
				if impl_trait_name == trait_name {
					// Check if any type argument is "Val"
					if has_marker_type_arg(trait_path, "Val") {
						return Some(item_impl);
					}
				}
			}
		}
		None
	})
}

/// Check if a trait path contains a type argument matching the given marker name.
fn has_marker_type_arg(
	path: &syn::Path,
	marker_name: &str,
) -> bool {
	let Some(last_segment) = path.segments.last() else {
		return false;
	};
	let PathArguments::AngleBracketed(args) = &last_segment.arguments else {
		return false;
	};
	args.args.iter().any(|arg| {
		if let GenericArgument::Type(Type::Path(type_path)) = arg {
			type_path.path.get_ident().is_some_and(|ident| ident == marker_name)
		} else {
			false
		}
	})
}

// -- Info extraction --

/// Extract dispatch trait info from a Val impl block and the trait definition.
fn extract_dispatch_info(
	trait_name: &str,
	val_impl: &syn::ItemImpl,
	trait_def: Option<&syn::ItemTrait>,
) -> DispatchTraitInfo {
	let brand_param = find_brand_param(val_impl);
	let kind_trait_name = trait_def.and_then(|td| {
		extract_kind_trait_name(td, brand_param.as_deref().unwrap_or(markers::DEFAULT_BRAND_PARAM))
	});
	let semantic_constraint =
		brand_param.as_ref().and_then(|bp| extract_semantic_constraint(val_impl, bp));
	let tuple_closure = is_tuple_closure(val_impl);

	let arrow_type = if tuple_closure {
		extract_tuple_arrow(val_impl)
	} else {
		extract_single_arrow(val_impl, brand_param.as_deref())
	};

	let secondary_constraints = brand_param
		.as_ref()
		.map(|bp| extract_secondary_constraints(val_impl, bp))
		.unwrap_or_default();

	let return_structure = trait_def
		.and_then(|td| extract_return_structure(td, brand_param.as_deref()))
		.unwrap_or(ReturnStructure::Plain("?".to_string()));

	let container_params = trait_def
		.map(|td| {
			extract_container_params(
				td,
				brand_param.as_deref().unwrap_or(markers::DEFAULT_BRAND_PARAM),
			)
		})
		.unwrap_or_default();

	DispatchTraitInfo {
		trait_name: trait_name.to_string(),
		brand_param: brand_param.unwrap_or_else(|| markers::DEFAULT_BRAND_PARAM.to_string()),
		kind_trait_name,
		semantic_constraint,
		secondary_constraints,
		arrow_type,
		tuple_closure,
		return_structure,
		container_params,
	}
}

/// Extract the Kind trait name from a dispatch trait definition's Brand param bounds.
///
/// Scans both inline bounds and where clause for a `Kind_*` prefixed trait.
fn extract_kind_trait_name(
	trait_def: &syn::ItemTrait,
	brand_param_name: &str,
) -> Option<String> {
	// Check inline bounds on generic params
	for param in &trait_def.generics.params {
		if let syn::GenericParam::Type(type_param) = param
			&& type_param.ident == brand_param_name
		{
			for bound in &type_param.bounds {
				if let syn::TypeParamBound::Trait(trait_bound) = bound {
					let name = trait_bound
						.path
						.segments
						.last()
						.map(|s| s.ident.to_string())
						.unwrap_or_default();
					if name.starts_with(markers::KIND_PREFIX) {
						return Some(name);
					}
				}
			}
		}
	}

	// Check where clause
	if let Some(where_clause) = &trait_def.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(pred_type) = predicate {
				let param_name = type_to_string(&pred_type.bounded_ty);
				if param_name == brand_param_name {
					for bound in &pred_type.bounds {
						if let syn::TypeParamBound::Trait(trait_bound) = bound {
							let name = trait_bound
								.path
								.segments
								.last()
								.map(|s| s.ident.to_string())
								.unwrap_or_default();
							if name.starts_with(markers::KIND_PREFIX) {
								return Some(name);
							}
						}
					}
				}
			}
		}
	}

	None
}

/// Extract container parameter mappings from a dispatch trait definition.
///
/// Uses position-based analysis of the dispatch trait's generic params.
/// Container params are those that appear after the element type params
/// and before the Marker param, and whose names are not `Brand`, `Marker`,
/// `FnBrand`, or single-letter element types.
fn extract_container_params(
	trait_def: &syn::ItemTrait,
	brand_param_name: &str,
) -> Vec<(String, Vec<String>)> {
	let params: Vec<String> = trait_def
		.generics
		.params
		.iter()
		.filter_map(|p| {
			if let syn::GenericParam::Type(tp) = p { Some(tp.ident.to_string()) } else { None }
		})
		.collect();

	// The dispatch trait param order is typically:
	// Brand, [FnBrand], ElementTypes..., ContainerTypes..., Marker
	// Where element types are single letters (A, B, C, etc.)
	// and container types start with F (FA, FB, FC, etc.)

	// Find the Brand and Marker positions
	let brand_pos = params.iter().position(|p| p == brand_param_name);
	let marker_pos = params.iter().position(|p| p == "Marker");

	let Some(bp) = brand_pos else {
		return Vec::new();
	};
	let end = marker_pos.unwrap_or(params.len());

	// Params between Brand+1 and Marker, skipping FnBrand and single-letter element types
	let mut element_types = Vec::new();
	let mut container_mappings = Vec::new();

	for param in params.iter().take(end).skip(bp + 1) {
		if param == markers::FN_BRAND_PARAM || param == markers::MARKER_PARAM {
			continue;
		}
		// Heuristic: container params have multi-char names starting with uppercase
		// that contain at least one letter after the first
		// Element types are single uppercase letters or well-known names like M
		if param.len() == 1 || param == markers::FN_BRAND_PARAM {
			element_types.push(param.clone());
		} else {
			// This is a container param. Map it to its element types by
			// looking at the naming convention: FA -> [A], FTA -> [A]
			// For bifunctor: the element types come from the trait's type args
			// For now, use position: the Nth container param maps to the Nth
			// subset of element types based on the brand's arity.
			container_mappings.push(param.clone());
		}
	}

	// Map each container to its element type(s) by position
	// For arity-1: FA maps to element_types[0], FB to element_types[1], etc.
	// For arity-2: FA maps to (element_types[0], element_types[1]), etc.
	// Determine arity from the number of element types per container
	let arity = if container_mappings.is_empty() {
		1
	} else {
		// If there are more element types than containers, it's likely arity > 1
		let ratio = element_types.len().checked_div(container_mappings.len()).unwrap_or(1);
		ratio.max(1)
	};

	let mut result = Vec::new();
	for (i, container) in container_mappings.iter().enumerate() {
		let start = i * arity;
		let end = (start + arity).min(element_types.len());
		let mapped_elements: Vec<String> =
			element_types.get(start .. end).unwrap_or_default().to_vec();
		if !mapped_elements.is_empty() {
			result.push((container.clone(), mapped_elements));
		}
	}

	result
}

/// Extract the return type structure from the dispatch trait's `dispatch` method.
fn extract_return_structure(
	trait_def: &syn::ItemTrait,
	brand_param: Option<&str>,
) -> Option<ReturnStructure> {
	// Find the dispatch method
	for item in &trait_def.items {
		let syn::TraitItem::Fn(method) = item else {
			continue;
		};
		if method.sig.ident != "dispatch" {
			continue;
		}

		let syn::ReturnType::Type(_, return_ty) = &method.sig.output else {
			return Some(ReturnStructure::Plain("()".to_string()));
		};

		let ret_str = quote::quote!(#return_ty).to_string();

		// Check if it's a simple type (no Apply!, no Kind_)
		if !ret_str.contains("Apply") && !ret_str.contains("Kind_") {
			let clean = ret_str.replace(' ', "");
			return Some(ReturnStructure::Plain(clean));
		}

		// Count Of< applications in the trait's return type.
		// The trait definition uses Brand directly, not InferableBrand, so
		// counting Of< is reliable here.
		let of_count = ret_str.matches("Of <").count() + ret_str.matches("Of<").count();

		if of_count >= 2 {
			// Nested: outer brand is a different param than Brand
			let outer = extract_non_brand_param(&ret_str, brand_param);
			let inner_args = extract_last_of_type_args_clean(&ret_str);
			return Some(ReturnStructure::Nested {
				outer_param: outer.unwrap_or_else(|| "G".to_string()),
				inner_args,
			});
		}

		// Simple Apply: Brand applied to type args
		let type_args = extract_last_of_type_args_clean(&ret_str);
		return Some(ReturnStructure::Applied(type_args));
	}

	None
}

/// Extract type args from the last Of<...> in a clean trait return type string.
fn extract_last_of_type_args_clean(ret_str: &str) -> Vec<String> {
	let Some(of_pos) = ret_str.rfind("Of") else {
		return Vec::new();
	};
	let after_of = &ret_str[of_pos ..];

	let Some(start) = after_of.find('<') else {
		return Vec::new();
	};
	let inner = &after_of[start + 1 ..];

	let mut depth = 1;
	let mut end = 0;
	for (i, c) in inner.char_indices() {
		match c {
			'<' | '(' => depth += 1,
			'>' | ')' => {
				depth -= 1;
				if depth == 0 {
					end = i;
					break;
				}
			}
			_ => {}
		}
	}

	let args_str = &inner[.. end];
	args_str
		.split(',')
		.map(|s| s.trim().to_string())
		.filter(|s| !s.is_empty() && !s.starts_with('\''))
		.collect()
}

/// Extract a non-Brand type parameter from a return type string.
fn extract_non_brand_param(
	ret_str: &str,
	brand_param: Option<&str>,
) -> Option<String> {
	for part in ret_str.split("as") {
		let trimmed = part.trim();
		if let Some(bracket_pos) = trimmed.rfind('<') {
			let candidate = trimmed[bracket_pos + 1 ..].trim();
			if candidate.chars().all(|c| c.is_alphanumeric() || c == '_')
				&& !candidate.is_empty()
				&& !candidate.starts_with("Kind")
				&& brand_param.is_none_or(|bp| candidate != bp)
			{
				return Some(candidate.to_string());
			}
		}
	}
	None
}

/// Find the Brand type parameter name by looking for a non-infrastructure,
/// non-Fn, non-Kind trait bound on a type parameter.
fn find_brand_param(val_impl: &syn::ItemImpl) -> Option<String> {
	let Some(where_clause) = &val_impl.generics.where_clause else {
		return None;
	};

	for predicate in &where_clause.predicates {
		if let WherePredicate::Type(pred_type) = predicate {
			let param_name = type_to_string(&pred_type.bounded_ty);

			// Skip lifetime-only bounds (e.g., A: 'a)
			let has_trait_bound =
				pred_type.bounds.iter().any(|b| matches!(b, syn::TypeParamBound::Trait(_)));
			if !has_trait_bound {
				continue;
			}

			// Check if any bound is a semantic type class (not Fn, not Kind, not infrastructure)
			for bound in &pred_type.bounds {
				if let syn::TypeParamBound::Trait(trait_bound) = bound {
					let bound_name = trait_bound
						.path
						.segments
						.last()
						.map(|s| s.ident.to_string())
						.unwrap_or_default();

					if is_semantic_type_class(&bound_name) {
						return Some(param_name);
					}
				}
			}
		}
	}

	// Also check inline bounds on generic params
	for param in &val_impl.generics.params {
		if let syn::GenericParam::Type(type_param) = param {
			let param_name = type_param.ident.to_string();
			for bound in &type_param.bounds {
				if let syn::TypeParamBound::Trait(trait_bound) = bound {
					let bound_name = trait_bound
						.path
						.segments
						.last()
						.map(|s| s.ident.to_string())
						.unwrap_or_default();

					if is_semantic_type_class(&bound_name) {
						return Some(param_name);
					}
				}
			}
		}
	}

	None
}

/// Check if a trait name represents a semantic type class constraint
/// (as opposed to infrastructure like Fn, Kind, Send, etc.).
fn is_semantic_type_class(name: &str) -> bool {
	// Not a Fn trait
	if name == "Fn" || name == "FnMut" || name == "FnOnce" {
		return false;
	}
	// Not a Kind trait
	if name.starts_with(markers::KIND_PREFIX) {
		return false;
	}
	// Not an InferableBrand trait
	if name.starts_with(markers::INFERABLE_BRAND_PREFIX) {
		return false;
	}
	// Not infrastructure
	if INFRASTRUCTURE_TRAITS.contains(&name) {
		return false;
	}
	// Not a dispatch trait (avoid self-referential detection)
	if name.ends_with(markers::DISPATCH_SUFFIX) {
		return false;
	}
	true
}

/// Extract the primary semantic constraint from the Brand parameter.
fn extract_semantic_constraint(
	val_impl: &syn::ItemImpl,
	brand_param: &str,
) -> Option<String> {
	// Check where clause
	if let Some(where_clause) = &val_impl.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(pred_type) = predicate {
				let param_name = type_to_string(&pred_type.bounded_ty);
				if param_name == brand_param {
					for bound in &pred_type.bounds {
						if let syn::TypeParamBound::Trait(trait_bound) = bound {
							let name = trait_bound
								.path
								.segments
								.last()
								.map(|s| s.ident.to_string())
								.unwrap_or_default();
							if is_semantic_type_class(&name) {
								return Some(name);
							}
						}
					}
				}
			}
		}
	}

	// Check inline bounds
	for param in &val_impl.generics.params {
		if let syn::GenericParam::Type(type_param) = param
			&& type_param.ident == brand_param
		{
			for bound in &type_param.bounds {
				if let syn::TypeParamBound::Trait(trait_bound) = bound {
					let name = trait_bound
						.path
						.segments
						.last()
						.map(|s| s.ident.to_string())
						.unwrap_or_default();
					if is_semantic_type_class(&name) {
						return Some(name);
					}
				}
			}
		}
	}

	None
}

/// Extract secondary constraints (non-Brand, non-closure, non-infrastructure).
fn extract_secondary_constraints(
	val_impl: &syn::ItemImpl,
	brand_param: &str,
) -> Vec<(String, String)> {
	let mut result = Vec::new();

	if let Some(where_clause) = &val_impl.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(pred_type) = predicate {
				let param_name = type_to_string(&pred_type.bounded_ty);

				// Skip the Brand param itself
				if param_name == brand_param {
					continue;
				}

				for bound in &pred_type.bounds {
					if let syn::TypeParamBound::Trait(trait_bound) = bound {
						let name = trait_bound
							.path
							.segments
							.last()
							.map(|s| s.ident.to_string())
							.unwrap_or_default();

						// Only include semantic type classes
						if is_semantic_type_class(&name) && !is_fn_like(&name) {
							result.push((param_name.clone(), name));
						}
					}
				}
			}
		}
	}

	result
}

/// Check if the impl is for a multi-element tuple type (e.g., `for (F, G)`).
/// Unit tuples `()` and single-element tuples are not tuple closures.
fn is_tuple_closure(val_impl: &syn::ItemImpl) -> bool {
	if let Type::Tuple(tuple) = &*val_impl.self_ty { tuple.elems.len() >= 2 } else { false }
}

/// Extract arrow type from a single-closure dispatch impl.
fn extract_single_arrow(
	val_impl: &syn::ItemImpl,
	brand_param: Option<&str>,
) -> Option<DispatchArrow> {
	// Search where clause for Fn bounds
	if let Some(where_clause) = &val_impl.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(pred_type) = predicate {
				for bound in &pred_type.bounds {
					if let Some(arrow) = extract_fn_arrow_from_bound(bound, brand_param) {
						return Some(arrow);
					}
				}
			}
		}
	}

	// Search inline bounds on generic params
	for param in &val_impl.generics.params {
		if let syn::GenericParam::Type(type_param) = param {
			for bound in &type_param.bounds {
				if let Some(arrow) = extract_fn_arrow_from_bound(bound, brand_param) {
					return Some(arrow);
				}
			}
		}
	}

	None
}

/// Extract arrow types from a tuple-closure dispatch impl (e.g., bimap with (F, G)).
fn extract_tuple_arrow(val_impl: &syn::ItemImpl) -> Option<DispatchArrow> {
	let mut all_inputs = Vec::new();
	let mut last_output = ArrowOutput::Plain("()".to_string());

	if let Some(where_clause) = &val_impl.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(pred_type) = predicate {
				for bound in &pred_type.bounds {
					if let Some(arrow) = extract_fn_arrow_from_bound(bound, None) {
						// For tuple closures, each sub-arrow becomes an input
						let sub_arrow_str = format_arrow_as_string(&arrow);
						all_inputs.push(DispatchArrowParam::TypeParam(sub_arrow_str));
						last_output = arrow.output.clone();
					}
				}
			}
		}
	}

	// Also check inline bounds
	for param in &val_impl.generics.params {
		if let syn::GenericParam::Type(type_param) = param {
			for bound in &type_param.bounds {
				if let Some(arrow) = extract_fn_arrow_from_bound(bound, None) {
					let sub_arrow_str = format_arrow_as_string(&arrow);
					all_inputs.push(DispatchArrowParam::TypeParam(sub_arrow_str));
					last_output = arrow.output.clone();
				}
			}
		}
	}

	if all_inputs.is_empty() {
		None
	} else {
		Some(DispatchArrow {
			inputs: all_inputs,
			output: last_output,
		})
	}
}

/// Extract a DispatchArrow from a single trait bound if it's a Fn trait.
fn extract_fn_arrow_from_bound(
	bound: &syn::TypeParamBound,
	brand_param: Option<&str>,
) -> Option<DispatchArrow> {
	let syn::TypeParamBound::Trait(trait_bound) = bound else {
		return None;
	};

	let segment = trait_bound.path.segments.last()?;
	let name = segment.ident.to_string();

	if name != "Fn" && name != "FnMut" && name != "FnOnce" {
		return None;
	}

	let PathArguments::Parenthesized(args) = &segment.arguments else {
		return None;
	};

	let inputs: Vec<DispatchArrowParam> =
		args.inputs.iter().map(|ty| type_to_arrow_param(ty, brand_param)).collect();

	let output = match &args.output {
		ReturnType::Default => ArrowOutput::Plain("()".to_string()),
		ReturnType::Type(_, ty) => classify_arrow_output(ty, brand_param),
	};

	Some(DispatchArrow {
		inputs,
		output,
	})
}

/// Convert a type to a DispatchArrowParam, detecting associated types like Brand::Index.
fn type_to_arrow_param(
	ty: &Type,
	brand_param: Option<&str>,
) -> DispatchArrowParam {
	if let Type::Path(type_path) = ty {
		// Check for Brand::Index pattern (two-segment path)
		let segments: Vec<_> = type_path.path.segments.iter().collect();
		if let [first_seg, second_seg] = segments.as_slice() {
			let first = first_seg.ident.to_string();
			let second = second_seg.ident.to_string();
			if brand_param.is_some_and(|bp| bp == first) {
				return DispatchArrowParam::AssociatedType {
					assoc_name: second,
				};
			}
		}
	}
	DispatchArrowParam::TypeParam(type_to_string(ty))
}

/// Simplify a type for HM rendering. Strips lifetimes, Apply! macros, etc.
/// Classify an arrow output type as plain, brand-applied, or other-brand-applied.
fn classify_arrow_output(
	ty: &Type,
	brand_param: Option<&str>,
) -> ArrowOutput {
	let type_str = quote::quote!(#ty).to_string();

	// Check if this is a macro (Apply!) invocation
	if let Type::Macro(_) = ty {
		// The output contains Apply! macro. Check if it references Brand or another param.
		if let Some(bp) = brand_param
			&& type_str.contains(bp)
		{
			// Brand-applied: e.g., Apply!(<Brand as Kind!(...)>::Of<'a, B>)
			// Extract the type args from the last Of<>
			let args = extract_last_of_type_args_clean(&type_str);
			if !args.is_empty() {
				return ArrowOutput::BrandApplied(args);
			}
		}
		// Check for other-brand patterns (e.g., <F as Kind!(...)>::Of<'a, B>)
		if let Some(other_brand) = extract_non_brand_param(&type_str, brand_param) {
			let args = extract_last_of_type_args_clean(&type_str);
			if !args.is_empty() {
				return ArrowOutput::OtherApplied {
					brand: other_brand,
					args,
				};
			}
		}
	}

	// Plain type
	ArrowOutput::Plain(simplify_type_for_hm(ty))
}

fn simplify_type_for_hm(ty: &Type) -> String {
	match ty {
		Type::Path(type_path) => {
			let name =
				type_path.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
			// Check for Option<X> and Result<X, Y> patterns
			if let Some(last) = type_path.path.segments.last()
				&& let PathArguments::AngleBracketed(args) = &last.arguments
			{
				let type_args: Vec<String> = args
					.args
					.iter()
					.filter_map(|a| {
						if let GenericArgument::Type(t) = a {
							Some(simplify_type_for_hm(t))
						} else {
							None
						}
					})
					.collect();
				if type_args.is_empty() {
					return name;
				}
				return format!("{} {}", name, type_args.join(" "));
			}
			name
		}
		Type::Tuple(tuple) => {
			let elems: Vec<String> = tuple.elems.iter().map(simplify_type_for_hm).collect();
			format!("({})", elems.join(", "))
		}
		Type::Reference(reference) => {
			let inner = simplify_type_for_hm(&reference.elem);
			format!("&{inner}")
		}
		_ => quote::quote!(#ty).to_string(),
	}
}

/// Format a DispatchArrow as a string for embedding in tuple closures.
fn format_arrow_as_string(arrow: &DispatchArrow) -> String {
	let inputs: Vec<String> = arrow
		.inputs
		.iter()
		.map(|p| match p {
			DispatchArrowParam::TypeParam(s) => s.clone(),
			DispatchArrowParam::AssociatedType {
				assoc_name,
			} => assoc_name.clone(),
		})
		.collect();

	let input_str = if inputs.len() == 1 {
		inputs.first().cloned().unwrap_or_default()
	} else {
		format!("({})", inputs.join(", "))
	};

	let output_str = match &arrow.output {
		ArrowOutput::Plain(s) => s.clone(),
		ArrowOutput::BrandApplied(args) => format!("Brand {}", args.join(" ")),
		ArrowOutput::OtherApplied {
			brand,
			args,
		} => format!("{brand} {}", args.join(" ")),
	};

	format!("{input_str} -> {output_str}")
}

// -- Helpers --

/// Check if a name looks like a Fn-like trait (not a type class).
fn is_fn_like(name: &str) -> bool {
	name == "Fn" || name == "FnMut" || name == "FnOnce"
}

/// Convert a Type to its string representation.
fn type_to_string(ty: &Type) -> String {
	quote::quote!(#ty).to_string().replace(' ', "")
}

#[cfg(test)]
#[expect(
	clippy::unwrap_used,
	clippy::expect_used,
	clippy::indexing_slicing,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use super::*;

	fn make_items(code: &str) -> Vec<Item> {
		let file: syn::File = syn::parse_str(code).expect("Failed to parse test code");
		file.items
	}

	#[test]
	fn test_simple_dispatch_trait() {
		let items = make_items(
			r#"
			trait FunctorDispatch<'a, Brand, A, B, FA, Marker> {
				fn dispatch(self, fa: FA) -> ();
			}
			impl<'a, Brand, A, B, F> FunctorDispatch<'a, Brand, A, B, (), Val> for F
			where
				Brand: Functor,
				A: 'a,
				B: 'a,
				F: Fn(A) -> B + 'a,
			{
				fn dispatch(self, fa: ()) -> () {}
			}
			struct Val;
			"#,
		);

		let result = analyze_dispatch_traits(&items);
		assert_eq!(result.len(), 1);

		let info = result.get("FunctorDispatch").unwrap();
		assert_eq!(info.semantic_constraint.as_deref(), Some("Functor"));
		assert!(info.arrow_type.is_some());
		assert!(!info.tuple_closure);

		let arrow = info.arrow_type.as_ref().unwrap();
		assert_eq!(arrow.inputs.len(), 1);
		assert!(matches!(arrow.output, ArrowOutput::Plain(ref s) if s == "B"));
	}

	#[test]
	fn test_closureless_dispatch() {
		let items = make_items(
			r#"
			trait AltDispatch<'a, Brand, A, Marker> {
				fn dispatch(self, other: Self) -> ();
			}
			impl<'a, Brand, A> AltDispatch<'a, Brand, A, Val> for ()
			where
				Brand: Alt,
				A: 'a,
			{
				fn dispatch(self, other: Self) -> () {}
			}
			struct Val;
			"#,
		);

		let result = analyze_dispatch_traits(&items);
		let info = result.get("AltDispatch").unwrap();
		assert_eq!(info.semantic_constraint.as_deref(), Some("Alt"));
		assert!(info.arrow_type.is_none());
		assert!(!info.tuple_closure);
	}

	#[test]
	fn test_tuple_closure_dispatch() {
		let items = make_items(
			r#"
			trait BimapDispatch<'a, Brand, A, B, C, D, FA, Marker> {
				fn dispatch(self, fa: FA) -> ();
			}
			impl<'a, Brand, A, B, C, D, F, G>
				BimapDispatch<'a, Brand, A, B, C, D, (), Val> for (F, G)
			where
				Brand: Bifunctor,
				A: 'a,
				B: 'a,
				C: 'a,
				D: 'a,
				F: Fn(A) -> B + 'a,
				G: Fn(C) -> D + 'a,
			{
				fn dispatch(self, fa: ()) -> () {}
			}
			struct Val;
			"#,
		);

		let result = analyze_dispatch_traits(&items);
		let info = result.get("BimapDispatch").unwrap();
		assert_eq!(info.semantic_constraint.as_deref(), Some("Bifunctor"));
		assert!(info.arrow_type.is_some());
		assert!(info.tuple_closure);

		let arrow = info.arrow_type.as_ref().unwrap();
		// Tuple closure: each sub-arrow becomes an input
		assert_eq!(arrow.inputs.len(), 2);
	}

	#[test]
	fn test_secondary_constraints() {
		let items = make_items(
			r#"
			trait TraverseDispatch<'a, FnBrand, Brand, A, B, F, FA, Marker> {
				fn dispatch(self, fa: FA) -> ();
			}
			impl<'a, FnBrand, Brand, A, B, F, Func>
				TraverseDispatch<'a, FnBrand, Brand, A, B, F, (), Val> for Func
			where
				Brand: Traversable,
				A: 'a,
				B: 'a,
				F: Applicative,
				Func: Fn(A) -> () + 'a,
			{
				fn dispatch(self, fa: ()) -> () {}
			}
			struct Val;
			"#,
		);

		let result = analyze_dispatch_traits(&items);
		let info = result.get("TraverseDispatch").unwrap();
		assert_eq!(info.semantic_constraint.as_deref(), Some("Traversable"));
		assert_eq!(info.secondary_constraints.len(), 1);
		assert_eq!(info.secondary_constraints[0], ("F".to_string(), "Applicative".to_string()));
	}
}
