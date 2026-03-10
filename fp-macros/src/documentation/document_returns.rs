use {
	crate::{
		core::{
			Result as OurResult,
			constants::attributes::DOCUMENT_RETURNS,
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

/// Worker for the `document_returns` macro.
pub fn document_returns_worker(
	attr: TokenStream,
	item: TokenStream,
) -> OurResult<TokenStream> {
	let description: syn::LitStr = syn::parse2(attr)?;
	let mut ast = RustAst::parse(item).map_err(crate::core::Error::Parse)?;

	if ast.signature().is_some() {
		process_document_returns_on_ast(&mut ast, &description);
	} else {
		return Err(syn::Error::new_spanned(
			ast,
			format!("{DOCUMENT_RETURNS} can only be applied to functions"),
		)
		.into());
	}

	Ok(quote!(#ast))
}

fn process_document_returns_on_ast(
	ast: &mut RustAst,
	description: &syn::LitStr,
) {
	let description_value = description.value();
	let docs = vec![
		(String::new(), "### Returns\n".to_string()),
		(String::new(), format!("{description_value}\n")),
	];

	let attrs = ast.attributes();
	let insert_idx = find_insertion_index(attrs, description.span());
	insert_doc_comments_batch(attrs, docs, insert_idx);
}
