use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprTuple, LitStr, Token, parse_quote, spanned::Spanned};

pub enum DocArg {
	Desc(LitStr),
	Override(LitStr, LitStr),
}

impl Parse for DocArg {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		if input.peek(syn::token::Paren) {
			let tuple: ExprTuple = input.parse()?;
			if tuple.elems.len() != 2 {
				return Err(Error::new(
					tuple.span(),
					"Expected a tuple of (Name, Description), e.g., (\"arg\", \"description\")",
				));
			}
			let name = match &tuple.elems[0] {
				Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.clone(),
				_ => {
					return Err(Error::new(
						tuple.elems[0].span(),
						"Expected a string literal for the parameter name",
					));
				}
			};
			let desc = match &tuple.elems[1] {
				Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.clone(),
				_ => {
					return Err(Error::new(
						tuple.elems[1].span(),
						"Expected a string literal for the description",
					));
				}
			};
			Ok(DocArg::Override(name, desc))
		} else {
			let lit: LitStr = input.parse()?;
			Ok(DocArg::Desc(lit))
		}
	}
}

pub struct GenericArgs {
	pub entries: Punctuated<DocArg, Token![,]>,
}

impl Parse for GenericArgs {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(GenericArgs { entries: Punctuated::parse_terminated(input)? })
	}
}

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
