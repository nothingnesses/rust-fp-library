//! # Generic Parameter Handling
//!
//! This module provides utilities for extracting and analyzing generic parameters
//! from Rust syntax structures. It offers a unified interface for working with
//! different kinds of generic parameters (lifetimes, types, const parameters)
//! in procedural macros.
//!
//! ## Overview
//!
//! Generic parameter handling is scattered across multiple functions in the
//! documentation macro system. This module provides the foundational utilities,
//! while more complex analysis (such as bound extraction) is handled in
//! [`analyze_generics`].
//!
//! ## Parameter Categories
//!
//! Rust supports three kinds of generic parameters:
//!
//! 1. **Lifetime Parameters**: `'a`, `'static`, etc.
//! 2. **Type Parameters**: `T`, `U`, `A`, `B`, etc.
//! 3. **Const Parameters**: `const N: usize`, etc.
//!
//! Different functions in this module extract different subsets of these parameters.
//!
//! ## Available Interface
//!
//! The module provides free functions for generic analysis:
//!
//! ### Type Parameters Only - [`get_type_parameters`]
//!
//! Extracts only type parameter names, filtering out lifetimes and const parameters.
//! This is the most commonly used operation in the documentation system.
//!
//! **Use when:** You need type parameter names for type variable tracking,
//! generic substitution, or HM type conversion.
//!
//! **Example:**
//! ```rust,ignore
//! // Given: fn foo<'a, T, U, const N: usize>(x: &'a T) -> [U; N]
//! let generics: syn::Generics = /* ... */;
//! let type_params = get_type_parameters(&generics);
//! // Returns: ["T", "U"]
//! // Omits: 'a (lifetime), N (const)
//! ```
//!
//! **Used in:**
//! - [`function_utils::type_to_hm`] - Building the `generic_names` set for type conversion
//! - [`document_module::resolution::get_self_type_info`] - Extracting impl block type parameters
//! - [`document_module::resolution::normalize_type`] - Type parameter normalization
//!
//! ### All Parameters - [`get_all_parameters`]
//!
//! Extracts names of all generic parameters including lifetimes, types, and const parameters.
//!
//! **Use when:** You need the complete list of generic parameter names for
//! comprehensive analysis or display purposes.
//!
//! **Example:**
//! ```rust,ignore
//! // Given: fn foo<'a, T, U, const N: usize>(x: &'a T) -> [U; N]
//! let generics: syn::Generics = /* ... */;
//! let all_params = get_all_parameters(&generics);
//! // Returns: ["'a", "T", "U", "N"]
//! // Includes all parameter kinds
//! ```
//!
//! **Used in:**
//! - [`document_module::generation::process_doc_type_params`] - Comprehensive parameter tracking
//! - Useful for debugging and diagnostic purposes
//!
//! ## Relationship to Other Generic Handling
//!
//! This module provides **name extraction only**. More complex generic parameter
//! analysis is performed elsewhere:
//!
//! ### [`analyze_generics`]
//!
//! **Purpose:** Analyzes type parameter bounds and extracts function signatures
//! from trait bounds (e.g., `F: Fn(A) -> B`).
//!
//! **Returns:** A `HashMap<String, HMType>` mapping type parameter names to their
//! function signatures (if they have `Fn`-trait bounds).
//!
//! **Example:**
//! ```rust,ignore
//! fn map<A, B, F>(f: F, fa: Self) -> Self
//! where
//!     F: Fn(A) -> B,
//!     A: Clone,
//! {
//!     // analyze_generics returns:
//!     // { "F": Arrow(Variable("A"), Variable("B")) }
//!     // Note: A and B have no Fn bounds, so they're not in the map
//! }
//! ```
//!
//! ### [`document_module::resolution::merge_generics`]
//!
//! **Purpose:** Merges generic parameters from impl blocks with function signatures,
//! maintaining proper ordering (lifetimes, types, consts).
//!
//! **Example:**
//! ```rust,ignore
//! impl<'a, T> Functor for CatList<T> {
//!     fn map<'b, U, F>(self, f: F) -> Self
//!     // merge_generics produces: <'a, 'b, T, U, F>
//!     // Order: lifetimes first, then types
//! }
//! ```
//!
//! ## Usage Patterns
//!
//! ### Pattern 1: Type Variable Tracking
//!
//! The most common pattern is to extract type parameters for tracking which
//! identifiers are type variables vs. concrete types:
//!
//! ```rust,ignore
//! fn process_signature(sig: &syn::Signature) {
//!     // Extract type parameter names as a set
//!     let generic_names = get_type_parameters_set(&sig.generics);
//!
//!     // Now use generic_names to distinguish type variables from concrete types
//!     for ty in &sig.inputs {
//!         if let Some(ident) = ty.path.get_ident() {
//!             if generic_names.contains(&ident) {
//!                 // This is a type variable (e.g., T, A)
//!             } else {
//!                 // This is a concrete type (e.g., Option, Vec)
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ### Pattern 2: Impl Block Analysis
//!
//! Extract type parameters from impl blocks for Self resolution:
//!
//! ```rust,ignore
//! fn analyze_impl(item_impl: &syn::ItemImpl) {
//!     // For: impl<A, B> Functor for CatList<A>
//!     let impl_type_params = get_type_parameters(&item_impl.generics);
//!     // Returns: ["A", "B"]
//!
//!     // Can now track which parameters belong to the impl
//!     // vs. individual methods
//! }
//! ```
//!
//! ### Pattern 3: Combined with Bound Analysis
//!
//! Often combined with bound analysis for complete generic understanding:
//!
//! ```rust,ignore
//! use crate::analysis::generics::{get_type_parameters_set, analyze_fn_bounds};
//!
//! fn analyze_function(sig: &syn::Signature, config: &Config) {
//!     // Get type parameter names as a set
//!     let generic_names = get_type_parameters_set(&sig.generics);
//!
//!     // Get function bounds for those parameters
//!     let fn_bounds = analyze_fn_bounds(sig, config);
//!
//!     // Now we have:
//!     // - generic_names: Which identifiers are type variables
//!     // - fn_bounds: What function signatures they represent (if any)
//! }
//! ```
//!
//! ## Design Rationale
//!
//! ### Why Free Functions?
//!
//! The module uses free functions instead of a struct because:
//!
//! 1. **Simplicity**: Most operations are pure transformations of `syn::Generics`.
//!
//! 2. **Rust Idioms**: Procedural macro analysis often uses free functions or
//!    trait-based extensions for syntax tree nodes.
//!
//! 3. **Reduced Boilerplate**: Eliminates the need to instantiate an analyzer
//!    struct for simple extractions.
//!
//! ### Why Separate Functions?
//!
//! The module provides separate functions ([`get_type_parameters`],
//! [`get_all_parameters`], [`get_type_parameters_set`],
//! [`analyze_fn_bounds`]) because:
//!
//! 1. **Type parameters are special**: In HM-style type signatures, only type
//!    parameters appear as variables. Lifetimes and const parameters are not
//!    part of the HM type system.
//!
//! 2. **Performance**: Most call sites only need type parameters, so dedicated
//!    functions avoid unnecessary allocations and conversions.
//!
//! 3. **Clarity**: Explicit function names make the intent clear at call sites.
//!
//! ### Why analyze_fn_bounds Takes Extra Parameters?
//!
//! The [`analyze_fn_bounds`] function requires a `Signature` and `Config` because:
//!
//! 1. **Where Clauses**: Function bounds can appear in where clauses, which are part of the
//!    signature, not just the `Generics`.
//!
//! 2. **Context-Dependent**: Bound analysis requires configuration (e.g., which traits to ignore)
//!    and cross-references between parameters.
//!
//! 3. **Separation of Concerns**: Simple extraction (type params, all params) doesn't need
//!    external context, while semantic analysis (bounds) does.

use {
	crate::{
		analysis::traits::get_fn_type_from_bound,
		core::config::Config,
		hm::HmAst,
	},
	std::collections::{
		HashMap,
		HashSet,
	},
	syn::{
		GenericParam,
		Generics,
		Signature,
		Type,
		TypeParamBound,
		WherePredicate,
	},
};

/// Extracts only type parameter names.
///
/// This returns type parameters like `T`, `U`, filtering out lifetimes
/// and const parameters.
///
/// * `generics` - The generics to extract type parameters from
///
/// ### Returns
///
/// A vector of type parameter names as strings.
///
/// ### Example
///
/// For `<'a, T, U, const N: usize>`, returns `["T", "U"]`.
pub fn get_type_parameters(generics: &Generics) -> Vec<String> {
	generics
		.params
		.iter()
		.filter_map(|p| match p {
			GenericParam::Type(t) => Some(t.ident.to_string()),
			_ => None,
		})
		.collect()
}

/// Extracts all generic parameter names.
///
/// This returns all parameters including lifetimes, type parameters,
/// and const parameters.
///
/// * `generics` - The generics to extract parameters from
///
/// ### Returns
///
/// A vector of all parameter names as strings.
///
/// ### Example
///
/// For `<'a, T, U, const N: usize>`, returns `["'a", "T", "U", "N"]`.
pub fn get_all_parameters(generics: &Generics) -> Vec<String> {
	generics
		.params
		.iter()
		.map(|p| match p {
			GenericParam::Type(t) => t.ident.to_string(),
			GenericParam::Lifetime(l) => l.lifetime.to_string(),
			GenericParam::Const(c) => c.ident.to_string(),
		})
		.collect()
}

/// Extracts type parameter names as a HashSet.
///
/// This is a convenience function for when you need to perform membership
/// checks to determine if an identifier is a type variable.
///
/// * `generics` - The generics to extract type parameters from
///
/// ### Returns
///
/// A `HashSet<String>` of type parameter names.
///
/// ### Example
///
/// ```rust,ignore
/// let type_params = type_parameters_to_set(&sig.generics);
///
/// if type_params.contains("T") {
///     // "T" is a type variable
/// }
/// ```
pub fn type_parameters_to_set(generics: &Generics) -> HashSet<String> {
	get_type_parameters(generics).into_iter().collect()
}

/// Analyzes function trait bounds for generic parameters.
///
/// This function extracts function signatures from trait bounds like
/// `F: Fn(A) -> B`, returning a map from type parameter names to
/// their HM type representations.
///
/// * `sig` - The function signature containing the generics and where clause
/// * `config` - Configuration for type conversion
///
/// ### Returns
///
/// A `HashMap<String, HMType>` mapping type parameter names to their
/// function signatures (only includes parameters with `Fn*` trait bounds).
///
/// ### Example
///
/// For a function signature:
/// ```rust,ignore
/// fn map<A, B, F>(f: F, fa: Self) -> Self
/// where
///     F: Fn(A) -> B,
///     A: Clone,
/// ```
///
/// Returns:
/// ```rust,ignore
/// {
///     "F": Arrow(Variable("a"), Variable("b"))
/// }
/// ```
///
/// Note: `A` and `B` have no `Fn*` bounds, so they're not in the result.
pub fn analyze_fn_bounds(
	sig: &Signature,
	config: &Config,
) -> HashMap<String, HmAst> {
	let mut fn_bounds = HashMap::new();
	let generic_names = type_parameters_to_set(&sig.generics);

	// Collect Fn bounds from generic parameter bounds
	for param in &sig.generics.params {
		if let GenericParam::Type(type_param) = param {
			let name = type_param.ident.to_string();
			for bound in &type_param.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound
					&& let Some(sig_ty) =
						get_fn_type_from_bound(trait_bound, &fn_bounds, &generic_names, config)
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
				// SAFETY: segments.len() == 1 checked above
				#[allow(clippy::indexing_slicing)]
				let name = type_path.path.segments[0].ident.to_string();
				for bound in &predicate_type.bounds {
					if let TypeParamBound::Trait(trait_bound) = bound
						&& let Some(sig_ty) =
							get_fn_type_from_bound(trait_bound, &fn_bounds, &generic_names, config)
					{
						fn_bounds.insert(name.clone(), sig_ty);
					}
				}
			}
		}
	}

	fn_bounds
}

/// Analyze a function signature to extract generic names and function bounds.
///
/// Returns a tuple of (generic_names, fn_bounds) for use in type conversion.
pub fn analyze_generics(
	sig: &syn::Signature,
	config: &Config,
) -> (std::collections::HashSet<String>, std::collections::HashMap<String, HmAst>) {
	let generic_names = type_parameters_to_set(&sig.generics);
	let fn_bounds = analyze_fn_bounds(sig, config);

	(generic_names, fn_bounds)
}
