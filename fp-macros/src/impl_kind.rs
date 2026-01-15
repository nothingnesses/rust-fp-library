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
