//! Implementation of the `impl_kind!` macro.
//!
//! This module handles the parsing and expansion of the `impl_kind!` macro, which is used
//! to implement a generated `Kind` trait for a specific brand type.

use {
	super::{
		AssociatedType as AssociatedTypeInput,
		AssociatedTypeBase,
		AssociatedTypes,
		generate_name,
	},
	crate::{
		core::Result,
		support::{
			attributes,
			parsing::{
				parse_many,
				parse_non_empty,
			},
		},
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Generics,
		Token,
		Type,
		WhereClause,
		braced,
		parse::{
			Parse,
			ParseStream,
		},
	},
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
pub struct ImplKindInput {
	/// Attributes (including doc comments) for the impl block.
	pub attributes: Vec<syn::Attribute>,
	/// Generics for the impl block (e.g., `impl<T>`).
	pub impl_generics: Generics,
	/// The `for` keyword.
	pub _for_token: Token![for],
	/// The brand type being implemented (e.g., `MyBrand`).
	pub brand: Type,
	/// The brace token surrounding the associated type definitions.
	pub _brace_token: syn::token::Brace,
	/// The associated type definitions inside the braces.
	pub definitions: Vec<AssociatedType>,
}

/// Represents a single associated type definition inside `impl_kind!`.
///
/// Example: `type Of<A> = MyType<A>;`
pub struct AssociatedType {
	/// The common signature parts.
	pub signature: AssociatedTypeBase,
	/// The `=` token.
	pub _eq_token: Token![=],
	/// The concrete type being assigned (e.g., `MyType<A>`).
	pub target_type: Type,
	/// Optional where clause.
	pub where_clause: Option<WhereClause>,
	/// The semicolon.
	pub semi_token: Token![;],
}

impl Parse for ImplKindInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let attributes = input.call(syn::Attribute::parse_outer)?;

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

		let definitions = parse_many(&content)?;
		let definitions = parse_non_empty(
			definitions,
			"Kind implementation must have at least one associated type definition",
		)?;

		Ok(ImplKindInput {
			attributes,
			impl_generics,
			_for_token: for_token,
			brand,
			_brace_token: brace_token,
			definitions,
		})
	}
}

impl Parse for AssociatedType {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let signature =
			AssociatedTypeBase::parse_signature(input, |i| i.peek(Token![=]) || i.peek(Token![;]))?;

		let eq_token: Token![=] = input.parse()?;
		let target_type: Type = input.parse()?;

		let where_clause: Option<WhereClause> =
			if input.peek(Token![where]) { Some(input.parse()?) } else { None };

		let semi_token: Token![;] = input.parse()?;

		Ok(AssociatedType {
			signature,
			_eq_token: eq_token,
			target_type,
			where_clause,
			semi_token,
		})
	}
}

/// Generates the implementation for the `impl_kind!` macro.
///
/// This function takes the parsed input, determines the correct `Kind` trait based on
/// the signature of the associated types, and generates the `impl` block.
pub fn impl_kind_worker(input: ImplKindInput) -> Result<TokenStream> {
	let brand = &input.brand;
	let impl_generics = &input.impl_generics;

	// Convert to KindInput for name generation
	let kind_input = AssociatedTypes {
		associated_types: input
			.definitions
			.iter()
			.map(|def| AssociatedTypeInput {
				signature: def.signature.clone(),
				semi_token: def.semi_token,
			})
			.collect(),
	};
	let kind_trait_name = generate_name(&kind_input)?;

	let assoc_types_impl = input.definitions.iter().map(|def| {
		let ident = &def.signature.name;
		let generics = &def.signature.generics;
		let target = &def.target_type;
		let where_clause = &def.where_clause;
		// Filter out documentation-specific attributes to avoid "unused attribute" warnings
		let attrs = attributes::filter_doc_attributes(&def.signature.attributes);

		quote! {
			#(#attrs)*
			type #ident #generics = #target #where_clause;
		}
	});

	// Generate doc comment
	let doc_comment =
		format!("Generated implementation of `{kind_trait_name}` for `{}`.", quote!(#brand));

	let (impl_generics_impl, _, impl_generics_where) = impl_generics.split_for_impl();

	let attrs = &input.attributes;
	let has_doc = attrs.iter().any(|attr| attr.path().is_ident("doc"));
	let maybe_separator = if has_doc {
		quote! { #[doc = ""] }
	} else {
		quote! {}
	};

	Ok(quote! {
		#[doc = #doc_comment]
		#maybe_separator
		#(#attrs)*
		impl #impl_generics_impl #kind_trait_name for #brand #impl_generics_where {
			#(#assoc_types_impl)*
		}
	})
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
		assert_eq!(parsed.definitions[0].signature.name.to_string(), "Of");
	}

	#[test]
	fn test_parse_impl_kind_multiple() {
		let input = "for MyBrand {
			type Of<A> = MyType<A>;
			type SendOf<B> = MySendType<B>;
		}";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		assert_eq!(parsed.definitions.len(), 2);
		assert_eq!(parsed.definitions[0].signature.name.to_string(), "Of");
		assert_eq!(parsed.definitions[1].signature.name.to_string(), "SendOf");
	}

	#[test]
	fn test_impl_kind_generation() {
		let input = "for OptionBrand { type Of<'a, A: 'a>: 'a = Option<A>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
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

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
	}

	#[test]
	fn test_impl_kind_with_multiple_impl_generics() {
		let input = "impl<E: Clone, F: Send> for MyBrand<E, F> { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E : Clone , F : Send > Kind_"));
		assert!(output_str.contains("for MyBrand < E , F >"));
	}

	#[test]
	fn test_impl_kind_with_bounded_impl_generic() {
		let input = "impl<E: std::fmt::Debug> for ResultBrand<E> { type Of<A> = Result<A, E>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
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

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E > Kind_"));
		assert!(output_str.contains("for ResultBrand < E >"));
		assert!(output_str.contains("where E : std :: fmt :: Debug"));
	}

	#[test]
	fn test_impl_kind_with_multiple_where_bounds() {
		let input = "impl<E, F> for MyBrand<E, F> where E: Clone, F: Send { type Of<A> = MyType<A, E, F>; }";
		let parsed: ImplKindInput = syn::parse_str(input).expect("Failed to parse ImplKindInput");

		let output = impl_kind_worker(parsed).expect("impl_kind_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("impl < E , F >"));
		assert!(output_str.contains("where E : Clone , F : Send"));
	}
}
