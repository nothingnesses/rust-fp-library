use crate::{
	analysis::GenericAnalyzer,
	core::{
		config::Config,
		constants::known_attrs::{self, DOCUMENT_SIGNATURE, DOCUMENT_TYPE_PARAMETERS},
		error_handling::ErrorCollector,
	},
	documentation::document_signature::generate_signature,
	resolution::resolver::{
		SelfSubstitutor, extract_concrete_type_name, extract_self_type_info, merge_generics,
	},
	support::{
		attributes::{find_attr_value_checked, find_attribute},
		syntax::{DocArg, GenericArgs, validate_doc_args},
	},
};
use quote::quote;
use syn::{Error, ImplItem, Item, Result, parse_quote, spanned::Spanned, visit_mut::VisitMut};

/// Process the `#[document_signature]` attribute on a method.
#[allow(clippy::too_many_arguments)]
pub(super) fn process_document_signature(
	method: &mut syn::ImplItemFn,
	attr_pos: usize,
	self_ty: &syn::Type,
	self_ty_path: &str,
	_trait_name: Option<&str>,
	trait_path_str: Option<&str>,
	document_use: Option<&str>,
	item_impl_generics: &syn::Generics,
	config: &Config,
) -> Vec<Error> {
	method.attrs.remove(attr_pos);

	let mut errors = Vec::new();
	let mut synthetic_sig = method.sig.clone();

	// Extract base type name and generic parameters from impl
	let (base_type_name, impl_generic_params) = extract_self_type_info(self_ty, item_impl_generics);

	// Resolve Self
	let mut substitutor = SelfSubstitutor::new(
		self_ty,
		self_ty_path,
		trait_path_str,
		document_use,
		config,
		base_type_name.clone(),
		impl_generic_params.clone(),
	);
	substitutor.visit_signature_mut(&mut synthetic_sig);

	// Collect any resolution errors
	errors.extend(substitutor.errors);

	// Merge generics
	merge_generics(&mut synthetic_sig, item_impl_generics);

	// Create a modified config with concrete type information
	let mut sig_config = config.clone();

	// Extract and add the concrete type name
	if let Some(concrete_type_name) = extract_concrete_type_name(self_ty, config) {
		sig_config.concrete_types.insert(concrete_type_name.clone());
		sig_config.self_type_name = Some(concrete_type_name);
	}

	let signature_data = generate_signature(&synthetic_sig, &sig_config);

	let doc_comment = format!("`{}`", signature_data);
	let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
	method.attrs.insert(attr_pos, doc_attr);

	errors
}

/// Process the `#[document_type_parameters]` attribute on a method.
pub(super) fn process_doc_type_params(
	method: &mut syn::ImplItemFn,
	attr_pos: usize,
	item_impl_generics: &syn::Generics,
) -> Vec<Error> {
	let attr = method.attrs.remove(attr_pos);
	let mut errors = Vec::new();

	// Parse the arguments from the attribute
	if let Ok(args) = attr.parse_args::<GenericArgs>() {
		// Get all generics: impl generics + method generics
		let mut all_generics = syn::Generics::default();

		// Add impl generics first
		for param in &item_impl_generics.params {
			all_generics.params.push(param.clone());
		}

		// Add method generics
		for param in &method.sig.generics.params {
			all_generics.params.push(param.clone());
		}

		let targets = GenericAnalyzer::all_params(&all_generics);

		let entries: Vec<_> = args.entries.iter().collect();
		if let Err(e) = validate_doc_args(targets.len(), entries.len(), attr.span()) {
			errors.push(e);
		} else {
			for (i, (name_from_target, entry)) in targets.iter().zip(entries).enumerate() {
				let (name, desc) = match entry {
					DocArg::Override(n, d) => (n.value(), d.value()),
					DocArg::Desc(d) => (name_from_target.clone(), d.value()),
				};

				let doc_comment = format!("* `{}`: {}", name, desc);
				let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
				method.attrs.insert(attr_pos + i, doc_attr);
			}
		}
	} else {
		errors.push(Error::new(attr.span(), "Failed to parse document_type_parameters arguments"));
	}

	errors
}

pub(super) fn generate_docs(
	items: &mut [Item],
	config: &Config,
) -> Result<()> {
	let mut errors = ErrorCollector::new();

	for item in items {
		if let Item::Impl(item_impl) = item {
			let self_ty = &*item_impl.self_ty;
			let self_ty_path = quote!(#self_ty).to_string();
			let trait_path = item_impl.trait_.as_ref().map(|(_, path, _)| path);
			let trait_name =
				trait_path.and_then(|p| p.segments.last().map(|s| s.ident.to_string()));
			let trait_path_str = trait_path.map(|p| quote!(#p).to_string());

			let impl_document_use =
				match find_attr_value_checked(&item_impl.attrs, known_attrs::DOCUMENT_USE) {
					Ok(v) => v,
					Err(e) => {
						errors.push(syn::Error::from(e));
						None
					}
				};

			for impl_item in &mut item_impl.items {
				if let ImplItem::Fn(method) = impl_item {
					let method_document_use =
						match find_attr_value_checked(&method.attrs, known_attrs::DOCUMENT_USE) {
							Ok(v) => v,
							Err(e) => {
								errors.push(syn::Error::from(e));
								None
							}
						};
					let document_use = method_document_use.or_else(|| impl_document_use.clone());

					// 1. Handle HM Signature
					if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_SIGNATURE) {
						let method_errors = process_document_signature(
							method,
							attr_pos,
							self_ty,
							&self_ty_path,
							trait_name.as_deref(),
							trait_path_str.as_deref(),
							document_use.as_deref(),
							&item_impl.generics,
							config,
						);
						errors.extend(method_errors);
					}

					// 2. Handle Doc Type Params
					if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_TYPE_PARAMETERS)
					{
						let method_errors =
							process_doc_type_params(method, attr_pos, &item_impl.generics);
						errors.extend(method_errors);
					}
				}
			}
		}
	}

	errors.finish()
}
