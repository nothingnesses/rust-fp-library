use {
	super::generation::generate_documentation,
	crate::{
		analysis::get_all_parameters,
		core::{
			Result as OurResult,
			config::Config,
			constants::attributes::{
				DOCUMENT_ATTR_ORDER,
				DOCUMENT_EXAMPLES,
				DOCUMENT_MODULE,
				DOCUMENT_PARAMETERS,
				DOCUMENT_RETURNS,
				DOCUMENT_SIGNATURE,
				DOCUMENT_TYPE_PARAMETERS,
			},
			error_handling::ErrorCollector,
		},
		resolution::get_context,
		support::{
			attributes::has_attribute,
			method_utils::{
				has_non_receiver_parameters,
				impl_has_receiver_methods,
			},
			parsing::{
				parse_many,
				parse_non_empty,
				parse_with_dispatch,
			},
		},
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::{
		Item,
		ItemMod,
		parse::{
			Parse,
			ParseStream,
		},
		spanned::Spanned,
		visit_mut::{
			self,
			VisitMut,
		},
	},
};

pub struct DocumentModuleInput {
	pub items: Vec<Item>,
}

impl Parse for DocumentModuleInput {
	fn parse(input: ParseStream) -> syn::Result<Self> {
		let items = parse_many(input)?;
		let items = parse_non_empty(items, "Module documentation must contain at least one item")?;
		Ok(DocumentModuleInput {
			items,
		})
	}
}

/// Configuration for document_module validation
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ValidationMode {
	/// Validation enabled - emit warnings for missing documentation (default)
	#[default]
	On,
	/// Validation disabled - no warnings
	Off,
}

/// Parse validation mode from attribute arguments
fn parse_validation_mode(attr: TokenStream) -> syn::Result<ValidationMode> {
	if attr.is_empty() {
		return Ok(ValidationMode::default());
	}

	let attr_str = attr.to_string();
	match attr_str.trim() {
		"no_validation" => Ok(ValidationMode::Off),
		_ => Err(syn::Error::new(
			attr.span(),
			format!("Unknown validation mode '{attr_str}'. Valid option: 'no_validation'"),
		)),
	}
}

/// Represents the parsed input format for document_module macro.
enum ParsedInput {
	/// Outer attribute on a module: #[document_module] mod foo { ... }
	ModuleWrapper(ItemMod, syn::token::Brace, Vec<Item>),
	/// Direct items (used for const block pattern): const _: () = { ... items ... };
	DirectItems(Vec<Item>),
}

/// Try to parse input as a module with outer attribute.
fn try_parse_module_wrapper(item: TokenStream) -> Option<ParsedInput> {
	if let Ok(mut item_mod) = syn::parse2::<ItemMod>(item) {
		if let Some((brace, mod_items)) = item_mod.content.take() {
			return Some(ParsedInput::ModuleWrapper(item_mod, brace, mod_items));
		} else {
			// mod foo; case - we can't see the content easily
			// Return None to fall through to error handling
			return None;
		}
	}
	None
}

/// Try to parse input as direct items (const block pattern).
fn try_parse_direct_items(item: TokenStream) -> Option<ParsedInput> {
	if let Ok(input) = syn::parse2::<DocumentModuleInput>(item) {
		return Some(ParsedInput::DirectItems(input.items));
	}
	None
}

/// Try to parse input as a const block: const _: () = { ... };
fn try_parse_const_block(item: TokenStream) -> Result<ParsedInput, syn::Error> {
	let item_const = syn::parse2::<syn::ItemConst>(item)?;

	if let syn::Expr::Block(expr_block) = *item_const.expr {
		let items: Vec<Item> = expr_block
			.block
			.stmts
			.into_iter()
			.filter_map(|stmt| match stmt {
				syn::Stmt::Item(item) => Some(item),
				_ => None,
			})
			.collect();
		Ok(ParsedInput::DirectItems(items))
	} else {
		Err(syn::Error::new(
			item_const.span(),
			format!(
				"{DOCUMENT_MODULE} on a const item requires a block expression: const _: () = {{ ... }};"
			),
		))
	}
}

/// Parse the input token stream into one of the supported formats.
fn parse_document_module_input(item: TokenStream) -> Result<ParsedInput, syn::Error> {
	// Try to parse as ItemMod first (more specific), then fall back to DocumentModuleInput
	// This is critical: ItemMod must be checked first, otherwise `#[document_module] mod inner { ... }`
	// would be parsed as DocumentModuleInput containing a single module item, losing the wrapper.
	parse_with_dispatch(
		item,
		vec![
			Box::new(|tokens| {
				try_parse_module_wrapper(tokens).ok_or_else(|| {
					syn::Error::new(proc_macro2::Span::call_site(), "Not a module wrapper")
				})
			}),
			Box::new(|tokens| {
				try_parse_direct_items(tokens).ok_or_else(|| {
					syn::Error::new(proc_macro2::Span::call_site(), "Not direct items")
				})
			}),
			Box::new(try_parse_const_block),
		],
		&format!(
			"{DOCUMENT_MODULE} must be applied to a module or a const block (e.g., const _: () = {{ ... }})."
		),
	)
}

pub fn document_module_worker(
	attr: TokenStream,
	item: TokenStream,
) -> OurResult<TokenStream> {
	let parsed_input = parse_document_module_input(item)?;

	let (module_wrapper, mut items) = match parsed_input {
		ParsedInput::ModuleWrapper(module, brace, items) => (Some((module, brace)), items),
		ParsedInput::DirectItems(items) => (None, items),
	};

	// Parse validation mode from attribute
	let validation_mode = parse_validation_mode(attr)?;

	let mut config = Config::default();

	// Pass 1: Context Extraction (handles both top-level and nested)
	get_context(&items, &mut config)?;

	// Also recursively extract from nested modules
	apply_to_nested_modules(&mut items, get_context, &mut config)?;

	// Pass 1.5: Validation (emit warnings for missing documentation attributes)
	let warning_tokens: Vec<TokenStream> = if validation_mode != ValidationMode::Off {
		let warnings = validate_documentation(&items);

		// Also recursively validate nested modules
		let nested_warnings = validate_nested_modules(&items);

		// Convert errors to compile-time errors
		warnings.into_iter().chain(nested_warnings).map(|e| e.to_compile_error()).collect()
	} else {
		Vec::new()
	};

	// Pass 2: Documentation Generation (handles both top-level and nested)
	generate_documentation(&mut items, &config)?;

	// Also recursively generate docs for nested modules (immutable config)
	apply_to_nested_modules_immut(&mut items, generate_documentation, &config)?;

	// Reconstruct module wrapper if needed (outer attribute case)
	let items_output = if let Some((mut module, brace)) = module_wrapper {
		module.content = Some((brace, items));
		quote!(#module)
	} else {
		quote!(#(#items)*)
	};

	// Combine warnings with output if validation is enabled
	// Warnings are emitted first so they appear in the compiler output
	let output = if !warning_tokens.is_empty() {
		quote! {
			#(#warning_tokens)*
			#items_output
		}
	} else {
		items_output
	};

	Ok(output)
}

/// Apply an operation to all nested modules recursively with mutable config.
fn apply_to_nested_modules<F>(
	items: &mut [Item],
	operation: F,
	config: &mut Config,
) -> syn::Result<()>
where
	F: Fn(&[Item], &mut Config) -> syn::Result<()> + Copy, {
	let mut errors = ErrorCollector::new();
	let mut visitor = ModuleVisitor {
		operation,
		config,
		errors: &mut errors,
	};

	for item in items {
		visitor.visit_item_mut(item);
	}

	errors.finish()
}

/// Apply an operation to all nested modules recursively with immutable config.
fn apply_to_nested_modules_immut<F>(
	items: &mut [Item],
	operation: F,
	config: &Config,
) -> syn::Result<()>
where
	F: Fn(&mut [Item], &Config) -> syn::Result<()> + Copy, {
	let mut errors = ErrorCollector::new();
	let mut visitor = ModuleVisitorImmut {
		operation,
		config,
		errors: &mut errors,
	};

	for item in items {
		visitor.visit_item_mut(item);
	}

	errors.finish()
}

/// Generic visitor for applying operations to nested modules (mutable config).
struct ModuleVisitor<'a, F>
where
	F: Fn(&[Item], &mut Config) -> syn::Result<()>, {
	operation: F,
	config: &'a mut Config,
	errors: &'a mut ErrorCollector,
}

impl<'a, F> VisitMut for ModuleVisitor<'a, F>
where
	F: Fn(&[Item], &mut Config) -> syn::Result<()>,
{
	fn visit_item_mod_mut(
		&mut self,
		module: &mut ItemMod,
	) {
		if let Some((_, ref items)) = module.content {
			if let Err(e) = (self.operation)(items, self.config) {
				self.errors.push(e);
			}
			// Recursively process nested modules
			visit_mut::visit_item_mod_mut(self, module);
		}
	}
}

/// Generic visitor for applying operations to nested modules (immutable config).
struct ModuleVisitorImmut<'a, F>
where
	F: Fn(&mut [Item], &Config) -> syn::Result<()>, {
	operation: F,
	config: &'a Config,
	errors: &'a mut ErrorCollector,
}

impl<'a, F> VisitMut for ModuleVisitorImmut<'a, F>
where
	F: Fn(&mut [Item], &Config) -> syn::Result<()>,
{
	fn visit_item_mod_mut(
		&mut self,
		module: &mut ItemMod,
	) {
		if let Some((_, ref mut items)) = module.content {
			if let Err(e) = (self.operation)(items, self.config) {
				self.errors.push(e);
			}
			// Recursively process nested modules
			visit_mut::visit_item_mod_mut(self, module);
		}
	}
}

/// Check that none of the ordered documentation attributes appear more than once.
fn validate_no_duplicate_doc_attrs(
	attrs: &[syn::Attribute],
	item_span: proc_macro2::Span,
	item_label: &str,
	warnings: &mut ErrorCollector,
) {
	for name in DOCUMENT_ATTR_ORDER {
		let count = attrs.iter().filter(|a| a.path().is_ident(name)).count();
		if count > 1 {
			warnings.push(syn::Error::new(
				item_span,
				format!(
					"{item_label} has `#[{name}]` applied {count} times; it may only appear once",
				),
			));
		}
	}
}

/// Check that the ordered documentation attributes appear in the canonical order:
/// document_signature → document_type_parameters → document_parameters →
/// document_returns → document_examples.
fn validate_doc_attr_order(
	attrs: &[syn::Attribute],
	item_span: proc_macro2::Span,
	item_label: &str,
	warnings: &mut ErrorCollector,
) {
	// Collect the index of the first occurrence of each ordered attribute.
	let positions: Vec<Option<usize>> = DOCUMENT_ATTR_ORDER
		.iter()
		.map(|name| attrs.iter().position(|a| a.path().is_ident(name)))
		.collect();

	// For every pair (i, j) where i comes before j in the canonical order,
	// their attribute positions must satisfy pos[i] < pos[j].
	for i in 0 .. DOCUMENT_ATTR_ORDER.len() {
		for j in (i + 1) .. DOCUMENT_ATTR_ORDER.len() {
			if let (Some(pos_i), Some(pos_j)) = (positions[i], positions[j]) {
				if pos_i > pos_j {
					warnings.push(syn::Error::new(
						item_span,
						format!(
							"{item_label} has `#[{}]` before `#[{}]`, but the required order is: {}",
							DOCUMENT_ATTR_ORDER[j],
							DOCUMENT_ATTR_ORDER[i],
							DOCUMENT_ATTR_ORDER
								.iter()
								.filter(|name| positions
									[DOCUMENT_ATTR_ORDER.iter().position(|n| n == *name).unwrap()]
								.is_some())
								.copied()
								.map(|n| format!("`#[{n}]`"))
								.collect::<Vec<_>>()
								.join(" → "),
						),
					));
					// Report at most one ordering violation per item to avoid noise.
					return;
				}
			}
		}
	}
}

/// Validate that a method has appropriate documentation attributes.
fn validate_method_documentation(
	method: &syn::ImplItemFn,
	warnings: &mut ErrorCollector,
) {
	let method_name = &method.sig.ident;
	let method_generics = &method.sig.generics;
	let label = format!("Method `{method_name}`");

	// Check for duplicate and out-of-order documentation attributes
	validate_no_duplicate_doc_attrs(&method.attrs, method.span(), &label, warnings);
	validate_doc_attr_order(&method.attrs, method.span(), &label, warnings);

	// Check for document_signature
	if !has_attribute(&method.attrs, DOCUMENT_SIGNATURE) {
		let warning = syn::Error::new(
			method.span(),
			format!("Method `{method_name}` should have #[{DOCUMENT_SIGNATURE}] attribute"),
		);
		warnings.push(warning);
	}

	// Check for document_type_parameters if method has type parameters
	let has_type_params = !method_generics.params.is_empty();
	let has_doc_type_params = has_attribute(&method.attrs, DOCUMENT_TYPE_PARAMETERS);

	if has_type_params && !has_doc_type_params {
		let type_param_names: Vec<String> = get_all_parameters(method_generics);
		let warning = syn::Error::new(
			method.span(),
			format!(
				"Method `{method_name}` has type parameters <{}> but no #[{DOCUMENT_TYPE_PARAMETERS}] attribute",
				type_param_names.join(", "),
			),
		);
		warnings.push(warning);
	}

	// Check for document_parameters if method has non-receiver parameters
	if has_non_receiver_parameters(method) && !has_attribute(&method.attrs, DOCUMENT_PARAMETERS) {
		let warning = syn::Error::new(
			method.span(),
			format!(
				"Method `{method_name}` has parameters but no #[{DOCUMENT_PARAMETERS}] attribute",
			),
		);
		warnings.push(warning);
	}

	// Check for document_returns if method has a return type
	if let syn::ReturnType::Type(..) = method.sig.output {
		if !has_attribute(&method.attrs, DOCUMENT_RETURNS) {
			let warning = syn::Error::new(
				method.span(),
				format!(
					"Method `{method_name}` has a return type but no #[{DOCUMENT_RETURNS}] attribute",
				),
			);
			warnings.push(warning);
		}
	}

	// Check for document_examples (required on all methods)
	if !has_attribute(&method.attrs, DOCUMENT_EXAMPLES) {
		warnings.push(syn::Error::new(
			method.span(),
			format!(
				"Method `{method_name}` should have a #[{DOCUMENT_EXAMPLES}] attribute; the attribute should contain a string showing example code usage of the function annotated by the attribute",
			),
		));
	}
}

/// Validate that an impl block has appropriate documentation attributes.
fn validate_impl_documentation(
	item_impl: &syn::ItemImpl,
	warnings: &mut ErrorCollector,
) {
	// Check for duplicate and out-of-order documentation attributes on the impl block itself
	validate_no_duplicate_doc_attrs(&item_impl.attrs, item_impl.span(), "Impl block", warnings);
	validate_doc_attr_order(&item_impl.attrs, item_impl.span(), "Impl block", warnings);

	let impl_generics = &item_impl.generics;
	let has_type_params = !impl_generics.params.is_empty();
	let has_doc_type_params = has_attribute(&item_impl.attrs, DOCUMENT_TYPE_PARAMETERS);

	// Check if any methods have receivers
	let has_methods_with_receivers = impl_has_receiver_methods(item_impl);

	// Warn if impl has type parameters but no document_type_parameters
	if has_type_params && !has_doc_type_params {
		let type_param_names: Vec<String> = get_all_parameters(impl_generics);
		let warning = syn::Error::new(
			item_impl.span(),
			format!(
				"Impl block has type parameters <{}> but no #[{DOCUMENT_TYPE_PARAMETERS}] attribute",
				type_param_names.join(", "),
			),
		);
		warnings.push(warning);
	}

	// Warn if impl has methods with receivers but no document_parameters at impl level
	// Note: This checks for impl-level document_parameters, which documents the receiver type
	if has_methods_with_receivers && !has_attribute(&item_impl.attrs, DOCUMENT_PARAMETERS) {
		let warning = syn::Error::new(
			item_impl.span(),
			format!(
				"Impl block contains methods with receiver parameters but no #[{DOCUMENT_PARAMETERS}] attribute",
			),
		);
		warnings.push(warning);
	}

	// Validate each method in the impl block
	for impl_item in &item_impl.items {
		if let syn::ImplItem::Fn(method) = impl_item {
			validate_method_documentation(method, warnings);
		}
	}
}

/// Validate that a free function has a `document_examples` attribute.
fn validate_fn_documentation(
	item_fn: &syn::ItemFn,
	warnings: &mut ErrorCollector,
) {
	let fn_name = &item_fn.sig.ident;
	if !has_attribute(&item_fn.attrs, DOCUMENT_EXAMPLES) {
		warnings.push(syn::Error::new(
			item_fn.span(),
			format!(
				"Function `{fn_name}` should have a #[{DOCUMENT_EXAMPLES}] attribute; the attribute should contain a string showing example code usage of the function annotated by the attribute",
			),
		));
	}
}

/// Validate documentation attributes on all items.
///
/// This function checks that impl blocks, their methods, and free functions have
/// appropriate documentation attributes based on their characteristics (type
/// parameters, parameters, etc.).
///
/// Returns a list of warnings (as syn::Error objects) that can be emitted
/// or collected for reporting.
fn validate_documentation(items: &[Item]) -> Vec<syn::Error> {
	let mut warnings = ErrorCollector::new();

	for item in items {
		match item {
			Item::Impl(item_impl) => validate_impl_documentation(item_impl, &mut warnings),
			Item::Fn(item_fn) => validate_fn_documentation(item_fn, &mut warnings),
			_ => {}
		}
	}

	warnings.into_errors()
}

/// Recursively validate all nested modules and collect warnings.
fn validate_nested_modules(items: &[Item]) -> Vec<syn::Error> {
	let mut warnings = Vec::new();

	for item in items {
		if let Item::Mod(module) = item
			&& let Some((_, ref nested_items)) = module.content
		{
			// Validate this module's items
			warnings.extend(validate_documentation(nested_items));

			// Recursively validate nested modules
			warnings.extend(validate_nested_modules(nested_items));
		}
	}

	warnings
}
