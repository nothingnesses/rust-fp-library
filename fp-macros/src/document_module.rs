use crate::{
	apply::ApplyInput,
	doc_utils::{DocArg, GenericArgs, validate_doc_args},
	function_utils::Config,
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

	quote!(#(#items)*)
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
						if let Some((prev_generics, prev_target)) =
							config.projections.get(&(brand_path.clone(), None, assoc_name.clone()))
						{
							let normalized_prev =
								normalize_type(prev_target.clone(), prev_generics);
							let normalized_curr =
								normalize_type(def.target_type.clone(), &def.generics);

							if normalized_prev.to_token_stream().to_string()
								!= normalized_curr.to_token_stream().to_string()
							{
								errors.push(Error::new(
									def.ident.span(),
									format!(
										"Conflicting definitions for same associated type: {}",
										assoc_name
									),
								));
							}
						}

						config.projections.insert(
							(brand_path.clone(), None, assoc_name.clone()),
							(def.generics.clone(), def.target_type.clone()),
						);

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

			let impl_doc_use = match find_attr_value_checked(&item_impl.attrs, "doc_use") {
				Ok(v) => v,
				Err(e) => {
					errors.push(e);
					None
				}
			};

			for impl_item in &mut item_impl.items {
				if let ImplItem::Fn(method) = impl_item {
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

						// Resolve Self
						let mut substitutor = SelfSubstitutor {
							self_ty,
							self_ty_path: &self_ty_path,
							trait_path: trait_path_str.as_deref(),
							doc_use: doc_use.as_deref(),
							config,
							errors: Vec::new(),
						};
						substitutor.visit_signature_mut(&mut synthetic_sig);

						// Collect any resolution errors
						errors.extend(substitutor.errors);

						// Merge generics
						merge_generics(&mut synthetic_sig, &item_impl.generics);

						// Add trait bound: SelfTy: Trait (only if it's a trait impl)
						if let Some(trait_path) = trait_path {
							let where_clause = synthetic_sig.generics.make_where_clause();
							where_clause.predicates.push(parse_quote!(#self_ty: #trait_path));
						}

						let signature_data =
							generate_signature(&synthetic_sig, trait_name.as_deref(), config);
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
						*i = target.clone();
					} else {
						// Fallback to self_ty if projection not found
						*i = self.self_ty.clone();
					}
				} else {
					// No default found - report error with available types
					self.errors.push(create_missing_default_error(
						tp.span(),
						self.self_ty_path,
						self.trait_path,
						self.config,
					));
					// Fallback to self_ty
					*i = self.self_ty.clone();
				}
			} else if let Some(first) = tp.path.segments.first()
				&& first.ident == "Self"
				&& tp.path.segments.len() > 1
			{
				let assoc_name = tp.path.segments[1].ident.to_string();
				if let Some((_generics, target)) = self
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
					*i = target.clone();
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
				let self_ty = self.self_ty;
				let attrs = &r.attrs;
				if let Some(reference) = &r.reference {
					let lt = &reference.1;
					if r.mutability.is_some() {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt mut #self_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					} else {
						let pat: syn::Pat = parse_quote!(self);
						let ty: syn::Type = parse_quote!(&#lt #self_ty);
						*input = syn::FnArg::Typed(syn::PatType {
							attrs: attrs.clone(),
							pat: Box::new(pat),
							colon_token: Default::default(),
							ty: Box::new(ty),
						});
					}
				} else {
					let pat: syn::Pat = parse_quote!(self);
					let ty: syn::Type = parse_quote!(#self_ty);
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
	for (param, arg) in generics.params.iter().zip(args.iter()) {
		if let (syn::GenericParam::Type(tp), syn::GenericArgument::Type(at)) = (param, arg) {
			mapping.insert(tp.ident.to_string(), at.clone());
		}
	}

	struct SubstitutionVisitor<'a> {
		mapping: &'a HashMap<String, syn::Type>,
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
	}

	let mut visitor = SubstitutionVisitor { mapping: &mapping };
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
