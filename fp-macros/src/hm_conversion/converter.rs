//! Type-to-HM conversion logic.
//!
//! This module contains the main conversion logic and TypeVisitor trait
//! for transforming Rust types into Hindley-Milner representations.

use crate::{
	analysis::traits::format_brand_name,
	common::last_path_segment,
	config::Config,
	common::errors::known_types,
	hm_conversion::HMType,
};
use std::collections::{HashMap, HashSet};
use syn::{
	GenericArgument, PathArguments, ReturnType, TraitBound, Type,
};

/// A visitor trait for traversing and transforming Rust type syntax trees.
///
/// This trait provides a flexible way to traverse `syn::Type` structures and produce
/// custom output types. Unlike `syn::Visit` and `syn::VisitMut` which are designed for
/// in-place inspection or mutation, `TypeVisitor` is designed for **type transformation**
/// - converting from one type representation to another.
///
/// ## Design Rationale
///
/// **Why a custom trait instead of `syn::Visit`/`syn::VisitMut`?**
/// - `syn::Visit` and `syn::VisitMut` only support traversal with no return values or in-place mutation
/// - Many use cases require transforming types to different representations (e.g., `syn::Type` → `HMType`)
/// - The associated `Output` type enables flexible return values from each visitor method
///
/// **Comparison with syn's visitor traits:**
/// - `syn::Visit` - Read-only traversal, no return values
/// - `syn::VisitMut` - In-place mutation of the same type
/// - `TypeVisitor` - Transform to a different output type (our use case)
///
/// ## All Methods Have Default Implementations
///
/// All visitor methods now have sensible default implementations that return `Default::default()`.
/// This design choice:
/// - Eliminates boilerplate for visitors that only care about specific type constructs
/// - Provides consistent behavior across all type variants
/// - Allows incremental implementation - override only what you need
///
/// ## Usage Patterns
///
/// ### Pattern 1: Transformation (e.g., HMTypeBuilder)
/// Convert types to a custom representation:
/// ```rust,ignore
/// struct MyTransformer { /* config */ }
/// impl TypeVisitor for MyTransformer {
///     type Output = MyType;
///
///     fn visit_path(&mut self, path: &TypePath) -> MyType {
///         // Transform path to MyType
///     }
///     // Override only the methods you need
/// }
/// ```
///
/// ### Pattern 2: Collection (e.g., CurriedParamExtractor)
/// Collect information via side effects:
/// ```rust,ignore
/// struct ParamCollector<'a> {
///     params: &'a mut Vec<Param>,
/// }
/// impl TypeVisitor for ParamCollector<'_> {
///     type Output = ();  // No return value, uses side effects
///
///     fn visit_path(&mut self, path: &TypePath) {
///         // Collect parameters into self.params
///     }
/// }
/// ```
///
/// ## Implementation Examples
///
/// See [`HMTypeBuilder`] (in visitors/transformation.rs) for a transformation visitor.
pub trait TypeVisitor {
	/// The output type produced by visiting each type construct.
	///
	/// Common choices:
	/// - Custom type representation (e.g., `HMType`)
	/// - `()` for side-effect-only visitors
	/// - `Option<T>` for optional extraction
	///
	/// The `Default` bound enables all visitor methods to have default no-op implementations.
	type Output: Default;

	/// Main entry point for visiting a type.
	///
	/// Dispatches to the appropriate specialized visitor method based on the type variant.
	/// Override this method only if you need to intercept all type visits or add logging.
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

	/// Visit a type path (e.g., `Vec<T>`, `std::option::Option`, `Self::Assoc`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Concrete types
	/// - Generic parameters
	/// - Associated types
	/// - Qualified paths (`<T as Trait>::Assoc`)
	fn visit_path(
		&mut self,
		_type_path: &syn::TypePath,
	) -> Self::Output {
		Default::default()
	}

	/// Visit a type macro invocation (e.g., `Apply!(Brand, T, U)`, `vec![T]`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Higher-kinded type applications (`Apply!`)
	/// - Custom type macros
	/// - Declarative macro expansions
	fn visit_macro(
		&mut self,
		_type_macro: &syn::TypeMacro,
	) -> Self::Output {
		Default::default()
	}

	/// Visit a reference type (e.g., `&T`, `&mut T`, `&'a T`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Immutable references
	/// - Mutable references
	/// - Lifetime-annotated references
	fn visit_reference(
		&mut self,
		_type_ref: &syn::TypeReference,
	) -> Self::Output {
		Default::default()
	}

	/// Visit an impl trait type (e.g., `impl Trait`, `impl FnOnce(T) -> U`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Return position impl trait
	/// - Trait bounds extraction
	/// - Function trait patterns
	fn visit_impl_trait(
		&mut self,
		_impl_trait: &syn::TypeImplTrait,
	) -> Self::Output {
		Default::default()
	}

	/// Visit a trait object type (e.g., `dyn Trait`, `dyn Fn(T) -> U + Send`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Dynamic dispatch types
	/// - Trait bound combinations
	/// - Auto trait markers (Send, Sync)
	fn visit_trait_object(
		&mut self,
		_trait_obj: &syn::TypeTraitObject,
	) -> Self::Output {
		Default::default()
	}

	/// Visit a bare function pointer type (e.g., `fn(T) -> U`, `unsafe fn()`, `extern "C" fn()`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Function pointers
	/// - Unsafe functions
	/// - Foreign function interfaces
	fn visit_bare_fn(
		&mut self,
		_bare_fn: &syn::TypeBareFn,
	) -> Self::Output {
		Default::default()
	}

	/// Visit a tuple type (e.g., `()`, `(T,)`, `(T, U)`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Unit type `()`
	/// - Single-element tuples
	/// - Multi-element tuples
	fn visit_tuple(
		&mut self,
		_tuple: &syn::TypeTuple,
	) -> Self::Output {
		Default::default()
	}

	/// Visit an array type (e.g., `[T; N]`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Fixed-size arrays
	/// - Const generic lengths
	fn visit_array(
		&mut self,
		_array: &syn::TypeArray,
	) -> Self::Output {
		Default::default()
	}

	/// Visit a slice type (e.g., `[T]`).
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// Override this to handle:
	/// - Dynamically-sized slices
	fn visit_slice(
		&mut self,
		_slice: &syn::TypeSlice,
	) -> Self::Output {
		Default::default()
	}

	/// Visit any other type variant not covered by specialized methods.
	///
	/// **Default:** Returns `Default::default()` (no-op)
	///
	/// This catches:
	/// - `Type::Ptr` (raw pointers)
	/// - `Type::Never` (`!`)
	/// - `Type::Paren` (parenthesized types)
	/// - `Type::Group` (grouped types)
	/// - `Type::Infer` (`_`)
	/// - `Type::Verbatim` (unparsed tokens)
	/// - Future syn type variants
	fn visit_other(
		&mut self,
		_ty: &syn::Type,
	) -> Self::Output {
		Default::default()
	}
}

// ============================================================================
// Main Conversion Entry Point
// ============================================================================

/// Convert a Rust type to its Hindley-Milner representation.
///
/// This is the main entry point for type conversion. It creates an HMTypeBuilder
/// visitor and uses it to transform the type.
pub fn type_to_hm(
	ty: &Type,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	let mut visitor = crate::hm_conversion::visitors::transformation::HMTypeBuilder { fn_bounds, generic_names, config };
	visitor.visit(ty)
}

// ============================================================================
// Helper Functions for Type Conversion
// ============================================================================

/// Helper function to check if a type path is PhantomData
pub(crate) fn is_phantom_data_path(type_path: &syn::TypePath) -> bool {
	if let Some(segment) = type_path.path.segments.last() {
		segment.ident == known_types::PHANTOM_DATA
	} else {
		false
	}
}

/// Helper function to check if a type name is a smart pointer (Box, Arc, Rc)
pub(crate) fn is_smart_pointer(name: &str) -> bool {
	matches!(name, "Box" | "Arc" | "Rc")
}

/// Helper function to extract the inner type from a smart pointer if present
pub(crate) fn extract_smart_pointer_inner(segment: &syn::PathSegment) -> Option<&syn::Type> {
	if let PathArguments::AngleBracketed(args) = &segment.arguments
		&& let Some(GenericArgument::Type(inner_ty)) = args.args.first()
	{
		return Some(inner_ty);
	}
	None
}

/// Convert a trait bound to an HM type representation.
///
/// This is used for processing trait bounds in impl trait and trait object types.
pub fn trait_bound_to_hm_type(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	let Some(segment) = last_path_segment(&trait_bound.path) else {
		// Defensive fallback for malformed trait bounds
		return HMType::Variable("trait".to_string());
	};
	let name = segment.ident.to_string();

	if name == "Fn" || name == "FnMut" || name == "FnOnce" {
		return trait_bound_to_hm_arrow(trait_bound, fn_bounds, generic_names, config);
	}

	let name = if generic_names.contains(&name) || name == known_types::SELF {
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

/// Convert a trait bound with parenthesized syntax (Fn/FnMut/FnOnce) to an arrow type.
pub fn trait_bound_to_hm_arrow(
	trait_bound: &TraitBound,
	fn_bounds: &HashMap<String, HMType>,
	generic_names: &HashSet<String>,
	config: &Config,
) -> HMType {
	let Some(segment) = last_path_segment(&trait_bound.path) else {
		// Defensive fallback for malformed input (should never occur with valid Rust)
		return HMType::Variable("fn".to_string());
	};
	if let PathArguments::Parenthesized(args) = &segment.arguments {
		// Erase HRTB lifetimes from trait bound
		let _ = &trait_bound.lifetimes;

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
