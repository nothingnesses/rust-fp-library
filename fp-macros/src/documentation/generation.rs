use crate::{
	analysis::extract_all_params,
	core::{
		config::Config,
		constants::attributes::{DOCUMENT_SIGNATURE, DOCUMENT_TYPE_PARAMETERS, DOCUMENT_USE},
		error_handling::ErrorCollector,
	},
	documentation::document_signature::generate_signature,
	resolution::{
		ImplKey,
		resolver::{
			SelfSubstitutor, extract_concrete_type_name, extract_self_type_info, merge_generics,
		},
	},
	support::{
		attributes::find_attribute,
		parsing::parse_unique_attr_value,
		syntax::{DocArg, GenericArgs, format_parameter_doc, validate_doc_args},
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

	let doc_comment = format!("`{signature_data}`");
	let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
	method.attrs.insert(attr_pos, doc_attr);

	errors
}

/// Process the `#[document_type_parameters]` attribute on a method.
pub(super) fn process_document_type_parameters(
	method: &mut syn::ImplItemFn,
	attr_pos: usize,
) -> Vec<Error> {
	let attr = method.attrs.remove(attr_pos);
	let mut errors = Vec::new();

	// Get method-only generics (not including impl generics)
	let method_param_names: Vec<String> = extract_all_params(&method.sig.generics);

	// Error if method has no type parameters
	if method_param_names.is_empty() {
		errors.push(Error::new(
			attr.span(),
			format!(
				"{DOCUMENT_TYPE_PARAMETERS} cannot be used on method '{}' with no type parameters",
				method.sig.ident
			),
		));
		return errors;
	}

	// Try to parse the arguments from the attribute
	let args_result = attr.parse_args::<GenericArgs>();

	if let Ok(args) = args_result {
		let entries: Vec<_> = args.entries.iter().collect();

		// Validate and emit method-level docs only
		if let Err(e) = validate_doc_args(method_param_names.len(), entries.len(), attr.span()) {
			errors.push(e);
		} else {
			for (i, (name_from_target, entry)) in method_param_names.iter().zip(entries).enumerate()
			{
				let (name, desc) = match entry {
					DocArg::Override(n, d) => (n.value(), d.value()),
					DocArg::Desc(d) => (name_from_target.clone(), d.value()),
				};

				let doc_comment = format_parameter_doc(&name, &desc);
				let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
				method.attrs.insert(attr_pos + i, doc_attr);
			}
		}
	} else {
		// Parse failed - error
		errors.push(Error::new(
			attr.span(),
			format!("Failed to parse {DOCUMENT_TYPE_PARAMETERS} arguments"),
		));
	}

	errors
}

/// Helper to parse an attribute value, collecting errors instead of propagating them.
fn parse_attr_or_none(
	attrs: &[syn::Attribute],
	name: &str,
	errors: &mut ErrorCollector,
) -> Option<String> {
	parse_unique_attr_value(attrs, name).unwrap_or_else(|e| {
		errors.push(syn::Error::from(e));
		None
	})
}

/// Process method-level documentation (signatures and type parameters).
fn process_method_documentation(
	method: &mut syn::ImplItemFn,
	self_ty: &syn::Type,
	self_ty_path: &str,
	trait_name: Option<&str>,
	trait_path_str: Option<&str>,
	impl_document_use: Option<&str>,
	item_impl_generics: &syn::Generics,
	config: &Config,
	errors: &mut ErrorCollector,
) {
	let method_document_use = parse_attr_or_none(&method.attrs, DOCUMENT_USE, errors);
	let document_use = method_document_use.or_else(|| impl_document_use.map(String::from));

	// 1. Handle HM Signature
	if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_SIGNATURE) {
		let method_errors = process_document_signature(
			method,
			attr_pos,
			self_ty,
			self_ty_path,
			trait_name,
			trait_path_str,
			document_use.as_deref(),
			item_impl_generics,
			config,
		);
		errors.extend(method_errors);
	}

	// 2. Handle Doc Type Params
	if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_TYPE_PARAMETERS) {
		let method_errors = process_document_type_parameters(method, attr_pos);
		errors.extend(method_errors);
	}
}

/// Process a single impl block for documentation generation.
fn process_impl_block(
	item_impl: &mut syn::ItemImpl,
	config: &Config,
	errors: &mut ErrorCollector,
) {
	let self_ty = &*item_impl.self_ty;
	let self_ty_path = quote!(#self_ty).to_string();
	let trait_path = item_impl.trait_.as_ref().map(|(_, path, _)| path);
	let trait_name = trait_path.and_then(|p| p.segments.last().map(|s| s.ident.to_string()));
	let trait_path_str = trait_path.map(|p| quote!(#p).to_string());

	// Generate impl-level documentation for type parameters if attribute is present
	if let Some(attr_pos) = find_attribute(&item_impl.attrs, DOCUMENT_TYPE_PARAMETERS) {
		// Create impl key and process in one go to avoid borrow conflicts
		let impl_key = ImplKey::from_paths(&self_ty_path, trait_path_str.as_deref());

		// Get the stored impl-level docs from config
		if let Some(impl_docs) = config.impl_type_param_docs.get(&impl_key) {
			// Remove the attribute
			item_impl.attrs.remove(attr_pos);

			// Generate documentation comments for each impl-level type parameter
			for (i, (param_name, desc)) in impl_docs.iter().enumerate() {
				let doc_comment = format_parameter_doc(param_name, desc);
				let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
				item_impl.attrs.insert(attr_pos + i, doc_attr);
			}
		} else {
			// This shouldn't happen as context extraction should have caught this
			// But remove the attribute anyway to prevent downstream issues
			item_impl.attrs.remove(attr_pos);
		}
	}

	// Parse impl-level document_use attribute
	let impl_document_use = parse_attr_or_none(&item_impl.attrs, DOCUMENT_USE, errors);

	// Process each method in the impl block
	for impl_item in &mut item_impl.items {
		if let ImplItem::Fn(method) = impl_item {
			process_method_documentation(
				method,
				self_ty,
				&self_ty_path,
				trait_name.as_deref(),
				trait_path_str.as_deref(),
				impl_document_use.as_deref(),
				&item_impl.generics,
				config,
				errors,
			);
		}
	}
}

/// Generate documentation for all items.
///
/// This is the main entry point for documentation generation. It processes each impl block
/// in the provided items, generating documentation for type parameters, method signatures,
/// and other attributes.
pub(super) fn generate_documentation(
	items: &mut [Item],
	config: &Config,
) -> Result<()> {
	let mut errors = ErrorCollector::new();

	for item in items {
		if let Item::Impl(item_impl) = item {
			process_impl_block(item_impl, config, &mut errors);
		}
	}

	errors.finish()
}
