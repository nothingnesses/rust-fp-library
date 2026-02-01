use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
	Attribute, Error, Expr, ExprTuple, ImplItemFn, ItemFn, LitStr, Signature, Token, TraitItemFn,
	parse_quote, spanned::Spanned,
};

pub enum GenericItem {
	Fn(ItemFn),
	TraitFn(TraitItemFn),
	ImplFn(ImplItemFn),
	Struct(syn::ItemStruct),
	Enum(syn::ItemEnum),
	Union(syn::ItemUnion),
	Trait(syn::ItemTrait),
	Type(syn::ItemType),
}

impl GenericItem {
	pub fn parse(item: TokenStream) -> syn::Result<Self> {
		if let Ok(f) = syn::parse2::<ItemFn>(item.clone()) {
			Ok(GenericItem::Fn(f))
		} else if let Ok(f) = syn::parse2::<TraitItemFn>(item.clone()) {
			Ok(GenericItem::TraitFn(f))
		} else if let Ok(f) = syn::parse2::<ImplItemFn>(item.clone()) {
			Ok(GenericItem::ImplFn(f))
		} else if let Ok(f) = syn::parse2::<syn::ItemStruct>(item.clone()) {
			Ok(GenericItem::Struct(f))
		} else if let Ok(f) = syn::parse2::<syn::ItemEnum>(item.clone()) {
			Ok(GenericItem::Enum(f))
		} else if let Ok(f) = syn::parse2::<syn::ItemUnion>(item.clone()) {
			Ok(GenericItem::Union(f))
		} else if let Ok(f) = syn::parse2::<syn::ItemTrait>(item.clone()) {
			Ok(GenericItem::Trait(f))
		} else if let Ok(f) = syn::parse2::<syn::ItemType>(item) {
			Ok(GenericItem::Type(f))
		} else {
			Err(Error::new(
				proc_macro2::Span::call_site(),
				"Unsupported item type for documentation macros",
			))
		}
	}

	pub fn attrs(&mut self) -> &mut Vec<Attribute> {
		match self {
			GenericItem::Fn(f) => &mut f.attrs,
			GenericItem::TraitFn(f) => &mut f.attrs,
			GenericItem::ImplFn(f) => &mut f.attrs,
			GenericItem::Struct(f) => &mut f.attrs,
			GenericItem::Enum(f) => &mut f.attrs,
			GenericItem::Union(f) => &mut f.attrs,
			GenericItem::Trait(f) => &mut f.attrs,
			GenericItem::Type(f) => &mut f.attrs,
		}
	}

	pub fn generics(&self) -> &syn::Generics {
		match self {
			GenericItem::Fn(f) => &f.sig.generics,
			GenericItem::TraitFn(f) => &f.sig.generics,
			GenericItem::ImplFn(f) => &f.sig.generics,
			GenericItem::Struct(f) => &f.generics,
			GenericItem::Enum(f) => &f.generics,
			GenericItem::Union(f) => &f.generics,
			GenericItem::Trait(f) => &f.generics,
			GenericItem::Type(f) => &f.generics,
		}
	}

	pub fn sig(&self) -> Option<&Signature> {
		match self {
			GenericItem::Fn(f) => Some(&f.sig),
			GenericItem::TraitFn(f) => Some(&f.sig),
			GenericItem::ImplFn(f) => Some(&f.sig),
			_ => None,
		}
	}
}

impl ToTokens for GenericItem {
	fn to_tokens(
		&self,
		tokens: &mut TokenStream,
	) {
		match self {
			GenericItem::Fn(f) => f.to_tokens(tokens),
			GenericItem::TraitFn(f) => f.to_tokens(tokens),
			GenericItem::ImplFn(f) => f.to_tokens(tokens),
			GenericItem::Struct(f) => f.to_tokens(tokens),
			GenericItem::Enum(f) => f.to_tokens(tokens),
			GenericItem::Union(f) => f.to_tokens(tokens),
			GenericItem::Trait(f) => f.to_tokens(tokens),
			GenericItem::Type(f) => f.to_tokens(tokens),
		}
	}
}

pub enum DocArg {
	Desc(LitStr),
	Override(LitStr, LitStr),
}

impl syn::parse::Parse for DocArg {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
	pub entries: syn::punctuated::Punctuated<DocArg, Token![,]>,
}

impl syn::parse::Parse for GenericArgs {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		Ok(GenericArgs { entries: syn::punctuated::Punctuated::parse_terminated(input)? })
	}
}

pub fn generate_doc_comments<F>(
	attr: TokenStream,
	item_tokens: TokenStream,
	get_targets: F,
) -> TokenStream
where
	F: FnOnce(&GenericItem) -> Result<Vec<String>, Error>,
{
	let mut generic_item = match GenericItem::parse(item_tokens) {
		Ok(i) => i,
		Err(e) => return e.to_compile_error(),
	};

	let args = match syn::parse2::<GenericArgs>(attr.clone()) {
		Ok(a) => a,
		Err(e) => return e.to_compile_error(),
	};

	let targets = match get_targets(&generic_item) {
		Ok(t) => t,
		Err(e) => return e.to_compile_error(),
	};

	let entries: Vec<_> = args.entries.into_iter().collect();

	if targets.len() != entries.len() {
		return Error::new(
			attr.span(),
			format!("Expected {} description arguments, found {}.", targets.len(), entries.len()),
		)
		.to_compile_error();
	}

	for (name_from_target, entry) in targets.iter().zip(entries) {
		let (name, desc) = match entry {
			DocArg::Override(n, d) => (n.value(), d.value()),
			DocArg::Desc(d) => (name_from_target.clone(), d.value()),
		};

		let doc_comment = format!("* `{}`: {}", name, desc);
		insert_doc_comment(generic_item.attrs(), doc_comment, proc_macro2::Span::call_site());
	}

	quote::quote! {
		#generic_item
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
