//! Common signature components for Higher-Kinded Type (HKT) associated types.

use {
	proc_macro2::TokenStream,
	quote::ToTokens,
	syn::{
		Attribute, Generics, Ident, Token, TypeParamBound, parse::ParseStream,
		punctuated::Punctuated,
	},
};

/// Common components of an associated type definition used across HKT macros.
///
/// This includes the attributes, `type` keyword, name, generics, and optional bounds.
/// Example: `#[doc] type Of<'a, T: 'a>: Display`
#[derive(Debug, Clone)]
pub struct AssociatedTypeBase {
	/// Attributes (e.g., doc comments).
	pub attributes: Vec<Attribute>,
	/// The `type` keyword.
	pub type_token: Token![type],
	/// The name of the associated type (e.g., `Of`).
	pub name: Ident,
	/// Generics for the associated type (e.g., `<'a, T: 'a>`).
	pub generics: Generics,
	/// Optional colon for output bounds.
	pub colon_token: Option<Token![:]>,
	/// Bounds on the associated type output (e.g., `Display`).
	pub output_bounds: Punctuated<TypeParamBound, Token![+]>,
}

impl ToTokens for AssociatedTypeBase {
	fn to_tokens(
		&self,
		tokens: &mut TokenStream,
	) {
		for attr in &self.attributes {
			attr.to_tokens(tokens);
		}
		self.type_token.to_tokens(tokens);
		self.name.to_tokens(tokens);
		self.generics.to_tokens(tokens);
		if let Some(colon) = &self.colon_token {
			colon.to_tokens(tokens);
			self.output_bounds.to_tokens(tokens);
		}
	}
}

impl AssociatedTypeBase {
	/// Parses the signature part of an associated type definition.
	///
	/// This parses everything from attributes up to (but not including) the `=` or `;`.
	pub fn parse_signature(
		input: ParseStream,
		terminator_check: impl Fn(ParseStream) -> bool,
	) -> syn::Result<Self> {
		let attributes = input.call(Attribute::parse_outer)?;
		let type_token: Token![type] = input.parse()?;
		let name: Ident = input.parse()?;
		let generics: Generics = input.parse()?;

		let mut colon_token: Option<Token![:]> = None;
		let mut output_bounds = Punctuated::new();

		if input.peek(Token![:]) {
			colon_token = Some(input.parse()?);
			loop {
				if terminator_check(input) {
					break;
				}
				output_bounds.push_value(input.parse()?);
				if input.peek(Token![+]) {
					output_bounds.push_punct(input.parse()?);
				} else {
					break;
				}
			}
		}

		Ok(AssociatedTypeBase {
			attributes,
			type_token,
			name,
			generics,
			colon_token,
			output_bounds,
		})
	}
}
