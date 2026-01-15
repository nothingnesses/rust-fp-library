//! Implementation of the `impl_kind!` macro.
//!
//! This module handles the parsing and expansion of the `impl_kind!` macro, which is used
//! to implement a generated `Kind` trait for a specific brand type.

use crate::{
	generate::generate_name,
	parse::{KindInput, TypeInput},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
	GenericParam, Generics, Ident, Token, Type, TypeParamBound, braced,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};
/// Input structure for the `impl_kind!` macro.
///
/// Parses syntax like:
/// ```ignore
/// impl<T> for MyBrand {
///     type Of<A> = MyType<A>;
/// }
/// ```
///
/// Or with where clause:
/// ```ignore
/// impl<E> for ResultBrand<E> where E: Debug {
///     type Of<A> = Result<A, E>;
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
	/// The brace token surrounding the associated type definition.
	pub brace_token: syn::token::Brace,
	/// The associated type definition inside the braces.
	pub definition: KindAssocType,
}

/// Represents the associated type definition inside `impl_kind!`.
///
/// Example: `type Of<A> = MyType<A>;`
#[allow(dead_code)]
pub struct KindAssocType {
	/// The `type` keyword.
	pub type_token: Token![type],
	/// The name of the associated type (must be `Of`).
	pub of_ident: Ident,
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

		let definition: KindAssocType = content.parse()?;

		Ok(ImplKindInput { impl_generics, for_token, brand, brace_token, definition })
	}
}

impl Parse for KindAssocType {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let type_token: Token![type] = input.parse()?;
		let of_ident: Ident = input.parse()?;
		if of_ident != "Of" {
			return Err(syn::Error::new(of_ident.span(), "Expected associated type name 'Of'"));
		}

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

		Ok(KindAssocType {
			type_token,
			of_ident,
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
/// the signature of the associated type `Of`, and generates the `impl` block.
pub fn impl_kind_impl(input: ImplKindInput) -> TokenStream {
	let brand = &input.brand;
	let definition = &input.definition;
	let impl_generics = &input.impl_generics;

	// Convert to KindInput for name generation
	let lifetimes: Punctuated<_, Token![,]> = definition
		.generics
		.params
		.iter()
		.filter_map(|p| match p {
			GenericParam::Lifetime(lt) => Some(lt.lifetime.clone()),
			_ => None,
		})
		.collect();

	let types: Punctuated<_, Token![,]> = definition
		.generics
		.params
		.iter()
		.filter_map(|p| match p {
			GenericParam::Type(ty) => {
				Some(TypeInput { ident: ty.ident.clone(), bounds: ty.bounds.clone() })
			}
			_ => None,
		})
		.collect();

	let kind_input = KindInput { lifetimes, types, output_bounds: definition.bounds.clone() };

	let kind_trait_name = generate_name(&kind_input);

	let of_generics = &definition.generics;
	let of_target = &definition.target_type;

	// Generate doc comment
	let doc_comment =
		format!("Generated implementation of `{}` for `{}`.", kind_trait_name, quote!(#brand));

	let (impl_generics_impl, _, impl_generics_where) = impl_generics.split_for_impl();

	quote! {
		#[doc = #doc_comment]
		impl #impl_generics_impl #kind_trait_name for #brand #impl_generics_where {
			type Of #of_generics = #of_target;
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// ===========================================================================
	// impl_kind! Parsing and Generation Tests (Original)
	// ===========================================================================

	/// Tests basic parsing of impl_kind! input.
	///
	/// Verifies that the parser correctly identifies the associated type name "Of".
	#[test]
	fn test_parse_impl_kind() {
		let input = "for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		assert_eq!(parsed.definition.of_ident.to_string(), "Of");
	}

	/// Tests code generation for impl_kind!.
	///
	/// Verifies that the generated impl block contains the correct trait name,
	/// brand type, and associated type definition.
	#[test]
	fn test_impl_kind_generation() {
		let input = "for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl Kind_"));
		assert!(output_str.contains("for OptionBrand"));
		// Note: Bounds on associated types in impl blocks are not emitted because
		// they have no effect in Rust (bounds are only valid in trait definitions).
		assert!(output_str.contains("type Of < 'a , A : 'a > = Option < A >"));
	}

	// ===========================================================================
	// impl_kind! with generics Tests
	// ===========================================================================

	/// Tests impl_kind! with a single impl-level generic parameter.
	///
	/// Verifies that `impl<E> for ResultBrand<E>` correctly passes the
	/// generic parameter to both the impl block and the brand type.
	#[test]
	fn test_impl_kind_with_impl_generics() {
		let input = "impl<E> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		assert_eq!(parsed.definition.of_ident.to_string(), "Of");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
	}

	/// Tests impl_kind! with multiple bounded impl-level generics.
	///
	/// Verifies that bounds on impl generics (e.g., `E: Clone, F: Send`)
	/// are preserved in the generated output.
	#[test]
	fn test_impl_kind_with_multiple_impl_generics() {
		let input = "impl<E: Clone, F: Send> for MyBrand<E, F> { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E : Clone , F : Send > Kind_"));
		assert!(output_str.contains("for MyBrand < E , F >"));
	}

	/// Tests impl_kind! with a path-qualified bound.
	///
	/// Verifies that bounds like `std::fmt::Debug` are correctly preserved.
	#[test]
	fn test_impl_kind_with_bounded_impl_generic() {
		// Test that bounds on impl generics are preserved
		let input = "impl<E: std::fmt::Debug> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E : std :: fmt :: Debug > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
	}

	/// Tests impl_kind! with multiple lifetimes and multiple type parameters.
	///
	/// Verifies that complex signatures with multiple lifetimes and types
	/// are correctly handled.
	#[test]
	fn test_impl_kind_multiple_lifetimes_and_types() {
		let input = "for MyBrand { type Of<'a, 'b, A: 'a, B: 'b> = MyType<'a, 'b, A, B>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl Kind_"));
		assert!(output_str.contains("type Of < 'a , 'b , A : 'a , B : 'b >"));
	}

	// ===========================================================================
	// impl_kind! with where clauses Tests
	// ===========================================================================

	/// Tests impl_kind! with a single where clause bound.
	///
	/// Verifies that `where E: Debug` is correctly parsed and emitted
	/// in the generated impl block.
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

	/// Tests impl_kind! with multiple where clause predicates.
	///
	/// Verifies that `where E: Clone, F: Send` with multiple predicates
	/// is correctly parsed and emitted.
	#[test]
	fn test_impl_kind_with_multiple_where_bounds() {
		let input = "impl<E, F> for MyBrand<E, F> where E: Clone, F: Send { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E , F >"));
		assert!(output_str.contains("where E : Clone , F : Send"));
	}

	/// Tests impl_kind! with multiple trait bounds in a single where predicate.
	///
	/// Verifies that `where E: Clone + Send + Sync` with multiple traits
	/// on one parameter is correctly handled.
	#[test]
	fn test_impl_kind_with_complex_where_bounds() {
		let input = "impl<E> for ResultBrand<E> where E: Clone + Send + Sync { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_impl(parsed);
		let output_str = output.to_string();

		assert!(output_str.contains("where E : Clone + Send + Sync"));
	}
}
