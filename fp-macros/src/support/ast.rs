use {
	crate::support::parsing::parse_with_dispatch,
	proc_macro2::TokenStream,
	quote::ToTokens,
	syn::{
		Attribute,
		ImplItemFn,
		ItemFn,
		Signature,
		TraitItemFn,
	},
};

/// A generic item that can represent various Rust syntax items.
///
/// This enum provides a unified interface for working with different item types
/// that can be documented or processed by macros.
pub enum RustAst {
	Fn(ItemFn),
	TraitFn(TraitItemFn),
	ImplFn(ImplItemFn),
	Impl(syn::ItemImpl),
	Struct(syn::ItemStruct),
	Enum(syn::ItemEnum),
	Union(syn::ItemUnion),
	Trait(syn::ItemTrait),
	Type(syn::ItemType),
}

impl RustAst {
	/// Parse a token stream into a GenericItem.
	///
	/// Attempts to parse the token stream as various item types in order,
	/// returning the first successful parse.
	pub fn parse(item: TokenStream) -> syn::Result<Self> {
		parse_with_dispatch(
			item,
			vec![
				Box::new(|i| syn::parse2::<ItemFn>(i).map(RustAst::Fn)),
				Box::new(|i| syn::parse2::<TraitItemFn>(i).map(RustAst::TraitFn)),
				Box::new(|i| syn::parse2::<ImplItemFn>(i).map(RustAst::ImplFn)),
				Box::new(|i| syn::parse2::<syn::ItemImpl>(i).map(RustAst::Impl)),
				Box::new(|i| syn::parse2::<syn::ItemStruct>(i).map(RustAst::Struct)),
				Box::new(|i| syn::parse2::<syn::ItemEnum>(i).map(RustAst::Enum)),
				Box::new(|i| syn::parse2::<syn::ItemUnion>(i).map(RustAst::Union)),
				Box::new(|i| syn::parse2::<syn::ItemTrait>(i).map(RustAst::Trait)),
				Box::new(|i| syn::parse2::<syn::ItemType>(i).map(RustAst::Type)),
			],
			"Unsupported item type for documentation macros",
		)
	}

	/// Get a mutable reference to the item's attributes.
	pub fn attributes(&mut self) -> &mut Vec<Attribute> {
		match self {
			RustAst::Fn(f) => &mut f.attrs,
			RustAst::TraitFn(f) => &mut f.attrs,
			RustAst::ImplFn(f) => &mut f.attrs,
			RustAst::Impl(f) => &mut f.attrs,
			RustAst::Struct(f) => &mut f.attrs,
			RustAst::Enum(f) => &mut f.attrs,
			RustAst::Union(f) => &mut f.attrs,
			RustAst::Trait(f) => &mut f.attrs,
			RustAst::Type(f) => &mut f.attrs,
		}
	}

	/// Get a reference to the item's generics.
	pub fn generics(&self) -> &syn::Generics {
		match self {
			RustAst::Fn(f) => &f.sig.generics,
			RustAst::TraitFn(f) => &f.sig.generics,
			RustAst::ImplFn(f) => &f.sig.generics,
			RustAst::Impl(f) => &f.generics,
			RustAst::Struct(f) => &f.generics,
			RustAst::Enum(f) => &f.generics,
			RustAst::Union(f) => &f.generics,
			RustAst::Trait(f) => &f.generics,
			RustAst::Type(f) => &f.generics,
		}
	}

	/// Get the signature if this is a function item.
	///
	/// Returns `Some` for function-like items (Fn, TraitFn, ImplFn),
	/// and `None` for other item types.
	pub fn signature(&self) -> Option<&Signature> {
		match self {
			RustAst::Fn(f) => Some(&f.sig),
			RustAst::TraitFn(f) => Some(&f.sig),
			RustAst::ImplFn(f) => Some(&f.sig),
			_ => None,
		}
	}
}

impl ToTokens for RustAst {
	fn to_tokens(
		&self,
		tokens: &mut TokenStream,
	) {
		match self {
			RustAst::Fn(f) => f.to_tokens(tokens),
			RustAst::TraitFn(f) => f.to_tokens(tokens),
			RustAst::ImplFn(f) => f.to_tokens(tokens),
			RustAst::Impl(f) => f.to_tokens(tokens),
			RustAst::Struct(f) => f.to_tokens(tokens),
			RustAst::Enum(f) => f.to_tokens(tokens),
			RustAst::Union(f) => f.to_tokens(tokens),
			RustAst::Trait(f) => f.to_tokens(tokens),
			RustAst::Type(f) => f.to_tokens(tokens),
		}
	}
}
