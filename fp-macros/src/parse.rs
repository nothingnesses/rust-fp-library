//! Input parsing for Kind macros.
//!
//! This module defines the input structures and parsing logic for the `Kind!` and `def_kind!` macros.
//! It handles parsing of lifetimes, type parameters with bounds, and output bounds.

use syn::{
	Ident, Lifetime, Result, Token, TypeParamBound,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

/// Represents the parsed input for a Kind signature.
///
/// This structure captures the lifetimes, type parameters (with bounds), and output bounds
/// that define a Higher-Kinded Type signature.
pub struct KindInput {
	/// Lifetimes involved in the signature.
	pub lifetimes: Punctuated<Lifetime, Token![,]>,
	/// Type parameters involved in the signature.
	pub types: Punctuated<TypeInput, Token![,]>,
	/// Bounds on the output type.
	pub output_bounds: Punctuated<TypeParamBound, Token![+]>,
}

/// Represents a single type parameter in a Kind signature.
pub struct TypeInput {
	/// The identifier of the type parameter.
	pub ident: Ident,
	/// Bounds on the type parameter.
	pub bounds: Punctuated<TypeParamBound, Token![+]>,
}

impl Parse for KindInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let content;
		let _ = syn::parenthesized!(content in input);
		let lifetimes = content.parse_terminated(Lifetime::parse, Token![,])?;

		input.parse::<Token![,]>()?;

		let content;
		let _ = syn::parenthesized!(content in input);
		let types = content.parse_terminated(TypeInput::parse, Token![,])?;

		input.parse::<Token![,]>()?;

		let content;
		let _ = syn::parenthesized!(content in input);
		let output_bounds = content.parse_terminated(TypeParamBound::parse, Token![+])?;

		Ok(KindInput { lifetimes, types, output_bounds })
	}
}

impl Parse for TypeInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let ident: Ident = input.parse()?;
		let bounds = if input.peek(Token![:]) {
			input.parse::<Token![:]>()?;
			Punctuated::parse_terminated_with(input, TypeParamBound::parse)?
		} else {
			Punctuated::new()
		};
		Ok(TypeInput { ident, bounds })
	}
}
