use syn::{
	Error,
	Expr,
	ExprTuple,
	LitStr,
	Token,
	parse::{
		Parse,
		ParseStream,
	},
	punctuated::Punctuated,
	spanned::Spanned,
};

/// A documentation argument for parameter documentation.
///
/// Can be either a simple description or an override with name and description.
pub enum DocumentationParameter {
	/// A description string (e.g., `"description"`)
	Description(LitStr),
	/// An override with name and description (e.g., `("name", "description")`)
	Override(LitStr, LitStr),
}

impl Parse for DocumentationParameter {
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
				Expr::Lit(syn::ExprLit {
					lit: syn::Lit::Str(s), ..
				}) => s.clone(),
				_ => {
					return Err(Error::new(
						tuple.elems[0].span(),
						"Expected a string literal for the parameter name",
					));
				}
			};
			let desc = match &tuple.elems[1] {
				Expr::Lit(syn::ExprLit {
					lit: syn::Lit::Str(s), ..
				}) => s.clone(),
				_ => {
					return Err(Error::new(
						tuple.elems[1].span(),
						"Expected a string literal for the description",
					));
				}
			};
			Ok(DocumentationParameter::Override(name, desc))
		} else {
			let lit: LitStr = input.parse()?;
			Ok(DocumentationParameter::Description(lit))
		}
	}
}

/// Generic arguments for documentation macros.
///
/// Represents a comma-separated list of `DocumentationParameter` entries.
pub struct DocumentationParameters {
	pub entries: Punctuated<DocumentationParameter, Token![,]>,
}

impl Parse for DocumentationParameters {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		Ok(DocumentationParameters {
			entries: Punctuated::parse_terminated(input)?,
		})
	}
}
