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
		documentation::document_signature::{
			SignatureData,
			generate_signature,
		},
		hm::HmAst,
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
		GenericParam,
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

	let Some(dispatch_info) = find_dispatch_trait_in_sig(&item_fn.sig, config) else {
		// No dispatch trait found; leave #[document_signature] for the standalone macro
		return;
	};

	// Remove the attribute so the standalone macro does not also process it
	item_fn.attrs.remove(attr_pos);

	// Generate dispatch-aware signature
	let sig_data = generate_dispatch_signature(&item_fn.sig, &dispatch_info, config);

	let doc_comment = format!("`{sig_data}`");
	let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
	item_fn.attrs.insert(attr_pos, doc_attr);

	let header_attr: syn::Attribute = parse_quote!(#[doc = r#"### Type Signature
"#]);
	item_fn.attrs.insert(attr_pos, header_attr);
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

/// Generate a dispatch-aware HM signature from the function signature and dispatch info.
fn generate_dispatch_signature(
	sig: &syn::Signature,
	dispatch_info: &DispatchTraitInfo,
	_config: &Config,
) -> SignatureData {
	// Collect type parameters, excluding infrastructure ones (Marker, FnBrand)
	// and container params with InferableBrand bounds (these become F A in the output)
	let inferable_params = find_inferable_brand_params(sig);
	let mut forall = Vec::new();
	let brand_var = "F".to_string();

	for param in &sig.generics.params {
		if let GenericParam::Type(type_param) = param {
			let name = type_param.ident.to_string();
			// Skip infrastructure params
			if name == crate::core::constants::markers::MARKER_PARAM
				|| name == crate::core::constants::markers::FN_BRAND_PARAM
			{
				continue;
			}
			// Skip InferableBrand-bound params (they become F A)
			if inferable_params.contains(&name) {
				// Use the first inferable param's name to derive the brand variable
				if forall.iter().all(|v: &String| v != &brand_var) {
					forall.push(brand_var.clone());
				}
				continue;
			}
			forall.push(name);
		}
	}

	// Build constraints
	let mut constraints = Vec::new();
	if let Some(ref semantic) = dispatch_info.semantic_constraint {
		constraints.push(format!("{semantic} {brand_var}"));
	}
	for (param, constraint) in &dispatch_info.secondary_constraints {
		constraints.push(format!("{constraint} {param}"));
	}

	// Build parameters
	let mut params = Vec::new();

	// Add the arrow type (if not closureless)
	if let Some(ref arrow) = dispatch_info.arrow_type {
		let arrow_hm = dispatch_arrow_to_hm(arrow, dispatch_info.tuple_closure);
		params.push(arrow_hm);
	}

	// Add non-dispatch, non-infrastructure parameters
	for input in &sig.inputs {
		let FnArg::Typed(pat_type) = input else {
			continue;
		};
		// Skip the impl Dispatch parameter (already handled via arrow_type)
		if matches!(&*pat_type.ty, Type::ImplTrait(_)) {
			continue;
		}
		let ty = &pat_type.ty;
		let type_str = quote!(#ty).to_string();
		// Skip PhantomData-like params
		if type_str.contains("PhantomData") {
			continue;
		}
		params.push(type_to_param_hm(&pat_type.ty, &inferable_params, &brand_var));
	}

	// Build return type from dispatch trait's ReturnStructure
	let return_type = build_dispatch_return_type(dispatch_info, &brand_var);

	SignatureData {
		forall,
		constraints,
		params,
		return_type,
	}
}

/// Find all type parameters that have an InferableBrand bound.
fn find_inferable_brand_params(sig: &syn::Signature) -> Vec<String> {
	let mut result = Vec::new();

	if let Some(where_clause) = &sig.generics.where_clause {
		for predicate in &where_clause.predicates {
			if let syn::WherePredicate::Type(pred_type) = predicate {
				let bounded_ty = &pred_type.bounded_ty;
				for bound in &pred_type.bounds {
					if let TypeParamBound::Trait(trait_bound) = bound {
						let name = trait_bound
							.path
							.segments
							.last()
							.map(|s| s.ident.to_string())
							.unwrap_or_default();
						if name.starts_with("InferableBrand_") {
							let clean_name = quote!(#bounded_ty).to_string().replace(' ', "");
							result.push(clean_name);
						}
					}
				}
			}
		}
	}

	// Also check inline bounds
	for param in &sig.generics.params {
		if let GenericParam::Type(type_param) = param {
			for bound in &type_param.bounds {
				if let TypeParamBound::Trait(trait_bound) = bound {
					let name = trait_bound
						.path
						.segments
						.last()
						.map(|s| s.ident.to_string())
						.unwrap_or_default();
					if name.starts_with("InferableBrand_") {
						result.push(type_param.ident.to_string());
					}
				}
			}
		}
	}

	result
}

/// Convert a DispatchArrow to an HmAst.
fn dispatch_arrow_to_hm(
	arrow: &crate::analysis::dispatch::DispatchArrow,
	tuple_closure: bool,
) -> HmAst {
	use crate::analysis::dispatch::DispatchArrowParam;

	if tuple_closure {
		// For tuple closures (bimap, etc.), the inputs are sub-arrows
		let sub_arrows: Vec<HmAst> = arrow
			.inputs
			.iter()
			.map(|p| match p {
				DispatchArrowParam::TypeParam(s) => HmAst::Variable(s.clone()),
				DispatchArrowParam::AssociatedType {
					assoc_name,
				} => HmAst::Variable(assoc_name.clone()),
			})
			.collect();
		HmAst::Tuple(sub_arrows)
	} else {
		let inputs: Vec<HmAst> = arrow
			.inputs
			.iter()
			.map(|p| match p {
				DispatchArrowParam::TypeParam(s) => HmAst::Variable(s.clone()),
				DispatchArrowParam::AssociatedType {
					assoc_name,
				} => HmAst::Variable(assoc_name.clone()),
			})
			.collect();

		let input = if inputs.len() == 1 {
			inputs.into_iter().next().unwrap_or(HmAst::Unit)
		} else {
			HmAst::Tuple(inputs)
		};

		let output = arrow_output_to_hm(&arrow.output);
		HmAst::Arrow(Box::new(input), Box::new(output))
	}
}

/// Convert an ArrowOutput to an HmAst.
fn arrow_output_to_hm(output: &crate::analysis::dispatch::ArrowOutput) -> HmAst {
	use crate::analysis::dispatch::ArrowOutput;
	match output {
		ArrowOutput::Plain(s) => HmAst::Variable(s.clone()),
		ArrowOutput::BrandApplied(args) => {
			let hm_args: Vec<HmAst> = args.iter().map(|a| HmAst::Variable(a.clone())).collect();
			HmAst::Constructor(
				crate::core::constants::markers::DEFAULT_BRAND_PARAM.to_string(),
				hm_args,
			)
		}
		ArrowOutput::OtherApplied {
			brand,
			args,
		} => {
			let hm_args: Vec<HmAst> = args.iter().map(|a| HmAst::Variable(a.clone())).collect();
			HmAst::Constructor(brand.clone(), hm_args)
		}
	}
}

/// Convert a parameter type to an HmAst, substituting InferableBrand params with F A.
fn type_to_param_hm(
	ty: &Type,
	inferable_params: &[String],
	brand_var: &str,
) -> HmAst {
	let type_str = quote!(#ty).to_string().replace(' ', "");

	// Check if this is an InferableBrand-bound type param
	for param in inferable_params {
		if type_str == *param {
			return inferable_param_to_hm(param, brand_var);
		}
	}

	HmAst::Variable(type_str)
}

/// Convert an InferableBrand-bound param name to F A (or P A C for arity-2).
fn inferable_param_to_hm(
	param: &str,
	brand_var: &str,
) -> HmAst {
	// FA -> F A, FB -> F B, FTA -> F A (strip the F prefix, keep the rest)
	let element = if param.starts_with('F') && param.len() >= 2 { &param[1 ..] } else { param };
	HmAst::Constructor(brand_var.to_string(), vec![HmAst::Variable(element.to_string())])
}

/// Build the HM return type using the dispatch trait's `ReturnStructure`.
fn build_dispatch_return_type(
	dispatch_info: &DispatchTraitInfo,
	brand_var: &str,
) -> HmAst {
	use crate::analysis::dispatch::ReturnStructure;

	match &dispatch_info.return_structure {
		ReturnStructure::Plain(var) => HmAst::Variable(var.clone()),
		ReturnStructure::Applied(args) => {
			let hm_args: Vec<HmAst> = args.iter().map(|a| HmAst::Variable(a.clone())).collect();
			HmAst::Constructor(brand_var.to_string(), hm_args)
		}
		ReturnStructure::Nested {
			outer_param,
			inner_args,
		} => {
			let inner_hm_args: Vec<HmAst> =
				inner_args.iter().map(|a| HmAst::Variable(a.clone())).collect();
			let inner = HmAst::Constructor(brand_var.to_string(), inner_hm_args);
			HmAst::Constructor(outer_param.clone(), vec![inner])
		}
	}
}
