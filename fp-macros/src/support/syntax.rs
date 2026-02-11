use crate::{
	analysis::{
		extract_apply_macro_info,
		patterns::extract_fn_brand_info,
		traits::{TraitCategory, classify_trait},
	},
	core::{config::Config, constants::types},
	support::{
		parsing::{parse_parameter_documentation_pairs, try_parse_one_of},
		type_visitor::TypeVisitor,
	},
};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
	Attribute, Error, Expr, ExprTuple, ImplItemFn, ItemFn, LitStr, PathArguments, ReturnType,
	Signature, Token, TraitBound, TraitItemFn, Type, TypeParamBound,
	parse::{Parse, ParseStream},
	parse_quote,
	punctuated::Punctuated,
	spanned::Spanned,
};

pub enum GenericItem {
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

impl GenericItem {
	pub fn parse(item: TokenStream) -> syn::Result<Self> {
		try_parse_one_of(
			item,
			vec![
				Box::new(|i| syn::parse2::<ItemFn>(i).map(GenericItem::Fn)),
				Box::new(|i| syn::parse2::<TraitItemFn>(i).map(GenericItem::TraitFn)),
				Box::new(|i| syn::parse2::<ImplItemFn>(i).map(GenericItem::ImplFn)),
				Box::new(|i| syn::parse2::<syn::ItemImpl>(i).map(GenericItem::Impl)),
				Box::new(|i| syn::parse2::<syn::ItemStruct>(i).map(GenericItem::Struct)),
				Box::new(|i| syn::parse2::<syn::ItemEnum>(i).map(GenericItem::Enum)),
				Box::new(|i| syn::parse2::<syn::ItemUnion>(i).map(GenericItem::Union)),
				Box::new(|i| syn::parse2::<syn::ItemTrait>(i).map(GenericItem::Trait)),
				Box::new(|i| syn::parse2::<syn::ItemType>(i).map(GenericItem::Type)),
			],
			"Unsupported item type for documentation macros",
		)
	}

	pub fn attrs(&mut self) -> &mut Vec<Attribute> {
		match self {
			GenericItem::Fn(f) => &mut f.attrs,
			GenericItem::TraitFn(f) => &mut f.attrs,
			GenericItem::ImplFn(f) => &mut f.attrs,
			GenericItem::Impl(f) => &mut f.attrs,
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
			GenericItem::Impl(f) => &f.generics,
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
			GenericItem::Impl(f) => f.to_tokens(tokens),
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

pub fn generate_doc_comments<F>(
	attr: TokenStream,
	item_tokens: TokenStream,
	get_targets: F,
) -> crate::core::Result<TokenStream>
where
	F: FnOnce(&GenericItem) -> Result<Vec<String>, Error>,
{
	let mut generic_item = GenericItem::parse(item_tokens).map_err(crate::core::Error::Parse)?;

	let args = syn::parse2::<GenericArgs>(attr.clone()).map_err(crate::core::Error::Parse)?;

	let targets = get_targets(&generic_item)?;
	let entries: Vec<_> = args.entries.into_iter().collect();

	// Parse and validate using the new function in parsing.rs
	let pairs = parse_parameter_documentation_pairs(targets, entries, attr.span())?;

	for (name_from_target, entry) in pairs {
		let (name, desc) = match entry {
			DocArg::Override(n, d) => (n.value(), d.value()),
			DocArg::Desc(d) => (name_from_target, d.value()),
		};

		let doc_comment = format_parameter_doc(&name, &desc);
		insert_doc_comment(generic_item.attrs(), doc_comment, proc_macro2::Span::call_site());
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

// ============================================================================
// Logical Parameter Extraction
// ============================================================================

/// Represents a parameter in a function signature, either explicit or implicit.
#[derive(Clone, Debug)]
pub enum LogicalParam {
	/// A parameter that appears explicitly in the function signature (e.g., `x: i32`)
	Explicit(syn::Pat),
	/// A parameter that is implicit from trait bounds or other context (e.g., from Fn trait bounds)
	///
	/// This variant is constructed during curried parameter extraction and matched in
	/// documentation generation to represent implicit parameters as `_` in signatures.
	///
	/// Note: The `syn::Type` field is currently not accessed during matching, but is preserved
	/// for potential future use in generating more detailed documentation. This causes a
	/// compiler warning about unused fields, which is expected and acceptable.
	#[allow(dead_code)]
	Implicit(syn::Type),
}

/// Helper function to check if a type is PhantomData
pub fn is_phantom_data(ty: &Type) -> bool {
	match ty {
		Type::Path(type_path) => {
			if let Some(segment) = type_path.path.segments.last() {
				return segment.ident == types::PHANTOM_DATA;
			}
			false
		}
		Type::Reference(type_ref) => is_phantom_data(&type_ref.elem),
		_ => false,
	}
}

/// Extract all logical parameters from a function signature.
///
/// This includes both explicit parameters and implicit curried parameters from the return type.
pub fn get_logical_params(
	sig: &syn::Signature,
	config: &Config,
) -> Vec<LogicalParam> {
	let mut params = Vec::new();

	// 1. Explicit arguments
	for input in &sig.inputs {
		match input {
			syn::FnArg::Receiver(_) => continue, // Skip self
			syn::FnArg::Typed(pat_type) => {
				if !is_phantom_data(&pat_type.ty) {
					params.push(LogicalParam::Explicit((*pat_type.pat).clone()));
				}
			}
		}
	}

	// 2. Curried arguments from return type
	extract_curried_params(&sig.output, &mut params, config);

	params
}

fn extract_curried_params(
	output: &ReturnType,
	params: &mut Vec<LogicalParam>,
	config: &Config,
) {
	if let ReturnType::Type(_, ty) = output {
		extract_from_type(ty, params, config);
	}
}

fn extract_from_type(
	ty: &Type,
	params: &mut Vec<LogicalParam>,
	config: &Config,
) {
	let mut visitor = CurriedParamExtractor { params, config };
	visitor.visit(ty);
}

/// Helper function to safely get the last segment of a path.
pub fn last_path_segment(path: &syn::Path) -> Option<&syn::PathSegment> {
	path.segments.last()
}

struct CurriedParamExtractor<'a> {
	params: &'a mut Vec<LogicalParam>,
	config: &'a Config,
}

impl<'a> TypeVisitor for CurriedParamExtractor<'a> {
	type Output = ();

	fn default_output(&self) -> Self::Output {}

	fn visit_path(
		&mut self,
		type_path: &syn::TypePath,
	) {
		// Check for FnBrand pattern using shared helper
		if let Some(fn_brand_info) = extract_fn_brand_info(type_path, self.config) {
			// Add all input types as implicit parameters
			for input_ty in fn_brand_info.inputs {
				self.params.push(LogicalParam::Implicit(input_ty));
			}
			// Recursively visit the output type for nested currying
			self.visit(&fn_brand_info.output);
		}
	}

	fn visit_macro(
		&mut self,
		type_macro: &syn::TypeMacro,
	) {
		// Apply! macro support is handled by extracting its info, but we don't
		// currently extract curried parameters from Apply! projections.
		// This could be enhanced in the future if needed.
		let _ = extract_apply_macro_info(type_macro);
	}

	fn visit_impl_trait(
		&mut self,
		impl_trait: &syn::TypeImplTrait,
	) {
		for bound in &impl_trait.bounds {
			if let TypeParamBound::Trait(trait_bound) = bound {
				self.visit_trait_bound_helper(trait_bound);
			}
		}
	}

	fn visit_trait_object(
		&mut self,
		trait_obj: &syn::TypeTraitObject,
	) {
		for bound in &trait_obj.bounds {
			if let TypeParamBound::Trait(trait_bound) = bound {
				self.visit_trait_bound_helper(trait_bound);
			}
		}
	}

	fn visit_bare_fn(
		&mut self,
		bare_fn: &syn::TypeBareFn,
	) {
		for input in &bare_fn.inputs {
			self.params.push(LogicalParam::Implicit(input.ty.clone()));
		}
		if let ReturnType::Type(_, ty) = &bare_fn.output {
			self.visit(ty);
		}
	}
}

impl<'a> CurriedParamExtractor<'a> {
	fn visit_trait_bound_helper(
		&mut self,
		trait_bound: &TraitBound,
	) {
		let Some(segment) = last_path_segment(&trait_bound.path) else {
			return; // Skip malformed trait bounds
		};
		let name = segment.ident.to_string();

		if let TraitCategory::FnTrait = classify_trait(&name, self.config)
			&& let PathArguments::Parenthesized(args) = &segment.arguments
		{
			for input in &args.inputs {
				self.params.push(LogicalParam::Implicit(input.clone()));
			}
			if let ReturnType::Type(_, ty) = &args.output {
				self.visit(ty);
			}
		}
	}
}
