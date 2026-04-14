//! Dispatch trait analysis for HM signature generation.
//!
//! Extracts semantic information (type class constraints, arrow types) from
//! dispatch trait impl blocks. This information is used by `#[document_module]`
//! to generate correct HM signatures for inference wrapper functions.

use {
	crate::{
		analysis::patterns::get_apply_macro_parameters,
		core::constants::markers,
		hkt::{
			ApplyInput,
			apply::apply_worker,
		},
	},
	std::collections::HashMap,
	syn::{
		GenericArgument,
		ImplItem,
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
	/// Container param mapping: maps trait type param names to their element
	/// types and position in the trait's generic param list.
	/// E.g., for lift2: [ContainerParam { name: "FA", position: 4, elements: ["A"] }].
	/// Derived from the dispatch trait's generic parameter positions.
	pub container_params: Vec<ContainerParam>,
	/// Associated type definitions from the Val impl block.
	/// Maps associated type names to their element types extracted from Apply!.
	/// E.g., for ApplyFirstDispatch: [("FB", ["B"])].
	pub associated_types: Vec<(String, Vec<String>)>,
	/// Element types extracted from the Val impl's self type when it is an Apply! macro.
	/// Used for closureless dispatch where the container IS the self type.
	/// E.g., for SeparateDispatch: Some(["Result<O,E>"]).
	pub self_type_elements: Option<Vec<String>>,
	/// Semantic type parameter names from the trait definition, in declaration order.
	/// Excludes FnBrand, Marker, and multi-letter container params (FA, FB, etc.).
	/// Used to order the forall type variables in the HM signature.
	/// E.g., for TraverseDispatch: ["Brand", "A", "B", "F"].
	pub type_param_order: Vec<String>,
}

/// A container type parameter from the dispatch trait definition.
#[derive(Debug, Clone)]
pub struct ContainerParam {
	/// The trait param name (e.g., "FA", "FB").
	pub name: String,
	/// Position index among type params in the trait definition (excluding lifetimes).
	pub position: usize,
	/// The element types extracted from the Apply! macro in the Val impl
	/// (e.g., ["A"] for FA, ["B"] for FB).
	pub element_types: Vec<String>,
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
	/// Returns a tuple of brand applications (e.g., `(F A, F B)` for partition/separate).
	Tuple(Vec<Vec<String>>),
	/// Returns a nested application containing a tuple of brand applications
	/// (e.g., `M (F E, F O)` for wilt).
	NestedTuple { outer_param: String, inner_elements: Vec<Vec<String>> },
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
	/// A sub-arrow from a tuple closure (e.g., one of the Fn bounds in bimap's (F, G)).
	SubArrow(DispatchArrow),
}

// -- Constants --

/// Trait names that are infrastructure, not semantic type class constraints.
const INFRASTRUCTURE_TRAITS: &[&str] =
	&["Send", "Sync", "Clone", "Copy", "Debug", "Display", "Sized", "LiftFn", "SendLiftFn"];

// -- Apply! macro parsing helpers --

/// Extract type argument names from an Apply! macro invocation.
///
/// Uses `get_apply_macro_parameters` (the proper token-stream parser) to
/// extract the type args from `Apply!(<Brand as Kind!(...)>::Of<'a, A, B>)`.
/// Returns the type arg names as strings (e.g., `["A", "B"]`).
fn extract_apply_type_args(ty: &Type) -> Option<Vec<String>> {
	let Type::Macro(type_macro) = ty else {
		return None;
	};
	let (_brand, args) = get_apply_macro_parameters(type_macro)?;
	Some(args.iter().map(|t| quote::quote!(#t).to_string().replace(' ', "")).collect())
}

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
	// Prefer the trait definition for brand_param (direct source: the param with a Kind_* bound).
	// Fall back to scanning the Val impl's where clause (indirect: finds param with semantic bound).
	let brand_param =
		trait_def.and_then(find_brand_param_from_trait_def).or_else(|| find_brand_param(val_impl));
	let kind_trait_name = trait_def.and_then(|td| {
		extract_kind_trait_name(td, brand_param.as_deref().unwrap_or(markers::DEFAULT_BRAND_PARAM))
	});
	let semantic_constraint =
		brand_param.as_ref().and_then(|bp| extract_semantic_constraint(val_impl, bp));
	let tuple_closure = is_tuple_closure(val_impl);

	let arrow_type = if tuple_closure {
		extract_tuple_arrow(val_impl, brand_param.as_deref())
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

	let container_params =
		trait_def.map(|td| extract_container_params(td, val_impl)).unwrap_or_default();

	let associated_types = extract_associated_types(val_impl);
	let self_type_elements = extract_self_type_elements(val_impl);
	let type_param_order =
		trait_def.map(|td| extract_type_param_order(td, &container_params)).unwrap_or_default();

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
		associated_types,
		self_type_elements,
		type_param_order,
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

/// Extract container parameter mappings from the Val impl block.
///
/// Aligns the trait definition's type param names with the Val impl's
/// trait type arguments. Container params are those whose corresponding
/// impl argument is an `Apply!` macro invocation. The element types
/// are extracted from the `Of<'a, ElementType>` pattern in the Apply.
fn extract_container_params(
	trait_def: &syn::ItemTrait,
	val_impl: &syn::ItemImpl,
) -> Vec<ContainerParam> {
	// Get trait type param names (excluding lifetimes)
	let trait_params: Vec<String> = trait_def
		.generics
		.params
		.iter()
		.filter_map(|p| {
			if let syn::GenericParam::Type(tp) = p { Some(tp.ident.to_string()) } else { None }
		})
		.collect();

	// Get Val impl's trait type arguments (excluding lifetimes and the marker)
	let Some((_, trait_path, _)) = &val_impl.trait_ else {
		return Vec::new();
	};
	let Some(last_seg) = trait_path.segments.last() else {
		return Vec::new();
	};
	let PathArguments::AngleBracketed(impl_args) = &last_seg.arguments else {
		return Vec::new();
	};

	// Collect type arguments (skip lifetime args)
	let impl_type_args: Vec<&Type> = impl_args
		.args
		.iter()
		.filter_map(|arg| if let GenericArgument::Type(ty) = arg { Some(ty) } else { None })
		.collect();

	// Align: trait_params[i] corresponds to impl_type_args[i]
	// (both lists exclude lifetimes, so they should align)
	let mut result = Vec::new();

	for (i, param_name) in trait_params.iter().enumerate() {
		// Skip Brand, FnBrand, Marker, and single-letter element types
		if param_name == markers::DEFAULT_BRAND_PARAM
			|| param_name == markers::FN_BRAND_PARAM
			|| param_name == markers::MARKER_PARAM
			|| param_name.len() == 1
		{
			continue;
		}

		// Check if the corresponding impl arg is an Apply! macro (container type)
		if let Some(impl_arg) = impl_type_args.get(i)
			&& let Some(element_types) = extract_apply_type_args(impl_arg)
			&& !element_types.is_empty()
		{
			result.push(ContainerParam {
				name: param_name.clone(),
				position: i,
				element_types,
			});
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

		return Some(classify_return_type(return_ty, brand_param));
	}

	None
}

/// Classify a return type into a ReturnStructure using the proper Apply! parser.
fn classify_return_type(
	ty: &Type,
	brand_param: Option<&str>,
) -> ReturnStructure {
	// Tuple return (e.g., partition, separate)
	if let Type::Tuple(tuple) = ty {
		let mut tuple_elements = Vec::new();
		for elem in &tuple.elems {
			if let Some(args) = extract_apply_type_args(elem) {
				tuple_elements.push(args);
			} else {
				// Non-Apply element in tuple; treat as plain
				let elem_str = quote::quote!(#elem).to_string().replace(' ', "");
				tuple_elements.push(vec![elem_str]);
			}
		}
		if !tuple_elements.is_empty() {
			return ReturnStructure::Tuple(tuple_elements);
		}
	}

	// Apply! macro return
	if let Type::Macro(type_macro) = ty
		&& let Some((brand, raw_args)) = get_apply_macro_parameters(type_macro)
	{
		let args: Vec<String> =
			raw_args.iter().map(|t| quote::quote!(#t).to_string().replace(' ', "")).collect();
		let brand_name = match &brand {
			Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()),
			_ => None,
		};

		// Check if brand is the same as the Brand param (simple application)
		// or a different param (nested application)
		let is_brand = brand_param.is_some_and(|bp| brand_name.as_deref() == Some(bp));

		if is_brand {
			// Simple: Brand applied to type args (e.g., Brand B, Brand B D)
			return ReturnStructure::Applied(args);
		}

		// Nested: outer brand is different from Brand (e.g., F (Brand B))
		let outer_name = brand_name.unwrap_or_else(|| "G".to_string());

		// Check if the inner arg is a tuple of Apply! types (e.g., wilt returns M (Brand E, Brand O))
		if let [single_arg] = raw_args.as_slice()
			&& let Type::Tuple(tuple) = single_arg
			&& tuple.elems.len() >= 2
		{
			let mut inner_elements = Vec::new();
			for elem in &tuple.elems {
				if let Some(nested_args) = extract_apply_type_args(elem) {
					inner_elements.push(nested_args);
				} else {
					let s = quote::quote!(#elem).to_string().replace(' ', "");
					inner_elements.push(vec![s]);
				}
			}
			return ReturnStructure::NestedTuple {
				outer_param: outer_name,
				inner_elements,
			};
		}

		// The inner args are the type args of the outer Apply, which may
		// themselves be Apply! macros. Extract the innermost Brand application.
		let mut inner_args = Vec::new();
		for raw_arg in &raw_args {
			if let Some(nested_args) = extract_apply_type_args(raw_arg) {
				inner_args = nested_args;
			} else {
				let arg_str = quote::quote!(#raw_arg).to_string().replace(' ', "");
				inner_args.push(arg_str);
			}
		}
		return ReturnStructure::Nested {
			outer_param: outer_name,
			inner_args,
		};
	}

	// Not a macro or tuple; treat as plain type
	let ret_str = quote::quote!(#ty).to_string().replace(' ', "");
	ReturnStructure::Plain(ret_str)
}

/// Find the Brand type parameter from the trait definition by looking for
/// the type param with a `Kind_*` bound. This is the direct/authoritative source.
fn find_brand_param_from_trait_def(trait_def: &syn::ItemTrait) -> Option<String> {
	// Check inline bounds on generic params
	for param in &trait_def.generics.params {
		if let syn::GenericParam::Type(type_param) = param {
			for bound in &type_param.bounds {
				if let syn::TypeParamBound::Trait(trait_bound) = bound
					&& trait_bound
						.path
						.segments
						.last()
						.is_some_and(|s| s.ident.to_string().starts_with(markers::KIND_PREFIX))
				{
					return Some(type_param.ident.to_string());
				}
			}
		}
	}

	// Check where clause
	if let Some(where_clause) = &trait_def.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(pred_type) = predicate {
				for bound in &pred_type.bounds {
					if let syn::TypeParamBound::Trait(trait_bound) = bound
						&& trait_bound
							.path
							.segments
							.last()
							.is_some_and(|s| s.ident.to_string().starts_with(markers::KIND_PREFIX))
					{
						return Some(type_to_string(&pred_type.bounded_ty));
					}
				}
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
fn extract_tuple_arrow(
	val_impl: &syn::ItemImpl,
	brand_param: Option<&str>,
) -> Option<DispatchArrow> {
	let mut all_inputs = Vec::new();
	let mut last_output = ArrowOutput::Plain("()".to_string());

	if let Some(where_clause) = &val_impl.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let WherePredicate::Type(pred_type) = predicate {
				for bound in &pred_type.bounds {
					if let Some(arrow) = extract_fn_arrow_from_bound(bound, brand_param) {
						last_output = arrow.output.clone();
						all_inputs.push(DispatchArrowParam::SubArrow(arrow));
					}
				}
			}
		}
	}

	// Also check inline bounds
	for param in &val_impl.generics.params {
		if let syn::GenericParam::Type(type_param) = param {
			for bound in &type_param.bounds {
				if let Some(arrow) = extract_fn_arrow_from_bound(bound, brand_param) {
					last_output = arrow.output.clone();
					all_inputs.push(DispatchArrowParam::SubArrow(arrow));
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
	// Use the proper Apply! parser for macro types
	if let Type::Macro(type_macro) = ty
		&& let Some((brand, args)) = get_apply_macro_parameters(type_macro)
	{
		let brand_name = match &brand {
			Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()),
			_ => None,
		};

		let arg_strings: Vec<String> =
			args.iter().map(|t| quote::quote!(#t).to_string().replace(' ', "")).collect();

		if !arg_strings.is_empty() {
			let is_brand = brand_param.is_some_and(|bp| brand_name.as_deref() == Some(bp));
			if is_brand {
				return ArrowOutput::BrandApplied(arg_strings);
			}
			if let Some(name) = brand_name {
				return ArrowOutput::OtherApplied {
					brand: name,
					args: arg_strings,
				};
			}
		}
	}

	// Plain type: store as valid Rust token text (not HM-simplified)
	// so it can be parsed back to syn::Type in the synthetic signature builder
	ArrowOutput::Plain(quote::quote!(#ty).to_string())
}

/// Extract the semantic type param names from the trait definition, in declaration order.
///
/// Filters out lifetimes, FnBrand, Marker, and multi-letter container params (those
/// that appear in `container_params`). The result preserves the trait author's intended
/// ordering for the HM forall clause.
fn extract_type_param_order(
	trait_def: &syn::ItemTrait,
	container_params: &[ContainerParam],
) -> Vec<String> {
	let container_names: Vec<&str> = container_params.iter().map(|cp| cp.name.as_str()).collect();

	trait_def
		.generics
		.params
		.iter()
		.filter_map(|p| {
			if let syn::GenericParam::Type(tp) = p {
				let name = tp.ident.to_string();
				// Skip infrastructure params
				if name == markers::FN_BRAND_PARAM || name == markers::MARKER_PARAM {
					return None;
				}
				// Skip container params (multi-letter params that map to Apply! types)
				if container_names.contains(&name.as_str()) {
					return None;
				}
				Some(name)
			} else {
				None
			}
		})
		.collect()
}

/// Extract associated type definitions from a Val impl block.
///
/// Finds `type FB = Apply!(<Brand as Kind>::Of<'a, B>)` items and extracts
/// the associated type name and element types from the Apply! macro.
fn extract_associated_types(val_impl: &syn::ItemImpl) -> Vec<(String, Vec<String>)> {
	let mut result = Vec::new();
	for item in &val_impl.items {
		if let ImplItem::Type(type_item) = item {
			let name = type_item.ident.to_string();
			if let Some(args) = extract_apply_type_args(&type_item.ty) {
				result.push((name, args));
			}
		}
	}
	result
}

/// Extract element types from the Val impl's self type when it is an Apply! macro.
///
/// For closureless dispatch where the trait is implemented on the container type
/// (e.g., `impl SeparateDispatch<...> for Apply!(<Brand as Kind>::Of<'a, Result<O, E>>)`),
/// the self type's Apply! args give the correct container element types.
///
/// Inner Apply! macros are resolved to their expanded qualified path form
/// (e.g., `Apply!(<OptionBrand as Kind!(...)>::Of<'a, A>)` becomes
/// `<OptionBrand as Kind_hash>::Of<'a, A>`). This allows the HM pipeline to
/// simplify them (e.g., to `Option A`).
fn extract_self_type_elements(val_impl: &syn::ItemImpl) -> Option<Vec<String>> {
	let Type::Macro(type_macro) = &*val_impl.self_ty else {
		return None;
	};
	let (_brand, args) = get_apply_macro_parameters(type_macro)?;
	Some(
		args.iter()
			.map(|t| {
				// If the arg is itself an Apply! macro, resolve it to a qualified path
				if let Type::Macro(inner_macro) = t
					&& let Ok(apply_input) =
						syn::parse2::<ApplyInput>(inner_macro.mac.tokens.clone())
					&& let Ok(resolved) = apply_worker(apply_input)
				{
					return resolved.to_string();
				}
				quote::quote!(#t).to_string().replace(' ', "")
			})
			.collect(),
	)
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

	#[test]
	fn test_container_params_with_apply() {
		let items = make_items(
			r#"
			trait FunctorDispatch<'a, Brand: Kind_cdc7cd43dac7585f, A: 'a, B: 'a, FA, Marker> {
				fn dispatch(self, fa: FA) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
			}
			impl<'a, Brand, A, B, F>
				FunctorDispatch<
					'a,
					Brand,
					A,
					B,
					Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
					Val,
				> for F
			where
				Brand: Functor,
				A: 'a,
				B: 'a,
				F: Fn(A) -> B + 'a,
			{
				fn dispatch(self, fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) { todo!() }
			}
			struct Val;
			"#,
		);

		let result = analyze_dispatch_traits(&items);
		let info = result.get("FunctorDispatch").unwrap();

		// Container params should map FA -> ["A"] (not ["A", "B"])
		assert_eq!(
			info.container_params.len(),
			1,
			"Expected 1 container param, got {:?}",
			info.container_params
		);
		assert_eq!(info.container_params[0].name, "FA");
		assert_eq!(info.container_params[0].element_types, vec!["A".to_string()]);

		// Return structure should be Applied(["B"])
		assert!(
			matches!(info.return_structure, ReturnStructure::Applied(ref args) if args == &["B"]),
			"Expected Applied([B]), got {:?}",
			info.return_structure
		);
	}
}
