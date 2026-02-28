use {
	crate::{
		core::{
			Result as OurResult,
			constants::{
				attributes::DOCUMENT_EXAMPLES,
				macros::ASSERTION_MACROS,
			},
		},
		support::{
			ast::RustAst,
			generate_documentation::{
				find_insertion_index,
				insert_doc_comments_batch,
			},
		},
	},
	proc_macro2::TokenStream,
	quote::quote,
};

/// Check whether `code` contains at least one assertion macro invocation.
fn contains_assertion(code: &str) -> bool {
	ASSERTION_MACROS.iter().any(|mac| code.contains(mac))
}

/// Worker for the `document_examples` macro.
pub fn document_examples_worker(
	attr: TokenStream,
	item: TokenStream,
) -> OurResult<TokenStream> {
	// Error case 2: attribute argument must be a string literal.
	let example_code: syn::LitStr = syn::parse2(attr).map_err(|e| {
		syn::Error::new(
			e.span(),
			format!(
				"#[{DOCUMENT_EXAMPLES}] requires a string argument; the string should show example code usage of the function"
			),
		)
	})?;

	// Error case 3: the string must contain an assertion macro invocation.
	let code = example_code.value();
	if !contains_assertion(&code) {
		return Err(syn::Error::new(
			example_code.span(),
			format!(
				"The example code provided in #[{DOCUMENT_EXAMPLES}] should contain assertions about the expected output of the function"
			),
		)
		.into());
	}

	let mut ast = RustAst::parse(item).map_err(crate::core::Error::Parse)?;

	if ast.signature().is_none() {
		return Err(syn::Error::new_spanned(
			ast,
			format!("{DOCUMENT_EXAMPLES} can only be applied to functions"),
		)
		.into());
	}

	process_document_examples_on_ast(&mut ast, &example_code);

	Ok(quote!(#ast))
}

fn process_document_examples_on_ast(
	ast: &mut RustAst,
	example_code: &syn::LitStr,
) {
	let code = example_code.value();

	let mut docs =
		vec![(String::new(), "### Examples\n".to_string()), (String::new(), "```".to_string())];

	for line in code.lines() {
		docs.push((String::new(), line.to_string()));
	}

	docs.push((String::new(), "```\n".to_string()));

	let attrs = ast.attributes();
	let insert_idx = find_insertion_index(attrs, example_code.span());
	insert_doc_comments_batch(attrs, docs, insert_idx);
}
