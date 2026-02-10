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

	let doc_comment = format!("`{signature_data}`");
	let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
	method.attrs.insert(attr_pos, doc_attr);

	errors
}

/// Process the `#[document_type_parameters]` attribute on a method.
pub(super) fn process_doc_type_params(
	method: &mut syn::ImplItemFn,
	attr_pos: usize,
	item_impl_generics: &syn::Generics,
	impl_key: &ImplKey,
	config: &Config,
) -> Vec<Error> {
	let attr = method.attrs.remove(attr_pos);
	let mut errors = Vec::new();

	// Extract impl generic parameter names
	let impl_param_names: Vec<String> = extract_all_params(item_impl_generics);

	// Get impl-level type parameter docs (optional if impl has no type parameters)
	let impl_docs = config.impl_type_param_docs.get(impl_key);

	// If impl has type parameters but no impl-level docs, require them
	if !impl_param_names.is_empty() && impl_docs.is_none() {
		errors.push(Error::new(
			attr.span(),
			format!(
				"{DOCUMENT_TYPE_PARAMETERS} on methods requires impl-level type parameter documentation \
				when the impl block has type parameters. \
				Add #[document_type_parameters(...)] to the impl block first."
			),
		));
		return errors;
	}

	// Try to parse the arguments from the attribute
	let args_result = attr.parse_args::<GenericArgs>();

	if let Ok(args) = args_result {
		// Get method-only generics (not including impl generics)
		let method_param_names: Vec<String> = extract_all_params(&method.sig.generics);

		let entries: Vec<_> = args.entries.iter().collect();

		// If we have impl docs, validate that method doesn't redocument impl parameters
		if let Some(impl_docs_vec) = impl_docs {
			if entries.len() > method_param_names.len() {
				let impl_param_list = impl_param_names.join(", ");
				errors.push(Error::new(
					attr.span(),
					format!(
						"Method documents {} parameters but only has {} method-level generic parameters. \
						Impl-level parameters ({impl_param_list}) are already documented at the impl level and should not be redocumented here.",
						entries.len(),
						method_param_names.len()
					),
				));
				return errors;
			}

			// Emit docs: impl docs first, then method docs
			let mut doc_index = 0;

			// Emit impl-level docs
			for (param_name, desc) in impl_docs_vec {
				let doc_comment = format!("* `{param_name}`: {desc}");
				let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
				method.attrs.insert(attr_pos + doc_index, doc_attr);
				doc_index += 1;
			}

			// Emit method-level docs
			if let Err(e) = validate_doc_args(method_param_names.len(), entries.len(), attr.span())
			{
				errors.push(e);
			} else {
				for (name_from_target, entry) in method_param_names.iter().zip(entries) {
					let (name, desc) = match entry {
						DocArg::Override(n, d) => (n.value(), d.value()),
						DocArg::Desc(d) => (name_from_target.clone(), d.value()),
					};

					let doc_comment = format!("* `{name}`: {desc}");
					let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
					method.attrs.insert(attr_pos + doc_index, doc_attr);
					doc_index += 1;
				}
			}
		} else {
			// No impl docs - document all parameters (impl + method)
			let mut all_generics = syn::Generics::default();

			// Add impl generics first
			for param in &item_impl_generics.params {
				all_generics.params.push(param.clone());
			}

			// Add method generics
			for param in &method.sig.generics.params {
				all_generics.params.push(param.clone());
			}

			let targets = extract_all_params(&all_generics);

			if let Err(e) = validate_doc_args(targets.len(), entries.len(), attr.span()) {
				errors.push(e);
			} else {
				for (i, (name_from_target, entry)) in targets.iter().zip(entries).enumerate() {
					let (name, desc) = match entry {
						DocArg::Override(n, d) => (n.value(), d.value()),
						DocArg::Desc(d) => (name_from_target.clone(), d.value()),
					};

					let doc_comment = format!("* `{name}`: {desc}");
					let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
					method.attrs.insert(attr_pos + i, doc_attr);
				}
			}
		}
	} else {
		// Parse failed - check if this is because the attribute is empty
		if let Some(impl_docs_vec) = impl_docs {
			// Emit impl-level docs only
			for (i, (param_name, desc)) in impl_docs_vec.iter().enumerate() {
				let doc_comment = format!("* `{param_name}`: {desc}");
				let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
				method.attrs.insert(attr_pos + i, doc_attr);
			}
		} else {
			// No impl docs and parse failed - error
			errors.push(Error::new(
				attr.span(),
				format!("Failed to parse {DOCUMENT_TYPE_PARAMETERS} arguments"),
			));
		}
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
			// Remove impl-level document_type_parameters attribute if present
			item_impl.attrs.retain(|attr| !attr.path().is_ident(DOCUMENT_TYPE_PARAMETERS));

			let self_ty = &*item_impl.self_ty;
			let self_ty_path = quote!(#self_ty).to_string();
			let trait_path = item_impl.trait_.as_ref().map(|(_, path, _)| path);
			let trait_name =
				trait_path.and_then(|p| p.segments.last().map(|s| s.ident.to_string()));
			let trait_path_str = trait_path.map(|p| quote!(#p).to_string());

			let impl_document_use = match parse_unique_attr_value(&item_impl.attrs, DOCUMENT_USE) {
				Ok(v) => v,
				Err(e) => {
					errors.push(syn::Error::from(e));
					None
				}
			};

			for impl_item in &mut item_impl.items {
				if let ImplItem::Fn(method) = impl_item {
					let method_document_use =
						match parse_unique_attr_value(&method.attrs, DOCUMENT_USE) {
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
						// Create impl key for looking up impl-level docs
						let impl_key = if let Some(ref t_path) = trait_path_str {
							ImplKey::with_trait(&self_ty_path, t_path)
						} else {
							ImplKey::new(&self_ty_path)
						};

						let method_errors = process_doc_type_params(
							method,
							attr_pos,
							&item_impl.generics,
							&impl_key,
							config,
						);
						errors.extend(method_errors);
					}
				}
			}
		}
	}

	errors.finish()
}
