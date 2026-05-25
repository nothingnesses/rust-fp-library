use {
	crate::{
		core::{
			Result as OurResult,
			constants::{
				attributes::DOCUMENT_EXAMPLES,
				documentation::{
					ASSERTION_MACROS,
					REASON,
					RUST_CODE_TAGS,
					SINGLE_ARGUMENT_ASSERTION_MACROS,
					SKIP_CALL_CHECK,
					TWO_ARGUMENT_ASSERTION_MACROS,
					WILDCARD_ONLY_ASSERTION_PREFIXES,
					WILDCARD_STRUCT_MATCH_FRAGMENT,
				},
			},
		},
		support::{
			ast::RustAst,
			attributes::reject_duplicate_attribute,
			generate_documentation::insert_doc_comment,
		},
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Expr,
		LitStr,
		Token,
		parse::Parser,
		visit::Visit,
	},
};

#[derive(Debug, Default)]
struct DocumentExamplesOptions {
	skip_call_check: bool,
	reason: Option<String>,
}

fn parse_document_examples_options(attr: TokenStream) -> OurResult<DocumentExamplesOptions> {
	if attr.is_empty() {
		return Ok(DocumentExamplesOptions::default());
	}

	let mut options = DocumentExamplesOptions::default();

	let parser = syn::meta::parser(|meta| {
		if meta.path.is_ident(SKIP_CALL_CHECK) {
			if options.skip_call_check {
				return Err(meta.error(format!("duplicate `{SKIP_CALL_CHECK}` argument")));
			}
			if meta.input.peek(Token![=]) {
				return Err(meta.error(format!("`{SKIP_CALL_CHECK}` does not take a value")));
			}
			options.skip_call_check = true;
			Ok(())
		} else if meta.path.is_ident(REASON) {
			if options.reason.is_some() {
				return Err(meta.error(format!("duplicate `{REASON}` argument")));
			}
			let value = meta.value()?;
			let reason = value.parse::<LitStr>()?;
			let reason_value = reason.value();
			if reason_value.trim().is_empty() {
				return Err(syn::Error::new(
					reason.span(),
					format!("`{REASON}` for #[{DOCUMENT_EXAMPLES}] cannot be empty"),
				));
			}
			options.reason = Some(reason_value);
			Ok(())
		} else {
			Err(meta.error(format!(
				"unsupported #[{DOCUMENT_EXAMPLES}] argument; expected `{SKIP_CALL_CHECK}` or `{REASON} = \"...\"`",
			)))
		}
	});
	parser.parse2(attr)?;

	match (options.skip_call_check, options.reason.is_some()) {
		(true, true) => Ok(options),
		(true, false) => Err(syn::Error::new(
			proc_macro2::Span::call_site(),
			format!(
				"#[{DOCUMENT_EXAMPLES}({SKIP_CALL_CHECK})] must include `{REASON} = \"...\"`; use #[{DOCUMENT_EXAMPLES}({SKIP_CALL_CHECK}, {REASON} = \"explain why direct call validation is impossible\")]",
			),
		)
		.into()),
		(false, true) => Err(syn::Error::new(
			proc_macro2::Span::call_site(),
			format!("`{REASON}` is only supported when `{SKIP_CALL_CHECK}` is present"),
		)
		.into()),
		(false, false) => Ok(options),
	}
}

/// Check whether `code` contains at least one assertion macro invocation.
fn contains_assertion(code: &str) -> bool {
	ASSERTION_MACROS.iter().any(|mac| code.contains(mac))
}

/// Check whether `code` contains an assertion over only literals and
/// operators.
///
/// Assertions like `assert_eq!(2 + 1, 3)` and `assert!(1 + 1 == 2)`
/// can be varied infinitely, so exact string patterns are the wrong
/// tool. This parser-based check rejects assertions whose checked
/// expression is built only from literals, grouping, unary operators,
/// binary operators, casts, arrays, tuples, and references. A useful
/// example should assert a value produced by the documented API.
fn contains_literal_only_assertion(code: &str) -> bool {
	let wrapped = format!("{{\n{}\n}}", normalize_doctest_code_for_parsing(code));
	let Ok(block) = syn::parse_str::<syn::Block>(&wrapped) else {
		return false;
	};

	let mut visitor = LiteralOnlyAssertionVisitor {
		found_literal_only_assertion: false,
	};
	visitor.visit_block(&block);
	visitor.found_literal_only_assertion
}

fn normalize_doctest_code_for_parsing(code: &str) -> String {
	let mut normalized = String::new();

	for line in code.lines() {
		let trimmed = line.trim_start();
		let indent_len = line.len() - trimmed.len();
		let visible_line = trimmed.strip_prefix("# ").unwrap_or(trimmed);

		if visible_line.trim_start().starts_with("#![") {
			continue;
		}

		normalized.push_str(&line[.. indent_len]);
		normalized.push_str(visible_line);
		normalized.push('\n');
	}

	normalized
}

struct LiteralOnlyAssertionVisitor {
	found_literal_only_assertion: bool,
}

impl<'ast> Visit<'ast> for LiteralOnlyAssertionVisitor {
	fn visit_macro(
		&mut self,
		mac: &'ast syn::Macro,
	) {
		if assertion_macro_is_literal_only(mac) {
			self.found_literal_only_assertion = true;
			return;
		}

		syn::visit::visit_macro(self, mac);
	}

	fn visit_expr_macro(
		&mut self,
		expr_macro: &'ast syn::ExprMacro,
	) {
		if assertion_macro_is_literal_only(&expr_macro.mac) {
			self.found_literal_only_assertion = true;
			return;
		}

		syn::visit::visit_expr_macro(self, expr_macro);
	}
}

fn assertion_macro_is_literal_only(mac: &syn::Macro) -> bool {
	let Ok(args) = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated
		.parse2(mac.tokens.clone())
	else {
		return false;
	};

	if SINGLE_ARGUMENT_ASSERTION_MACROS.iter().any(|name| mac.path.is_ident(name)) {
		return args.first().is_some_and(expr_is_literal_only);
	}

	if TWO_ARGUMENT_ASSERTION_MACROS.iter().any(|name| mac.path.is_ident(name)) {
		return args.first().zip(args.iter().nth(1)).is_some_and(|(left, right)| {
			expr_is_literal_only(left) && expr_is_literal_only(right)
		});
	}

	false
}

fn expr_is_literal_only(expr: &syn::Expr) -> bool {
	match expr {
		syn::Expr::Array(expr) => expr.elems.iter().all(expr_is_literal_only),
		syn::Expr::Binary(expr) =>
			expr_is_literal_only(&expr.left) && expr_is_literal_only(&expr.right),
		syn::Expr::Cast(expr) => expr_is_literal_only(&expr.expr),
		syn::Expr::Group(expr) => expr_is_literal_only(&expr.expr),
		syn::Expr::Lit(_) => true,
		syn::Expr::Paren(expr) => expr_is_literal_only(&expr.expr),
		syn::Expr::Reference(expr) => expr_is_literal_only(&expr.expr),
		syn::Expr::Tuple(expr) => expr.elems.iter().all(expr_is_literal_only),
		syn::Expr::Unary(expr) => expr_is_literal_only(&expr.expr),
		_ => false,
	}
}

/// Check whether `code` contains a wildcard-only variant assertion.
///
/// Assertions like `assert!(matches!(value, Variant { .. }))` prove
/// only that a constructor was produced. They do not demonstrate how
/// the documented item is supposed to be used or verify the fields,
/// closures, or interpreted result that the example produces.
fn contains_wildcard_only_assertion(code: &str) -> bool {
	let stripped: String = code.chars().filter(|c| !c.is_whitespace()).collect();
	WILDCARD_ONLY_ASSERTION_PREFIXES
		.iter()
		.any(|prefix| stripped.contains(&prefix.replace(' ', "")))
		&& stripped.contains(WILDCARD_STRUCT_MATCH_FRAGMENT)
}

/// Check whether `code` contains a call to the documented function or method.
fn contains_call_to_item(
	code: &str,
	item_name: &str,
) -> bool {
	let wrapped = format!("{{\n{}\n}}", normalize_doctest_code_for_parsing(code));
	let Ok(block) = syn::parse_str::<syn::Block>(&wrapped) else {
		return false;
	};

	let mut visitor = ItemCallVisitor {
		item_name,
		found_call: false,
	};
	visitor.visit_block(&block);
	visitor.found_call
}

struct ItemCallVisitor<'a> {
	item_name: &'a str,
	found_call: bool,
}

impl<'ast> Visit<'ast> for ItemCallVisitor<'_> {
	fn visit_macro(
		&mut self,
		mac: &'ast syn::Macro,
	) {
		if self.found_call {
			return;
		}

		if macro_tokens_contain_call(mac, self.item_name) {
			self.found_call = true;
			return;
		}

		syn::visit::visit_macro(self, mac);
	}

	fn visit_expr_call(
		&mut self,
		expr_call: &'ast syn::ExprCall,
	) {
		if self.found_call {
			return;
		}

		if expr_path_ends_with(&expr_call.func, self.item_name) {
			self.found_call = true;
			return;
		}

		syn::visit::visit_expr_call(self, expr_call);
	}

	fn visit_expr_method_call(
		&mut self,
		expr_method_call: &'ast syn::ExprMethodCall,
	) {
		if self.found_call {
			return;
		}

		if expr_method_call.method == self.item_name {
			self.found_call = true;
			return;
		}

		syn::visit::visit_expr_method_call(self, expr_method_call);
	}

	fn visit_item_fn(
		&mut self,
		_item_fn: &'ast syn::ItemFn,
	) {
	}

	fn visit_impl_item_fn(
		&mut self,
		_impl_item_fn: &'ast syn::ImplItemFn,
	) {
	}
}

fn macro_tokens_contain_call(
	mac: &syn::Macro,
	item_name: &str,
) -> bool {
	let Ok(args) = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated
		.parse2(mac.tokens.clone())
	else {
		return false;
	};

	args.iter().any(|expr| expr_contains_call(expr, item_name))
}

fn expr_contains_call(
	expr: &Expr,
	item_name: &str,
) -> bool {
	let mut visitor = ItemCallVisitor {
		item_name,
		found_call: false,
	};
	visitor.visit_expr(expr);
	visitor.found_call
}

fn expr_path_ends_with(
	expr: &Expr,
	item_name: &str,
) -> bool {
	if let Expr::Path(expr_path) = expr
		&& let Some(segment) = expr_path.path.segments.last()
	{
		segment.ident == item_name
	} else {
		false
	}
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
				"#[{DOCUMENT_EXAMPLES}] requires at least one Rust code block in the doc comments (using ``` or ```rust fences). Examples should show how the documented item is supposed to be used and contain assertions about the expected outputs using assertion macros such as assert_eq!, assert!, etc."
			),
		)
		.into());
	}

	Ok(())
}

/// Validate that every Rust code block contains at least one assertion
/// and that the assertion is non-trivial.
fn validate_code_blocks(code_blocks: &[String]) -> OurResult<()> {
	validate_code_blocks_exist(code_blocks)?;

	for (i, code) in code_blocks.iter().enumerate() {
		if !contains_assertion(code) {
			return Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				format!(
					"Code block {} in the doc comments for #[{DOCUMENT_EXAMPLES}] must show how the documented item is supposed to be used and contain at least one assertion about the expected outputs using assertion macros such as assert_eq!, assert!, etc.",
					i + 1,
				),
			)
			.into());
		}

		if contains_literal_only_assertion(code) {
			return Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				format!(
					"Code block {} in the doc comments for #[{DOCUMENT_EXAMPLES}] contains an assertion over only literals and operators; replace it with a meaningful assertion that verifies a value produced by the documented item",
					i + 1,
				),
			)
			.into());
		}

		if contains_wildcard_only_assertion(code) {
			return Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				format!(
					"Code block {} in the doc comments for #[{DOCUMENT_EXAMPLES}] contains a wildcard-only variant assertion (e.g., `assert!(matches!(value, Variant {{ .. }}))`); destructure the value and assert the relevant fields, closures, or interpreted result instead",
					i + 1,
				),
			)
			.into());
		}
	}

	Ok(())
}

/// Validate that every Rust code block contains a call to the documented item.
fn validate_code_blocks_call_item(
	code_blocks: &[String],
	item_name: &str,
) -> OurResult<()> {
	for (i, code) in code_blocks.iter().enumerate() {
		if !contains_call_to_item(code, item_name) {
			return Err(syn::Error::new(
				proc_macro2::Span::call_site(),
				format!(
					"Code block {} in the doc comments for #[{DOCUMENT_EXAMPLES}] must contain a call to the documented function or method `{item_name}`. Examples should show meaningful usage of the function or method being documented and assert expected outcomes using assertion macros such as assert_eq!, assert!, etc. Use #[{DOCUMENT_EXAMPLES}({SKIP_CALL_CHECK}, {REASON} = \"...\")] only when the example intentionally demonstrates related behaviour without calling `{item_name}` directly.",
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
	let options = parse_document_examples_options(attr)?;

	let mut ast = RustAst::parse(item).map_err(crate::core::Error::Parse)?;

	let item_name = ast.signature().map(|signature| signature.ident.to_string());

	// Check for duplicate #[document_examples]
	reject_duplicate_attribute(ast.attributes(), DOCUMENT_EXAMPLES)?;

	// Extract and validate doc comment code blocks
	let doc_content = extract_doc_content(ast.attributes());
	let code_blocks = extract_rust_code_blocks(&doc_content);

	if let Some(item_name) = item_name {
		// Functions require assertion macros in code blocks
		validate_code_blocks(&code_blocks)?;
		if !options.skip_call_check {
			validate_code_blocks_call_item(&code_blocks, &item_name)?;
		}
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

#[cfg(test)]
#[expect(clippy::expect_used, reason = "Tests use panicking Result assertions for clarity.")]
mod tests {
	use {
		super::{
			contains_call_to_item,
			parse_document_examples_options,
		},
		quote::quote,
	};

	#[test]
	fn parses_empty_document_examples_options() {
		let options =
			parse_document_examples_options(quote! {}).expect("empty options should parse");

		assert!(!options.skip_call_check);
		assert_eq!(options.reason, None);
	}

	#[test]
	fn parses_skip_call_check_with_reason() {
		let options = parse_document_examples_options(quote! {
			skip_call_check, reason = "private boundary helper is exercised indirectly"
		})
		.expect("skip_call_check with reason should parse");

		assert!(options.skip_call_check);
		assert_eq!(
			options.reason.as_deref(),
			Some("private boundary helper is exercised indirectly")
		);
	}

	#[test]
	fn rejects_skip_call_check_without_reason() {
		let error = parse_document_examples_options(quote! { skip_call_check })
			.expect_err("skip_call_check without reason should fail");

		assert!(error.to_string().contains("must include `reason = \"...\"`"));
	}

	#[test]
	fn rejects_reason_without_skip_call_check() {
		let error = parse_document_examples_options(quote! { reason = "not used" })
			.expect_err("reason without skip_call_check should fail");

		assert!(error.to_string().contains("only supported when `skip_call_check` is present"));
	}

	#[test]
	fn detects_free_function_call() {
		let code = r#"
let value = documented_function();
assert_eq!(value, 1);
"#;

		assert!(contains_call_to_item(code, "documented_function"));
	}

	#[test]
	fn detects_qualified_function_call() {
		let code = r#"
let value = crate::module::documented_function();
assert_eq!(value, 1);
"#;

		assert!(contains_call_to_item(code, "documented_function"));
	}

	#[test]
	fn detects_call_when_doctest_has_inner_attribute() {
		let code = r#"
#![recursion_limit = "512"]
let value = TypeName::documented_function();
assert_eq!(value, 1);
"#;

		assert!(contains_call_to_item(code, "documented_function"));
	}

	#[test]
	fn detects_method_call() {
		let code = r#"
let value = receiver.documented_method();
assert_eq!(value, 1);
"#;

		assert!(contains_call_to_item(code, "documented_method"));
	}

	#[test]
	fn detects_hidden_doctest_call() {
		let code = r#"
# let value = documented_function();
assert_eq!(value, 1);
"#;

		assert!(contains_call_to_item(code, "documented_function"));
	}

	#[test]
	fn detects_call_inside_assertion_macro() {
		let code = r#"
assert_eq!(documented_function(), 1);
"#;

		assert!(contains_call_to_item(code, "documented_function"));
	}

	#[test]
	fn detects_method_call_inside_assertion_macro() {
		let code = r#"
assert_eq!(receiver.documented_method(), 1);
"#;

		assert!(contains_call_to_item(code, "documented_method"));
	}

	#[test]
	fn ignores_function_definitions() {
		let code = r#"
fn documented_function() -> i32 {
	1
}
let value = 1;
assert_eq!(value, 1);
"#;

		assert!(!contains_call_to_item(code, "documented_function"));
	}

	#[test]
	fn ignores_method_definitions() {
		let code = r#"
struct Example;
impl Example {
	fn documented_method(&self) -> i32 {
		self.documented_method()
	}
}
let value = 1;
assert_eq!(value, 1);
"#;

		assert!(!contains_call_to_item(code, "documented_method"));
	}
}
