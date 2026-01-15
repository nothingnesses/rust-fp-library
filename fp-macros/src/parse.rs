//! Input parsing for Kind macros.
//!
//! This module defines the input structures and parsing logic for the `Kind!` and `def_kind!` macros.
//! It handles parsing of lifetimes, type parameters with bounds, and output bounds.

use syn::{
	Ident, Lifetime, Result, Token, TypeParamBound,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
};

pub struct KindInput {
	pub lifetimes: Punctuated<Lifetime, Token![,]>,
	pub types: Punctuated<TypeInput, Token![,]>,
	pub output_bounds: Punctuated<TypeParamBound, Token![+]>,
}

pub struct TypeInput {
	pub ident: Ident,
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
