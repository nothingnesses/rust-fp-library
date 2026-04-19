//! Implementation of the `trait_kind!` macro.
//!
//! This module handles the generation of a new `Kind` trait based on a signature.

use {
	super::AssociatedTypes,
	crate::{
		analysis::generics::{
			extract_lifetime_names,
			extract_type_idents,
		},
		core::Result,
		documentation::templates::DocumentationBuilder,
		generate_inferable_brand_name,
		generate_name,
	},
	proc_macro2::TokenStream,
	quote::quote,
};

/// Generates the implementation for the `trait_kind!` macro.
///
/// This function takes the parsed input and generates a trait definition
/// for a Higher-Kinded Type signature with multiple associated types.
pub fn trait_kind_worker(input: AssociatedTypes) -> Result<TokenStream> {
	let name = generate_name(&input)?;
	let inferable_brand_name = generate_inferable_brand_name(&input)?;

	let assoc_types_tokens = input.associated_types.iter().map(|assoc| {
		let ident = &assoc.signature.name;
		let generics = &assoc.signature.generics;
		let output_bounds = &assoc.signature.output_bounds;
		let attrs = &assoc.signature.attributes;
		let output_bounds_tokens =
			if output_bounds.is_empty() { quote!() } else { quote!(: #output_bounds) };

		quote! {
			#(#attrs)*
			type #ident #generics #output_bounds_tokens;
		}
	});

	// Build documentation using the DocumentationBuilder
	let doc_string = DocumentationBuilder::new(&name, &input.associated_types).build();

	// -- InferableBrand trait generation --
	//
	// Extract generics from the first associated type. The InferableBrand trait's
	// parameters are: the associated type's lifetimes, then Brand (bounded
	// by the Kind trait), then the associated type's type parameters.

	// The parser validates that associated_types is non-empty (parse_non_empty in input.rs),
	// so this indexing is safe. Clippy's indexing_slicing lint is suppressed accordingly.
	#[expect(clippy::indexing_slicing, reason = "validated non-empty by parser")]
	let first_generics = &input.associated_types[0].signature.generics;

	let lifetime_defs: Vec<_> = first_generics
		.params
		.iter()
		.filter_map(|p| if let syn::GenericParam::Lifetime(lt) = p { Some(lt) } else { None })
		.collect();

	let type_defs: Vec<_> = first_generics
		.params
		.iter()
		.filter_map(|p| if let syn::GenericParam::Type(tp) = p { Some(tp) } else { None })
		.collect();

	let lifetime_names = extract_lifetime_names(first_generics);
	let type_idents = extract_type_idents(first_generics);

	let inferable_brand_doc_summary = format!(
		r#"Reverse mapping from concrete types to brands for `{name}`.

This trait has Brand as a trait parameter, allowing multiple implementations
per concrete type keyed on different Brand values. This enables
closure-directed inference for multi-brand types like `Result`.

The associated `Marker` type projects whether the container is owned
(`Val`) or borrowed (`Ref`). Direct implementations for owned types set
`Marker = Val`; the blanket implementation for `&T` sets `Marker = Ref`.

**Marker-agreement invariant:** all implementations for a given `Self`
type must agree on the same `Marker` value. Owned types always produce
`Val`; references always produce `Ref`. This invariant is enforced by
construction since `impl_kind!` is the sole generator of implementations.

InferableBrand enables closure-directed brand inference for both single-brand
and multi-brand types."#,
	);

	let inferable_brand_blanket_doc = format!(
		r#"Blanket implementation projecting `Marker = Ref` for borrowed containers.

Delegates the Brand resolution to the underlying type's `{inferable_brand_name}`
implementation while setting `Marker = Ref` to route dispatch to the
by-reference trait method."#,
	);

	Ok(quote! {
		#[doc = #doc_string]
		#[expect(non_camel_case_types, reason = "Generated name uses hash suffix for uniqueness")]
		pub trait #name {
			#(#assoc_types_tokens)*
		}

		#[doc = #inferable_brand_doc_summary]
		#[expect(non_camel_case_types, reason = "Generated name uses hash suffix for uniqueness")]
		#[diagnostic::on_unimplemented(
			message = "cannot infer brand for `{Self}`",
			note = "for multi-brand types, annotate the closure's input type to disambiguate",
			note = "if that does not help, use the `explicit::` variant with a turbofish to specify the brand manually"
		)]
		pub trait #inferable_brand_name<#(#lifetime_defs,)* __InferableBrand_Brand: #name #(, #type_defs)*> {
			/// Dispatch marker: [`Val`](::fp_library::dispatch::Val) for owned types,
			/// [`Ref`](::fp_library::dispatch::Ref) for references.
			type Marker;
		}

		#[doc = #inferable_brand_blanket_doc]
		#[expect(non_camel_case_types, reason = "Generated name uses hash suffix for uniqueness")]
		impl<#(#lifetime_defs,)* __InferableBrand_T: ?Sized, __InferableBrand_Brand: #name #(, #type_defs)*>
			#inferable_brand_name<#(#lifetime_names,)* __InferableBrand_Brand #(, #type_idents)*>
		for &__InferableBrand_T
		where
			__InferableBrand_T: #inferable_brand_name<#(#lifetime_names,)* __InferableBrand_Brand #(, #type_idents)*>,
		{
			type Marker = ::fp_library::dispatch::Ref;
		}
	})
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use super::*;

	/// Helper function to parse a KindInput from a string.
	fn parse_kind_input(input: &str) -> AssociatedTypes {
		syn::parse_str(input).expect("Failed to parse KindInput")
	}

	// ===========================================================================
	// trait_kind! Tests
	// ===========================================================================

	/// Tests trait_kind! with a single associated type.
	#[test]
	fn test_trait_kind_simple() {
		let input = parse_kind_input("type Of<A>;");
		let output = trait_kind_worker(input).expect("trait_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("pub trait Kind_"));
		assert!(output_str.contains("type Of < A > ;"));
	}

	/// Tests trait_kind! with multiple associated types.
	#[test]
	fn test_trait_kind_multiple() {
		let input = parse_kind_input(
			"
			type Of<'a, T>: Display;
			type SendOf<U>: Send;
		",
		);
		let output = trait_kind_worker(input).expect("trait_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("pub trait Kind_"));
		assert!(output_str.contains("type Of < 'a , T > : Display ;"));
		assert!(output_str.contains("type SendOf < U > : Send ;"));
	}

	/// Tests trait_kind! with complex bounds.
	#[test]
	fn test_trait_kind_complex() {
		let input = parse_kind_input("type Of<'a, T: 'a + Clone>: Debug + Display;");
		let output = trait_kind_worker(input).expect("trait_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("type Of < 'a , T : 'a + Clone > : Debug + Display ;"));
	}

	/// Tests that documentation correctly renders type parameter bounds.
	/// This specifically tests for the bug where `#ty.bounds` was incorrectly
	/// used in quote!, resulting in output like "A: A : 'a.bounds" instead of "A: 'a".
	#[test]
	fn test_trait_kind_doc_type_param_bounds() {
		let input = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let output = trait_kind_worker(input).expect("trait_kind_worker failed");
		let output_str = output.to_string();

		// Verify the documentation contains correct type parameter bounds
		assert!(
			output_str.contains(r#"**Type parameters** (1): `A: 'a`"#),
			"Expected documentation to contain 'Type parameters (1): `A: 'a`', got: {output_str}"
		);

		// Ensure the buggy output is not present
		assert!(
			!output_str.contains(".bounds"),
			"Documentation should not contain '.bounds', got: {output_str}"
		);
		assert!(
			!output_str.contains("A: A"),
			"Documentation should not contain 'A: A', got: {}",
			output_str
		);
	}

	/// Tests documentation for type parameters without bounds.
	#[test]
	fn test_trait_kind_doc_type_param_no_bounds() {
		let input = parse_kind_input("type Of<A>;");
		let output = trait_kind_worker(input).expect("trait_kind_worker failed");
		let output_str = output.to_string();

		// Type parameter without bounds should just show the identifier
		assert!(
			output_str.contains(r#"**Type parameters** (1): `A`"#),
			"Expected documentation to contain 'Type parameters (1): `A`', got: {}",
			output_str
		);
	}

	/// Tests that documentation correctly renders the impl_kind! example.
	#[test]
	fn test_trait_kind_doc_impl_example() {
		let input = parse_kind_input("type Of<'a, T>; type SendOf<U>;");
		let output = trait_kind_worker(input).expect("trait_kind_worker failed");
		let output_str = output.to_string();

		// Verify the documentation contains the correct impl_kind! example
		// We check for the presence of the generated lines.
		// Based on our logic:
		// quote! -> type Of < 'a , T > = ConcreteType ;
		// cleanup -> type Of<'a, T> = ConcreteType;

		assert!(
			output_str.contains("type Of<'a, T> = ConcreteType;"),
			"Expected 'type Of<'a, T> = ConcreteType;' in documentation, got: {}",
			output_str
		);
		assert!(
			output_str.contains("type SendOf<U> = ConcreteType;"),
			"Expected 'type SendOf<U> = ConcreteType;' in documentation, got: {}",
			output_str
		);
	}

	/// Tests documentation for multiple type parameters with various bounds.
	#[test]
	fn test_trait_kind_doc_multiple_type_params() {
		let input = parse_kind_input("type Of<'a, T: Clone, U: 'a + Send>: Debug;");
		let output = trait_kind_worker(input).expect("trait_kind_worker failed");
		let output_str = output.to_string();

		// Verify lifetimes doc
		assert!(
			output_str.contains(r#"**Lifetimes** (1): `'a`"#),
			"Expected documentation to contain lifetime 'a, got: {}",
			output_str
		);

		// Verify type parameters doc (both T and U with their bounds)
		assert!(
			output_str.contains(r#"**Type parameters** (2):"#),
			"Expected 2 type parameters, got: {}",
			output_str
		);
		assert!(
			output_str.contains("`T: Clone`"),
			"Expected T: Clone in documentation, got: {}",
			output_str
		);
		assert!(
			output_str.contains("`U: 'a + Send`"),
			"Expected U: 'a + Send in documentation, got: {}",
			output_str
		);

		// Verify output bounds doc
		assert!(
			output_str.contains(r#"**Output bounds**: `Debug`"#),
			"Expected output bounds Debug, got: {}",
			output_str
		);
	}
}
