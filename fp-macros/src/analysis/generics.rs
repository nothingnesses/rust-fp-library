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
//! [`function_utils::analyze_generics`].
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
//! The primary interface is the [`GenericAnalyzer`] struct, which provides methods for:
//!
//! ### Type Parameters Only - [`GenericAnalyzer::type_params`]
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
//! let analyzer = GenericAnalyzer::new(&generics);
//! let type_params = analyzer.type_params();
//! // Returns: ["T", "U"]
//! // Omits: 'a (lifetime), N (const)
//! ```
//!
//! **Used in:**
//! - [`function_utils::type_to_hm`] - Building the `generic_names` set for type conversion
//! - [`document_module::resolution::extract_self_type_info`] - Extracting impl block type parameters
//! - [`document_module::resolution::normalize_type`] - Type parameter normalization
//!
//! ### All Parameters - [`GenericAnalyzer::all_params`]
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
//! let analyzer = GenericAnalyzer::new(&generics);
//! let all_params = analyzer.all_params();
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
//! ### [`function_utils::analyze_generics`]
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
//! use crate::generic_utils::GenericAnalyzer;
//!
//! fn process_signature(sig: &syn::Signature) {
//!     // Create analyzer and extract type parameter names
//!     let analyzer = GenericAnalyzer::new(&sig.generics);
//!     let generic_names = analyzer.type_params_set();
//!
//!     // Now use generic_names to distinguish type variables from concrete types
//!     for ty in &sig.inputs {
//!         if let Some(ident) = extract_type_ident(ty) {
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
//! use crate::generic_utils::GenericAnalyzer;
//!
//! fn analyze_impl(item_impl: &syn::ItemImpl) {
//!     // For: impl<A, B> Functor for CatList<A>
//!     let analyzer = GenericAnalyzer::new(&item_impl.generics);
//!     let impl_type_params = analyzer.type_params();
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
//! use crate::generic_utils::GenericAnalyzer;
//! use crate::function_utils::analyze_generics;
//!
//! fn analyze_function(sig: &syn::Signature, config: &Config) {
//!     // Create analyzer for comprehensive analysis
//!     let analyzer = GenericAnalyzer::new(&sig.generics);
//!
//!     // Get type parameter names as a set
//!     let generic_names = analyzer.type_params_set();
//!
//!     // Get function bounds for those parameters
//!     let fn_bounds = analyzer.fn_bounds(sig, config);
//!
//!     // Now we have:
//!     // - generic_names: Which identifiers are type variables
//!     // - fn_bounds: What function signatures they represent (if any)
//! }
//! ```
//!
//! ## Design Rationale
//!
//! ### Why a Unified Analyzer?
//!
//! The [`GenericAnalyzer`] struct provides a unified interface because:
//!
//! 1. **Consistency**: All generic parameter operations go through a single entry point,
//!    making the API more discoverable and easier to use correctly.
//!
//! 2. **Type Safety**: The analyzer holds a reference to `Generics`, ensuring all methods
//!    operate on the same generic parameter list.
//!
//! 3. **Extensibility**: New analysis methods can be added to the analyzer without
//!    proliferating top-level functions.
//!
//! ### Why Separate Methods?
//!
//! The analyzer provides separate methods ([`type_params`](GenericAnalyzer::type_params),
//! [`all_params`](GenericAnalyzer::all_params), [`type_params_set`](GenericAnalyzer::type_params_set),
//! [`fn_bounds`](GenericAnalyzer::fn_bounds)) because:
//!
//! 1. **Type parameters are special**: In HM-style type signatures, only type
//!    parameters appear as variables. Lifetimes and const parameters are not
//!    part of the HM type system.
//!
//! 2. **Performance**: Most call sites only need type parameters, so dedicated
//!    methods avoid unnecessary allocations and conversions.
//!
//! 3. **Clarity**: Explicit method names make the intent clear at call sites.
//!
//! ### Why fn_bounds Takes Extra Parameters?
//!
//! The [`fn_bounds`](GenericAnalyzer::fn_bounds) method requires a `Signature` and `Config` because:
//!
//! 1. **Where Clauses**: Function bounds can appear in where clauses, which are part of the
//!    signature, not just the `Generics`.
//!
//! 2. **Context-Dependent**: Bound analysis requires configuration (e.g., which traits to ignore)
//!    and cross-references between parameters.
//!
//! 3. **Separation of Concerns**: Simple extraction (type params, all params) doesn't need
//!    external context, while semantic analysis (bounds) does.
//!
//! ## Future Improvements
//!
//! Potential enhancements to the generic parameter handling system:
//!
//! 1. **Const Parameter Integration**: Currently const parameters are largely ignored
//!    in documentation generation. Future work could:
//!    - Add HM-style representation for const generics
//!    - Track const parameter dependencies
//!    - Generate documentation for const bounds
//!
//! 2. **Lifetime Tracking**: More sophisticated lifetime tracking could improve
//!    error messages and diagnostics by:
//!    - Detecting lifetime conflicts in generic bounds
//!    - Providing better error messages for lifetime issues
//!    - Tracking lifetime variance in type transformations
//!
//! 3. **Generic Constraint Analysis**: Extend bound analysis beyond `Fn*` traits:
//!    - Track other trait bounds (Clone, Debug, Send, etc.)
//!    - Analyze associated type constraints
//!    - Detect conflicting bounds

use std::collections::{HashMap, HashSet};
use syn::{GenericParam, Generics, Signature, Type, TypeParamBound, WherePredicate};

use crate::config::Config;
use crate::hm_conversion::HMType;
use crate::analysis::traits::{TraitCategory, classify_trait};
use crate::hm_conversion::converter::trait_bound_to_hm_arrow;
use crate::common::errors::known_types;
use crate::common::last_path_segment;

/// Unified analyzer for generic parameter extraction and analysis.
///
/// This struct provides a consistent namespace for all generic parameter operations,
/// consolidating the previously scattered functionality across multiple functions.
///
/// ## Design Rationale
///
/// Previously, generic parameter handling was scattered across standalone functions
/// in this module and [`analyze_generics`](crate::function_utils::analyze_generics)
/// in `function_utils`. This analyzer consolidates all generic parameter operations
/// into a single, cohesive interface using static methods.
///
/// ## Usage
///
/// ### Basic Type Parameter Extraction
///
/// ```rust,ignore
/// use crate::generic_utils::GenericAnalyzer;
///
/// fn process_signature(sig: &syn::Signature) {
///     let type_params = GenericAnalyzer::type_params(&sig.generics);
///     // Returns Vec<String> of type parameter names
/// }
/// ```
///
/// ### Complete Generic Analysis
///
/// ```rust,ignore
/// use crate::generic_utils::GenericAnalyzer;
///
/// fn full_analysis(sig: &syn::Signature, config: &Config) {
///     // Get type parameter names
///     let type_params = GenericAnalyzer::type_params(&sig.generics);
///
///     // Get all parameter names (including lifetimes, const)
///     let all_params = GenericAnalyzer::all_params(&sig.generics);
///
///     // Get type parameter names as a set (for membership checks)
///     let generic_names = GenericAnalyzer::type_params_set(&sig.generics);
///
///     // Analyze function bounds (requires signature and config)
///     let fn_bounds = GenericAnalyzer::fn_bounds(sig, config);
/// }
/// ```
///
/// ## Methods
///
/// - [`type_params`](Self::type_params): Extract type parameter names only
/// - [`all_params`](Self::all_params): Extract all parameter names
/// - [`type_params_set`](Self::type_params_set): Type parameters as a HashSet
/// - [`fn_bounds`](Self::fn_bounds): Analyze function trait bounds
pub struct GenericAnalyzer;

impl GenericAnalyzer {
	/// Extracts only type parameter names.
	///
	/// This returns type parameters like `T`, `U`, filtering out lifetimes
	/// and const parameters.
	///
	/// ### Parameters
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
	pub fn type_params(generics: &Generics) -> Vec<String> {
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
	/// ### Parameters
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
	pub fn all_params(generics: &Generics) -> Vec<String> {
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
	/// This is a convenience method for when you need to perform membership
	/// checks to determine if an identifier is a type variable.
	///
	/// ### Parameters
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
	/// let type_params = GenericAnalyzer::type_params_set(&sig.generics);
	///
	/// if type_params.contains("T") {
	///     // "T" is a type variable
	/// }
	/// ```
	pub fn type_params_set(generics: &Generics) -> HashSet<String> {
		Self::type_params(generics).into_iter().collect()
	}

	/// Analyzes function trait bounds for generic parameters.
	///
	/// This method extracts function signatures from trait bounds like
	/// `F: Fn(A) -> B`, returning a map from type parameter names to
	/// their HM type representations.
	///
	/// ### Parameters
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
	pub fn fn_bounds(
		sig: &Signature,
		config: &Config,
	) -> HashMap<String, HMType> {
		let mut fn_bounds = HashMap::new();
		let generic_names = Self::type_params_set(&sig.generics);

		// Collect Fn bounds from generic parameter bounds
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

		fn_bounds
	}
}

// ============================================================================
// Additional Generic Analysis Functions
// ============================================================================

/// Analyze a function signature to extract generic names and function bounds.
///
/// Returns a tuple of (generic_names, fn_bounds) for use in type conversion.
pub fn analyze_generics(
	sig: &syn::Signature,
	config: &Config,
) -> (std::collections::HashSet<String>, std::collections::HashMap<String, HMType>) {
	// Use the unified GenericAnalyzer for consistent generic parameter handling
	let generic_names = GenericAnalyzer::type_params_set(&sig.generics);
	let fn_bounds = GenericAnalyzer::fn_bounds(sig, config);

	(generic_names, fn_bounds)
}

/// Extract the HM type from a trait bound if it represents a function type.
///
/// Returns Some(HMType) if the trait bound is a function trait (Fn, FnMut, FnOnce, or FnBrand),
/// None otherwise.
pub fn get_fn_type(
	trait_bound: &syn::TraitBound,
	fn_bounds: &std::collections::HashMap<String, HMType>,
	generic_names: &std::collections::HashSet<String>,
	config: &Config,
) -> Option<HMType> {
	let segment = last_path_segment(&trait_bound.path)?;
	let name = segment.ident.to_string();
	match classify_trait(&name, config) {
		TraitCategory::FnTrait => {
			Some(trait_bound_to_hm_arrow(trait_bound, fn_bounds, generic_names, config))
		}
		TraitCategory::FnBrand => Some(HMType::Variable(known_types::FN_BRAND_MARKER.to_string())),
		_ => None,
	}
}
