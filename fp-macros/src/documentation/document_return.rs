use {
	crate::core::{
		Result as OurResult,
		constants::attributes::DOCUMENT_RETURN,
	},
	proc_macro2::TokenStream,
	quote::quote,
	std::fmt::format,
	syn::{
		Item,
		parse_quote,
	},
};

/// Worker for the `document_return` macro.
pub fn document_return_worker(
	attr: TokenStream,
	item: TokenStream,
) -> OurResult<TokenStream> {
	let description: syn::LitStr = syn::parse2(attr)?;
	let mut item: Item = syn::parse2(item)?;

	if let Item::Fn(ref mut func) = item {
		process_document_return_on_fn(func, &description);
	} else {
		return Err(syn::Error::new_spanned(
			item,
			format!("{DOCUMENT_RETURN} can only be applied to functions"),
		)
		.into());
	}

	Ok(quote!(#item))
}

pub fn document_return_on_method(
	method: &mut syn::ImplItemFn,
	description: &str,
) {
	let header_comment = r#"### Returns
"#;
	let desc_comment = format!(
		"{description}
"
	);

	let header_attr: syn::Attribute = parse_quote!(#[doc = #header_comment]);
	let desc_attr: syn::Attribute = parse_quote!(#[doc = #desc_comment]);

	// Prepend docs
	method.attrs.insert(0, desc_attr);
	method.attrs.insert(0, header_attr);
}

fn process_document_return_on_fn(
	func: &mut syn::ItemFn,
	description: &syn::LitStr,
) {
	let description_value = description.value();
	let header_comment = r#"### Returns
"#;
	let desc_comment = format!(
		"{description_value}
"
	);

	let header_attr: syn::Attribute = parse_quote!(#[doc = #header_comment]);
	let desc_attr: syn::Attribute = parse_quote!(#[doc = #desc_comment]);

	// Prepend docs
	func.attrs.insert(0, desc_attr);
	func.attrs.insert(0, header_attr);
}
