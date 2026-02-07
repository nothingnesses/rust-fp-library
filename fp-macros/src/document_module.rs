use crate::{
	apply::ApplyInput,
	doc_utils::{DocArg, GenericArgs, validate_doc_args},
	function_utils::{Config, format_brand_name},
	hm_signature::generate_signature,
	impl_kind::ImplKindInput,
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashMap;
use syn::{
	Attribute, Error, GenericParam, ImplItem, Item, Result, Signature,
	parse::{Parse, ParseStream},
	parse_quote,
	spanned::Spanned,
	visit_mut::{self, VisitMut},
};

pub struct DocumentModuleInput {
	pub items: Vec<Item>,
}

impl Parse for DocumentModuleInput {
	fn parse(input: ParseStream) -> Result<Self> {
		let mut items = Vec::new();
		while !input.is_empty() {
			items.push(input.parse()?);
		}
		Ok(DocumentModuleInput { items })
	}
}

pub fn document_module_impl(
	_attr: TokenStream,
	item: TokenStream,
) -> TokenStream {
	// eprintln!("\n[document_module] START document_module_impl");
	// eprintln!("[document_module] INPUT: {}", item.to_string());
	// Try to parse as a list of items (inner attribute case)
	let mut items = if let Ok(input) = syn::parse2::<DocumentModuleInput>(item.clone()) {
		input.items
	} else if let Ok(item_mod) = syn::parse2::<syn::ItemMod>(item.clone()) {
		// Outer attribute on a module case
		if let Some((_, mod_items)) = item_mod.content {
			mod_items
		} else {
			// mod foo; case - we can't see the content easily
			return syn::Error::new(
				item_mod.span(),
				"document_module cannot see the content of file modules when used as an outer attribute. Use an inner attribute #![document_module] instead, or wrap the content in a mod block.",
			).to_compile_error();
		}
	} else if let Ok(item_const) = syn::parse2::<syn::ItemConst>(item) {
		// Outer attribute on a const block case: const _: () = { ... };
		if let syn::Expr::Block(expr_block) = *item_const.expr {
			expr_block
				.block
				.stmts
				.into_iter()
				.filter_map(|stmt| match stmt {
					syn::Stmt::Item(item) => Some(item),
					_ => None,
				})
				.collect()
		} else {
			return syn::Error::new(
				item_const.span(),
				"document_module on a const item requires a block expression: const _: () = { ... };",
			)
			.to_compile_error();
		}
	} else {
		return syn::Error::new(
			proc_macro2::Span::call_site(),
			"document_module must be applied to a module, a const block, or used as an inner attribute in a module.",
		).to_compile_error();
	};

	let mut config = Config::default();

	// Pass 1: Context Extraction
	if let Err(e) = extract_context(&items, &mut config) {
		return e.to_compile_error();
	}

	// Pass 2: Documentation Generation
	if let Err(e) = generate_docs(&mut items, &config) {
		return e.to_compile_error();
	}

	let result = quote!(#(#items)*);
	// eprintln!("[document_module] END document_module_impl");
	result
}

fn extract_context(
	items: &[Item],
	config: &mut Config,
) -> Result<()> {
	let mut errors: Vec<Error> = Vec::new();

	// Track defaults per (Type, Trait) to detect conflicts across split impl blocks
	let mut scoped_defaults_tracker: std::collections::HashMap<
		(String, String),
		Vec<(String, proc_macro2::Span)>,
	> = std::collections::HashMap::new();

	for item in items {
		match item {
			Item::Macro(m) if m.mac.path.is_ident("impl_kind") => {
				if let Ok(impl_kind) = m.mac.parse_body::<ImplKindInput>() {
					let brand_path = impl_kind.brand.to_token_stream().to_string();
					for def in &impl_kind.definitions {
						let assoc_name = def.ident.to_string();

						// Check for circular references (Self:: forbidden in RHS)
						if type_uses_self_assoc(&def.target_type) {
							errors.push(Error::new(
								def.target_type.span(),
								"Self:: reference forbidden in associated type definition",
							));
						}

						// Check for collisions in impl_kind!
						// We use normalized types to detect semantic collisions even if generic names differ
						let normalized_target =
							normalize_type(def.target_type.clone(), &def.generics);
						let key = (brand_path.clone(), None, assoc_name.clone());
						if let Some((prev_generics, prev_type)) = config.projections.get(&key) {
							let prev_normalized = normalize_type(prev_type.clone(), prev_generics);
							if quote!(#prev_normalized).to_string()
								!= quote!(#normalized_target).to_string()
							{
								errors.push(Error::new(
									def.ident.span(),
									format!(
										"Conflicting implementation for {}: already defined with different type",
										assoc_name
									),
								));
							}
						}

						config
							.projections
							.insert(key, (def.generics.clone(), def.target_type.clone()));

						if has_attr(&def.attrs, "doc_default")
							&& let Some(prev) = config
								.module_defaults
								.insert(brand_path.clone(), assoc_name.clone())
						{
							errors.push(Error::new(
								def.ident.span(),
								format!(
									"Conflicting module default for {}: {} and {}",
									brand_path, prev, assoc_name
								),
							));
						}
					}
				}
			}
			Item::Impl(item_impl) => {
				let self_ty_path = item_impl.self_ty.to_token_stream().to_string();
				let trait_path = item_impl
					.trait_
					.as_ref()
					.map(|(_, path, _)| path.to_token_stream().to_string());

				// Split impl block merging: merge associated types across multiple impl blocks
				for item in &item_impl.items {
					if let ImplItem::Type(assoc_type) = item {
						let assoc_name = assoc_type.ident.to_string();

						// Store projection (multiple impl blocks can define same assoc type)
						config.projections.insert(
							(self_ty_path.clone(), trait_path.clone(), assoc_name.clone()),
							(assoc_type.generics.clone(), assoc_type.ty.clone()),
						);

						// Track doc_default across split impl blocks
						if has_attr(&assoc_type.attrs, "doc_default")
							&& let Some(t_path) = &trait_path
						{
							let key = (self_ty_path.clone(), t_path.clone());
							scoped_defaults_tracker
								.entry(key)
								.or_default()
								.push((assoc_name.clone(), assoc_type.ident.span()));
						}
					}
				}
			}
			_ => {}
		}
	}

	// Check for conflicting defaults across split impl blocks
	for ((self_ty, trait_path), defaults) in scoped_defaults_tracker {
		if defaults.len() > 1 {
			let names: Vec<_> = defaults.iter().map(|(n, _)| n.as_str()).collect();
			for (_name, span) in &defaults {
				errors.push(Error::new(
					*span,
					format!(
						"Multiple #[doc_default] annotations for ({}, {}): {}",
						self_ty,
						trait_path,
						names.join(", ")
					),
				));
			}
		} else if let Some((name, _)) = defaults.first() {
			config.scoped_defaults.insert((self_ty, trait_path), name.clone());
		}
	}

	if errors.is_empty() {
		Ok(())
	} else {
		let mut combined_error = errors.remove(0);
		for err in errors {
			combined_error.combine(err);
		}
		Err(combined_error)
	}
}

/// Extract the concrete type name from a Type for use in HM signatures
fn extract_concrete_type_name(
	ty: &syn::Type,
	config: &Config,
) -> Option<String> {
	match ty {
		syn::Type::Path(type_path) => {
			if let Some(segment) = type_path.path.segments.first() {
				let name = segment.ident.to_string();
				// Apply brand name formatting
				Some(format_brand_name(&name, config))
			} else {
				None
			}
		}
		_ => None,
	}
}

/// Extract base type name and generic parameter names from impl self type
/// For `impl<A> CatList<A>`, returns ("CatList", ["A"])
fn extract_self_type_info(
	self_ty: &syn::Type,
	impl_generics: &syn::Generics,
) -> (Option<String>, Vec<String>) {
	let base_name = match self_ty {
		syn::Type::Path(type_path) => {
			type_path.path.segments.last().map(|seg| seg.ident.to_string())
		}
		_ => None,
	};

	let generic_names: Vec<String> = impl_generics
		.params
		.iter()
		.filter_map(|p| match p {
			syn::GenericParam::Type(t) => Some(t.ident.to_string()),
			_ => None,
		})
		.collect();

	(base_name, generic_names)
}

/// Build a parameterized type from a base name and generic parameters
/// For ("CatList", ["A"]), returns `CatList<A>`
fn build_parameterized_type(
	base_name: &str,
	generic_params: &[String],
) -> syn::Type {
	if generic_params.is_empty() {
		parse_quote!(#base_name)
	} else {
		let params: Vec<syn::Ident> = generic_params
			.iter()
			.map(|p| syn::Ident::new(p, proc_macro2::Span::call_site()))
			.collect();
		parse_quote!(#base_name<#(#params),*>)
	}
}

fn generate_docs(
	items: &mut [Item],
	config: &Config,
) -> Result<()> {
	let mut errors = Vec::new();

	for item in items {
		if let Item::Impl(item_impl) = item {
			let self_ty = &*item_impl.self_ty;
			let self_ty_path = quote!(#self_ty).to_string();
			let trait_path = item_impl.trait_.as_ref().map(|(_, path, _)| path);
			let trait_name = trait_path.map(|p| p.segments.last().unwrap().ident.to_string());
			let trait_path_str = trait_path.map(|p| quote!(#p).to_string());
			
			// eprintln!("\n[document_module] Processing impl block for: {}", self_ty_path);
			// eprintln!("[document_module] Trait: {:?}", trait_name);
			// eprintln!("[document_module] Number of impl items: {}", item_impl.items.len());

			// for (i, attr) in item_impl.attrs.iter().enumerate() {
			// 	eprintln!("[document_module] Impl Attr {}: {:?}", i, attr.path().to_token_stream().to_string());
			// }

			let impl_doc_use = match find_attr_value_checked(&item_impl.attrs, "doc_use") {
				Ok(v) => v,
				Err(e) => {
					errors.push(e);
					None
				}
			};

			for impl_item in &mut item_impl.items {
				if let ImplItem::Fn(method) = impl_item {
					// eprintln!("[document_module] Processing method: {}", method.sig.ident);
					// for (i, attr) in method.attrs.iter().enumerate() {
					// 	eprintln!("[document_module] Method Attr {}: {:?}", i, attr.path().to_token_stream().to_string());
					// }
					// eprintln!("[document_module] Has hm_signature attr: {}", find_attribute(&method.attrs, "hm_signature").is_some());
					let method_doc_use = match find_attr_value_checked(&method.attrs, "doc_use") {
						Ok(v) => v,
						Err(e) => {
							errors.push(e);
							None
						}
					};
					let doc_use = method_doc_use.or_else(|| impl_doc_use.clone());

					// 1. Handle HM Signature
					if let Some(attr_pos) = find_attribute(&method.attrs, "hm_signature") {
						method.attrs.remove(attr_pos);

						let mut synthetic_sig = method.sig.clone();

						// Extract base type name and generic parameters from impl
						let (base_type_name, impl_generic_params) =
							extract_self_type_info(self_ty, &item_impl.generics);

						// DEBUG: Log the input state
						eprintln!("\n=== DEBUG: HM Signature Generation ===");
						eprintln!("Method: {}", method.sig.ident);
						eprintln!("Base type: {:?}", base_type_name);
						eprintln!("Impl generics: {:?}", impl_generic_params);
						eprintln!("Original return type: {}", quote::quote!(#(&synthetic_sig.output)));

						// Resolve Self
						let mut substitutor = SelfSubstitutor {
							self_ty,
							self_ty_path: &self_ty_path,
							trait_path: trait_path_str.as_deref(),
							doc_use: doc_use.as_deref(),
							config,
							errors: Vec::new(),
							base_type_name: base_type_name.clone(),
							impl_generic_params: impl_generic_params.clone(),
						};
						substitutor.visit_signature_mut(&mut synthetic_sig);

						eprintln!("After substitution return type: {}", quote::quote!(#(&synthetic_sig.output)));

						// Collect any resolution errors
						errors.extend(substitutor.errors);

						// Merge generics
						merge_generics(&mut synthetic_sig, &item_impl.generics);

						eprintln!("After merge, generics: {:?}", synthetic_sig.generics.params.iter().map(|p| quote::quote!(#p).to_string()).collect::<Vec<_>>());

						// Add trait bound: SelfTy: Trait (only if it's a trait impl)
						if let Some(trait_path) = trait_path {
							let where_clause = synthetic_sig.generics.make_where_clause();
							where_clause.predicates.push(parse_quote!(#self_ty: #trait_path));
						}

						// Create a modified config with concrete type information
						let mut sig_config = config.clone();

						// Extract and add the concrete type name
						if let Some(concrete_type_name) =
							extract_concrete_type_name(self_ty, config)
						{
							sig_config.concrete_types.insert(concrete_type_name.clone());
							sig_config.self_type_name = Some(concrete_type_name);
						}

						let signature_data =
							generate_signature(&synthetic_sig, trait_name.as_deref(), &sig_config);
						
						eprintln!("Generated signature: {}", signature_data);
						eprintln!("=== END DEBUG ===\n");
						
						let doc_comment = format!("`{}`", signature_data);
						let doc_attr: syn::Attribute = parse_quote!(#[doc = #doc_comment]);
						method.attrs.insert(attr_pos, doc_attr);
					}

					// 2. Handle Doc Type Params
					if let Some(attr_pos) = find_attribute(&method.attrs, "doc_type_params") {
						let attr = method.attrs.remove(attr_pos);

						// Parse the arguments from the attribute
						if let Ok(args) = attr.parse_args::<GenericArgs>() {
							// Get all generics: impl generics + method generics
							let mut all_generics = syn::Generics::default();

							// Add impl generics first
							for param in &item_impl.generics.params {
								all_generics.params.push(param.clone());
							}

							// Add method generics
							for param in &method.sig.generics.params {
								all_generics.params.push(param.clone());
							}

							let targets: Vec<String> = all_generics
								.params
								.iter()
								.map(|p| match p {
									GenericParam::Type(t) => t.ident.to_string(),
									GenericParam::Lifetime(l) => l.lifetime.ident.to_string(),
									GenericParam::Const(c) => c.ident.to_string(),
								})
								.collect();

							let entries: Vec<_> = args.entries.iter().collect();
							if let Err(e) =
								validate_doc_args(targets.len(), entries.len(), attr.span())
							{
								errors.push(e);
							} else {
								for (i, (name_from_target, entry)) in
									targets.iter().zip(entries).enumerate()
								{
									let (name, desc) = match entry {
										DocArg::Override(n, d) => (n.value(), d.value()),
										DocArg::Desc(d) => (name_from_target.clone(), d.value()),
									};

									let doc_comment = format!("* `{}`: {}", name, desc);
									let doc_attr: syn::Attribute =
										parse_quote!(#[doc = #doc_comment]);
									method.attrs.insert(attr_pos + i, doc_attr);
								}
							}
						} else {
							errors.push(Error::new(
								attr.span(),
								"Failed to parse doc_type_params arguments",
							));
						}
					}
				}
			}
		}
	}

	if errors.is_empty() {
		Ok(())
	} else {
		let mut combined_error: Error = errors.remove(0);
		for err in errors {
			combined_error.combine(err);
		}
		Err(combined_error)
	}
}

fn find_attribute(
	attrs: &[Attribute],
	name: &str,
) -> Option<usize> {
	attrs.iter().position(|attr| attr.path().is_ident(name))
}

fn find_attr_value_checked(
	attrs: &[Attribute],
	name: &str,
) -> Result<Option<String>> {
	let mut found = None;
	for attr in attrs {
		if attr.path().is_ident(name) {
			if found.is_some() {
				return Err(Error::new(
					attr.span(),
					format!("Multiple `#[{}]` attributes found on same item", name),
				));
			}
			if let syn::Meta::NameValue(nv) = &attr.meta
				&& let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value
			{
				found = Some(s.value());
			}
		}
	}
	Ok(found)
}

fn merge_generics(
	sig: &mut Signature,
	impl_generics: &syn::Generics,
) {
	let mut new_params = syn::punctuated::Punctuated::<GenericParam, syn::token::Comma>::new();
	for p in impl_generics.params.iter().chain(sig.generics.params.iter()) {
		if let GenericParam::Lifetime(_) = p {
			new_params.push(p.clone());
		}
	}
	for p in impl_generics.params.iter().chain(sig.generics.params.iter()) {
		if let GenericParam::Type(_) = p {
			new_params.push(p.clone());
		}
	}
	for p in impl_generics.params.iter().chain(sig.generics.params.iter()) {
		if let GenericParam::Const(_) = p {
			new_params.push(p.clone());
		}
	}
	sig.generics.params = new_params;

	if let Some(impl_where) = &impl_generics.where_clause {
		let where_clause = sig.generics.make_where_clause();
		for pred in &impl_where.predicates {
			where_clause.predicates.push(pred.clone());
		}
	}
}

struct SelfSubstitutor<'a> {
	self_ty: &'a syn::Type,
	self_ty_path: &'a str,
	trait_path: Option<&'a str>,
	doc_use: Option<&'a str>,
	config: &'a Config,
	errors: Vec<Error>,
	/// The base type name (e.g., "CatList") extracted from self_ty
	base_type_name: Option<String>,
	/// Generic parameter names from the impl block (e.g., ["A"])
	impl_generic_params: Vec<String>,
}

impl<'a> VisitMut for SelfSubstitutor<'a> {
	fn visit_type_mut(
		&mut self,
		i: &mut syn::Type,
	) {
		if let syn::Type::Path(tp) = i {
			if tp.path.is_ident("Self") {
				// Resolve bare Self
				let assoc_name = self
					.doc_use
					.map(|s| s.to_string())
					.or_else(|| {
						self.trait_path.and_then(|tp| {
							self.config
								.scoped_defaults
								.get(&(self.self_ty_path.to_string(), tp.to_string()))
								.cloned()
						})
					})
					.or_else(|| self.config.module_defaults.get(self.self_ty_path).cloned());

				if let Some(assoc) = assoc_name {
					if let Some((_generics, target)) = self
						.config
						.projections
						.get(&(
							self.self_ty_path.to_string(),
							self.trait_path.map(|s| s.to_string()),
							assoc.clone(),
						))
						.or_else(|| {
							self.config.projections.get(&(
								self.self_ty_path.to_string(),
								None,
								assoc.clone(),
							))
						}) {
						// For bare Self, we don't have generic arguments to substitute,
						// but we should still respect the definition's generics if any.
						// Actually, bare Self usually resolves to the primary association.
						*i = target.clone();
					} else {
						// Fallback: use parameterized concrete type if available
						if let Some(base_name) = &self.base_type_name {
							*i = build_parameterized_type(base_name, &self.impl_generic_params);
						} else {
							*i = self.self_ty.clone();
						}
					}
				} else {
					// No default found - use parameterized concrete type if available
					if let Some(base_name) = &self.base_type_name {
						*i = build_parameterized_type(base_name, &self.impl_generic_params);
					} else {
						// Report error with available types
						self.errors.push(create_missing_default_error(
							tp.span(),
							self.self_ty_path,
							self.trait_path,
							self.config,
						));
						// Fallback to self_ty
						*i = self.self_ty.clone();
					}
				}
			} else if let Some(first) = tp.path.segments.first()
				&& first.ident == "Self"
				&& tp.path.segments.len() > 1
			{
				let segment = &tp.path.segments[1];
				let assoc_name = segment.ident.to_string();
				if let Some((generics, target)) = self
					.config
					.projections
					.get(&(
						self.self_ty_path.to_string(),
						self.trait_path.map(|s| s.to_string()),
						assoc_name.clone(),
					))
					.or_else(|| {
						self.config.projections.get(&(
							self.self_ty_path.to_string(),
							None,
							assoc_name.clone(),
						))
					}) {
					if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
						*i = substitute_generics(target.clone(), generics, &args.args);
					} else {
						*i = target.clone();
					}
				} else {
					// Report error with available types
					self.errors.push(create_missing_assoc_type_error(
						tp.span(),
						self.self_ty_path,
						&assoc_name,
						self.trait_path,
						self.config,
					));
					// Fallback to qualified path
					let self_ty = self.self_ty;
					let mut new_path = tp.path.clone();
					new_path.segments = new_path.segments.into_iter().skip(1).collect();
					let segments = &new_path.segments;
					*i = parse_quote!(<#self_ty>::#segments);
				}
			}
		}
		visit_mut::visit_type_mut(self, i);
	}

	fn visit_type_macro_mut(
		&mut self,
		i: &mut syn::TypeMacro,
	) {
		if i.mac.path.is_ident("Apply")
			&& let Ok(mut apply_input) = syn::parse2::<ApplyInput>(i.mac.tokens.clone())
		{
			self.visit_type_mut(&mut apply_input.brand);
			for arg in apply_input.args.args.iter_mut() {
				if let syn::GenericArgument::Type(ty) = arg {
					self.visit_type_mut(ty);
				}
			}

			// If the brand resolved to a target type with generics, and Apply! has arguments,
			// we should perform substitution if possible.
			// However, document_module's SelfSubstitutor currently only replaces the type.
			// The HM signature generator will handle the projection if it remains a path.
			// But if we already replaced it with a concrete type (e.g. CatList),
			// we might have CatList instead of CatList<A>.

			// Let's check if brand was substituted.
			// We can use the same logic as visit_type_mut for Self segments.

			let brand = &apply_input.brand;
			let kind_input = &apply_input.kind_input;
			let assoc_name = &apply_input.assoc_name;
			let args = &apply_input.args;

			i.mac.tokens = quote! { <#brand as Kind!(#kind_input)>::#assoc_name #args };
		}
		visit_mut::visit_type_macro_mut(self, i);
	}

	fn visit_signature_mut(
		&mut self,
		i: &mut Signature,
	) {
		for input in &mut i.inputs {
			if let syn::FnArg::Receiver(r) = input {
				// Build the concrete parameterized type for the receiver
				let concrete_ty = if let Some(base_name) = &self.base_type_name {
					build_parameterized_type(base_name, &self.impl_generic_params)
				} else {
					self.self_ty.clone()
				};

				let attrs = &r.attrs;
				if let Some(reference) = &r.reference {
					let lt = &reference.1;
					if r.mutability.is_some() {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt mut #concrete_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					} else {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt #concrete_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					}
				} else {
					let pat: syn::Pat = parse_quote!(self);
					let ty: syn::Type = parse_quote!(#concrete_ty);
					*input = syn::FnArg::Typed(syn::PatType {
						attrs: attrs.clone(),
						pat: Box::new(pat),
						colon_token: Default::default(),
						ty: Box::new(ty),
					});
				}
			}
		}
		visit_mut::visit_signature_mut(self, i);
	}
}

fn has_attr(
	attrs: &[Attribute],
	name: &str,
) -> bool {
	attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn type_uses_self_assoc(ty: &syn::Type) -> bool {
	struct SelfAssocVisitor {
		found: bool,
	}
	impl syn::visit::Visit<'_> for SelfAssocVisitor {
		fn visit_type_path(
			&mut self,
			i: &syn::TypePath,
		) {
			if let Some(first) = i.path.segments.first()
				&& first.ident == "Self"
				&& i.path.segments.len() > 1
			{
				self.found = true;
			}
			syn::visit::visit_type_path(self, i);
		}
	}
	let mut visitor = SelfAssocVisitor { found: false };
	syn::visit::visit_type(&mut visitor, ty);
	visitor.found
}

pub(crate) fn substitute_generics(
	mut ty: syn::Type,
	generics: &syn::Generics,
	args: &syn::punctuated::Punctuated<syn::GenericArgument, syn::token::Comma>,
) -> syn::Type {
	let mut mapping = HashMap::new();
	let mut const_mapping = HashMap::new();

	for (param, arg) in generics.params.iter().zip(args.iter()) {
		match (param, arg) {
			(syn::GenericParam::Type(tp), syn::GenericArgument::Type(at)) => {
				mapping.insert(tp.ident.to_string(), at.clone());
			}
			(syn::GenericParam::Const(cp), syn::GenericArgument::Const(ca)) => {
				const_mapping.insert(cp.ident.to_string(), ca.clone());
			}
			(syn::GenericParam::Const(cp), syn::GenericArgument::Type(syn::Type::Path(tp)))
				if tp.path.get_ident().is_some() =>
			{
				// Sometimes const generics are passed as types in early parsing phases or macros
				let ident = tp.path.get_ident().unwrap();
				const_mapping.insert(cp.ident.to_string(), syn::parse_quote!(#ident));
			}
			_ => {}
		}
	}

	struct SubstitutionVisitor<'a> {
		mapping: &'a HashMap<String, syn::Type>,
		const_mapping: &'a HashMap<String, syn::Expr>,
	}
	impl VisitMut for SubstitutionVisitor<'_> {
		fn visit_type_mut(
			&mut self,
			i: &mut syn::Type,
		) {
			if let syn::Type::Path(tp) = i
				&& let Some(ident) = tp.path.get_ident()
				&& let Some(target) = self.mapping.get(&ident.to_string())
			{
				*i = target.clone();
				return;
			}
			visit_mut::visit_type_mut(self, i);
		}

		fn visit_expr_mut(
			&mut self,
			i: &mut syn::Expr,
		) {
			if let syn::Expr::Path(ep) = i
				&& let Some(ident) = ep.path.get_ident()
				&& let Some(target) = self.const_mapping.get(&ident.to_string())
			{
				*i = target.clone();
				return;
			}
			visit_mut::visit_expr_mut(self, i);
		}
	}

	let mut visitor = SubstitutionVisitor { mapping: &mapping, const_mapping: &const_mapping };
	visitor.visit_type_mut(&mut ty);
	ty
}

pub(crate) fn normalize_type(
	mut ty: syn::Type,
	generics: &syn::Generics,
) -> syn::Type {
	let mut mapping = HashMap::new();
	for (i, param) in generics.params.iter().enumerate() {
		if let syn::GenericParam::Type(tp) = param {
			let ident = quote::format_ident!("T{}", i);
			mapping.insert(tp.ident.to_string(), parse_quote!(#ident));
		}
	}

	struct NormalizationVisitor<'a> {
		mapping: &'a HashMap<String, syn::Type>,
	}
	impl VisitMut for NormalizationVisitor<'_> {
		fn visit_type_mut(
			&mut self,
			i: &mut syn::Type,
		) {
			if let syn::Type::Path(tp) = i
				&& let Some(ident) = tp.path.get_ident()
				&& let Some(target) = self.mapping.get(&ident.to_string())
			{
				*i = target.clone();
				return;
			}
			visit_mut::visit_type_mut(self, i);
		}
	}

	let mut visitor = NormalizationVisitor { mapping: &mapping };
	visitor.visit_type_mut(&mut ty);
	ty
}

fn get_available_types_for_brand(
	config: &Config,
	self_ty_path: &str,
	trait_path: Option<&str>,
) -> (Vec<String>, Vec<String>) {
	let mut in_this_impl = Vec::new();
	let mut in_other_traits = Vec::new();

	for (brand, trait_opt, assoc_name) in config.projections.keys() {
		if brand == self_ty_path {
			match (&trait_opt, trait_path) {
				(Some(t), Some(current)) if t == current => {
					in_this_impl.push(assoc_name.clone());
				}
				(Some(_), _) | (None, _) => {
					in_other_traits.push(assoc_name.clone());
				}
			}
		}
	}

	in_this_impl.sort();
	in_this_impl.dedup();
	in_other_traits.sort();
	in_other_traits.dedup();

	(in_this_impl, in_other_traits)
}

fn create_missing_default_error(
	span: proc_macro2::Span,
	self_ty_path: &str,
	trait_path: Option<&str>,
	config: &Config,
) -> Error {
	let (in_this_impl, in_other_traits) =
		get_available_types_for_brand(config, self_ty_path, trait_path);

	let mut message =
		format!("Cannot resolve bare `Self` for type `{}` - no default specified", self_ty_path);

	if !in_this_impl.is_empty() {
		message
			.push_str(&format!("\n  = note: Available in this impl: {}", in_this_impl.join(", ")));
	}

	if !in_other_traits.is_empty() {
		message.push_str(&format!(
			"\n  = note: Available in other traits: {}",
			in_other_traits.join(", ")
		));
	}

	message.push_str(
		"\n  = help: Mark one as default with #[doc_default], or use explicit #[doc_use = \"AssocName\"]",
	);

	Error::new(span, message)
}

fn create_missing_assoc_type_error(
	span: proc_macro2::Span,
	self_ty_path: &str,
	assoc_name: &str,
	trait_path: Option<&str>,
	config: &Config,
) -> Error {
	let (in_this_impl, in_other_traits) =
		get_available_types_for_brand(config, self_ty_path, trait_path);

	let mut message = format!("Cannot resolve `Self::{}` for type `{}`", assoc_name, self_ty_path);

	let all_available: Vec<String> =
		in_this_impl.iter().chain(in_other_traits.iter()).cloned().collect();

	if !all_available.is_empty() {
		message.push_str(&format!(
			"\n  = note: Available associated types: {}",
			all_available.join(", ")
		));
	} else {
		message.push_str("\n  = note: No associated types found for this type");
	}

	message.push_str(&format!(
		"\n  = help: Add an associated type definition:\n    impl_kind! {{\n        for {} {{\n            type {}<T> = YourType<T>;\n        }}\n    }}",
		self_ty_path, assoc_name
	));

	Error::new(span, message)
}
