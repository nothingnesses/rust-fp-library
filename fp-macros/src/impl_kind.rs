//! Implementation of the `impl_kind!` macro.
//!
//! This module handles the parsing and expansion of the `impl_kind!` macro, which is used
//! to implement a generated `Kind` trait for a specific brand type.

use crate::{
	generate::generate_name,
	parse::{KindAssocTypeInput, KindInput},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
	Generics, Ident, Token, Type, TypeParamBound, braced,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// Input structure for the `impl_kind!` macro.
///
/// Parses syntax like:
/// ```ignore
/// impl<T> for MyBrand {
///     type Of<A> = MyType<A>;
///     type SendOf<B> = MySendType<B>;
/// }
/// ```
#[allow(dead_code)]
pub struct ImplKindInput {
	/// Generics for the impl block (e.g., `impl<T>`).
	pub impl_generics: Generics,
	/// The `for` keyword.
	pub for_token: Token![for],
	/// The brand type being implemented (e.g., `MyBrand`).
	pub brand: Type,
	/// The brace token surrounding the associated type definitions.
	pub brace_token: syn::token::Brace,
	/// The associated type definitions inside the braces.
	pub definitions: Vec<KindAssocTypeImpl>,
}

/// Represents a single associated type definition inside `impl_kind!`.
///
/// Example: `type Of<A> = MyType<A>;`
#[allow(dead_code)]
pub struct KindAssocTypeImpl {
	/// The `type` keyword.
	pub type_token: Token![type],
	/// The name of the associated type.
	pub ident: Ident,
	/// Generics for the associated type (e.g., `<A>`).
	pub generics: Generics,
	/// Optional colon for bounds.
	pub colon_token: Option<Token![:]>,
	/// Bounds on the associated type.
	pub bounds: Punctuated<TypeParamBound, Token![+]>,
	/// The `=` token.
	pub eq_token: Token![=],
	/// The concrete type being assigned (e.g., `MyType<A>`).
	pub target_type: Type,
	/// The semicolon.
	pub semi_token: Token![;],
}

impl Parse for ImplKindInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let mut impl_generics = if input.peek(Token![impl]) {
			input.parse::<Token![impl]>()?;
			input.parse::<Generics>()?
		} else {
			Generics::default()
		};

		let for_token: Token![for] = input.parse()?;
		let brand: Type = input.parse()?;

		// Parse where clause if present (comes after brand, before braces)
		if input.peek(Token![where]) {
			impl_generics.where_clause = Some(input.parse()?);
		}

		let content;
		let brace_token = braced!(content in input);

		let mut definitions = Vec::new();
		while !content.is_empty() {
			definitions.push(content.parse()?);
		}

		Ok(ImplKindInput { impl_generics, for_token, brand, brace_token, definitions })
	}
}

impl Parse for KindAssocTypeImpl {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let type_token: Token![type] = input.parse()?;
		let ident: Ident = input.parse()?;
		let generics: Generics = input.parse()?;

		let mut colon_token: Option<Token![:]> = None;
		let mut bounds = Punctuated::new();

		if input.peek(Token![:]) {
			colon_token = Some(input.parse()?);
			loop {
				if input.peek(Token![=]) || input.peek(Token![;]) {
					break;
				}
				bounds.push_value(input.parse()?);
				if input.peek(Token![+]) {
					bounds.push_punct(input.parse()?);
				} else {
					break;
				}
			}
		}

		let eq_token: Token![=] = input.parse()?;
		let target_type: Type = input.parse()?;
		let semi_token: Token![;] = input.parse()?;

		Ok(KindAssocTypeImpl {
			type_token,
			ident,
			generics,
			colon_token,
			bounds,
			eq_token,
			target_type,
			semi_token,
		})
	}
}

/// Generates the implementation for the `impl_kind!` macro.
///
/// This function takes the parsed input, determines the correct `Kind` trait based on
/// the signature of the associated types, and generates the `impl` block.
pub fn impl_kind_impl(input: ImplKindInput) -> TokenStream {
	let brand = &input.brand;
	let impl_generics = &input.impl_generics;

	// Convert to KindInput for name generation
	let assoc_types_input: Vec<KindAssocTypeInput> = input
		.definitions
		.iter()
		.map(|def| KindAssocTypeInput {
			_type_token: def.type_token,
			ident: def.ident.clone(),
			generics: def.generics.clone(),
			_colon_token: def.colon_token,
			output_bounds: def.bounds.clone(),
			_semi_token: def.semi_token,
		})
		.collect();

	let kind_input = KindInput { assoc_types: assoc_types_input };
	let kind_trait_name = generate_name(&kind_input);

	let assoc_types_impl = input.definitions.iter().map(|def| {
		let ident = &def.ident;
		let generics = &def.generics;
		let target = &def.target_type;

		quote! {
			type #ident #generics = #target;
		}
	});

	// Generate doc comment
	let doc_comment =
		format!("Generated implementation of `{}` for `{}`.", kind_trait_name, quote!(#brand));

	let (impl_generics_impl, _, impl_generics_where) = impl_generics.split_for_impl();

	quote! {
		#[doc = #doc_comment]
		impl #impl_generics_impl #kind_trait_name for #brand #impl_generics_where {
			#(#assoc_types_impl)*
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// ===========================================================================
	// impl_kind! Parsing and Generation Tests
	// ===========================================================================

	#[test]
	fn test_parse_impl_kind_simple() {
		let input = "for OptionBrand { type Of<A> = Option<A>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		assert_eq!(parsed.definitions.len(), 1);
		assert_eq!(parsed.definitions[0].ident.to_string(), "Of");
	}

	#[test]
	fn test_parse_impl_kind_multiple() {
		let input = "for MyBrand { 
			type Of<A> = MyType<A>;
			type SendOf<B> = MySendType<B>;
		}";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		assert_eq!(parsed.definitions.len(), 2);
		assert_eq!(parsed.definitions[0].ident.to_string(), "Of");
		assert_eq!(parsed.definitions[1].ident.to_string(), "SendOf");
	}

	#[test]
	fn test_impl_kind_generation() {
		let input = "for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl Kind_"));
		assert!(output_str.contains("for OptionBrand"));
		assert!(output_str.contains("type Of < 'a , A : 'a > = Option < A >"));
	}

	// ===========================================================================
	// impl_kind! with generics Tests
	// ===========================================================================

	#[test]
	fn test_impl_kind_with_impl_generics() {
		let input = "impl<E> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
	}

	#[test]
	fn test_impl_kind_with_multiple_impl_generics() {
		let input = "impl<E: Clone, F: Send> for MyBrand<E, F> { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E : Clone , F : Send > Kind_"));
		assert!(output_str.contains("for MyBrand < E , F >"));
	}

	#[test]
	fn test_impl_kind_with_bounded_impl_generic() {
		let input = "impl<E: std::fmt::Debug> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E : std :: fmt :: Debug > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
	}

	// ===========================================================================
	// impl_kind! with where clauses Tests
	// ===========================================================================

	#[test]
	fn test_impl_kind_with_where_clause() {
		let input =
			"impl<E> for ResultBrand<E> where E: std::fmt::Debug { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
		assert!(output_str.contains("where E : std :: fmt :: Debug"));
	}

	#[test]
	fn test_impl_kind_with_multiple_where_bounds() {
		let input = "impl<E, F> for MyBrand<E, F> where E: Clone, F: Send { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E , F >"));
		assert!(output_str.contains("where E : Clone , F : Send"));
	}
}
