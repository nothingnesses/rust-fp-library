use {
	super::document_module_worker,
	proc_macro2::TokenStream,
	quote::quote,
	std::error::Error,
	syn::{
		ImplItem,
		Item,
	},
};

type TestResult = Result<(), Box<dyn Error>>;

fn run_document_module(tokens: TokenStream) -> Result<syn::File, Box<dyn Error>> {
	let output = document_module_worker(TokenStream::new(), tokens)?;
	let file = syn::parse2(output)?;
	Ok(file)
}

fn impl_method_names(file: &syn::File) -> Vec<String> {
	file.items
		.iter()
		.filter_map(|item| match item {
			Item::Impl(item_impl) => Some(item_impl),
			_ => None,
		})
		.flat_map(|item_impl| item_impl.items.iter())
		.filter_map(|item| match item {
			ImplItem::Fn(method) => Some(method.sig.ident.to_string()),
			_ => None,
		})
		.collect()
}

fn enum_names(file: &syn::File) -> Vec<String> {
	file.items
		.iter()
		.filter_map(|item| match item {
			Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
			_ => None,
		})
		.collect()
}

fn contains_macro_invocation(
	items: &[Item],
	name: &str,
) -> bool {
	items.iter().any(|item| match item {
		Item::Macro(item_macro) => item_macro.mac.path.is_ident(name),
		Item::Mod(module) => module
			.content
			.as_ref()
			.is_some_and(|(_, nested_items)| contains_macro_invocation(nested_items, name)),
		_ => false,
	})
}

#[test]
fn documented_helper_impls_expand_before_validation() -> TestResult {
	let file = run_document_module(quote! {
		struct Demo(i32);

		documented_helper_impls! {
			impl Demo {
				/// Returns the stored value.
				#[document_signature]
				#[document_returns("The stored value.")]
				#[document_examples]
				///
				/// ```
				/// let demo = Demo(7);
				/// assert_eq!(demo.value(), 7);
				/// ```
				pub fn value(&self) -> i32 {
					self.0
				}
			}
		}
	})?;

	assert!(
		impl_method_names(&file).iter().any(|name| name == "value"),
		"expanded impl method should be present in document_module output",
	);
	assert!(
		!contains_macro_invocation(&file.items, "documented_helper_impls"),
		"documented_helper_impls marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_effect_reader_expands_before_validation() -> TestResult {
	let file = run_document_module(quote! {
		define_effect! {
			effect Reader;
		}
	})?;

	let enum_names = enum_names(&file);
	assert!(
		enum_names.iter().any(|name| name == "Reader"),
		"generated Reader enum should be present",
	);
	assert!(
		enum_names.iter().any(|name| name == "SendReader"),
		"generated SendReader enum should be present",
	);
	assert!(
		enum_names.iter().any(|name| name == "BoxReader"),
		"generated BoxReader enum should be present",
	);
	assert!(
		impl_method_names(&file).iter().any(|name| name == "map"),
		"generated Functor impl methods should be present",
	);
	assert!(
		impl_method_names(&file).iter().any(|name| name == "send_map"),
		"generated SendFunctor impl method should be present",
	);
	assert!(
		!contains_macro_invocation(&file.items, "define_effect"),
		"define_effect marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_effect_reader_emits_documented_surface() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			define_effect! {
				effect Reader;
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("ReaderBrand"),
		"generated items should include ReaderBrand kind impls",
	);
	assert!(
		output_text.contains("BoxReaderBrand"),
		"generated items should include BoxReaderBrand kind impls",
	);
	assert!(
		output_text.contains("SendReaderBrand"),
		"generated items should include SendReaderBrand kind impls",
	);
	assert!(
		output_text.contains("### Type Signature"),
		"generated methods should run through document_module signature generation",
	);
	assert!(
		!output_text.contains("document_signature"),
		"document_module should consume document_signature on generated methods",
	);
	assert!(
		!output_text.contains("__document_module_generated"),
		"internal generated-item marker should be removed before output",
	);

	let returns_pos = output_text
		.find("### Returns")
		.ok_or_else(|| std::io::Error::other("generated output should include Returns docs"))?;
	let examples_pos = output_text
		.find("### Examples")
		.ok_or_else(|| std::io::Error::other("generated output should include Examples docs"))?;
	let example_code_pos = output_text.find("let original").ok_or_else(|| {
		std::io::Error::other("generated output should retain the Reader clone example")
	})?;
	assert!(
		returns_pos < examples_pos && examples_pos < example_code_pos,
		"generated method docs should keep Returns and Examples headings before the example code",
	);

	Ok(())
}

#[test]
fn define_effect_rejects_unsupported_effects() -> TestResult {
	let error = match document_module_worker(
		TokenStream::new(),
		quote! {
			define_effect! {
				effect State;
			}
		},
	) {
		Ok(_) => {
			return Err(std::io::Error::other(
				"define_effect should reject unsupported effect names",
			)
			.into());
		}
		Err(error) => error,
	};

	assert!(
		error.to_string().contains("currently only supports `effect Reader;`"),
		"error should explain the supported first slice; got: {error}",
	);

	Ok(())
}

#[test]
fn documented_helper_impls_are_documented_after_expansion() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			struct Demo(i32);

			documented_helper_impls! {
				impl Demo {
					/// Returns the stored value.
					#[document_signature]
					#[document_returns("The stored value.")]
					#[document_examples]
					///
					/// ```
					/// let demo = Demo(7);
					/// assert_eq!(demo.value(), 7);
					/// ```
					pub fn value(&self) -> i32 {
						self.0
					}
				}
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("### Type Signature"),
		"generated methods should run through document_module signature generation",
	);
	assert!(
		output_text.contains("_fp_macros_warning"),
		"generated impl blocks should run through document_module validation",
	);
	assert!(
		!output_text.contains("document_signature"),
		"document_module should consume document_signature on generated methods",
	);

	Ok(())
}

#[test]
fn documented_helper_impls_reject_non_impl_items() -> TestResult {
	let error = match document_module_worker(
		TokenStream::new(),
		quote! {
			documented_helper_impls! {
				pub struct NotAnImpl;
			}
		},
	) {
		Ok(_) => {
			return Err(std::io::Error::other(
				"documented_helper_impls should reject non-impl input",
			)
			.into());
		}
		Err(error) => error,
	};

	assert!(
		error.to_string().contains("only accepts Rust impl blocks"),
		"error should explain the constrained generator input; got: {error}",
	);

	Ok(())
}
