//! Detection of named generic type parameters that could be `impl Trait`.
//!
//! This lint identifies function signatures where a named generic type parameter:
//! 1. Has trait bounds (not just lifetime bounds)
//! 2. Appears in exactly one parameter position
//! 3. Does not appear in the return type
//! 4. Is not cross-referenced by other type parameters' bounds

use {
	crate::analysis::patterns::get_apply_macro_parameters,
	syn::{
		GenericParam,
		Signature,
		Type,
		TypeParamBound,
	},
};

/// A named generic type parameter that could be replaced with `impl Trait`.
pub struct ImplTraitCandidate {
	/// The name of the type parameter (e.g., `"F"`).
	pub param_name: String,
	/// The span of the type parameter declaration.
	pub param_span: proc_macro2::Span,
	/// Display string of the combined bounds (e.g., `"Fn(A) -> B + 'a"`).
	pub bounds_display: String,
}

/// Find all named generic type parameters that could be replaced with `impl Trait`.
pub fn find_impl_trait_candidates(sig: &Signature) -> Vec<ImplTraitCandidate> {
	let mut candidates = Vec::new();

	for param in &sig.generics.params {
		let GenericParam::Type(type_param) = param else {
			continue;
		};

		let name = type_param.ident.to_string();

		// 1. Collect all bounds (inline + where clause). Skip if only lifetime bounds.
		let all_bounds = collect_all_bounds(type_param, &name, &sig.generics.where_clause);
		if !has_trait_bounds(&all_bounds) {
			continue;
		}

		// 2. Appears exactly once in a top-level parameter position (where impl Trait is valid).
		if !appears_once_at_top_level(sig, &name) {
			continue;
		}

		// 3. Absent from return type.
		if appears_in_return_type(sig, &name) {
			continue;
		}

		// 4. Not cross-referenced by other type parameters' bounds.
		if is_cross_referenced(sig, &name) {
			continue;
		}

		// 5. Not self-referential in bounds.
		// If the param's own bounds reference itself (e.g.,
		// `FA: JoinDispatch<..., <FA as InferableBrand<...>>::Marker>`), it cannot use
		// `impl Trait` because `impl Trait` cannot reference itself by name.
		if is_self_referential_in_bounds(&all_bounds, &name) {
			continue;
		}

		// 6. Not used in where-clause projections (e.g., `<FA as Trait>::Assoc: Bound`).
		// `impl Trait` cannot be referenced by name, so any where-clause predicate
		// whose `bounded_ty` contains the param inside a projection disqualifies it.
		if appears_in_where_clause_projection(sig, &name) {
			continue;
		}

		let bounds_display = format_bounds(&all_bounds);
		candidates.push(ImplTraitCandidate {
			param_name: name,
			param_span: type_param.ident.span(),
			bounds_display,
		});
	}

	candidates
}

/// Collect all bounds for a type parameter from both inline and where clause.
fn collect_all_bounds<'a>(
	type_param: &'a syn::TypeParam,
	name: &str,
	where_clause: &'a Option<syn::WhereClause>,
) -> Vec<&'a TypeParamBound> {
	let mut bounds: Vec<&TypeParamBound> = type_param.bounds.iter().collect();

	if let Some(wc) = where_clause {
		for pred in &wc.predicates {
			if let syn::WherePredicate::Type(pred_type) = pred
				&& type_is_ident(&pred_type.bounded_ty, name)
			{
				bounds.extend(pred_type.bounds.iter());
			}
		}
	}

	bounds
}

/// Check whether a set of bounds contains at least one trait bound (not just lifetimes).
fn has_trait_bounds(bounds: &[&TypeParamBound]) -> bool {
	bounds.iter().any(|b| matches!(b, TypeParamBound::Trait(_)))
}

/// Check if the type parameter appears in exactly one parameter at a top-level position
/// where `impl Trait` substitution would be syntactically valid.
///
/// `impl Trait` can only appear as the entire type of a function parameter (or behind
/// `&`/`&mut`). It cannot be nested inside generic types like `Option<F>` or `Apply!(...)`.
// SAFETY: matching.len() == 1 checked on the return expression
#[expect(clippy::indexing_slicing, reason = "matching.len() == 1 checked on return")]
fn appears_once_at_top_level(
	sig: &Signature,
	name: &str,
) -> bool {
	let matching: Vec<_> = sig
		.inputs
		.iter()
		.filter_map(|arg| if let syn::FnArg::Typed(pat_type) = arg { Some(pat_type) } else { None })
		.filter(|pat_type| contains_type_param(&pat_type.ty, name))
		.collect();

	matching.len() == 1 && is_top_level_type_param(&matching[0].ty, name)
}

/// Check if a type IS the named type parameter at the top level.
///
/// Returns true for positions where `impl Trait` substitution is syntactically valid:
/// `F`, `&F`, `&mut F`, `(F)`.
fn is_top_level_type_param(
	ty: &Type,
	name: &str,
) -> bool {
	match ty {
		Type::Path(type_path) => type_path.qself.is_none() && type_path.path.is_ident(name),
		Type::Reference(type_ref) => is_top_level_type_param(&type_ref.elem, name),
		Type::Paren(type_paren) => is_top_level_type_param(&type_paren.elem, name),
		Type::Group(type_group) => is_top_level_type_param(&type_group.elem, name),
		_ => false,
	}
}

/// Check if a type parameter appears in the return type.
fn appears_in_return_type(
	sig: &Signature,
	name: &str,
) -> bool {
	match &sig.output {
		syn::ReturnType::Default => false,
		syn::ReturnType::Type(_, ty) => contains_type_param(ty, name),
	}
}

/// Check if any other type parameter's bounds reference this name.
fn is_cross_referenced(
	sig: &Signature,
	name: &str,
) -> bool {
	for param in &sig.generics.params {
		let GenericParam::Type(other_param) = param else {
			continue;
		};
		if other_param.ident == name {
			continue;
		}

		// Check inline bounds of other params
		if bounds_contain_type_param(other_param.bounds.iter(), name) {
			return true;
		}
	}

	// Check where clause predicates for other params
	if let Some(wc) = &sig.generics.where_clause {
		for pred in &wc.predicates {
			if let syn::WherePredicate::Type(pred_type) = pred
				// Only check bounds of predicates that are NOT for our param
				&& !type_is_ident(&pred_type.bounded_ty, name)
				&& bounds_contain_type_param(pred_type.bounds.iter(), name)
			{
				return true;
			}
		}
	}

	false
}

/// Check if the type parameter appears inside its own bounds.
///
/// When a param's bounds reference itself (e.g.,
/// `FA: JoinDispatch<..., <FA as InferableBrand<...>>::Marker>`), the param cannot use
/// `impl Trait` because `impl Trait` cannot reference itself by name.
fn is_self_referential_in_bounds(
	bounds: &[&TypeParamBound],
	name: &str,
) -> bool {
	for bound in bounds {
		if let TypeParamBound::Trait(trait_bound) = bound
			&& trait_bound_contains_type_param(trait_bound, name)
		{
			return true;
		}
	}
	false
}

/// Check if the type parameter appears inside a where-clause projection's subject.
///
/// When a predicate's `bounded_ty` is a projection like `<FA as Trait>::Assoc`,
/// the type param `FA` cannot use `impl Trait` because `impl Trait` cannot be
/// referenced by name in where clauses. This catches predicates where the param
/// appears inside `bounded_ty` but is not the bare ident itself (those are
/// already handled as simple bound collection in `collect_all_bounds`).
fn appears_in_where_clause_projection(
	sig: &Signature,
	name: &str,
) -> bool {
	if let Some(wc) = &sig.generics.where_clause {
		for pred in &wc.predicates {
			if let syn::WherePredicate::Type(pred_type) = pred
				// Skip predicates where the bounded_ty IS the bare param (e.g., `FA: Trait`).
				// Those are normal bounds, not projections.
				&& !type_is_ident(&pred_type.bounded_ty, name)
				// Check if the param appears inside the bounded_ty (e.g., `<FA as Trait>::Assoc`)
				&& contains_type_param(&pred_type.bounded_ty, name)
			{
				return true;
			}
		}
	}
	false
}

/// Check if a sequence of bounds contains a reference to the named type parameter.
fn bounds_contain_type_param<'a>(
	bounds: impl Iterator<Item = &'a TypeParamBound>,
	name: &str,
) -> bool {
	for bound in bounds {
		if let TypeParamBound::Trait(trait_bound) = bound
			&& trait_bound_contains_type_param(trait_bound, name)
		{
			return true;
		}
	}
	false
}

/// Format bounds as a display string.
fn format_bounds(bounds: &[&TypeParamBound]) -> String {
	use quote::ToTokens;
	bounds.iter().map(|b| b.to_token_stream().to_string()).collect::<Vec<_>>().join(" + ")
}

/// Check if a `syn::Type` is a simple identifier matching `name`.
fn type_is_ident(
	ty: &Type,
	name: &str,
) -> bool {
	if let Type::Path(type_path) = ty
		&& type_path.qself.is_none()
		&& type_path.path.is_ident(name)
	{
		return true;
	}
	false
}

/// Recursively check if a type contains a reference to the named type parameter.
///
/// Walks through all `syn::Type` variants including:
/// - `Type::Path` - checks ident and recurses into generic args
/// - `Type::Macro` - parses `Apply!` macros and recurses into args
/// - `Type::Reference` - recurses into element
/// - `Type::Tuple` - recurses into each element
/// - `Type::ImplTrait` / `Type::TraitObject` - recurses into bounds
/// - `Type::BareFn` - recurses into inputs and output
/// - `Type::Array` / `Type::Slice` - recurses into element
/// - `Type::Paren` / `Type::Group` - recurses into inner type
pub fn contains_type_param(
	ty: &Type,
	name: &str,
) -> bool {
	match ty {
		Type::Path(type_path) => {
			// Check if this path IS the type parameter
			if type_path.qself.is_none() && type_path.path.is_ident(name) {
				return true;
			}
			// Check qself
			if let Some(qself) = &type_path.qself
				&& contains_type_param(&qself.ty, name)
			{
				return true;
			}
			// Recurse into generic arguments
			for segment in &type_path.path.segments {
				if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
					for arg in &args.args {
						match arg {
							syn::GenericArgument::Type(inner_ty) => {
								if contains_type_param(inner_ty, name) {
									return true;
								}
							}
							syn::GenericArgument::AssocType(assoc) => {
								if contains_type_param(&assoc.ty, name) {
									return true;
								}
							}
							_ => {}
						}
					}
				}
			}
			false
		}
		Type::Macro(type_macro) => {
			// Parse Apply! macros and recurse into brand AND type arguments
			if let Some((brand, args)) = get_apply_macro_parameters(type_macro) {
				if contains_type_param(&brand, name) {
					return true;
				}
				for arg_ty in &args {
					if contains_type_param(arg_ty, name) {
						return true;
					}
				}
			}
			false
		}
		Type::Reference(type_ref) => contains_type_param(&type_ref.elem, name),
		Type::Tuple(type_tuple) =>
			type_tuple.elems.iter().any(|elem| contains_type_param(elem, name)),
		Type::ImplTrait(type_impl) => type_impl.bounds.iter().any(|bound| {
			if let TypeParamBound::Trait(trait_bound) = bound {
				trait_bound_contains_type_param(trait_bound, name)
			} else {
				false
			}
		}),
		Type::TraitObject(type_obj) => type_obj.bounds.iter().any(|bound| {
			if let TypeParamBound::Trait(trait_bound) = bound {
				trait_bound_contains_type_param(trait_bound, name)
			} else {
				false
			}
		}),
		Type::BareFn(type_fn) => {
			for input in &type_fn.inputs {
				if contains_type_param(&input.ty, name) {
					return true;
				}
			}
			if let syn::ReturnType::Type(_, ret_ty) = &type_fn.output
				&& contains_type_param(ret_ty, name)
			{
				return true;
			}
			false
		}
		Type::Array(type_array) => contains_type_param(&type_array.elem, name),
		Type::Slice(type_slice) => contains_type_param(&type_slice.elem, name),
		Type::Paren(type_paren) => contains_type_param(&type_paren.elem, name),
		Type::Group(type_group) => contains_type_param(&type_group.elem, name),
		_ => false,
	}
}

/// Check if a trait bound contains a reference to the named type parameter.
fn trait_bound_contains_type_param(
	trait_bound: &syn::TraitBound,
	name: &str,
) -> bool {
	for segment in &trait_bound.path.segments {
		if segment.ident == name {
			return true;
		}
		if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
			for arg in &args.args {
				match arg {
					syn::GenericArgument::Type(ty) =>
						if contains_type_param(ty, name) {
							return true;
						},
					syn::GenericArgument::AssocType(assoc) => {
						if contains_type_param(&assoc.ty, name) {
							return true;
						}
					}
					_ => {}
				}
			}
		}
		if let syn::PathArguments::Parenthesized(args) = &segment.arguments {
			for input in &args.inputs {
				if contains_type_param(input, name) {
					return true;
				}
			}
			if let syn::ReturnType::Type(_, ret_ty) = &args.output
				&& contains_type_param(ret_ty, name)
			{
				return true;
			}
		}
	}
	false
}

#[cfg(test)]
#[expect(
	clippy::unwrap_used,
	clippy::indexing_slicing,
	reason = "Tests use panicking operations for brevity and clarity"
)]
mod tests {
	use {
		super::*,
		syn::parse_str,
	};

	// =========================================================================
	// contains_type_param tests (7b)
	// =========================================================================

	#[test]
	fn test_simple_path() {
		let ty: Type = parse_str("F").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_simple_path_mismatch() {
		let ty: Type = parse_str("F").unwrap();
		assert!(!contains_type_param(&ty, "G"));
	}

	#[test]
	fn test_nested_in_generic() {
		let ty: Type = parse_str("Option<F>").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_deeply_nested() {
		let ty: Type = parse_str("Vec<Option<F>>").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_reference() {
		let ty: Type = parse_str("&F").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_mutable_reference() {
		let ty: Type = parse_str("&mut F").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_tuple() {
		let ty: Type = parse_str("(A, F, B)").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_tuple_absent() {
		let ty: Type = parse_str("(A, B)").unwrap();
		assert!(!contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_bare_fn() {
		let ty: Type = parse_str("fn(F) -> B").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_bare_fn_return() {
		let ty: Type = parse_str("fn(A) -> F").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_impl_trait_bound() {
		let ty: Type = parse_str("impl Iterator<Item = F>").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_dyn_trait_bound() {
		let ty: Type = parse_str("dyn Fn(F) -> B").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_array() {
		let ty: Type = parse_str("[F; 3]").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_slice() {
		let ty: Type = parse_str("[F]").unwrap();
		assert!(contains_type_param(&ty, "F"));
	}

	#[test]
	fn test_no_match_in_complex() {
		let ty: Type = parse_str("Vec<Option<&str>>").unwrap();
		assert!(!contains_type_param(&ty, "F"));
	}

	// =========================================================================
	// find_impl_trait_candidates tests (7c)
	// =========================================================================

	fn parse_sig(s: &str) -> Signature {
		// Wrap in a dummy function to parse
		let item: syn::ItemFn = parse_str(&format!("{s} {{}}")).unwrap();
		item.sig
	}

	// Should produce candidates

	#[test]
	fn test_basic_fn_bound() {
		let sig = parse_sig("fn new<F>(f: F) where F: FnOnce() -> A");
		let candidates = find_impl_trait_candidates(&sig);
		assert_eq!(candidates.len(), 1);
		assert_eq!(candidates[0].param_name, "F");
		assert!(candidates[0].bounds_display.contains("FnOnce"));
	}

	#[test]
	fn test_inline_bounds() {
		let sig = parse_sig("fn apply<F: Fn(A) -> B>(f: F, a: A) -> B");
		let candidates = find_impl_trait_candidates(&sig);
		assert_eq!(candidates.len(), 1);
		assert_eq!(candidates[0].param_name, "F");
	}

	#[test]
	fn test_multiple_candidates() {
		let sig = parse_sig("fn foo<F: Fn(A), G: Fn(B)>(f: F, g: G)");
		let candidates = find_impl_trait_candidates(&sig);
		assert_eq!(candidates.len(), 2);
		let names: Vec<&str> = candidates.iter().map(|c| c.param_name.as_str()).collect();
		assert!(names.contains(&"F"));
		assert!(names.contains(&"G"));
	}

	#[test]
	fn test_mixed_where_and_inline() {
		let sig = parse_sig("fn bar<B: 'static, F>(f: F) -> Out where F: FnOnce(A) -> B + 'static");
		let candidates = find_impl_trait_candidates(&sig);
		// F is a candidate (appears once in params, not in return type)
		// B is NOT a candidate (only lifetime bounds)
		assert_eq!(candidates.len(), 1);
		assert_eq!(candidates[0].param_name, "F");
	}

	#[test]
	fn test_lifetime_only_bound_skipped() {
		let sig = parse_sig("fn baz<B: 'static>(x: B) -> B");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	// Should NOT produce candidates

	#[test]
	fn test_in_return_type() {
		let sig = parse_sig("fn identity<T: Clone>(x: T) -> T");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_multiple_param_positions() {
		let sig = parse_sig("fn combine<T: Clone>(a: T, b: T) -> T");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_no_trait_bounds() {
		let sig = parse_sig("fn wrap<T>(x: T) -> Box<T>");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_cross_referenced() {
		let sig = parse_sig("fn foo<F: Clone, G: Fn(F)>(f: F, g: G)");
		let candidates = find_impl_trait_candidates(&sig);
		// F is cross-referenced by G's bound, so F is not a candidate
		// G is still a candidate (appears once, not in return, not cross-referenced)
		let names: Vec<&str> = candidates.iter().map(|c| c.param_name.as_str()).collect();
		assert!(!names.contains(&"F"));
		assert!(names.contains(&"G"));
	}

	#[test]
	fn test_only_lifetime_bounds() {
		let sig = parse_sig("fn bar<T: 'a>(x: T)");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_self_receiver_ignored() {
		let sig = parse_sig("fn method<F: Fn()>(self_: &Self, f: F, f2: F)");
		// Note: we can't use `&self` in a free fn parse, so we test with two typed params
		// F appears in 2 positions -> not a candidate
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	// =========================================================================
	// Edge case tests (7f)
	// =========================================================================

	#[test]
	fn test_no_generics() {
		let sig = parse_sig("fn foo(x: i32) -> i32");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_empty_where_clause() {
		// A function with `where` but no predicates
		let sig = parse_sig("fn foo<T: Clone>(x: T) -> T where");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty()); // T in return type
	}

	#[test]
	fn test_self_receiver_not_counted() {
		// Use a trait-like signature: &self should not count as a parameter position
		let item: syn::TraitItemFn = parse_str("fn method<F: Fn()>(&self, f: F);").unwrap();
		let candidates = find_impl_trait_candidates(&item.sig);
		// F appears once (in `f`), self is skipped -> candidate
		assert_eq!(candidates.len(), 1);
		assert_eq!(candidates[0].param_name, "F");
	}

	#[test]
	fn test_multiple_bounds_displayed() {
		let sig = parse_sig("fn foo<F: Clone + Send + Fn()>(f: F)");
		let candidates = find_impl_trait_candidates(&sig);
		assert_eq!(candidates.len(), 1);
		assert!(candidates[0].bounds_display.contains("Clone"));
		assert!(candidates[0].bounds_display.contains("Send"));
		assert!(candidates[0].bounds_display.contains("Fn"));
	}

	#[test]
	fn test_where_clause_cross_ref() {
		let sig = parse_sig("fn foo<A: Clone, B>(a: A, b: B) where B: From<A>");
		let candidates = find_impl_trait_candidates(&sig);
		// A is cross-referenced by B's where-clause bound
		let names: Vec<&str> = candidates.iter().map(|c| c.param_name.as_str()).collect();
		assert!(!names.contains(&"A"));
	}

	// =========================================================================
	// is_top_level_type_param tests
	// =========================================================================

	#[test]
	fn test_top_level_bare_ident() {
		let ty: Type = parse_str("F").unwrap();
		assert!(is_top_level_type_param(&ty, "F"));
	}

	#[test]
	fn test_top_level_reference() {
		let ty: Type = parse_str("&F").unwrap();
		assert!(is_top_level_type_param(&ty, "F"));
	}

	#[test]
	fn test_top_level_mut_reference() {
		let ty: Type = parse_str("&mut F").unwrap();
		assert!(is_top_level_type_param(&ty, "F"));
	}

	#[test]
	fn test_not_top_level_in_option() {
		let ty: Type = parse_str("Option<F>").unwrap();
		assert!(!is_top_level_type_param(&ty, "F"));
	}

	#[test]
	fn test_not_top_level_in_vec() {
		let ty: Type = parse_str("Vec<F>").unwrap();
		assert!(!is_top_level_type_param(&ty, "F"));
	}

	#[test]
	fn test_not_top_level_in_tuple() {
		let ty: Type = parse_str("(A, F)").unwrap();
		assert!(!is_top_level_type_param(&ty, "F"));
	}

	#[test]
	fn test_not_top_level_in_associated_type() {
		let ty: Type = parse_str("<F as Trait>::Assoc").unwrap();
		assert!(!is_top_level_type_param(&ty, "F"));
	}

	// =========================================================================
	// appears_once_at_top_level integration tests
	// =========================================================================

	#[test]
	fn test_nested_param_not_candidate() {
		// F only appears nested in Option<F>, not at top level -> not a candidate
		let sig = parse_sig("fn foo<F: Clone>(x: Option<F>)");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_reference_param_is_candidate() {
		// F appears as &F -> top-level, valid for impl Trait
		let sig = parse_sig("fn foo<F: Clone>(x: &F)");
		let candidates = find_impl_trait_candidates(&sig);
		assert_eq!(candidates.len(), 1);
		assert_eq!(candidates[0].param_name, "F");
	}

	#[test]
	fn test_associated_type_projection_not_candidate() {
		// F only appears in <F as Trait>::Assoc position -> not top-level
		let sig = parse_sig("fn foo<F: Iterator>(x: <F as Iterator>::Item)");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	// =========================================================================
	// Self-referential bounds tests
	// =========================================================================

	#[test]
	fn test_self_referential_bound_not_candidate() {
		// FA's own bound references FA via a projection: cannot use impl Trait
		let sig = parse_sig(
			"fn join<FA, A>(mma: FA) where FA: InferableBrand<A> + Dispatch<A, <FA as InferableBrand<A>>::Marker>",
		);
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_non_self_referential_bound_is_candidate() {
		// FA's bounds do not reference FA itself
		let sig = parse_sig("fn foo<FA: Clone + Send>(x: FA)");
		let candidates = find_impl_trait_candidates(&sig);
		assert_eq!(candidates.len(), 1);
		assert_eq!(candidates[0].param_name, "FA");
	}

	// =========================================================================
	// Where-clause projection tests
	// =========================================================================

	#[test]
	fn test_where_clause_projection_not_candidate() {
		// FA appears inside a where-clause predicate's bounded_ty as a projection
		let sig = parse_sig("fn foo<FA: Clone>(x: FA) where <FA as Iterator>::Item: Send");
		let candidates = find_impl_trait_candidates(&sig);
		assert!(candidates.is_empty());
	}

	#[test]
	fn test_simple_where_bound_still_candidate() {
		// FA has a simple where-clause bound (FA: Clone), not a projection
		let sig = parse_sig("fn foo<FA>(x: FA) where FA: Clone");
		let candidates = find_impl_trait_candidates(&sig);
		assert_eq!(candidates.len(), 1);
		assert_eq!(candidates[0].param_name, "FA");
	}
}
