//! Implementation of the `#[kind]` attribute macro.
//!
//! This module handles adding a `Kind` supertrait bound to a trait definition
//! based on a signature provided in the attribute arguments.

use {
	super::AssociatedTypes,
	crate::{
		core::Result,
		generate_name,
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		ItemTrait,
		parse_quote,
	},
};

/// Generates the implementation for the `#[kind]` attribute macro.
///
/// This function takes the parsed attribute arguments (a Kind signature) and
/// the annotated trait definition, then adds the corresponding `Kind_` trait
/// as a supertrait bound.
pub fn kind_attr_worker(
	attr: AssociatedTypes,
	mut item: ItemTrait,
) -> Result<TokenStream> {
	let name = generate_name(&attr)?;

	if !item.supertraits.is_empty() {
		item.supertraits.push_punct(<syn::Token![+]>::default());
	}
	item.supertraits.push_value(parse_quote!(#name));

	Ok(quote!(#item))
}

#[cfg(test)]
mod tests {
	use super::*;

	fn parse_kind_input(input: &str) -> AssociatedTypes {
		syn::parse_str(input).expect("Failed to parse KindInput")
	}

	fn parse_trait(input: &str) -> ItemTrait {
		syn::parse_str(input).expect("Failed to parse ItemTrait")
	}

	#[test]
	fn test_kind_attr_adds_supertrait() {
		let attr = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let item = parse_trait("pub trait Functor { fn map(); }");
		let output = kind_attr_worker(attr, item).expect("kind_attr_worker failed");
		let output_str = output.to_string();

		assert!(
			output_str.contains("Kind_"),
			"Expected Kind_ supertrait in output, got: {output_str}"
		);
		assert!(
			output_str.contains("pub trait Functor"),
			"Expected trait definition preserved, got: {output_str}"
		);
	}

	#[test]
	fn test_kind_attr_preserves_existing_supertraits() {
		let attr = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let item = parse_trait("pub trait Monad: Applicative { fn bind(); }");
		let output = kind_attr_worker(attr, item).expect("kind_attr_worker failed");
		let output_str = output.to_string();

		assert!(
			output_str.contains("Applicative"),
			"Expected existing supertrait preserved, got: {output_str}"
		);
		assert!(output_str.contains("Kind_"), "Expected Kind_ supertrait added, got: {output_str}");
		assert!(output_str.contains("+"), "Expected + between supertraits, got: {output_str}");
	}

	#[test]
	fn test_kind_attr_deterministic_name() {
		let attr1 = parse_kind_input("type Of<'a, A: 'a>: 'a;");
		let attr2 = parse_kind_input("type Of<'a, T: 'a>: 'a;");
		let item1 = parse_trait("trait Foo {}");
		let item2 = parse_trait("trait Bar {}");

		let output1 = kind_attr_worker(attr1, item1).expect("kind_attr_worker failed");
		let output2 = kind_attr_worker(attr2, item2).expect("kind_attr_worker failed");

		// Both should generate the same Kind_ name (parameter names don't matter)
		let name1 = output1
			.to_string()
			.split("Kind_")
			.nth(1)
			.unwrap()
			.split_whitespace()
			.next()
			.unwrap()
			.to_string();
		let name2 = output2
			.to_string()
			.split("Kind_")
			.nth(1)
			.unwrap()
			.split_whitespace()
			.next()
			.unwrap()
			.to_string();

		assert_eq!(
			name1, name2,
			"Same signature with different param names should produce same hash"
		);
	}
}
