use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use syn::{
	Ident, Lifetime, Result, Token, TypeParamBound,
	parse::{Parse, ParseStream},
	parse_macro_input,
	punctuated::Punctuated,
};

struct KindInput {
	lifetimes: Punctuated<Lifetime, Token![,]>,
	types: Punctuated<TypeInput, Token![,]>,
	output_bounds: Punctuated<TypeParamBound, Token![+]>,
}

struct TypeInput {
	ident: Ident,
	bounds: Punctuated<TypeParamBound, Token![+]>,
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

struct Canonicalizer {
	lifetime_map: BTreeMap<String, usize>,
}

impl Canonicalizer {
	fn new(
		lifetimes: &Punctuated<Lifetime, Token![,]>,
		_types: &Punctuated<TypeInput, Token![,]>,
	) -> Self {
		let mut lifetime_map = BTreeMap::new();
		for (i, lt) in lifetimes.iter().enumerate() {
			lifetime_map.insert(lt.ident.to_string(), i);
		}

		Self { lifetime_map }
	}

	fn canonicalize_bound(
		&self,
		bound: &TypeParamBound,
	) -> String {
		match bound {
			TypeParamBound::Lifetime(lt) => {
				let idx = self.lifetime_map.get(&lt.ident.to_string()).expect("Unknown lifetime");
				format!("l{}", idx)
			}
			TypeParamBound::Trait(tr) => {
				// Simplified trait bound handling: just take the last segment
				let last_segment = tr.path.segments.last().unwrap();
				format!("t{}", last_segment.ident)
			}
			_ => panic!("Unsupported bound type"),
		}
	}

	fn canonicalize_bounds(
		&self,
		bounds: &Punctuated<TypeParamBound, Token![+]>,
	) -> String {
		let mut parts: Vec<String> = bounds.iter().map(|b| self.canonicalize_bound(b)).collect();
		parts.sort(); // Ensure deterministic order
		parts.join("")
	}
}

fn generate_name(input: &KindInput) -> Ident {
	let canon = Canonicalizer::new(&input.lifetimes, &input.types);

	let l_count = input.lifetimes.len();
	let t_count = input.types.len();

	let mut name = format!("Kind_L{}_T{}", l_count, t_count);

	// Type bounds
	for (i, ty) in input.types.iter().enumerate() {
		if !ty.bounds.is_empty() {
			let bounds_str = canon.canonicalize_bounds(&ty.bounds);
			name.push_str(&format!("_B{}{}", i, bounds_str));
		}
	}

	// Output bounds
	if !input.output_bounds.is_empty() {
		let bounds_str = canon.canonicalize_bounds(&input.output_bounds);
		name.push_str(&format!("_O{}", bounds_str));
	}

	format_ident!("{}", name)
}

#[proc_macro]
#[allow(non_snake_case)]
pub fn Kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as KindInput);
	let name = generate_name(&input);
	quote!(#name).into()
}

#[proc_macro]
pub fn def_kind(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as KindInput);
	let name = generate_name(&input);

	let lifetimes = &input.lifetimes;

	let types_with_bounds = input.types.iter().map(|t| {
		let ident = &t.ident;
		let bounds = &t.bounds;
		if bounds.is_empty() {
			quote! { #ident }
		} else {
			quote! { #ident: #bounds }
		}
	});

	let output_bounds = &input.output_bounds;
	let output_bounds_tokens =
		if output_bounds.is_empty() { quote!() } else { quote!(: #output_bounds) };

	let doc_string = format!("Auto-generated Kind trait: {}", name);

	let generics_inner = if input.lifetimes.is_empty() {
		quote! { #(#types_with_bounds),* }
	} else if input.types.is_empty() {
		quote! { #lifetimes }
	} else {
		quote! { #lifetimes, #(#types_with_bounds),* }
	};

	let expanded = quote! {
		#[doc = #doc_string]
		#[allow(non_camel_case_types)]
		pub trait #name {
			type Of < #generics_inner > #output_bounds_tokens;
		}
	};

	expanded.into()
}
