use {
	crate::{
		analysis::{
			dispatch::DispatchTraitInfo,
			get_all_parameters,
		},
		core::{
			config::Config,
			constants::attributes::{
				ALLOW_NAMED_GENERICS,
				DOCUMENT_SIGNATURE,
				DOCUMENT_TYPE_PARAMETERS,
				DOCUMENT_USE,
			},
			error_handling::{
				CollectErrors,
				ErrorCollector,
			},
		},
		documentation::document_signature::generate_signature,
		resolution::{
			ImplKey,
			resolver::{
				SelfSubstitutor,
				get_concrete_type_name,
				get_self_type_info,
				merge_generics,
			},
		},
		support::{
			attributes::{
				AttributeExt,
				count_attributes,
				find_attribute,
			},
			documentation_parameters::{
				DocumentationParameter,
				DocumentationParameters,
			},
			generate_documentation::format_parameter_doc,
			parsing::{
				self,
				parse_parameter_documentation_pairs,
			},
		},
	},
	quote::quote,
	syn::{
		FnArg,
		ImplItem,
		Item,
		Result,
		TraitItem,
		Type,
		TypeParamBound,
		parse_quote,
		spanned::Spanned,
		visit_mut::VisitMut,
	},
};

/// Generate a Hindley-Milner type signature and insert it as doc comments.
///
/// This is the shared core used by both impl method and trait method signature processing.
fn insert_signature_docs(
	attrs: &mut Vec<syn::Attribute>,
	attr_pos: usize,
	sig: &syn::Signature,
	config: &Config,
) {
	let signature_data = generate_signature(sig, config);

	let doc_comment = format!("`{signature_data}`");
	let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
	attrs.insert(attr_pos, doc_attr);

	// Add section header
	let header_attr: syn::Attribute = parse_quote!(#[doc = r#"### Type Signature
"#]);
	attrs.insert(attr_pos, header_attr);
}

/// Process the `#[document_signature]` attribute on an impl method.
///
/// Performs Self-type substitution and generics merging before delegating
/// to [`insert_signature_docs`] for the shared doc comment insertion.
#[expect(clippy::too_many_arguments, reason = "Documentation generation requires many parameters")]
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
	errors: &mut ErrorCollector,
) {
	method.attrs.remove(attr_pos);

	let mut synthetic_sig = method.sig.clone();

	// Extract base type name and generic parameters from impl
	let (base_type_name, impl_generic_params) = get_self_type_info(self_ty, item_impl_generics);

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
	if let Some(concrete_type_name) = get_concrete_type_name(self_ty, config) {
		sig_config.concrete_types.insert(concrete_type_name.clone());
		sig_config.self_type_name = Some(concrete_type_name);
	}

	insert_signature_docs(&mut method.attrs, attr_pos, &synthetic_sig, &sig_config);
}

/// Process the `#[document_type_parameters]` attribute, shared core.
///
/// Works with any item that has `attrs` and generic parameters - methods (impl or trait),
/// trait definitions, or any other generic item.
fn process_type_parameters_core(
	attrs: &mut Vec<syn::Attribute>,
	generics: &syn::Generics,
	item_label: &str,
	attr_pos: usize,
	errors: &mut ErrorCollector,
) {
	let attr = attrs.remove(attr_pos);

	let param_names: Vec<String> = get_all_parameters(generics);

	// Error if item has no type parameters - use collect_our_result
	if errors
		.collect_our_result(|| {
			parsing::parse_has_documentable_items(
				param_names.len(),
				attr.span(),
				DOCUMENT_TYPE_PARAMETERS,
				&format!("{item_label} with no type parameters"),
			)
		})
		.is_none()
	{
		// Error occurred, return early
		return;
	}

	// Try to parse the arguments from the attribute
	if let Some(args) = errors.collect(|| attr.parse_args::<DocumentationParameters>()) {
		let entries: Vec<_> = args.entries.into_iter().collect();

		if let Some(pairs) = errors.collect_our_result(|| {
			parse_parameter_documentation_pairs(param_names, entries, attr.span())
		}) {
			let mut docs = Vec::new();
			docs.push((
				String::new(),
				r#"### Type Parameters
"#
				.to_string(),
			));

			for (name_from_target, entry) in pairs {
				let (name, desc) = match entry {
					DocumentationParameter::Override(n, d) => (n.value(), d.value()),
					DocumentationParameter::Description(d) => (name_from_target, d.value()),
				};
				docs.push((name, desc));
			}

			for (i, (name, desc)) in docs.into_iter().enumerate() {
				let doc_comment = format_parameter_doc(&name, &desc);
				let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
				attrs.insert(attr_pos + i, doc_attr);
			}
		}
	} else {
		// Parse failed - add a custom error message with context
		errors.push(syn::Error::new(
			attr.span(),
			format!("Failed to parse {DOCUMENT_TYPE_PARAMETERS} arguments"),
		));
	}
}

/// Process the `#[document_type_parameters]` attribute on an impl method.
pub(super) fn process_document_type_parameters(
	method: &mut syn::ImplItemFn,
	attr_pos: usize,
	errors: &mut ErrorCollector,
) {
	process_type_parameters_core(
		&mut method.attrs,
		&method.sig.generics,
		&format!("method '{}'", method.sig.ident),
		attr_pos,
		errors,
	);
}

/// Process method-level documentation (signatures and type parameters).
#[expect(clippy::too_many_arguments, reason = "Documentation generation requires many parameters")]
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
	// Strip #[allow_named_generics] - consumed during lint pass, must not remain in output
	method.attrs.retain(|attr| !attr.path().is_ident(ALLOW_NAMED_GENERICS));

	let method_document_use = method.attrs.find_value_or_collect(DOCUMENT_USE, errors);
	let document_use = method_document_use.or_else(|| impl_document_use.map(String::from));

	// 1. Handle HM Signature
	if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_SIGNATURE) {
		if count_attributes(&method.attrs, DOCUMENT_SIGNATURE) > 1 {
			errors.push(syn::Error::new(
				method.sig.ident.span(),
				format!(
					"#[{DOCUMENT_SIGNATURE}] can only be used once per item. Remove the duplicate attribute on method `{}`",
					method.sig.ident
				),
			));
		} else {
			process_document_signature(
				method,
				attr_pos,
				self_ty,
				self_ty_path,
				trait_name,
				trait_path_str,
				document_use.as_deref(),
				item_impl_generics,
				config,
				errors,
			);
		}
	}

	// 2. Handle Doc Type Params
	if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_TYPE_PARAMETERS) {
		if count_attributes(&method.attrs, DOCUMENT_TYPE_PARAMETERS) > 1 {
			errors.push(syn::Error::new(
				method.sig.ident.span(),
				format!(
					"#[{DOCUMENT_TYPE_PARAMETERS}] can only be used once per item. Remove the duplicate attribute on method `{}`",
					method.sig.ident
				),
			));
		} else {
			process_document_type_parameters(method, attr_pos, errors);
		}
	}

	// 3. Document parameters is now handled directly in document_parameters.rs
	// No processing needed in document_module
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
	if count_attributes(&item_impl.attrs, DOCUMENT_TYPE_PARAMETERS) > 1 {
		errors.push(syn::Error::new(
			item_impl.self_ty.span(),
			format!(
				"#[{DOCUMENT_TYPE_PARAMETERS}] can only be used once per item. Remove the duplicate attribute on impl block for `{self_ty_path}`",
			),
		));
	} else if let Some(attr_pos) = find_attribute(&item_impl.attrs, DOCUMENT_TYPE_PARAMETERS) {
		// Create impl key and process in one go to avoid borrow conflicts
		let impl_key = ImplKey::from_paths(&self_ty_path, trait_path_str.as_deref());

		// Get the stored impl-level docs from config
		if let Some(impl_docs) = config.impl_type_param_docs.get(&impl_key) {
			// Remove the attribute
			item_impl.attrs.remove(attr_pos);

			// Generate documentation comments for each impl-level type parameter
			let mut docs = Vec::new();
			docs.push((
				String::new(),
				r#"### Type Parameters
"#
				.to_string(),
			));
			for (param_name, desc) in impl_docs.iter() {
				docs.push((param_name.clone(), desc.clone()));
			}

			for (i, (name, desc)) in docs.into_iter().enumerate() {
				let doc_comment = format_parameter_doc(&name, &desc);
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
	let impl_document_use = item_impl.attrs.find_value_or_collect(DOCUMENT_USE, errors);

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

/// Process a trait method's documentation (signatures and type parameters).
///
/// Unlike impl methods, trait methods have no Self-type context, so signature
/// generation uses the method signature directly without substitution.
fn process_trait_method_documentation(
	method: &mut syn::TraitItemFn,
	config: &Config,
	errors: &mut ErrorCollector,
) {
	// 1. Handle HM Signature - no Self substitution needed
	if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_SIGNATURE) {
		if count_attributes(&method.attrs, DOCUMENT_SIGNATURE) > 1 {
			errors.push(syn::Error::new(
				method.sig.ident.span(),
				format!(
					"#[{DOCUMENT_SIGNATURE}] can only be used once per item. Remove the duplicate attribute on method `{}`",
					method.sig.ident
				),
			));
		} else {
			method.attrs.remove(attr_pos);
			insert_signature_docs(&mut method.attrs, attr_pos, &method.sig, config);
		}
	}

	// 2. Handle Doc Type Params
	if let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_TYPE_PARAMETERS) {
		if count_attributes(&method.attrs, DOCUMENT_TYPE_PARAMETERS) > 1 {
			errors.push(syn::Error::new(
				method.sig.ident.span(),
				format!(
					"#[{DOCUMENT_TYPE_PARAMETERS}] can only be used once per item. Remove the duplicate attribute on method `{}`",
					method.sig.ident
				),
			));
		} else {
			process_type_parameters_core(
				&mut method.attrs,
				&method.sig.generics,
				&format!("method '{}'", method.sig.ident),
				attr_pos,
				errors,
			);
		}
	}
}

/// Process a trait definition for documentation generation.
fn process_trait_block(
	item_trait: &mut syn::ItemTrait,
	config: &Config,
	errors: &mut ErrorCollector,
) {
	// Handle trait-level #[document_type_parameters]
	if let Some(attr_pos) = find_attribute(&item_trait.attrs, DOCUMENT_TYPE_PARAMETERS) {
		if count_attributes(&item_trait.attrs, DOCUMENT_TYPE_PARAMETERS) > 1 {
			errors.push(syn::Error::new(
				item_trait.ident.span(),
				format!(
					"#[{DOCUMENT_TYPE_PARAMETERS}] can only be used once per item. Remove the duplicate attribute on trait `{}`",
					item_trait.ident
				),
			));
		} else {
			process_type_parameters_core(
				&mut item_trait.attrs,
				&item_trait.generics,
				&format!("trait '{}'", item_trait.ident),
				attr_pos,
				errors,
			);
		}
	}

	// Process each method in the trait
	for item in &mut item_trait.items {
		if let TraitItem::Fn(method) = item {
			process_trait_method_documentation(method, config, errors);
		}
	}
}

/// Generate documentation for all items.
///
/// This is the main entry point for documentation generation. It processes impl blocks
/// and trait definitions in the provided items, generating documentation for type
/// parameters, method signatures, and other attributes.
pub(super) fn generate_documentation(
	items: &mut [Item],
	config: &Config,
) -> Result<()> {
	let mut errors = ErrorCollector::new();

	for item in items {
		match item {
			Item::Impl(item_impl) => process_impl_block(item_impl, config, &mut errors),
			Item::Trait(item_trait) => process_trait_block(item_trait, config, &mut errors),
			Item::Fn(item_fn) => {
				// Strip #[allow_named_generics] - consumed during lint pass, must not remain
				// in output
				item_fn.attrs.retain(|attr| !attr.path().is_ident(ALLOW_NAMED_GENERICS));

				// If this function has #[document_signature] and references a dispatch trait,
				// generate a dispatch-aware HM signature and remove the attribute so the
				// standalone macro does not also process it.
				process_fn_dispatch_signature(item_fn, config);
			}
			_ => {}
		}
	}

	errors.finish()
}

// -- Dispatch-aware free function signature generation --

/// Process `#[document_signature]` on a free function if it references a dispatch trait.
///
/// If the function has `#[document_signature]` and an `impl *Dispatch<...>` parameter,
/// removes the attribute and inserts a dispatch-aware HM signature as doc comments.
/// If no dispatch trait is found, the attribute is left for the standalone macro.
fn process_fn_dispatch_signature(
	item_fn: &mut syn::ItemFn,
	config: &Config,
) {
	let Some(attr_pos) = find_attribute(&item_fn.attrs, DOCUMENT_SIGNATURE) else {
		return;
	};

	// If the attribute has a string argument (manual override), use it directly
	if let Some(attr) = item_fn.attrs.get(attr_pos)
		&& let Some(manual_sig) = extract_manual_signature(attr)
	{
		item_fn.attrs.remove(attr_pos);
		let doc_comment = format!("`{manual_sig}`");
		let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
		item_fn.attrs.insert(attr_pos, doc_attr);
		let header_attr: syn::Attribute = parse_quote!(#[doc = r#"### Type Signature
"#]);
		item_fn.attrs.insert(attr_pos, header_attr);
		return;
	}

	let Some(dispatch_info) = find_dispatch_trait_in_sig(&item_fn.sig, config) else {
		// No dispatch trait found; leave #[document_signature] for the standalone macro
		return;
	};

	// Build synthetic signature; if it fails (e.g., missing Kind hash), leave
	// the attribute for the standalone macro
	let Some(synthetic_sig) = build_synthetic_signature(&item_fn.sig, &dispatch_info) else {
		return;
	};

	// Remove the attribute so the standalone macro does not also process it
	item_fn.attrs.remove(attr_pos);

	// Generate HM signature from the synthetic signature via the existing pipeline
	let sig_data = generate_signature(&synthetic_sig, config);

	let doc_comment = format!("`{sig_data}`");
	let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
	item_fn.attrs.insert(attr_pos, doc_attr);

	let header_attr: syn::Attribute = parse_quote!(#[doc = r#"### Type Signature
"#]);
	item_fn.attrs.insert(attr_pos, header_attr);
}

/// Extract a manual signature override from a `#[document_signature("...")]` attribute.
///
/// Returns `Some(String)` if the attribute has a string literal argument.
/// Returns `None` if the attribute has no arguments.
fn extract_manual_signature(attr: &syn::Attribute) -> Option<String> {
	let syn::Meta::List(meta_list) = &attr.meta else {
		return None;
	};
	let lit: syn::LitStr = syn::parse2(meta_list.tokens.clone()).ok()?;
	let value = lit.value();
	if value.is_empty() { None } else { Some(value) }
}

/// Find a dispatch trait referenced in a function's parameters via `impl *Dispatch<...>`.
fn find_dispatch_trait_in_sig(
	sig: &syn::Signature,
	config: &Config,
) -> Option<DispatchTraitInfo> {
	for input in &sig.inputs {
		let FnArg::Typed(pat_type) = input else {
			continue;
		};
		let Type::ImplTrait(impl_trait) = &*pat_type.ty else {
			continue;
		};
		for bound in &impl_trait.bounds {
			let TypeParamBound::Trait(trait_bound) = bound else {
				continue;
			};
			let Some(segment) = trait_bound.path.segments.last() else {
				continue;
			};
			let name = segment.ident.to_string();
			if let Some(info) = config.dispatch_traits.get(&name) {
				return Some(info.clone());
			}
		}
	}

	// Also check where-clause bounds for closureless dispatch
	// (the container type itself has a *Dispatch bound)
	if let Some(where_clause) = &sig.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let syn::WherePredicate::Type(pred_type) = predicate {
				for bound in &pred_type.bounds {
					if let TypeParamBound::Trait(trait_bound) = bound {
						let Some(segment) = trait_bound.path.segments.last() else {
							continue;
						};
						let name = segment.ident.to_string();
						if let Some(info) = config.dispatch_traits.get(&name) {
							return Some(info.clone());
						}
					}
				}
			}
		}
	}

	None
}

/// Build a synthetic `syn::Signature` that replaces dispatch machinery with
/// semantic equivalents. The result is fed to `generate_signature()` which
/// already handles Apply! simplification, qualified paths, and brand name
/// formatting.
///
/// Returns `None` if the Kind hash is not available (fallback to standalone macro).
fn build_synthetic_signature(
	_original_sig: &syn::Signature,
	dispatch_info: &DispatchTraitInfo,
) -> Option<syn::Signature> {
	let kind_trait_name = dispatch_info.kind_trait_name.as_ref()?;
	let brand_param = &dispatch_info.brand_param;
	let kind_ident: syn::Ident = syn::parse_str(kind_trait_name).ok()?;
	let brand_ident: syn::Ident = syn::parse_str(brand_param).ok()?;

	// Build generic params
	let mut generic_params: Vec<syn::GenericParam> = Vec::new();

	// Lifetime 'a
	generic_params.push(parse_quote!('a));

	// Brand: SemanticConstraint + Kind_hash
	if let Some(ref constraint_name) = dispatch_info.semantic_constraint {
		let constraint_ident: syn::Ident = syn::parse_str(constraint_name).ok()?;
		generic_params.push(parse_quote!(#brand_ident: #constraint_ident + #kind_ident));
	} else {
		generic_params.push(parse_quote!(#brand_ident: #kind_ident));
	}

	// Add type params in the order they appear in the dispatch trait definition.
	// This preserves the trait author's intended ordering for the forall clause.
	// Brand is already added above; skip it and add the remaining params.
	let mut all_element_types: Vec<String> = Vec::new();
	let secondary_map: std::collections::HashMap<&str, &str> =
		dispatch_info.secondary_constraints.iter().map(|(p, c)| (p.as_str(), c.as_str())).collect();

	for param_name in &dispatch_info.type_param_order {
		// Brand is already added as the first generic param
		if param_name == brand_param {
			continue;
		}

		// Secondary constraint params (e.g., F: Applicative, M: Applicative)
		if let Some(constraint_name) = secondary_map.get(param_name.as_str()) {
			let param_ident: syn::Ident = syn::parse_str(param_name).ok()?;
			let constraint_ident: syn::Ident = syn::parse_str(constraint_name).ok()?;
			generic_params.push(parse_quote!(#param_ident: #constraint_ident + #kind_ident));
			continue;
		}

		// Element type params
		if let Ok(param_ident) = syn::parse_str::<syn::Ident>(param_name) {
			generic_params.push(parse_quote!(#param_ident: 'a));
			all_element_types.push(param_name.clone());
		}
	}

	// Build function parameters by transforming the original signature's params.
	// - impl *Dispatch<...> -> impl Fn(inputs) -> output (from arrow type)
	// - Container types (FA, FB) -> <Brand as Kind_hash>::Of<'a, ElementType>
	// - Other params -> keep as-is
	//
	// The container_map maps FUNCTION type param names to element types.
	// This is built by matching the function's dispatch trait type args
	// (which use the function's param names like FA) against the
	// dispatch_info.container_params (which use the trait's param names like FTA).
	let container_map = build_container_map(_original_sig, dispatch_info);

	let mut fn_params: Vec<syn::FnArg> = Vec::new();

	for input in &_original_sig.inputs {
		let FnArg::Typed(pat_type) = input else {
			continue;
		};

		// Check if this is the impl Dispatch parameter -> replace with Fn closure
		if matches!(&*pat_type.ty, Type::ImplTrait(_))
			&& let Some(ref arrow) = dispatch_info.arrow_type
			&& let Some(closure_param) =
				build_closure_param(arrow, dispatch_info.tuple_closure, &brand_ident, &kind_ident)
		{
			fn_params.push(closure_param);
			continue;
		}
		// Skip impl Trait params that didn't produce a closure (e.g., placeholder)
		if matches!(&*pat_type.ty, Type::ImplTrait(_)) {
			continue;
		}

		// For tuple closure dispatch via where-clause (e.g., compose_kleisli_flipped
		// where (G, F): ComposeKleisliDispatch), detect tuple params of type vars
		// and replace with the closure tuple.
		if dispatch_info.tuple_closure
			&& matches!(&*pat_type.ty, Type::Tuple(tuple) if tuple.elems.len() >= 2)
			&& let Some(ref arrow) = dispatch_info.arrow_type
			&& let Some(closure_param) = build_closure_param(arrow, true, &brand_ident, &kind_ident)
		{
			fn_params.push(closure_param);
			continue;
		}

		// Check if this is a container type param -> replace with <Brand as Kind>::Of<...>
		let ty = &pat_type.ty;
		let type_str = quote!(#ty).to_string().replace(' ', "");
		if let Some(elements) = container_map.get(&type_str) {
			let pat = &pat_type.pat;
			let container_type = build_applied_type(&brand_ident, &kind_ident, elements)?;
			fn_params.push(parse_quote!(#pat: #container_type));
			continue;
		}

		// Check if this is a dispatch trait associated type projection
		// (e.g., <FA as ApplyFirstDispatch<...>>::FB -> Brand B)
		if let Type::Path(type_path) = &*pat_type.ty
			&& type_path.qself.is_some()
			&& let Some(last_seg) = type_path.path.segments.last()
		{
			let assoc_name = last_seg.ident.to_string();
			if let Some((_, elements)) =
				dispatch_info.associated_types.iter().find(|(name, _)| name == &assoc_name)
			{
				let pat = &pat_type.pat;
				let container_type = build_applied_type(&brand_ident, &kind_ident, elements)?;
				fn_params.push(parse_quote!(#pat: #container_type));
				continue;
			}
		}

		// For closureless dispatch or unrecognized container params:
		// if the type is an InferableBrand-bounded param, it's a container.
		// Fallback chain (most direct source first):
		// 1. self_type_elements: from the Val impl's self type (e.g., separate, compact)
		// 2. type_param_order: single-letter element types from the trait definition (e.g., alt)
		// 3. return structure: from the dispatch method's return type (last resort)
		if is_inferable_brand_param(&type_str, _original_sig) {
			use crate::analysis::dispatch::ReturnStructure;

			let element_types: Option<Vec<String>> = dispatch_info
				.self_type_elements
				.clone()
				.or_else(|| {
					// Extract single-letter element types from trait definition params,
					// excluding Brand and secondary constraint params.
					let elems: Vec<String> = dispatch_info
						.type_param_order
						.iter()
						.filter(|p| {
							*p != brand_param
								&& p.len() == 1 && !dispatch_info
								.secondary_constraints
								.iter()
								.any(|(sc, _)| sc == *p)
						})
						.cloned()
						.collect();
					if elems.is_empty() { None } else { Some(elems) }
				})
				.or_else(|| match &dispatch_info.return_structure {
					ReturnStructure::Applied(args) => Some(args.clone()),
					ReturnStructure::Nested {
						inner_args, ..
					} => Some(inner_args.clone()),
					ReturnStructure::Tuple(elements) => elements.first().cloned(),
					ReturnStructure::NestedTuple {
						inner_elements, ..
					} => inner_elements.first().cloned(),
					ReturnStructure::Plain(_) => None,
				});
			if let Some(ref elems) = element_types {
				let pat = &pat_type.pat;
				let container_type = build_applied_type(&brand_ident, &kind_ident, elems)?;
				fn_params.push(parse_quote!(#pat: #container_type));
				continue;
			}
		}

		// Keep other params as-is
		fn_params.push(input.clone());
	}

	// Build return type
	let return_type =
		build_return_type(&dispatch_info.return_structure, &brand_ident, &kind_ident)?;

	// Assemble the signature
	let generics = syn::Generics {
		lt_token: Some(Default::default()),
		params: generic_params.into_iter().collect(),
		gt_token: Some(Default::default()),
		where_clause: None,
	};

	Some(syn::Signature {
		constness: None,
		asyncness: None,
		unsafety: None,
		abi: None,
		fn_token: Default::default(),
		ident: syn::parse_str("synthetic").ok()?,
		generics,
		paren_token: Default::default(),
		inputs: fn_params.into_iter().collect(),
		variadic: None,
		output: syn::ReturnType::Type(Default::default(), Box::new(return_type)),
	})
}

/// Build a container map from the function's dispatch trait type args.
///
/// Uses the container_params' stored position indices to do a direct positional
/// lookup into the function's dispatch trait type args, avoiding heuristic scanning.
fn build_container_map(
	sig: &syn::Signature,
	dispatch_info: &DispatchTraitInfo,
) -> std::collections::HashMap<String, Vec<String>> {
	if dispatch_info.container_params.is_empty() {
		return std::collections::HashMap::new();
	}

	// Extract dispatch trait type args from whichever location has them:
	// either `impl *Dispatch<...>` parameter or where-clause bound.
	let fn_type_args = extract_dispatch_type_args(sig);
	if fn_type_args.is_empty() {
		return std::collections::HashMap::new();
	}

	// Use each container param's stored position to directly look up the
	// corresponding function type arg.
	let mut result = std::collections::HashMap::new();
	for cp in &dispatch_info.container_params {
		if let Some(fn_arg) = fn_type_args.get(cp.position) {
			result.insert(fn_arg.clone(), cp.element_types.clone());
		}
	}
	result
}

/// Extract the dispatch trait's type args from a function signature.
///
/// Checks both `impl *Dispatch<...>` parameters and where-clause bounds.
/// Returns the type args as stringified tokens (excluding lifetimes).
fn extract_dispatch_type_args(sig: &syn::Signature) -> Vec<String> {
	// Check impl Trait parameters
	for input in &sig.inputs {
		let FnArg::Typed(pat_type) = input else { continue };
		let Type::ImplTrait(impl_trait) = &*pat_type.ty else { continue };
		for bound in &impl_trait.bounds {
			if let Some(args) = extract_dispatch_trait_args(bound) {
				return args;
			}
		}
	}

	// Check where-clause bounds
	if let Some(where_clause) = &sig.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let syn::WherePredicate::Type(pred_type) = predicate {
				for bound in &pred_type.bounds {
					if let Some(args) = extract_dispatch_trait_args(bound) {
						return args;
					}
				}
			}
		}
	}

	Vec::new()
}

/// If a type param bound is a `*Dispatch<...>` trait, extract its type args.
fn extract_dispatch_trait_args(bound: &TypeParamBound) -> Option<Vec<String>> {
	let TypeParamBound::Trait(trait_bound) = bound else {
		return None;
	};
	let segment = trait_bound.path.segments.last()?;
	if !segment.ident.to_string().ends_with(crate::core::constants::markers::DISPATCH_SUFFIX) {
		return None;
	}
	let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
		return None;
	};
	Some(
		args.args
			.iter()
			.filter_map(|arg| {
				if let syn::GenericArgument::Type(ty) = arg {
					Some(quote!(#ty).to_string().replace(' ', ""))
				} else {
					None
				}
			})
			.collect(),
	)
}

/// Check if a type name is an InferableBrand-bounded param in the signature's where clause.
fn is_inferable_brand_param(
	type_name: &str,
	sig: &syn::Signature,
) -> bool {
	if let Some(where_clause) = &sig.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let syn::WherePredicate::Type(pred_type) = predicate {
				let bounded_ty = &pred_type.bounded_ty;
				let param_name = quote!(#bounded_ty).to_string().replace(' ', "");
				if param_name == type_name {
					for bound in &pred_type.bounds {
						if let TypeParamBound::Trait(trait_bound) = bound {
							let name = trait_bound
								.path
								.segments
								.last()
								.map(|s| s.ident.to_string())
								.unwrap_or_default();
							if name.starts_with("InferableBrand_") {
								return true;
							}
						}
					}
				}
			}
		}
	}
	false
}

/// Build a `<Brand as Kind_hash>::Of<'a, A, B, ...>` qualified path type.
fn build_applied_type(
	brand_ident: &syn::Ident,
	kind_ident: &syn::Ident,
	element_types: &[String],
) -> Option<syn::Type> {
	let mut args = vec![quote!('a)];
	for elem in element_types {
		let elem_type: syn::Type = syn::parse_str(elem).ok()?;
		args.push(quote!(#elem_type));
	}
	let args_tokens = quote!(#(#args),*);
	Some(parse_quote!(<#brand_ident as #kind_ident>::Of<#args_tokens>))
}

/// Build the closure parameter as `impl Fn(inputs) -> output`.
fn build_closure_param(
	arrow: &crate::analysis::dispatch::DispatchArrow,
	tuple_closure: bool,
	brand_ident: &syn::Ident,
	kind_ident: &syn::Ident,
) -> Option<syn::FnArg> {
	use crate::analysis::dispatch::{
		ArrowOutput,
		DispatchArrowParam,
	};

	if tuple_closure {
		// For tuple closures (bimap, etc.), each input is a sub-arrow.
		// Build bare fn pointer types in a tuple. The HM pipeline handles
		// fn(A) -> B via visit_bare_fn.
		let mut fn_types: Vec<syn::Type> = Vec::new();

		for param in &arrow.inputs {
			let sub_arrow = match param {
				DispatchArrowParam::SubArrow(arrow) => arrow,
				DispatchArrowParam::TypeParam(sub_arrow_str) => {
					// Legacy string path: split on " -> " and parse
					if let Some(arrow_pos) = sub_arrow_str.rfind(" -> ") {
						let input_str = &sub_arrow_str[.. arrow_pos];
						let output_str = &sub_arrow_str[arrow_pos + 4 ..];
						let input_str =
							input_str.trim().trim_start_matches('(').trim_end_matches(')');
						let input_types: Vec<syn::Type> = input_str
							.split(',')
							.filter_map(|s| syn::parse_str(s.trim()).ok())
							.collect();
						let output_type: syn::Type =
							syn::parse_str(output_str.trim()).unwrap_or_else(|_| parse_quote!(()));
						fn_types.push(parse_quote!(fn(#(#input_types),*) -> #output_type));
					}
					continue;
				}
				_ => continue,
			};

			// Build fn type from structured sub-arrow
			let mut input_types: Vec<syn::Type> = Vec::new();
			for sub_param in &sub_arrow.inputs {
				match sub_param {
					DispatchArrowParam::TypeParam(name) => {
						let ident: syn::Ident = syn::parse_str(name).ok()?;
						input_types.push(parse_quote!(#ident));
					}
					DispatchArrowParam::AssociatedType {
						assoc_name,
					} => {
						let assoc_ident: syn::Ident = syn::parse_str(assoc_name).ok()?;
						input_types.push(parse_quote!(#brand_ident::#assoc_ident));
					}
					DispatchArrowParam::SubArrow(_) => continue,
				}
			}

			let output_type: syn::Type = match &sub_arrow.output {
				ArrowOutput::Plain(s) => syn::parse_str(s).ok()?,
				ArrowOutput::BrandApplied(args) =>
					build_applied_type(brand_ident, kind_ident, args)?,
				ArrowOutput::OtherApplied {
					brand,
					args,
				} => {
					let other_ident: syn::Ident = syn::parse_str(brand).ok()?;
					build_applied_type(&other_ident, kind_ident, args)?
				}
			};

			fn_types.push(parse_quote!(fn(#(#input_types),*) -> #output_type));
		}

		if fn_types.is_empty() {
			return None;
		}

		return Some(parse_quote!(fg: (#(#fn_types),*)));
	}

	// Single closure: build impl Fn(A, B, ...) -> R
	let mut input_types: Vec<syn::Type> = Vec::new();
	for param in &arrow.inputs {
		match param {
			DispatchArrowParam::TypeParam(name) => {
				let ident: syn::Ident = syn::parse_str(name).ok()?;
				input_types.push(parse_quote!(#ident));
			}
			DispatchArrowParam::AssociatedType {
				assoc_name,
			} => {
				let assoc_ident: syn::Ident = syn::parse_str(assoc_name).ok()?;
				input_types.push(parse_quote!(#brand_ident::#assoc_ident));
			}
			DispatchArrowParam::SubArrow(_) => {
				// SubArrow is only used in tuple closures, not single closures
				continue;
			}
		}
	}

	let output_type: syn::Type = match &arrow.output {
		ArrowOutput::Plain(s) => syn::parse_str(s).ok()?,
		ArrowOutput::BrandApplied(args) => build_applied_type(brand_ident, kind_ident, args)?,
		ArrowOutput::OtherApplied {
			brand,
			args,
		} => {
			let other_ident: syn::Ident = syn::parse_str(brand).ok()?;
			build_applied_type(&other_ident, kind_ident, args)?
		}
	};

	Some(parse_quote!(f: impl Fn(#(#input_types),*) -> #output_type + 'a))
}

/// Build the return type from `ReturnStructure`.
fn build_return_type(
	ret: &crate::analysis::dispatch::ReturnStructure,
	brand_ident: &syn::Ident,
	kind_ident: &syn::Ident,
) -> Option<syn::Type> {
	use crate::analysis::dispatch::ReturnStructure;

	match ret {
		ReturnStructure::Plain(var) => Some(syn::parse_str(var).ok()?),
		ReturnStructure::Applied(args) => build_applied_type(brand_ident, kind_ident, args),
		ReturnStructure::Nested {
			outer_param,
			inner_args,
		} => {
			let outer_ident: syn::Ident = syn::parse_str(outer_param).ok()?;
			let inner_type = build_applied_type(brand_ident, kind_ident, inner_args)?;
			Some(parse_quote!(<#outer_ident as #kind_ident>::Of<'a, #inner_type>))
		}
		ReturnStructure::Tuple(elements) => {
			let elem_types: Vec<syn::Type> = elements
				.iter()
				.filter_map(|args| build_applied_type(brand_ident, kind_ident, args))
				.collect();
			Some(parse_quote!((#(#elem_types),*)))
		}
		ReturnStructure::NestedTuple {
			outer_param,
			inner_elements,
		} => {
			let outer_ident: syn::Ident = syn::parse_str(outer_param).ok()?;
			let tuple_types: Vec<syn::Type> = inner_elements
				.iter()
				.filter_map(|args| build_applied_type(brand_ident, kind_ident, args))
				.collect();
			let tuple_type: syn::Type = parse_quote!((#(#tuple_types),*));
			Some(parse_quote!(<#outer_ident as #kind_ident>::Of<'a, #tuple_type>))
		}
	}
}
