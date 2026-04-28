//! Code generation for the [`effects!`](crate::effects) and
//! [`raw_effects!`](crate::raw_effects) macros.
//!
//! Both macros accept a comma-separated list of brand types and emit a
//! right-nested
//! [`CoproductBrand`](https://docs.rs/fp-library/latest/fp_library/brands/struct.CoproductBrand.html)
//! chain terminated in
//! [`CNilBrand`](https://docs.rs/fp-library/latest/fp_library/brands/struct.CNilBrand.html).
//! The public `effects!` wraps each brand in
//! [`CoyonedaBrand`](https://docs.rs/fp-library/latest/fp_library/brands/struct.CoyonedaBrand.html);
//! the internal `raw_effects!` does not. Both share the lexical-sort
//! helper in [`crate::effects::row_sort`].
//!
//! The file is named `effects_macro.rs` (rather than `effects.rs`) to
//! avoid clippy's `module_inception` lint on the otherwise-nested
//! `crate::effects::effects` path. See
//! [`docs/plans/effects/deviations.md`](https://github.com/nothingnesses/rust-fp-library/blob/main/docs/plans/effects/deviations.md)
//! step 8 entry for the rename rationale.

use {
	crate::effects::row_sort::parse_and_sort_types,
	proc_macro2::TokenStream,
	quote::quote,
};

/// Worker for the public `effects!` macro: emits a Coyoneda-wrapped,
/// right-nested `CoproductBrand` chain terminating in `CNilBrand`.
///
/// Empty input produces just `CNilBrand`.
pub fn effects_worker(input: TokenStream) -> syn::Result<TokenStream> {
	let sorted = parse_and_sort_types(input)?;
	let mut acc: TokenStream = quote! { ::fp_library::brands::CNilBrand };
	for ty in sorted.into_iter().rev() {
		acc = quote! {
			::fp_library::brands::CoproductBrand<
				::fp_library::brands::CoyonedaBrand<#ty>,
				#acc
			>
		};
	}
	Ok(acc)
}

/// Worker for the internal `raw_effects!` macro: emits an un-wrapped,
/// right-nested `CoproductBrand` chain terminating in `CNilBrand`.
///
/// Each input type is emitted directly as a row variant without
/// `CoyonedaBrand` wrapping. Used by fp-library-internal code (test
/// fixtures, lower-level combinators) where the row already supplies
/// `Functor`-providing brands directly. Not part of the public surface.
///
/// Empty input produces just `CNilBrand`.
pub fn raw_effects_worker(input: TokenStream) -> syn::Result<TokenStream> {
	let sorted = parse_and_sort_types(input)?;
	let mut acc: TokenStream = quote! { ::fp_library::brands::CNilBrand };
	for ty in sorted.into_iter().rev() {
		acc = quote! {
			::fp_library::brands::CoproductBrand<#ty, #acc>
		};
	}
	Ok(acc)
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking operations for brevity and clarity")]
mod tests {
	use super::*;

	#[test]
	fn effects_empty_yields_cnil() {
		let out = effects_worker(quote! {}).expect("worker failed").to_string();
		assert_eq!(out, ":: fp_library :: brands :: CNilBrand");
	}

	#[test]
	fn effects_single_brand_wraps_in_coyoneda() {
		let out = effects_worker(quote! { IdentityBrand }).expect("worker failed").to_string();
		assert!(out.contains("CoyonedaBrand"));
		assert!(out.contains("IdentityBrand"));
		assert!(out.contains("CNilBrand"));
	}

	#[test]
	fn effects_two_brands_canonical_order_independent_of_input() {
		let a = effects_worker(quote! { OptionBrand, IdentityBrand })
			.expect("worker failed")
			.to_string();
		let b = effects_worker(quote! { IdentityBrand, OptionBrand })
			.expect("worker failed")
			.to_string();
		assert_eq!(a, b);
	}

	#[test]
	fn raw_effects_skips_coyoneda_wrap() {
		let out = raw_effects_worker(quote! { IdentityBrand }).expect("worker failed").to_string();
		assert!(!out.contains("CoyonedaBrand"));
		assert!(out.contains("IdentityBrand"));
		assert!(out.contains("CNilBrand"));
	}

	#[test]
	fn raw_effects_canonical_order_independent_of_input() {
		let a = raw_effects_worker(quote! { OptionBrand, IdentityBrand })
			.expect("worker failed")
			.to_string();
		let b = raw_effects_worker(quote! { IdentityBrand, OptionBrand })
			.expect("worker failed")
			.to_string();
		assert_eq!(a, b);
	}
}
