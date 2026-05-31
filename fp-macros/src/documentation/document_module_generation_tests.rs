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
		Item::Impl(item_impl) => item_impl.items.iter().any(|impl_item| match impl_item {
			ImplItem::Macro(item_macro) => item_macro.mac.path.is_ident(name),
			_ => false,
		}),
		Item::Mod(module) => module
			.content
			.as_ref()
			.is_some_and(|(_, nested_items)| contains_macro_invocation(nested_items, name)),
		_ => false,
	})
}

#[test]
fn define_run_wrapper_reader_methods_expand_before_validation() -> TestResult {
	let file = run_document_module(quote! {
		#[document_type_parameters(
			"The first-order effect row brand.",
			"The scoped-effect row brand.",
			"The result type."
		)]
		impl<R, S, A> Run<R, S, A>
		where
			R: 'static,
			S: 'static,
			A: 'static,
		{
			define_run_wrapper! {
				wrapper Run;
				effect Reader;
				method ask;
			}

			define_run_wrapper! {
				wrapper Run;
				effect Reader;
				method asks;
			}
		}

		#[document_type_parameters("The first-order effect row brand.", "The result type.")]
		#[document_parameters("The `Run` program to interpret.")]
		impl<R, A> Run<R, CNilBrand, A>
		where
			R: 'static,
			A: 'static,
		{
			define_run_wrapper! {
				wrapper Run;
				effect Reader;
				method run_reader;
			}
		}
	})?;

	let method_names = impl_method_names(&file);
	assert!(
		method_names.iter().any(|name| name == "ask"),
		"generated Run::ask method should be present",
	);
	assert!(
		method_names.iter().any(|name| name == "asks"),
		"generated Run::asks method should be present",
	);
	assert!(
		method_names.iter().any(|name| name == "run_reader"),
		"generated Run::run_reader method should be present",
	);
	assert!(
		!contains_macro_invocation(&file.items, "define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_run_state_methods_expand_before_validation() -> TestResult {
	let file = run_document_module(quote! {
		#[document_type_parameters(
			"The first-order effect row brand.",
			"The scoped-effect row brand.",
			"The result type."
		)]
		impl<R, S, A> Run<R, S, A>
		where
			R: 'static,
			S: 'static,
			A: 'static,
		{
			define_run_wrapper! {
				wrapper Run;
				effect State;
				method get;
			}
		}

		#[document_type_parameters("The first-order effect row brand.", "The scoped-effect row brand.")]
		impl<R, S> Run<R, S, ()>
		where
			R: 'static,
			S: 'static,
		{
			define_run_wrapper! {
				wrapper Run;
				effect State;
				method put;
			}

			define_run_wrapper! {
				wrapper Run;
				effect State;
				method modify;
			}
		}

		#[document_type_parameters("The first-order effect row brand.", "The result type.")]
		#[document_parameters("The `Run` program to interpret.")]
		impl<R, A> Run<R, CNilBrand, A>
		where
			R: 'static,
			A: 'static,
		{
			define_run_wrapper! {
				wrapper Run;
				effect State;
				method run_state;
			}
		}
	})?;

	let method_names = impl_method_names(&file);
	assert!(
		method_names.iter().any(|name| name == "get"),
		"generated Run::get method should be present",
	);
	assert!(
		method_names.iter().any(|name| name == "put"),
		"generated Run::put method should be present",
	);
	assert!(
		method_names.iter().any(|name| name == "modify"),
		"generated Run::modify method should be present",
	);
	assert!(
		method_names.iter().any(|name| name == "run_state"),
		"generated Run::run_state method should be present",
	);
	assert!(
		!contains_macro_invocation(&file.items, "define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_reader_methods_emit_documented_surface() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			#[document_type_parameters("The first-order effect row brand.", "The result type.")]
			#[document_parameters("The `Run` program to interpret.")]
			impl<R, A> Run<R, CNilBrand, A>
			where
				R: 'static,
				A: 'static,
			{
				define_run_wrapper! {
					wrapper Run;
					effect Reader;
					method run_reader;
				}
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("### Type Signature"),
		"generated Run helper should run through document_module signature generation",
	);
	assert!(
		output_text.contains("* `self`: The `Run` program to interpret."),
		"generated receiver docs should use the impl-level document_parameters text",
	);
	assert!(
		output_text.contains("* `env`: The environment value supplied to every Reader ask."),
		"generated method parameter docs should be present",
	);
	assert!(
		!output_text.contains("define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);
	assert!(
		!output_text.contains("__document_module_generated"),
		"internal generated-item marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_rcrun_reader_methods_emit_documented_surface() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			#[document_type_parameters("The first-order effect row brand.", "The result type.")]
			#[document_parameters("The `RcRun` program to interpret.")]
			impl<R, A> RcRun<R, CNilBrand, A>
			where
				R: 'static,
				A: 'static,
			{
				define_run_wrapper! {
					wrapper RcRun;
					effect Reader;
					method run_reader;
				}
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("RcCoyoneda"),
		"generated RcRun helper should use the RcCoyoneda Reader representation",
	);
	assert!(
		output_text.contains("* `self`: The `RcRun` program to interpret."),
		"generated RcRun receiver docs should use the impl-level document_parameters text",
	);
	assert!(
		output_text.contains("A first-order-only `RcRun` program with the Reader effect removed."),
		"generated RcRun returns docs should be present",
	);
	assert!(
		!output_text.contains("define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_arcrun_reader_methods_emit_documented_surface() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			#[document_type_parameters("The first-order effect row brand.", "The result type.")]
			#[document_parameters("The `ArcRun` program to interpret.")]
			impl<R, A> ArcRun<R, CNilBrand, A>
			where
				R: 'static,
				A: 'static,
			{
				define_run_wrapper! {
					wrapper ArcRun;
					effect Reader;
					method run_reader;
				}
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("ArcCoyoneda"),
		"generated ArcRun helper should use the ArcCoyoneda Reader representation",
	);
	assert!(
		output_text.contains("SendReaderBrand"),
		"generated ArcRun helper should use the SendReaderBrand Reader representation",
	);
	assert!(
		output_text.contains("* `self`: The `ArcRun` program to interpret."),
		"generated ArcRun receiver docs should use the impl-level document_parameters text",
	);
	assert!(
		!output_text.contains("define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_run_explicit_reader_methods_emit_documented_surface() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			#[document_type_parameters("The lifetime carried by the explicit wrapper.", "The first-order effect row brand.", "The result type.")]
			#[document_parameters("The `RunExplicit` program to interpret.")]
			impl<'a, R, A> RunExplicit<'a, R, CNilBrand, A>
			where
				R: 'static,
				A: 'a,
			{
				define_run_wrapper! {
					wrapper RunExplicit;
					effect Reader;
					method run_reader;
				}
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("Coyoneda"),
		"generated RunExplicit helper should use the explicit Coyoneda Reader representation",
	);
	assert!(
		output_text.contains("BoxReaderBrand"),
		"generated RunExplicit helper should use the BoxReaderBrand Reader representation",
	);
	assert!(
		output_text.contains("* `self`: The `RunExplicit` program to interpret."),
		"generated RunExplicit receiver docs should use the impl-level document_parameters text",
	);
	assert!(
		!output_text.contains("define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_rcrun_explicit_reader_methods_emit_documented_surface() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			#[document_type_parameters("The lifetime carried by the explicit wrapper.", "The first-order effect row brand.", "The result type.")]
			#[document_parameters("The `RcRunExplicit` program to interpret.")]
			impl<'a, R, A> RcRunExplicit<'a, R, CNilBrand, A>
			where
				R: 'static,
				A: 'a,
			{
				define_run_wrapper! {
					wrapper RcRunExplicit;
					effect Reader;
					method run_reader;
				}
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("RcCoyoneda"),
		"generated RcRunExplicit helper should use the explicit RcCoyoneda Reader representation",
	);
	assert!(
		output_text.contains("RcFreeExplicit"),
		"generated RcRunExplicit helper should preserve explicit RcFree projection bounds",
	);
	assert!(
		output_text.contains("* `self`: The `RcRunExplicit` program to interpret."),
		"generated RcRunExplicit receiver docs should use the impl-level document_parameters text",
	);
	assert!(
		!output_text.contains("define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_arcrun_explicit_reader_methods_emit_documented_surface() -> TestResult {
	let output = document_module_worker(
		TokenStream::new(),
		quote! {
			#[document_type_parameters("The lifetime carried by the explicit wrapper.", "The first-order effect row brand.", "The result type.")]
			#[document_parameters("The `ArcRunExplicit` program to interpret.")]
			impl<'a, R, A> ArcRunExplicit<'a, R, CNilBrand, A>
			where
				R: 'static,
				A: 'a,
			{
				define_run_wrapper! {
					wrapper ArcRunExplicit;
					effect Reader;
					method run_reader;
				}
			}
		},
	)?;

	let output_text = output.to_string();
	assert!(
		output_text.contains("ArcCoyoneda"),
		"generated ArcRunExplicit helper should use the explicit ArcCoyoneda Reader representation",
	);
	assert!(
		output_text.contains("ArcFreeExplicit"),
		"generated ArcRunExplicit helper should preserve explicit ArcFree projection bounds",
	);
	assert!(
		output_text.contains("SendReaderBrand"),
		"generated ArcRunExplicit helper should preserve SendReaderBrand",
	);
	assert!(
		output_text.contains("* `self`: The `ArcRunExplicit` program to interpret."),
		"generated ArcRunExplicit receiver docs should use the impl-level document_parameters text",
	);
	assert!(
		!output_text.contains("define_run_wrapper"),
		"define_run_wrapper marker should be removed before output",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_rejects_top_level_invocation() -> TestResult {
	let error = match document_module_worker(
		TokenStream::new(),
		quote! {
			define_run_wrapper! {
				wrapper Run;
				effect Reader;
				method ask;
			}
		},
	) {
		Ok(_) => {
			return Err(std::io::Error::other(
				"define_run_wrapper should reject item-position input",
			)
			.into());
		}
		Err(error) => error,
	};

	assert!(
		error.to_string().contains("must be used inside an impl block"),
		"error should explain where define_run_wrapper is supported; got: {error}",
	);

	Ok(())
}

#[test]
fn define_run_wrapper_rejects_unsupported_methods() -> TestResult {
	let error = match document_module_worker(
		TokenStream::new(),
		quote! {
			#[document_type_parameters(
				"The first-order effect row brand.",
				"The scoped-effect row brand.",
				"The result type."
			)]
			impl<R, S, A> Run<R, S, A>
			where
				R: 'static,
				S: 'static,
				A: 'static,
			{
				define_run_wrapper! {
					wrapper Run;
					effect Reader;
					method local;
				}
			}
		},
	) {
		Ok(_) => {
			return Err(std::io::Error::other(
				"define_run_wrapper should reject unsupported method names",
			)
			.into());
		}
		Err(error) => error,
	};

	assert!(
		error
			.to_string()
			.contains("currently only supports Reader methods `ask`, `asks`, and `run_reader`"),
		"error should explain the supported first slice; got: {error}",
	);

	Ok(())
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
fn define_effect_state_expands_before_validation() -> TestResult {
	let file = run_document_module(quote! {
		define_effect! {
			effect State;
		}
	})?;

	let enum_names = enum_names(&file);
	assert!(
		enum_names.iter().any(|name| name == "State"),
		"generated State enum should be present",
	);
	assert!(
		enum_names.iter().any(|name| name == "SendState"),
		"generated SendState enum should be present",
	);
	assert!(
		enum_names.iter().any(|name| name == "BoxState"),
		"generated BoxState enum should be present",
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
fn define_effect_rejects_unsupported_effects() -> TestResult {
	let error = match document_module_worker(
		TokenStream::new(),
		quote! {
			define_effect! {
				effect Writer;
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
		error.to_string().contains("currently only supports `effect Reader;` and `effect State;`"),
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
