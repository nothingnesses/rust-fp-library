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
#[allow(dead_code, reason = "Fields used in upcoming steps 7-8 for HM signature generation")]
pub struct DispatchTraitInfo {
	/// The dispatch trait name (e.g., "FunctorDispatch").
	pub trait_name: String,
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
}

/// Arrow type information extracted from a dispatch impl's Fn bound.
#[derive(Debug, Clone)]
pub struct DispatchArrow {
	/// The Fn bound's input parameter representations.
	pub inputs: Vec<DispatchArrowParam>,
	/// The Fn bound's output type as a token string.
	pub output: String,
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

/// Suffix used to identify dispatch traits by naming convention.
const DISPATCH_SUFFIX: &str = "Dispatch";

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
				if name.ends_with(DISPATCH_SUFFIX) {
					return Some(name);
				}
			}
			None
		})
		.collect();

	// For each dispatch trait, find its Val impl and extract info
	for trait_name in &dispatch_trait_names {
		if let Some(val_impl) = find_val_impl(items, trait_name) {
			let info = extract_dispatch_info(trait_name, val_impl);
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

/// Extract dispatch trait info from a Val impl block.
fn extract_dispatch_info(
	trait_name: &str,
	val_impl: &syn::ItemImpl,
) -> DispatchTraitInfo {
	let brand_param = find_brand_param(val_impl);
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

	DispatchTraitInfo {
		trait_name: trait_name.to_string(),
		semantic_constraint,
		secondary_constraints,
		arrow_type,
		tuple_closure,
	}
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
	if name.ends_with(DISPATCH_SUFFIX) {
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
	let mut last_output = String::new();

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
		ReturnType::Default => "()".to_string(),
		ReturnType::Type(_, ty) => simplify_type_for_hm(ty),
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

	format!("{} -> {}", input_str, arrow.output)
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
		assert_eq!(arrow.output, "B");
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
