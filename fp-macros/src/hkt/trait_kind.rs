//! Implementation of the `trait_kind!` macro.
//!
//! This module handles the generation of a new `Kind` trait based on a signature.

use {
	super::AssociatedTypes,
	crate::{core::Result, documentation::templates::DocumentationBuilder, generate_name},
	proc_macro2::TokenStream,
	quote::quote,
};

/// Generates the implementation for the `trait_kind!` macro.
///
/// This function takes the parsed input and generates a trait definition
/// for a Higher-Kinded Type signature with multiple associated types.
pub fn trait_kind_worker(input: AssociatedTypes) -> Result<TokenStream> {
	let name = generate_name(&input)?;

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

	Ok(quote! {
		#[doc = #doc_string]
		#[allow(non_camel_case_types)]
		pub trait #name {
			#(#assoc_types_tokens)*
		}
	})
}

#[cfg(test)]
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
