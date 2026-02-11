use crate::support::{
	ast::RustAst,
	documentation_parameters::{DocumentationParameter, DocumentationParameters},
	parsing::parse_parameter_documentation_pairs,
};
use proc_macro2::TokenStream;
use syn::{Error, parse_quote, spanned::Spanned};

/// Generate documentation comments for parameters.
///
/// This is the core function for generating parameter documentation. It:
/// 1. Parses the item and arguments
/// 2. Gets the list of parameters from the item
/// 3. Validates and pairs documentation with parameters
/// 4. Inserts doc comments into the item's attributes
///
/// # Parameters
/// - `attr`: The macro attribute tokens containing the documentation
/// - `item_tokens`: The item being documented
/// - `get_targets`: A function to extract parameter names from the item
pub fn generate_doc_comments<F>(
	attr: TokenStream,
	item_tokens: TokenStream,
	get_targets: F,
) -> crate::core::Result<TokenStream>
where
	F: FnOnce(&RustAst) -> Result<Vec<String>, Error>,
{
	let mut generic_item = RustAst::parse(item_tokens).map_err(crate::core::Error::Parse)?;

	let args =
		syn::parse2::<DocumentationParameters>(attr.clone()).map_err(crate::core::Error::Parse)?;

	let targets = get_targets(&generic_item)?;
	let entries: Vec<_> = args.entries.into_iter().collect();

	// Parse and validate using the function in parsing.rs
	let pairs = parse_parameter_documentation_pairs(targets, entries, attr.span())?;

	for (name_from_target, entry) in pairs {
		let (name, desc) = match entry {
			DocumentationParameter::Override(n, d) => (n.value(), d.value()),
			DocumentationParameter::Description(d) => (name_from_target, d.value()),
		};

		let doc_comment = format_parameter_doc(&name, &desc);
		insert_doc_comment(generic_item.attributes(), doc_comment, proc_macro2::Span::call_site());
	}

	Ok(quote::quote! {
		#generic_item
	})
}

/// Format a parameter documentation comment.
///
/// Creates a standardized documentation comment for a parameter with its description.
///
/// # Example
/// ```
/// # fn format_parameter_doc(name: &str, description: &str) -> String {
/// #     format!("* `{name}`: {description}")
/// # }
/// let doc = format_parameter_doc("T", "The element type");
/// assert_eq!(doc, "* `T`: The element type");
/// ```
pub fn format_parameter_doc(
	name: &str,
	description: &str,
) -> String {
	format!("* `{name}`: {description}")
}

/// Insert a documentation comment into an attribute list.
///
/// Inserts the doc comment at the appropriate position based on the macro invocation span.
pub fn insert_doc_comment(
	attrs: &mut Vec<syn::Attribute>,
	doc_comment: String,
	macro_span: proc_macro2::Span,
) {
	let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);

	// Find insertion point based on macro invocation position
	let mut insert_idx = attrs.len();

	for (i, attr) in attrs.iter().enumerate() {
		// If the attribute is after the macro invocation, insert before it
		if attr.span().start().line > macro_span.start().line {
			insert_idx = i;
			break;
		}
	}

	attrs.insert(insert_idx, doc_attr);
}

/// Generate and insert multiple doc comments in order.
///
/// This is a convenience function for batch-inserting documentation comments.
///
/// # Parameters
/// - `attrs`: The attribute list to insert into
/// - `docs`: Vec of (name, description) pairs to generate docs for
/// - `base_index`: The index where the first doc comment should be inserted
pub fn insert_doc_comments_batch(
	attrs: &mut Vec<syn::Attribute>,
	docs: Vec<(String, String)>,
	base_index: usize,
) {
	for (i, (name, desc)) in docs.into_iter().enumerate() {
		let doc_comment = format_parameter_doc(&name, &desc);
		let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
		attrs.insert(base_index + i, doc_attr);
	}
}

/// Get the documentation content from a doc attribute (test helper).
#[cfg(test)]
pub fn get_doc(attr: &syn::Attribute) -> String {
	if let syn::Meta::NameValue(nv) = &attr.meta
		&& let syn::Expr::Lit(lit) = &nv.value
		&& let syn::Lit::Str(s) = &lit.lit
	{
		return s.value();
	}
	panic!("Not a doc comment: {:?}", attr);
}
