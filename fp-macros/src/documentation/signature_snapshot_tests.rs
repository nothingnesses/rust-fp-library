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
dispatch_test!(contravariant_signatures, "contravariant.rs");
dispatch_test!(filterable_signatures, "filterable.rs");
dispatch_test!(filterable_with_index_signatures, "filterable_with_index.rs");
dispatch_test!(foldable_signatures, "foldable.rs");
dispatch_test!(foldable_with_index_signatures, "foldable_with_index.rs");
dispatch_test!(functor_signatures, "functor.rs");
dispatch_test!(functor_with_index_signatures, "functor_with_index.rs");
dispatch_test!(lift_signatures, "lift.rs");
dispatch_test!(semiapplicative_signatures, "semiapplicative.rs");
dispatch_test!(semimonad_signatures, "semimonad.rs");
dispatch_test!(traversable_signatures, "traversable.rs");
dispatch_test!(traversable_with_index_signatures, "traversable_with_index.rs");
dispatch_test!(witherable_signatures, "witherable.rs");

// -- Edge case tests for signature generation --
//
// These use synthetic dispatch module code (not real files) to test
// unusual inputs and graceful fallback behavior.

/// Run synthetic code through `document_module_worker` and extract signatures.
/// The code should be the body of a module (items, not wrapped in mod { }).
fn extract_synthetic_signatures(code: &str) -> BTreeMap<String, String> {
	let tokens: TokenStream =
		code.parse().unwrap_or_else(|e| panic!("Failed to parse synthetic code: {e}"));
	let result = document_module_worker(TokenStream::new(), tokens)
		.unwrap_or_else(|e| panic!("document_module_worker failed: {e}"));
	let file: syn::File = syn::parse2(result).expect("Failed to parse worker output");
	let mut signatures = BTreeMap::new();
	collect_fn_signatures(&file.items, &mut signatures);
	signatures
}

/// Check that a synthetic module produces NO HM signatures (e.g., fallback to standalone macro).
fn assert_no_dispatch_signatures(code: &str) {
	let tokens: TokenStream =
		code.parse().unwrap_or_else(|e| panic!("Failed to parse synthetic code: {e}"));
	let result = document_module_worker(TokenStream::new(), tokens)
		.unwrap_or_else(|e| panic!("document_module_worker failed: {e}"));
	let file: syn::File = syn::parse2(result).expect("Failed to parse worker output");
	let mut signatures = BTreeMap::new();
	collect_fn_signatures(&file.items, &mut signatures);
	assert!(signatures.is_empty(), "Expected no dispatch signatures, got: {signatures:?}");
}

#[test]
fn edge_missing_kind_hash_falls_back() {
	// Dispatch trait without Kind_* bound on Brand; build_synthetic_signature
	// should return None, leaving #[document_signature] for the standalone macro.
	// The standalone macro won't run in test context, so no "forall" doc is generated.
	assert_no_dispatch_signatures(
		r#"
		trait NoKindDispatch<'a, Brand, A: 'a, B: 'a, FA, Marker> {
			fn dispatch(self, fa: FA) -> ();
		}
		impl<'a, Brand, A, B, F>
			NoKindDispatch<'a, Brand, A, B, (), Val> for F
		where
			Brand: Functor,
			A: 'a,
			B: 'a,
			F: Fn(A) -> B + 'a,
		{
			fn dispatch(self, fa: ()) -> () {}
		}
		struct Val;

		#[document_signature]
		pub fn my_map<'a, FA, A: 'a, B: 'a, Marker>(
			f: impl NoKindDispatch<'a, FA, A, B, FA, Marker>,
			fa: FA,
		) -> ()
		{
			todo!()
		}
		"#,
	);
}

#[test]
fn edge_no_document_signature_attribute_skipped() {
	// Function WITHOUT #[document_signature] should not produce a signature
	assert_no_dispatch_signatures(
		r#"
		trait FunctorDispatch<'a, Brand: Kind_abc123, A: 'a, B: 'a, FA, Marker> {
			fn dispatch(self, fa: FA) -> ();
		}
		impl<'a, Brand, A, B, F>
			FunctorDispatch<'a, Brand, A, B, (), Val> for F
		where
			Brand: Functor,
			A: 'a,
			B: 'a,
			F: Fn(A) -> B + 'a,
		{
			fn dispatch(self, fa: ()) -> () {}
		}
		struct Val;

		pub fn map_no_doc<'a, FA, A: 'a, B: 'a, Marker>(
			f: impl FunctorDispatch<'a, FA, A, B, FA, Marker>,
			fa: FA,
		) -> ()
		{
			todo!()
		}
		"#,
	);
}

#[test]
fn edge_document_signature_without_dispatch_trait_left_for_standalone() {
	// Function with #[document_signature] but no dispatch trait reference.
	// The attribute should be left for the standalone macro (not removed).
	// Since the standalone macro doesn't run in test context, no "forall" appears.
	assert_no_dispatch_signatures(
		r#"
		struct Val;

		#[document_signature]
		pub fn plain_fn<A, B>(a: A) -> B {
			todo!()
		}
		"#,
	);
}

#[test]
fn edge_simple_dispatch_produces_correct_signature() {
	// Minimal functor-like dispatch; verify the full pipeline produces correct output
	let sigs = extract_synthetic_signatures(
		r#"
		trait MapDispatch<'a, Brand: Kind_abc123, A: 'a, B: 'a, FA, Marker> {
			fn dispatch(self, fa: FA) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>);
		}
		impl<'a, Brand, A, B, F>
			MapDispatch<
				'a,
				Brand,
				A,
				B,
				Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
				Val,
			> for F
		where
			Brand: Functor,
			A: 'a,
			B: 'a,
			F: Fn(A) -> B + 'a,
		{
			fn dispatch(self, fa: Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>)) -> Apply!(<Brand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) { todo!() }
		}
		struct Val;

		#[document_signature]
		pub fn my_map<'a, FA, A: 'a, B: 'a, Marker>(
			f: impl MapDispatch<'a, <FA as InferableBrand_abc123>::Brand, A, B, FA, Marker>,
			fa: FA,
		) -> <<FA as InferableBrand_abc123>::Brand as Kind_abc123>::Of<'a, B>
		where
			FA: InferableBrand_abc123
				+ MapDispatch<'a, <FA as InferableBrand_abc123>::Brand, A, B, FA, Marker>,
		{
			todo!()
		}
		"#,
	);

	assert_eq!(sigs.len(), 1);
	assert_eq!(
		sigs.get("my_map").unwrap(),
		"forall Brand A B. Functor Brand => (A -> B, Brand A) -> Brand B"
	);
}

#[test]
fn edge_bifunctor_two_element_container() {
	// Two-element brand application (bimap pattern with two-arg Kind)
	let sigs = extract_synthetic_signatures(
		r#"
		trait BimapDispatch<'a, Brand: Kind_abc123, A: 'a, B: 'a, C: 'a, D: 'a, FA, Marker> {
			fn dispatch(self, fa: FA) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>);
		}
		impl<'a, Brand, A, B, C, D, F, G>
			BimapDispatch<
				'a,
				Brand,
				A,
				B,
				C,
				D,
				Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>),
				Val,
			> for (F, G)
		where
			Brand: Bifunctor,
			A: 'a,
			B: 'a,
			C: 'a,
			D: 'a,
			F: Fn(A) -> B + 'a,
			G: Fn(C) -> D + 'a,
		{
			fn dispatch(self, fa: Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, A, C>)) -> Apply!(<Brand as Kind!( type Of<'a, A: 'a, B: 'a>: 'a; )>::Of<'a, B, D>) { todo!() }
		}
		struct Val;

		#[document_signature]
		pub fn my_bimap<'a, FA, A: 'a, B: 'a, C: 'a, D: 'a, Marker>(
			fg: impl BimapDispatch<'a, <FA as InferableBrand_abc123>::Brand, A, B, C, D, FA, Marker>,
			fa: FA,
		) -> <<FA as InferableBrand_abc123>::Brand as Kind_abc123>::Of<'a, B, D>
		where
			FA: InferableBrand_abc123,
		{
			todo!()
		}
		"#,
	);

	assert_eq!(sigs.len(), 1);
	assert_eq!(
		sigs.get("my_bimap").unwrap(),
		"forall Brand A B C D. Bifunctor Brand => ((A -> B, C -> D), Brand A C) -> Brand B D"
	);
}

#[test]
fn edge_manual_override_emits_provided_string() {
	// #[document_signature("custom signature")] should emit the string directly
	let sigs = extract_synthetic_signatures(
		r#"
		struct Val;

		#[document_signature("forall A B. (A -> B) -> A -> B")]
		pub fn my_fn<A, B>(f: fn(A) -> B, a: A) -> B {
			f(a)
		}
		"#,
	);

	assert_eq!(sigs.len(), 1);
	assert_eq!(sigs.get("my_fn").unwrap(), "forall A B. (A -> B) -> A -> B");
}

#[test]
fn edge_manual_override_in_dispatch_context() {
	// Manual override should take precedence over dispatch-aware generation
	let sigs = extract_synthetic_signatures(
		r#"
		trait MapDispatch<'a, Brand: Kind_abc123, A: 'a, B: 'a, FA, Marker> {
			fn dispatch(self, fa: FA) -> ();
		}
		impl<'a, Brand, A, B, F>
			MapDispatch<'a, Brand, A, B, (), Val> for F
		where
			Brand: Functor,
			A: 'a,
			B: 'a,
			F: Fn(A) -> B + 'a,
		{
			fn dispatch(self, fa: ()) -> () {}
		}
		struct Val;

		#[document_signature("forall F A B. Functor F => (A -> B, F A) -> F B")]
		pub fn my_map<'a, FA, A: 'a, B: 'a, Marker>(
			f: impl MapDispatch<'a, <FA as InferableBrand_abc123>::Brand, A, B, FA, Marker>,
			fa: FA,
		) -> ()
		where
			FA: InferableBrand_abc123,
		{
			todo!()
		}
		"#,
	);

	assert_eq!(sigs.len(), 1);
	// Should use the manual override, not the auto-generated signature
	assert_eq!(sigs.get("my_map").unwrap(), "forall F A B. Functor F => (A -> B, F A) -> F B");
}
