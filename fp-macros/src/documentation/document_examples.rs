use {
	crate::{
		core::{
			Result as OurResult,
			constants::{
				attributes::DOCUMENT_EXAMPLES,
				documentation::RUST_CODE_TAGS,
				macros::ASSERTION_MACROS,
			},
		},
		support::{
			ast::RustAst,
			generate_documentation::insert_doc_comment,
		},
	},
	proc_macro2::TokenStream,
	quote::quote,
};

/// Check whether `code` contains at least one assertion macro invocation.
fn contains_assertion(code: &str) -> bool {
	ASSERTION_MACROS.iter().any(|mac| code.contains(mac))
}

/// State machine for parsing doc comment code blocks.
enum ParseState {
	Normal,
	InRustBlock(Vec<String>),
	InSkippedBlock,
}

/// Extract the content of all `#[doc = "..."]` and `#[doc = concat!(...)]`
/// attributes.
fn extract_doc_content(attrs: &[syn::Attribute]) -> Vec<String> {
	attrs
		.iter()
		.filter_map(|attr| {
			if let syn::Meta::NameValue(nv) = &attr.meta
				&& nv.path.is_ident("doc")
			{
				if let syn::Expr::Lit(lit) = &nv.value
					&& let syn::Lit::Str(s) = &lit.lit
				{
					Some(s.value())
				} else if let syn::Expr::Macro(expr_macro) = &nv.value
					&& expr_macro.mac.path.is_ident("concat")
				{
					Some(extract_concat_string_literals(&expr_macro.mac.tokens))
				} else {
					None
				}
			} else {
				None
			}
		})
		.collect()
}

/// Extract string literal content from `concat!(...)` arguments.
///
/// Parses the token stream inside a `concat!()` invocation and
/// concatenates all string literal arguments. Non-literal arguments
/// (e.g., `stringify!(...)`) are skipped, since the string literal
/// portions are sufficient for detecting code fence boundaries and
/// assertion macros.
fn extract_concat_string_literals(tokens: &proc_macro2::TokenStream) -> String {
	let mut result = String::new();
	for token in tokens.clone() {
		if let proc_macro2::TokenTree::Literal(lit) = token
			&& let Ok(s) = syn::parse2::<syn::LitStr>(proc_macro2::TokenTree::Literal(lit).into())
		{
			result.push_str(&s.value());
		}
	}
	result
}

/// Extract Rust code blocks from doc comment lines.
///
/// Each doc comment attribute (`#[doc = "..."]`) contributes one line.
/// Code fences with tags in [`RUST_CODE_TAGS`] are collected; all other
/// fenced blocks (e.g. `compile_fail`, `ignore`, `text`) are skipped.
fn extract_rust_code_blocks(doc_lines: &[String]) -> Vec<String> {
	let mut blocks = Vec::new();
	let mut state = ParseState::Normal;

	for line in doc_lines {
		let trimmed = line.trim();

		state = match state {
			ParseState::Normal =>
				if let Some(stripped) = trimmed.strip_prefix("```") {
					let tag = stripped.trim();
					if RUST_CODE_TAGS.contains(&tag) {
						ParseState::InRustBlock(Vec::new())
					} else {
						ParseState::InSkippedBlock
					}
				} else {
					ParseState::Normal
				},
			ParseState::InRustBlock(mut lines) =>
				if trimmed == "```" {
					blocks.push(lines.join("\n"));
					ParseState::Normal
				} else {
					lines.push(line.clone());
					ParseState::InRustBlock(lines)
				},
			ParseState::InSkippedBlock =>
				if trimmed == "```" {
					ParseState::Normal
				} else {
					ParseState::InSkippedBlock
				},
		};
	}

	blocks
}

/// Validate that at least one Rust code block exists.
fn validate_code_blocks_exist(code_blocks: &[String]) -> OurResult<()> {
	if code_blocks.is_empty() {
		return Err(syn::Error::new(
			proc_macro2::Span::call_site(),
			format!(
				"#[{DOCUMENT_EXAMPLES}] requires at least one Rust code block in the doc comments (using ``` or ```rust fences)"
			),
		)
		.into());
	}

	Ok(())
}

/// Validate that every Rust code block contains at least one assertion.
fn validate_code_blocks(code_blocks: &[String]) -> OurResult<()> {
	validate_code_blocks_exist(code_blocks)?;

	for (i, code) in code_blocks.iter().enumerate() {
		if !contains_assertion(code) {
			return Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				format!(
					"Code block {} in the doc comments for #[{DOCUMENT_EXAMPLES}] must contain at least one assertion macro (e.g., assert_eq!, assert!)",
					i + 1,
				),
			)
			.into());
		}
	}

	Ok(())
}

/// Worker for the `document_examples` macro.
///
/// Expands `#[document_examples]` into a `### Examples` heading at the
/// attribute's position and validates that every Rust code block in the
/// item's doc comments contains at least one assertion macro invocation.
pub fn document_examples_worker(
	attr: TokenStream,
	item: TokenStream,
) -> OurResult<TokenStream> {
	if !attr.is_empty() {
		return Err(syn::Error::new(
			proc_macro2::Span::call_site(),
			format!(
				"#[{DOCUMENT_EXAMPLES}] does not accept arguments. Example code should be placed in doc comments after this attribute using fenced code blocks."
			),
		)
		.into());
	}

	let mut ast = RustAst::parse(item).map_err(crate::core::Error::Parse)?;

	let is_function = ast.signature().is_some();

	// Check for duplicate #[document_examples]
	let has_duplicate = ast.attributes().iter().any(|a| a.path().is_ident(DOCUMENT_EXAMPLES));
	if has_duplicate {
		return Err(syn::Error::new(
			proc_macro2::Span::call_site(),
			format!("#[{DOCUMENT_EXAMPLES}] should only be applied once per item"),
		)
		.into());
	}

	// Extract and validate doc comment code blocks
	let doc_content = extract_doc_content(ast.attributes());
	let code_blocks = extract_rust_code_blocks(&doc_content);

	if is_function {
		// Functions require assertion macros in code blocks
		validate_code_blocks(&code_blocks)?;
	} else {
		// Non-functions just need at least one code block
		validate_code_blocks_exist(&code_blocks)?;
	}

	// Insert ### Examples heading at the macro's position
	insert_doc_comment(
		ast.attributes(),
		"### Examples\n".to_string(),
		proc_macro2::Span::call_site(),
	);

	Ok(quote!(#ast))
}
