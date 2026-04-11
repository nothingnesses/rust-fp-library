//! Implementation of the `Apply!` macro.
//!
//! This module handles the parsing and expansion of the `Apply!` macro, which is used
//! to apply a Higher-Kinded Type (HKT) "brand" to a set of generic arguments.

use {
	super::AssociatedTypes,
	crate::{
		core::{
			Result,
			constants::macros::{
				INFERABLE_BRAND_MACRO,
				KIND_MACRO,
			},
		},
		generate_inferable_brand_name,
		generate_name,
	},
	proc_macro2::{
		Delimiter,
		TokenStream,
		TokenTree,
	},
	quote::quote,
	syn::{
		AngleBracketedGenericArguments,
		Ident,
		Token,
		Type,
		parse::{
			Parse,
			ParseStream,
		},
	},
};

/// Resolves `InferableBrand!(SIG)` invocations in a token stream.
///
/// Scans the token tree for an `InferableBrand` identifier followed by `!`
/// and a parenthesized group. Replaces the three-token sequence with the
/// resolved `InferableBrand_{hash}` identifier. This enables `Apply!` to
/// accept brands like `<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand`.
///
/// Processes recursively into groups (parentheses, brackets, braces) so that
/// nested occurrences are also resolved.
pub fn resolve_inferable_brand(input: TokenStream) -> std::result::Result<TokenStream, syn::Error> {
	let tokens: Vec<TokenTree> = input.into_iter().collect();
	let mut output = Vec::new();
	let mut i = 0;

	while i < tokens.len() {
		// Look for `InferableBrand ! ( ... )` pattern
		if let Some([TokenTree::Ident(ident), TokenTree::Punct(bang), TokenTree::Group(group), ..]) =
			tokens.get(i ..)
			&& ident == INFERABLE_BRAND_MACRO
			&& bang.as_char() == '!'
			&& group.delimiter() == Delimiter::Parenthesis
		{
			let sig: AssociatedTypes = syn::parse2(group.stream())?;
			let resolved = generate_inferable_brand_name(&sig)
				.map_err(|e| syn::Error::new(ident.span(), e.to_string()))?;
			output.push(TokenTree::Ident(resolved));
			i += 3;
			continue;
		}

		// Recurse into groups
		if let Some(TokenTree::Group(group)) = tokens.get(i) {
			let resolved_inner = resolve_inferable_brand(group.stream())?;
			let mut new_group = proc_macro2::Group::new(group.delimiter(), resolved_inner);
			new_group.set_span(group.span());
			output.push(TokenTree::Group(new_group));
		} else if let Some(token) = tokens.get(i) {
			output.push(token.clone());
		}
		i += 1;
	}

	Ok(output.into_iter().collect())
}

/// Input structure for the `Apply!` macro.
///
/// Syntax: `Apply!(<Brand as Kind!( type Of...; )>::Of<T, U>)`
///
/// The brand position also supports `InferableBrand!(SIG)` invocations,
/// which are resolved to `InferableBrand_{hash}` before parsing:
/// `Apply!(<<FA as InferableBrand!( type Of<'a, A: 'a>: 'a; )>::Brand as Kind!( type Of...; )>::Of<T, U>)`
#[derive(Debug)]
pub struct ApplyInput {
	/// The brand type (e.g., `OptionBrand`).
	pub brand: Type,
	/// The `Kind` signature definition.
	pub kind_input: AssociatedTypes,
	/// The associated type name to project (e.g., `Of`).
	pub assoc_name: Ident,
	/// The generic arguments for the projection (e.g., `<T, U>`).
	pub args: AngleBracketedGenericArguments,
}

impl Parse for ApplyInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		// Parse `<`
		input.parse::<Token![<]>()?;

		// Parse Brand
		let brand: Type = input.parse()?;

		// Parse `as`
		input.parse::<Token![as]>()?;

		// Parse `Kind` identifier
		let kind_ident: Ident = input.parse()?;
		if kind_ident != KIND_MACRO {
			return Err(syn::Error::new(kind_ident.span(), format!("expected `{KIND_MACRO}`")));
		}

		// Parse `!`
		input.parse::<Token![!]>()?;

		// Parse `(...)` containing KindInput
		let content;
		syn::parenthesized!(content in input);
		let kind_input: AssociatedTypes = content.parse()?;

		// Parse `>`
		input.parse::<Token![>]>()?;

		// Parse `::`
		input.parse::<Token![::]>()?;

		// Parse Assoc Name
		let assoc_name: Ident = input.parse()?;

		// Parse `<...>` Args
		let args: AngleBracketedGenericArguments = input.parse()?;

		Ok(ApplyInput {
			brand,
			kind_input,
			assoc_name,
			args,
		})
	}
}

/// Generates the implementation for the `Apply!` macro.
pub fn apply_worker(input: ApplyInput) -> Result<TokenStream> {
	let brand = &input.brand;
	let kind_name = generate_name(&input.kind_input)?;
	let assoc_name = &input.assoc_name;
	let args = &input.args;

	Ok(quote! {
		<#brand as #kind_name>::#assoc_name #args
	})
}

#[cfg(test)]
mod tests {
	use {
		super::*,
		syn::parse_str,
	};

	#[test]
	fn test_parse_apply_new_syntax() {
		let input = "<OptionBrand as Kind!(type Of<'a, T>: 'a;)>::Of<'static, i32>";
		let parsed: ApplyInput = parse_str(input).expect("Failed to parse ApplyInput");

		assert_eq!(parsed.assoc_name.to_string(), "Of");
		assert_eq!(parsed.kind_input.associated_types.len(), 1);
		assert_eq!(parsed.args.args.len(), 2);
	}

	#[test]
	fn test_apply_generation_new_syntax() {
		let input = "<OptionBrand as Kind!(type Of<'a, T>: 'a;)>::Of<'static, i32>";
		let parsed: ApplyInput = parse_str(input).expect("Failed to parse ApplyInput");

		let output = apply_worker(parsed).expect("apply_worker failed");
		let output_str = output.to_string();

		assert!(output_str.contains("< OptionBrand as Kind_"));
		assert!(output_str.contains(":: Of < 'static , i32 >"));
	}

	#[test]
	fn test_resolve_inferable_brand_in_apply() {
		// Simulate: Apply!(<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>)
		let input: TokenStream = "<<FA as InferableBrand!(type Of<'a, A: 'a>: 'a;)>::Brand as Kind!(type Of<'a, T: 'a>: 'a;)>::Of<'a, B>"
			.parse()
			.unwrap();
		let resolved = resolve_inferable_brand(input).expect("resolution failed");
		let resolved_str = resolved.to_string();

		// InferableBrand!(type Of<'a, A: 'a>: 'a;) should resolve to InferableBrand_cdc7cd43dac7585f
		assert!(
			resolved_str.contains("InferableBrand_cdc7cd43dac7585f"),
			"Expected resolved InferableBrand name, got: {resolved_str}"
		);
		// The resolved stream should be parseable as ApplyInput
		let parsed: ApplyInput =
			syn::parse2(resolved).expect("Failed to parse resolved stream as ApplyInput");
		let output = apply_worker(parsed).expect("apply_worker failed");
		let output_str = output.to_string();
		assert!(
			output_str.contains("InferableBrand_cdc7cd43dac7585f"),
			"Expected InferableBrand in output, got: {output_str}"
		);
	}
}
