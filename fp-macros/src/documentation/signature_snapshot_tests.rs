//! Snapshot regression tests for HM signature generation.
//!
//! Reads the full `dispatch.rs` module, runs it through `document_module_worker`
//! (the full proc macro pipeline), extracts the generated HM signature doc
//! comments, and asserts them against an insta snapshot. This ensures changes
//! to dispatch analysis, synthetic signature building, or the HM pipeline do
//! not silently break any of the 37 inference wrapper function signatures.

#![expect(
	clippy::unwrap_used,
	clippy::expect_used,
	clippy::panic,
	reason = "Tests use panicking operations for brevity and clarity"
)]

use {
	crate::documentation::document_module_worker,
	proc_macro2::TokenStream,
	std::collections::BTreeMap,
};

// -- Helpers --

/// Extract the body of `mod inner { ... }` from a dispatch submodule source file.
///
/// Each dispatch file (e.g., alt.rs) has `#[document_module] pub(crate) mod inner { ... }`.
/// This extracts the inner module's body using brace counting.
fn extract_inner_body(source: &str) -> &str {
	let inner_start = source
		.find("mod inner {")
		.or_else(|| source.find("mod inner{"))
		.expect("Could not find `mod inner` in source");
	let brace_start = source[inner_start ..].find('{').unwrap() + inner_start;
	let mut depth = 0;
	let mut inner_end = brace_start;
	for (i, ch) in source[brace_start ..].char_indices() {
		match ch {
			'{' => depth += 1,
			'}' => {
				depth -= 1;
				if depth == 0 {
					inner_end = brace_start + i;
					break;
				}
			}
			_ => {}
		}
	}
	&source[brace_start + 1 .. inner_end]
}

/// Run a dispatch submodule's inner body through `document_module_worker` and
/// extract all HM signatures from the generated doc comments on `Item::Fn` items.
///
/// Returns a sorted map of function_name -> signature_string.
fn extract_signatures(source: &str) -> BTreeMap<String, String> {
	let inner_body = extract_inner_body(source);
	let tokens: TokenStream =
		inner_body.parse().unwrap_or_else(|e| panic!("Failed to parse inner module body: {e}"));
	let result = document_module_worker(TokenStream::new(), tokens)
		.unwrap_or_else(|e| panic!("document_module_worker failed: {e}"));
	let file: syn::File = syn::parse2(result).expect("Failed to parse worker output");

	let mut signatures = BTreeMap::new();
	collect_fn_signatures(&file.items, &mut signatures);
	signatures
}

/// Recursively collect HM signatures from `Item::Fn` items, descending into submodules.
fn collect_fn_signatures(
	items: &[syn::Item],
	signatures: &mut BTreeMap<String, String>,
) {
	for item in items {
		match item {
			syn::Item::Fn(item_fn) => {
				let fn_name = item_fn.sig.ident.to_string();
				for attr in &item_fn.attrs {
					if let syn::Meta::NameValue(meta) = &attr.meta
						&& attr.path().is_ident("doc")
						&& let syn::Expr::Lit(expr_lit) = &meta.value
						&& let syn::Lit::Str(lit_str) = &expr_lit.lit
					{
						let value = lit_str.value();
						if value.contains("forall") {
							let sig = value.trim().trim_matches('`').to_string();
							signatures.insert(fn_name.clone(), sig);
						}
					}
				}
			}
			syn::Item::Mod(item_mod) => {
				// Skip the `explicit` submodule; we only want inference wrapper signatures
				if item_mod.ident == "explicit" {
					continue;
				}
				if let Some((_, inner_items)) = &item_mod.content {
					collect_fn_signatures(inner_items, signatures);
				}
			}
			_ => {}
		}
	}
}

/// Format signatures as a multi-line string for snapshotting.
fn format_signatures(sigs: &BTreeMap<String, String>) -> String {
	sigs.iter().map(|(name, sig)| format!("{name}: {sig}")).collect::<Vec<_>>().join("\n")
}

// -- Snapshot test --

/// Macro to define a dispatch signature test for a submodule file.
macro_rules! dispatch_test {
	($name:ident, $file:expr) => {
		#[test]
		fn $name() {
			let source = include_str!(concat!(
				env!("CARGO_MANIFEST_DIR"),
				"/../fp-library/src/dispatch/",
				$file,
			));
			let sigs = extract_signatures(source);
			assert!(!sigs.is_empty(), concat!("No HM signatures found in ", $file));
			let output = format_signatures(&sigs);
			insta::assert_snapshot!(output);
		}
	};
}

dispatch_test!(alt_signatures, "alt.rs");
dispatch_test!(apply_first_signatures, "apply_first.rs");
dispatch_test!(apply_second_signatures, "apply_second.rs");
dispatch_test!(bifoldable_signatures, "bifoldable.rs");
dispatch_test!(bifunctor_signatures, "bifunctor.rs");
dispatch_test!(bitraversable_signatures, "bitraversable.rs");
dispatch_test!(compactable_signatures, "compactable.rs");
dispatch_test!(filterable_signatures, "filterable.rs");
dispatch_test!(filterable_with_index_signatures, "filterable_with_index.rs");
dispatch_test!(foldable_signatures, "foldable.rs");
dispatch_test!(foldable_with_index_signatures, "foldable_with_index.rs");
dispatch_test!(functor_signatures, "functor.rs");
dispatch_test!(functor_with_index_signatures, "functor_with_index.rs");
dispatch_test!(lift_signatures, "lift.rs");
dispatch_test!(semimonad_signatures, "semimonad.rs");
dispatch_test!(traversable_signatures, "traversable.rs");
dispatch_test!(traversable_with_index_signatures, "traversable_with_index.rs");
dispatch_test!(witherable_signatures, "witherable.rs");
