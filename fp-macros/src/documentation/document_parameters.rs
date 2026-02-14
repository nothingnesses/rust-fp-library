use {
	crate::{
		core::{
			Error as CoreError, Result, config::get_config,
			constants::attributes::DOCUMENT_PARAMETERS, error_handling::ErrorCollector,
		},
		support::{
			Parameter,
			attributes::{find_attribute, remove_attribute_tokens},
			documentation_parameters::{DocumentationParameter, DocumentationParameters},
			generate_documentation::{generate_doc_comments, insert_doc_comments_batch},
			get_parameters, has_receiver, impl_has_receiver_methods, parsing,
		},
	},
	proc_macro2::TokenStream,
	quote::{ToTokens, quote},
	syn::{ImplItem, ImplItemFn, LitStr, parse::Parse, spanned::Spanned},
};

/// Parse single string literal for receiver documentation
struct ReceiverDoc {
	description: LitStr,
}

impl Parse for ReceiverDoc {
	fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
		let description = input.parse()?;
		Ok(ReceiverDoc { description })
	}
}

/// Process a method with #[document_parameters] attribute, applying receiver doc if needed
fn process_method_in_impl(
	method: &mut ImplItemFn,
	receiver_doc: &str,
	config: &crate::core::config::Config,
) -> Result<()> {
	// Find the #[document_parameters] attribute
	let Some(attr_pos) = find_attribute(&method.attrs, DOCUMENT_PARAMETERS) else {
		// No attribute on this method, skip it
		return Ok(());
	};

	// Remove the attribute and get its tokens
	let attr_tokens = remove_attribute_tokens(&mut method.attrs, attr_pos)?;

	// Get logical params (excluding receiver)
	let logical_params = get_parameters(&method.sig, config);

	// Check if method has receiver
	let has_receiver_param = has_receiver(method);

	// Error if no parameters at all
	if logical_params.is_empty() && !has_receiver_param {
		let _ = parsing::parse_has_documentable_items(
			0, // Explicit 0 to trigger error
			method.sig.ident.span(),
			DOCUMENT_PARAMETERS,
			&format!("method '{}' with no parameters", method.sig.ident),
		)?;
	}

	// Parse the arguments from the attribute (may be empty for receiver-only docs)
	let parse_result = syn::parse2::<DocumentationParameters>(attr_tokens.clone());

	// Get entries, which may be empty if only documenting receiver
	let entries: Vec<_> = if let Ok(args) = parse_result {
		args.entries.into_iter().collect()
	} else {
		// If parse fails and we have no logical params, it's likely just empty parens or no args
		// which is fine if we only have a receiver
		if logical_params.is_empty() && has_receiver_param {
			Vec::new()
		} else {
			return Err(CoreError::Parse(syn::Error::new(
				attr_tokens.span(),
				format!("Failed to parse {DOCUMENT_PARAMETERS} arguments"),
			)));
		}
	};

	// Validate entry count matches logical params (not including receiver)
	let (_expected, _provided) = parsing::parse_entry_count(
		logical_params.len(),
		entries.len(),
		attr_tokens.span(),
		"parameter",
	)?;

	// Generate parameter names for all params including receiver
	let mut param_names = Vec::new();
	let mut param_descs = Vec::new();

	// Add receiver if present
	if has_receiver_param {
		// Get receiver name from signature
		let receiver_name = if let Some(syn::FnArg::Receiver(recv)) = method.sig.inputs.first() {
			if recv.mutability.is_some() {
				"&mut self"
			} else if recv.reference.is_some() {
				"&self"
			} else {
				"self"
			}
		} else {
			"self"
		};
		param_names.push(receiver_name.to_string());
		param_descs.push(receiver_doc.to_string());
	}

	// Add other parameters
	for (param, entry) in logical_params.iter().zip(entries) {
		let (name, desc) = match (param, entry) {
			(Parameter::Explicit(_pat), DocumentationParameter::Override(n, d)) => {
				(n.value(), d.value())
			}
			(Parameter::Explicit(pat), DocumentationParameter::Description(d)) => {
				let name = pat.to_token_stream().to_string().replace(" , ", ", ");
				(name, d.value())
			}
			(Parameter::Implicit(_), DocumentationParameter::Override(n, d)) => {
				(n.value(), d.value())
			}
			(Parameter::Implicit(_), DocumentationParameter::Description(d)) => {
				("_".to_string(), d.value())
			}
		};
		param_names.push(name);
		param_descs.push(desc);
	}

	// Generate doc comments and insert them
	let mut docs: Vec<_> = param_names.into_iter().zip(param_descs).collect();

	// Add section header
	docs.insert(
		0,
		(
			String::new(),
			r#"### Parameters
"#
			.to_string(),
		),
	);

	insert_doc_comments_batch(&mut method.attrs, docs, attr_pos);

	Ok(())
}

/// Process impl block with #[document_parameters("receiver doc")]
fn process_impl_block(
	attr: TokenStream,
	mut item_impl: syn::ItemImpl,
) -> Result<TokenStream> {
	// Parse the receiver documentation
	let receiver_doc = syn::parse2::<ReceiverDoc>(attr.clone()).map_err(|e| {
		syn::Error::new(
			e.span(),
			format!(
				"{DOCUMENT_PARAMETERS} on impl blocks must have exactly one string literal for receiver documentation"
			),
		)
	})?;

	// Verify that the impl block has at least one method with a receiver
	if !impl_has_receiver_methods(&item_impl) {
		return Err(CoreError::Parse(syn::Error::new(
			attr.span(),
			format!(
				"{DOCUMENT_PARAMETERS} cannot be used on impl blocks with no methods that have receiver parameters"
			),
		)));
	}

	let receiver_desc = receiver_doc.description.value();
	let config = get_config();

	// Collect errors from all methods instead of returning early
	let mut errors = ErrorCollector::new();

	// Process each method that has #[document_parameters]
	for item in &mut item_impl.items {
		if let ImplItem::Fn(method) = item
			&& let Err(e) = process_method_in_impl(method, &receiver_desc, &config)
		{
			errors.push(e.into());
		}
	}

	// Finish and convert any collected errors
	errors.finish()?;

	Ok(quote!(#item_impl))
}

pub fn document_parameters_worker(
	attr: TokenStream,
	item_tokens: TokenStream,
) -> Result<TokenStream> {
	// Try parsing as impl block first
	if let Ok(item_impl) = syn::parse2::<syn::ItemImpl>(item_tokens.clone()) {
		return process_impl_block(attr, item_impl);
	}

	// Otherwise, process as a function with generate_doc_comments
	generate_doc_comments(attr, item_tokens, "Parameters", |generic_item| {
		let config = get_config();

		let sig = generic_item.signature().ok_or_else(|| {
			syn::Error::new(
				proc_macro2::Span::call_site(),
				format!("{DOCUMENT_PARAMETERS} can only be used on functions or impl blocks"),
			)
		})?;

		let logical_params = get_parameters(sig, &config);

		Ok(logical_params
			.into_iter()
			.map(|param| match param {
				Parameter::Explicit(pat) => {
					let s = pat.to_token_stream().to_string();
					s.replace(" , ", ", ")
				}
				Parameter::Implicit(_) => "_".to_string(),
			})
			.collect())
	})
}

#[cfg(test)]
mod tests {
	use {super::*, crate::support::generate_documentation::get_doc, quote::quote, syn::ItemFn};

	#[test]
	fn test_doc_params_basic() {
		let attr = quote! { "Arg 1", "Arg 2" };
		let item = quote! {
			fn foo(a: i32, b: String) {}
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		// 2 parameters + 1 header = 3 attributes
		assert_eq!(output_fn.attrs.len(), 3);
		assert_eq!(get_doc(&output_fn.attrs[0]), "### Parameters\n");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `a`: Arg 1");
		assert_eq!(get_doc(&output_fn.attrs[2]), "* `b`: Arg 2");
	}

	#[test]
	fn test_doc_params_trait() {
		let attr = quote! { "Arg 1" };
		let item = quote! {
			fn foo(a: i32);
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_fn: syn::TraitItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 2);
		assert_eq!(get_doc(&output_fn.attrs[0]), "### Parameters\n");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `a`: Arg 1");
	}

	#[test]
	fn test_doc_params_with_overrides() {
		let attr = quote! { ("custom_a", "Arg 1"), "Arg 2" };
		let item = quote! {
			fn foo(a: i32, b: String) {}
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 3);
		assert_eq!(get_doc(&output_fn.attrs[0]), "### Parameters\n");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `custom_a`: Arg 1");
		assert_eq!(get_doc(&output_fn.attrs[2]), "* `b`: Arg 2");
	}

	#[test]
	fn test_doc_params_curried() {
		let attr = quote! { "Arg 1", "Curried Arg" };
		let item = quote! {
			fn foo(a: i32) -> impl Fn(i32) -> i32 { todo!() }
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 3);
		assert_eq!(get_doc(&output_fn.attrs[0]), "### Parameters\n");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `a`: Arg 1");
		assert_eq!(get_doc(&output_fn.attrs[2]), "* `_`: Curried Arg");
	}

	#[test]
	fn test_doc_params_mismatch() {
		let attr = quote! { "Too few" };
		let item = quote! {
			fn foo(a: i32, b: i32) {}
		};

		let output = document_parameters_worker(attr, item).unwrap_err();
		let error = output.to_string();
		assert!(
			error.contains("Expected 2 description arguments (one for each parameter), found 1")
		);
	}

	#[test]
	fn test_doc_params_skips_self() {
		let attr = quote! { "Arg 1" };
		let item = quote! {
			fn foo(&self, a: i32) {}
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_fn: ItemFn = syn::parse2(output).unwrap();

		assert_eq!(output_fn.attrs.len(), 2);
		assert_eq!(get_doc(&output_fn.attrs[0]), "### Parameters\n");
		assert_eq!(get_doc(&output_fn.attrs[1]), "* `a`: Arg 1");
	}

	#[test]
	fn test_doc_params_impl_block_with_receiver_only() {
		// Method with only receiver, no other params
		let attr = quote! { "The receiver parameter" };
		let item = quote! {
			impl<A> MyType<A> {
				#[document_parameters]
				fn foo(&self) -> usize { 0 }
			}
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_impl: syn::ItemImpl = syn::parse2(output).unwrap();

		if let ImplItem::Fn(method) = &output_impl.items[0] {
			assert_eq!(method.attrs.len(), 2);
			assert_eq!(get_doc(&method.attrs[0]), "### Parameters\n");
			assert_eq!(get_doc(&method.attrs[1]), "* `&self`: The receiver parameter");
		} else {
			panic!("Expected method");
		}
	}

	#[test]
	fn test_doc_params_impl_block_with_receiver_and_params() {
		// Method with receiver and additional params
		let attr = quote! { "The list instance" };
		let item = quote! {
			impl<A> MyList<A> {
				#[document_parameters("The element to append")]
				fn push(&mut self, item: A) {}
			}
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_impl: syn::ItemImpl = syn::parse2(output).unwrap();

		if let ImplItem::Fn(method) = &output_impl.items[0] {
			assert_eq!(method.attrs.len(), 3);
			assert_eq!(get_doc(&method.attrs[0]), "### Parameters\n");
			assert_eq!(get_doc(&method.attrs[1]), "* `&mut self`: The list instance");
			assert_eq!(get_doc(&method.attrs[2]), "* `item`: The element to append");
		} else {
			panic!("Expected method");
		}
	}

	#[test]
	fn test_doc_params_impl_block_no_methods() {
		// Should error when impl block has no methods with receivers
		let attr = quote! { "The receiver" };
		let item = quote! {
			impl<A> MyType<A> {
				const VALUE: i32 = 42;
			}
		};

		let result = document_parameters_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("no methods that have receiver parameters"));
	}

	#[test]
	fn test_doc_params_impl_block_no_receiver_methods() {
		// Should error when impl block only has static methods
		let attr = quote! { "The receiver" };
		let item = quote! {
			impl<A> MyType<A> {
				fn new() -> Self { todo!() }
			}
		};

		let result = document_parameters_worker(attr, item);
		assert!(result.is_err());
		let error = result.unwrap_err().to_string();
		assert!(error.contains("no methods that have receiver parameters"));
	}

	#[test]
	fn test_doc_params_impl_block_static_method_ignored() {
		// Static methods without #[document_parameters] should be ignored
		let attr = quote! { "The receiver" };
		let item = quote! {
			impl<A> MyType<A> {
				#[document_parameters]
				fn foo(&self) -> usize { 0 }

				// This static method doesn't have #[document_parameters], so it's ignored
				fn new() -> Self { todo!() }
			}
		};

		let output = document_parameters_worker(attr, item).unwrap();
		let output_impl: syn::ItemImpl = syn::parse2(output).unwrap();

		// First method should have doc
		if let ImplItem::Fn(method) = &output_impl.items[0] {
			assert_eq!(method.attrs.len(), 2);
			assert_eq!(get_doc(&method.attrs[0]), "### Parameters\n");
			assert_eq!(get_doc(&method.attrs[1]), "* `&self`: The receiver");
		} else {
			panic!("Expected method");
		}

		// Second method should have no doc attributes
		if let ImplItem::Fn(method) = &output_impl.items[1] {
			assert_eq!(method.attrs.len(), 0);
		} else {
			panic!("Expected method");
		}
	}

	#[test]
	fn test_doc_params_standalone_function_no_params() {
		let attr = quote! {};
		let item = quote! {
			fn foo() {}
		};

		// Should error - function has no parameters
		let result = document_parameters_worker(attr, item);
		assert!(result.is_err());
	}
}
