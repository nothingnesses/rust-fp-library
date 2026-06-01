use {
	proc_macro2::{
		Literal,
		TokenStream,
	},
	quote::{
		format_ident,
		quote,
	},
};

struct ContinuationEffectNames {
	cell: &'static str,
	send_cell: &'static str,
	boxed_cell: &'static str,
	brand: &'static str,
	send_brand: &'static str,
	boxed_brand: &'static str,
	operation: &'static str,
	value_parameter: &'static str,
}

struct DirectPayloadEffectNames {
	cell: &'static str,
	brand: &'static str,
	operation: &'static str,
	payload_parameter: &'static str,
}

struct FixedMessageAbortEffectNames {
	cell: &'static str,
	brand: &'static str,
	operation: &'static str,
}

struct CoroutineEffectNames {
	cell: &'static str,
	send_cell: &'static str,
	boxed_cell: &'static str,
	brand: &'static str,
	send_brand: &'static str,
	boxed_brand: &'static str,
	operation: &'static str,
}

fn string_literal(value: &str) -> Literal {
	Literal::string(value)
}

fn generated_examples(reason: &'static str) -> TokenStream {
	let reason = string_literal(reason);

	quote! {
		#[document_examples(skip_call_check, reason = #reason)]
		#[doc = ""]
		#[doc = "```"]
		#[doc = "let values = vec![1, 2, 3];"]
		#[doc = "assert_eq!(values.len(), 3);"]
		#[doc = "```"]
	}
}

fn continuation_effect_items_tokens(names: ContinuationEffectNames) -> TokenStream {
	let cell = format_ident!("{}", names.cell);
	let send_cell = format_ident!("{}", names.send_cell);
	let boxed_cell = format_ident!("{}", names.boxed_cell);
	let brand = format_ident!("{}", names.brand);
	let send_brand = format_ident!("{}", names.send_brand);
	let boxed_brand = format_ident!("{}", names.boxed_brand);
	let operation = format_ident!("{}", names.operation);
	let value_parameter = format_ident!("{}", names.value_parameter);

	let clone_examples = generated_examples(
		"Generated effect Clone impl examples are smoke examples; wrapper-level helper tests exercise direct continuation use.",
	);
	let map_examples = generated_examples(
		"Generated effect Functor impl examples are smoke examples; wrapper-level helper tests exercise direct map use.",
	);
	let send_map_examples = generated_examples(
		"Generated effect SendFunctor impl examples are smoke examples; wrapper-level helper tests exercise direct send_map use.",
	);
	let boxed_map_examples = generated_examples(
		"Generated single-shot effect Functor impl examples are smoke examples; wrapper-level helper tests exercise direct map use.",
	);

	quote! {
		/// First-order continuation effect type.
		///
		/// The operation carries a continuation in `P`'s pointer kind.
		#[document_type_parameters(
			"The lifetime of the continuation and any references it captures.",
			"The pointer brand used for the continuation.",
			"The value supplied by the effect handler.",
			"The result type produced by running the effect."
		)]
		pub enum #cell<'a, P, #value_parameter, A>
		where
			P: ToDynCloneFn,
			#value_parameter: 'a,
			A: 'a, {
			/// Request a value from the handler and continue with it.
			#operation(<P as RefCountedPointer>::Of<'a, dyn 'a + Fn(#value_parameter) -> A>),
		}

		impl_kind! {
			impl<P: ToDynCloneFn, #value_parameter: 'static> for #brand<P, #value_parameter> {
				type Of<'a, A: 'a>: 'a = #cell<'a, P, #value_parameter, A>;
			}
		}

		#[document_type_parameters(
			"The lifetime of the continuation.",
			"The pointer brand used for the continuation.",
			"The value supplied by the effect handler.",
			"The result type."
		)]
		#[document_parameters("The effect to clone.")]
		impl<'a, P, #value_parameter, A> Clone for #cell<'a, P, #value_parameter, A>
		where
			P: ToDynCloneFn,
			#value_parameter: 'a,
			A: 'a,
		{
			/// Clones the effect by refcount-bumping the stored continuation pointer.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new effect sharing the continuation by refcount.")]
			#clone_examples
			fn clone(&self) -> Self {
				match self {
					#cell::#operation(k) => #cell::#operation(k.clone()),
				}
			}
		}

		#[document_type_parameters("The pointer brand used for the continuation.", "The value supplied by the handler.")]
		impl<P, #value_parameter> Functor for #brand<P, #value_parameter>
		where
			P: ToDynCloneFn,
			#value_parameter: 'static,
		{
			/// Maps `f` over the result type of this effect.
			///
			/// Composes `f` with the stored continuation.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuation.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters(
				"The function to compose with the continuation.",
				"The effect to map over."
			)]
			#[document_returns("A new effect with `f` composed onto the continuation.")]
			#map_examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#cell::#operation(k) =>
						#cell::#operation(<P as ToDynCloneFn>::new(move |value: #value_parameter| {
							f((*k)(value))
						})),
				}
			}
		}

		/// Thread-safe sibling with `Send + Sync`-bounded continuation trait objects.
		#[document_type_parameters(
			"The lifetime of the continuation and any references it captures.",
			"The pointer brand used for the continuation.",
			"The value supplied by the effect handler.",
			"The result type produced by running the effect."
		)]
		pub enum #send_cell<'a, P, #value_parameter, A>
		where
			P: ToDynSendFn,
			#value_parameter: 'a,
			A: 'a, {
			/// Request a value from the handler and continue with it.
			#operation(<P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(#value_parameter) -> A + Send + Sync,
			>),
		}

		impl_kind! {
			impl<P: ToDynSendFn, #value_parameter: 'static> for #send_brand<P, #value_parameter> {
				type Of<'a, A: 'a>: 'a = #send_cell<'a, P, #value_parameter, A>;
			}
		}

		#[document_type_parameters(
			"The lifetime of the continuation.",
			"The pointer brand used for the continuation.",
			"The value supplied by the effect handler.",
			"The result type."
		)]
		#[document_parameters("The effect to clone.")]
		impl<'a, P, #value_parameter, A> Clone for #send_cell<'a, P, #value_parameter, A>
		where
			P: ToDynSendFn,
			#value_parameter: 'a,
			A: 'a,
		{
			/// Clones the effect by refcount-bumping the stored continuation pointer.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new effect sharing the continuation by refcount.")]
			#clone_examples
			fn clone(&self) -> Self {
				match self {
					#send_cell::#operation(k) => #send_cell::#operation(k.clone()),
				}
			}
		}

		#[document_type_parameters("The pointer brand used for the continuation.", "The value supplied by the handler.")]
		impl<P, #value_parameter> SendFunctor for #send_brand<P, #value_parameter>
		where
			P: ToDynSendFn,
			#value_parameter: Send + Sync + 'static,
		{
			/// Maps `f` over the result type of this thread-safe effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuation.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters(
				"The function to compose with the continuation.",
				"The effect to map over."
			)]
			#[document_returns("A new effect with `f` composed onto the continuation.")]
			#send_map_examples
			fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
				f: impl Fn(A) -> B + Send + Sync + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#send_cell::#operation(k) =>
						#send_cell::#operation(<P as ToDynSendFn>::new(move |value: #value_parameter| {
							f((*k)(value))
						})),
				}
			}
		}

		/// Single-shot sibling with `dyn FnOnce`-bounded continuation trait objects.
		#[document_type_parameters(
			"The lifetime of the continuation and any references it captures.",
			"The pointer brand used for the continuation.",
			"The value supplied by the effect handler.",
			"The result type produced by running the effect."
		)]
		pub enum #boxed_cell<'a, P, #value_parameter, A>
		where
			P: ToDynFnOnce,
			#value_parameter: 'a,
			A: 'a, {
			/// Request a value from the handler and continue with it.
			#operation(<P as Pointer>::Of<'a, dyn 'a + FnOnce(#value_parameter) -> A>),
		}

		impl_kind! {
			impl<P: ToDynFnOnce, #value_parameter: 'static> for #boxed_brand<P, #value_parameter> {
				type Of<'a, A: 'a>: 'a = #boxed_cell<'a, P, #value_parameter, A>;
			}
		}

		#[document_type_parameters("The value supplied by the handler.")]
		impl<#value_parameter> Functor for #boxed_brand<BoxBrand, #value_parameter>
		where
			#value_parameter: 'static,
		{
			/// Maps `f` over the result type of this single-shot effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuation.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters(
				"The function to compose with the continuation.",
				"The effect to map over."
			)]
			#[document_returns("A new effect with `f` composed onto the continuation.")]
			#boxed_map_examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#boxed_cell::#operation(k) =>
						#boxed_cell::#operation(<BoxBrand as ToDynFnOnce>::new(
							move |value: #value_parameter| f(k(value)),
						)),
				}
			}
		}
	}
}

pub(super) fn fresh_effect_items_tokens() -> TokenStream {
	continuation_effect_items_tokens(ContinuationEffectNames {
		cell: "Fresh",
		send_cell: "SendFresh",
		boxed_cell: "BoxFresh",
		brand: "FreshBrand",
		send_brand: "SendFreshBrand",
		boxed_brand: "BoxFreshBrand",
		operation: "Fresh",
		value_parameter: "Id",
	})
}

pub(super) fn input_effect_items_tokens() -> TokenStream {
	continuation_effect_items_tokens(ContinuationEffectNames {
		cell: "Input",
		send_cell: "SendInput",
		boxed_cell: "BoxInput",
		brand: "InputBrand",
		send_brand: "SendInputBrand",
		boxed_brand: "BoxInputBrand",
		operation: "Input",
		value_parameter: "Item",
	})
}

fn coroutine_effect_items_tokens_from_names(names: CoroutineEffectNames) -> TokenStream {
	let cell = format_ident!("{}", names.cell);
	let send_cell = format_ident!("{}", names.send_cell);
	let boxed_cell = format_ident!("{}", names.boxed_cell);
	let brand = format_ident!("{}", names.brand);
	let send_brand = format_ident!("{}", names.send_brand);
	let boxed_brand = format_ident!("{}", names.boxed_brand);
	let operation = format_ident!("{}", names.operation);

	let examples = generated_examples(
		"Generated Coroutine trait impl examples are smoke examples; wrapper-level helper tests exercise status and resume behavior.",
	);

	quote! {
		/// Coroutine-yield first-order effect type.
		#[document_type_parameters(
			"The lifetime of the continuation and any references it captures.",
			"The pointer brand used for the continuation.",
			"The value yielded to the coroutine runner.",
			"The input value accepted when the suspended coroutine resumes.",
			"The result type produced after resuming the coroutine."
		)]
		pub enum #cell<'a, P, Out, In, A>
		where
			P: ToDynCloneFn,
			Out: 'a,
			In: 'a,
			A: 'a, {
			/// Yield an output value and continue when the runner supplies an input value.
			#operation(Out, <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(In) -> A>),
		}

		impl_kind! {
			impl<P: ToDynCloneFn, Out: 'static, In: 'static> for #brand<P, Out, In> {
				type Of<'a, A: 'a>: 'a = #cell<'a, P, Out, In, A>;
			}
		}

		#[document_type_parameters(
			"The lifetime of the continuation.",
			"The pointer brand used for the continuation.",
			"The yielded output type.",
			"The resume input type.",
			"The result type."
		)]
		#[document_parameters("The coroutine effect to clone.")]
		impl<'a, P, Out, In, A> Clone for #cell<'a, P, Out, In, A>
		where
			P: ToDynCloneFn,
			Out: Clone + 'a,
			In: 'a,
			A: 'a,
		{
			/// Clones the effect by cloning the yielded value and refcount-bumping the continuation.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new coroutine effect sharing the continuation by refcount.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					#cell::#operation(out, k) => #cell::#operation(out.clone(), k.clone()),
				}
			}
		}

		#[document_type_parameters(
			"The pointer brand used for the continuation.",
			"The yielded output type.",
			"The resume input type."
		)]
		impl<P, Out, In> Functor for #brand<P, Out, In>
		where
			P: ToDynCloneFn,
			Out: 'static,
			In: 'static,
		{
			/// Maps `f` over the result produced after the coroutine resumes.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuation.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters(
				"The function to compose with the resume continuation.",
				"The coroutine effect to map over."
			)]
			#[document_returns("A new coroutine effect with `f` composed onto the resume continuation.")]
			#examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#cell::#operation(out, k) =>
						#cell::#operation(out, <P as ToDynCloneFn>::new(move |input: In| {
							f((*k)(input))
						})),
				}
			}
		}

		/// Thread-safe sibling with `Send + Sync`-bounded continuation trait objects.
		#[document_type_parameters(
			"The lifetime of the continuation and any references it captures.",
			"The pointer brand used for the continuation.",
			"The value yielded to the coroutine runner.",
			"The input value accepted when the suspended coroutine resumes.",
			"The result type produced after resuming the coroutine."
		)]
		pub enum #send_cell<'a, P, Out, In, A>
		where
			P: ToDynSendFn,
			Out: 'a,
			In: 'a,
			A: 'a, {
			/// Yield an output value and continue when the runner supplies an input value.
			#operation(Out, <P as SendRefCountedPointer>::Of<
				'a,
				dyn 'a + Fn(In) -> A + Send + Sync,
			>),
		}

		impl_kind! {
			impl<P: ToDynSendFn, Out: 'static, In: 'static> for #send_brand<P, Out, In> {
				type Of<'a, A: 'a>: 'a = #send_cell<'a, P, Out, In, A>;
			}
		}

		#[document_type_parameters(
			"The lifetime of the continuation.",
			"The pointer brand used for the continuation.",
			"The yielded output type.",
			"The resume input type.",
			"The result type."
		)]
		#[document_parameters("The coroutine effect to clone.")]
		impl<'a, P, Out, In, A> Clone for #send_cell<'a, P, Out, In, A>
		where
			P: ToDynSendFn,
			Out: Clone + 'a,
			In: 'a,
			A: 'a,
		{
			/// Clones the effect by cloning the yielded value and refcount-bumping the continuation.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new coroutine effect sharing the continuation by refcount.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					#send_cell::#operation(out, k) => #send_cell::#operation(out.clone(), k.clone()),
				}
			}
		}

		#[document_type_parameters(
			"The pointer brand used for the continuation.",
			"The yielded output type.",
			"The resume input type."
		)]
		impl<P, Out, In> SendFunctor for #send_brand<P, Out, In>
		where
			P: ToDynSendFn,
			Out: Send + Sync + 'static,
			In: Send + Sync + 'static,
		{
			/// Maps `f` over the result produced after the coroutine resumes.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuation.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters(
				"The function to compose with the resume continuation.",
				"The coroutine effect to map over."
			)]
			#[document_returns("A new coroutine effect with `f` composed onto the resume continuation.")]
			#examples
			fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
				f: impl Fn(A) -> B + Send + Sync + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#send_cell::#operation(out, k) =>
						#send_cell::#operation(out, <P as ToDynSendFn>::new(move |input: In| {
							f((*k)(input))
						})),
				}
			}
		}

		/// Single-shot sibling with `dyn FnOnce`-bounded continuation trait objects.
		#[document_type_parameters(
			"The lifetime of the continuation and any references it captures.",
			"The pointer brand used for the continuation.",
			"The value yielded to the coroutine runner.",
			"The input value accepted when the suspended coroutine resumes.",
			"The result type produced after resuming the coroutine."
		)]
		pub enum #boxed_cell<'a, P, Out, In, A>
		where
			P: ToDynFnOnce,
			Out: 'a,
			In: 'a,
			A: 'a, {
			/// Yield an output value and continue when the runner supplies an input value.
			#operation(Out, <P as Pointer>::Of<'a, dyn 'a + FnOnce(In) -> A>),
		}

		impl_kind! {
			impl<P: ToDynFnOnce, Out: 'static, In: 'static> for #boxed_brand<P, Out, In> {
				type Of<'a, A: 'a>: 'a = #boxed_cell<'a, P, Out, In, A>;
			}
		}

		#[document_type_parameters("The yielded output type.", "The resume input type.")]
		impl<Out, In> Functor for #boxed_brand<BoxBrand, Out, In>
		where
			Out: 'static,
			In: 'static,
		{
			/// Maps `f` over the result produced after the coroutine resumes.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuation.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters(
				"The function to compose with the resume continuation.",
				"The coroutine effect to map over."
			)]
			#[document_returns("A new coroutine effect with `f` composed onto the resume continuation.")]
			#examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#boxed_cell::#operation(out, k) =>
						#boxed_cell::#operation(out, <BoxBrand as ToDynFnOnce>::new(
							move |input: In| f(k(input)),
						)),
				}
			}
		}

		/// Coroutine runner status for the default `Run` wrapper.
		#[document_type_parameters(
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		pub enum RunCoroutineStatus<R, S, Out, In, A>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Out: 'static,
			In: 'static,
			A: 'static, {
			/// The program completed without another Coroutine yield.
			Done(A),
			/// The program yielded an output value and can be resumed once with an input value.
			Continue(
				Out,
				<BoxBrand as Pointer>::Of<'static, dyn 'static + FnOnce(In) -> Run<R, S, A>>,
			),
		}

		/// Coroutine runner status for the multi-shot `RcRun` wrapper.
		#[document_type_parameters(
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		pub enum RcRunCoroutineStatus<R, S, Out, In, A>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Out: 'static,
			In: 'static,
			A: 'static, {
			/// The program completed without another Coroutine yield.
			Done(A),
			/// The program yielded an output value and can be resumed any number of times with an input value.
			Continue(
				Out,
				<RcBrand as RefCountedPointer>::Of<'static, dyn 'static + Fn(In) -> RcRun<R, S, A>>,
			),
		}

		#[document_type_parameters(
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		#[document_parameters("The coroutine status to clone.")]
		impl<R, S, Out, In, A> Clone for RcRunCoroutineStatus<R, S, Out, In, A>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Out: Clone + 'static,
			In: 'static,
			A: Clone + 'static,
		{
			/// Clones the multi-shot Coroutine status.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new status sharing the resume continuation by refcount.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					RcRunCoroutineStatus::Done(result) => RcRunCoroutineStatus::Done(result.clone()),
					RcRunCoroutineStatus::Continue(out, resume) =>
						RcRunCoroutineStatus::Continue(out.clone(), resume.clone()),
				}
			}
		}

		/// Coroutine runner status for the thread-safe multi-shot `ArcRun` wrapper.
		#[document_type_parameters(
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		pub enum ArcRunCoroutineStatus<R, S, Out, In, A>
		where
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + 'static,
			Out: Send + Sync + 'static,
			In: Send + Sync + 'static,
			A: 'static, {
			/// The program completed without another Coroutine yield.
			Done(A),
			/// The program yielded an output value and can be resumed any number of times with an input value.
			Continue(
				Out,
				<ArcBrand as SendRefCountedPointer>::Of<
					'static,
					dyn 'static + Fn(In) -> ArcRun<R, S, A> + Send + Sync,
				>,
			),
		}

		#[document_type_parameters(
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		#[document_parameters("The coroutine status to clone.")]
		impl<R, S, Out, In, A> Clone for ArcRunCoroutineStatus<R, S, Out, In, A>
		where
			NodeBrand<R, S>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
				> + 'static,
			Out: Clone + Send + Sync + 'static,
			In: Send + Sync + 'static,
			A: Clone + 'static,
		{
			/// Clones the thread-safe multi-shot Coroutine status.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new status sharing the resume continuation by atomic refcount.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					ArcRunCoroutineStatus::Done(result) => ArcRunCoroutineStatus::Done(result.clone()),
					ArcRunCoroutineStatus::Continue(out, resume) =>
						ArcRunCoroutineStatus::Continue(out.clone(), resume.clone()),
				}
			}
		}

		/// Coroutine runner status for the explicit `RunExplicit` wrapper.
		#[document_type_parameters(
			"The lifetime of the resume continuation.",
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		pub enum RunExplicitCoroutineStatus<'a, R, S, Out, In, A>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Out: 'a,
			In: 'a,
			A: 'a, {
			/// The program completed without another Coroutine yield.
			Done(A),
			/// The program yielded an output value and can be resumed once with an input value.
			Continue(
				Out,
				<BoxBrand as Pointer>::Of<'a, dyn 'a + FnOnce(In) -> RunExplicit<'a, R, S, A>>,
			),
		}

		/// Coroutine runner status for the multi-shot explicit `RcRunExplicit` wrapper.
		#[document_type_parameters(
			"The lifetime of the resume continuation.",
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		pub enum RcRunExplicitCoroutineStatus<'a, R, S, Out, In, A>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Out: 'a,
			In: 'a,
			A: 'a, {
			/// The program completed without another Coroutine yield.
			Done(A),
			/// The program yielded an output value and can be resumed any number of times with an input value.
			Continue(
				Out,
				<RcBrand as RefCountedPointer>::Of<
					'a,
					dyn 'a + Fn(In) -> RcRunExplicit<'a, R, S, A>,
				>,
			),
		}

		#[document_type_parameters(
			"The lifetime of the resume continuation.",
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		#[document_parameters("The coroutine status to clone.")]
		impl<'a, R, S, Out, In, A> Clone for RcRunExplicitCoroutineStatus<'a, R, S, Out, In, A>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Out: Clone + 'a,
			In: 'a,
			A: Clone + 'a,
		{
			/// Clones the multi-shot explicit Coroutine status.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new status sharing the resume continuation by refcount.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					RcRunExplicitCoroutineStatus::Done(result) =>
						RcRunExplicitCoroutineStatus::Done(result.clone()),
					RcRunExplicitCoroutineStatus::Continue(out, resume) =>
						RcRunExplicitCoroutineStatus::Continue(out.clone(), resume.clone()),
				}
			}
		}

		/// Coroutine runner status for the thread-safe multi-shot explicit `ArcRunExplicit` wrapper.
		#[document_type_parameters(
			"The lifetime of the resume continuation.",
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		pub enum ArcRunExplicitCoroutineStatus<'a, R, S, Out, In, A>
		where
			R: WrapDrop + SendFunctor + 'static,
			S: WrapDrop + SendFunctor + 'static,
			Out: Send + Sync + 'a,
			In: Send + Sync + 'a,
			A: 'a, {
			/// The program completed without another Coroutine yield.
			Done(A),
			/// The program yielded an output value and can be resumed any number of times with an input value.
			Continue(
				Out,
				<ArcBrand as SendRefCountedPointer>::Of<
					'a,
					dyn 'a + Fn(In) -> ArcRunExplicit<'a, R, S, A> + Send + Sync,
				>,
			),
		}

		#[document_type_parameters(
			"The lifetime of the resume continuation.",
			"The residual first-order effect row after removing Coroutine.",
			"The scoped-effect row brand.",
			"The yielded output type.",
			"The resume input type.",
			"The completed result type."
		)]
		#[document_parameters("The coroutine status to clone.")]
		impl<'a, R, S, Out, In, A> Clone for ArcRunExplicitCoroutineStatus<'a, R, S, Out, In, A>
		where
			R: WrapDrop + SendFunctor + 'static,
			S: WrapDrop + SendFunctor + 'static,
			Out: Clone + Send + Sync + 'a,
			In: Send + Sync + 'a,
			A: Clone + 'a,
		{
			/// Clones the thread-safe multi-shot explicit Coroutine status.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new status sharing the resume continuation by atomic refcount.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					ArcRunExplicitCoroutineStatus::Done(result) =>
						ArcRunExplicitCoroutineStatus::Done(result.clone()),
					ArcRunExplicitCoroutineStatus::Continue(out, resume) =>
						ArcRunExplicitCoroutineStatus::Continue(out.clone(), resume.clone()),
				}
			}
		}
	}
}

pub(super) fn coroutine_effect_items_tokens() -> TokenStream {
	coroutine_effect_items_tokens_from_names(CoroutineEffectNames {
		cell: "Coroutine",
		send_cell: "SendCoroutine",
		boxed_cell: "BoxCoroutine",
		brand: "CoroutineBrand",
		send_brand: "SendCoroutineBrand",
		boxed_brand: "BoxCoroutineBrand",
		operation: "Yield",
	})
}

pub(super) fn kv_store_effect_items_tokens() -> TokenStream {
	let examples = generated_examples(
		"Generated KVStore trait impl examples are smoke examples; wrapper-level helper tests exercise direct operation use.",
	);

	quote! {
		/// Key-value-store first-order effect type.
		#[document_type_parameters(
			"The lifetime of the continuations and any references they capture.",
			"The pointer brand used for the continuations.",
			"The key type.",
			"The value type.",
			"The result type produced by running the effect."
		)]
		pub enum KVStore<'a, P, K, V, A>
		where
			P: ToDynCloneFn,
			K: 'a,
			V: 'a,
			A: 'a, {
			/// Look up the current value for a key.
			Lookup(K, <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(Option<V>) -> A>),
			/// Insert, replace, or delete a key.
			Update(K, Option<V>, <P as RefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A>),
		}

		impl_kind! {
			impl<P: ToDynCloneFn, K: 'static, V: 'static> for KVStoreBrand<P, K, V> {
				type Of<'a, A: 'a>: 'a = KVStore<'a, P, K, V, A>;
			}
		}

		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The pointer brand used for the continuations.",
			"The key type.",
			"The value type.",
			"The result type."
		)]
		#[document_parameters("The effect to clone.")]
		impl<'a, P, K, V, A> Clone for KVStore<'a, P, K, V, A>
		where
			P: ToDynCloneFn,
			K: Clone + 'a,
			V: Clone + 'a,
			A: 'a,
		{
			/// Clones the effect by cloning payload values and refcount-bumping continuations.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new effect sharing continuation pointers.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					KVStore::Lookup(key, k) => KVStore::Lookup(key.clone(), k.clone()),
					KVStore::Update(key, value, k) => KVStore::Update(key.clone(), value.clone(), k.clone()),
				}
			}
		}

		#[document_type_parameters("The pointer brand used for the continuations.", "The key type.", "The value type.")]
		impl<P, K, V> Functor for KVStoreBrand<P, K, V>
		where
			P: ToDynCloneFn,
			K: 'static,
			V: 'static,
		{
			/// Maps `f` over the result type of this key-value-store effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuations.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters("The function to compose with each continuation.", "The effect to map over.")]
			#[document_returns("A new effect with `f` composed onto each continuation.")]
			#examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					KVStore::Lookup(key, k) =>
						KVStore::Lookup(key, <P as ToDynCloneFn>::new(move |value: Option<V>| f((*k)(value)))),
					KVStore::Update(key, value, k) =>
						KVStore::Update(key, value, <P as ToDynCloneFn>::new(move |unit: ()| f((*k)(unit)))),
				}
			}
		}

		/// Thread-safe sibling with `Send + Sync`-bounded continuation trait objects.
		#[document_type_parameters(
			"The lifetime of the continuations and any references they capture.",
			"The pointer brand used for the continuations.",
			"The key type.",
			"The value type.",
			"The result type produced by running the effect."
		)]
		pub enum SendKVStore<'a, P, K, V, A>
		where
			P: ToDynSendFn,
			K: 'a,
			V: 'a,
			A: 'a, {
			/// Look up the current value for a key.
			Lookup(K, <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(Option<V>) -> A + Send + Sync>),
			/// Insert, replace, or delete a key.
			Update(K, Option<V>, <P as SendRefCountedPointer>::Of<'a, dyn 'a + Fn(()) -> A + Send + Sync>),
		}

		impl_kind! {
			impl<P: ToDynSendFn, K: 'static, V: 'static> for SendKVStoreBrand<P, K, V> {
				type Of<'a, A: 'a>: 'a = SendKVStore<'a, P, K, V, A>;
			}
		}

		#[document_type_parameters(
			"The lifetime of the continuations.",
			"The pointer brand used for the continuations.",
			"The key type.",
			"The value type.",
			"The result type."
		)]
		#[document_parameters("The effect to clone.")]
		impl<'a, P, K, V, A> Clone for SendKVStore<'a, P, K, V, A>
		where
			P: ToDynSendFn,
			K: Clone + 'a,
			V: Clone + 'a,
			A: 'a,
		{
			/// Clones the effect by cloning payload values and refcount-bumping continuations.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new effect sharing continuation pointers.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					SendKVStore::Lookup(key, k) => SendKVStore::Lookup(key.clone(), k.clone()),
					SendKVStore::Update(key, value, k) =>
						SendKVStore::Update(key.clone(), value.clone(), k.clone()),
				}
			}
		}

		#[document_type_parameters("The pointer brand used for the continuations.", "The key type.", "The value type.")]
		impl<P, K, V> SendFunctor for SendKVStoreBrand<P, K, V>
		where
			P: ToDynSendFn,
			K: Send + Sync + 'static,
			V: Send + Sync + 'static,
		{
			/// Maps `f` over the result type of this thread-safe key-value-store effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuations.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters("The function to compose with each continuation.", "The effect to map over.")]
			#[document_returns("A new effect with `f` composed onto each continuation.")]
			#examples
			fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
				f: impl Fn(A) -> B + Send + Sync + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					SendKVStore::Lookup(key, k) =>
						SendKVStore::Lookup(key, <P as ToDynSendFn>::new(move |value: Option<V>| f((*k)(value)))),
					SendKVStore::Update(key, value, k) =>
						SendKVStore::Update(key, value, <P as ToDynSendFn>::new(move |unit: ()| f((*k)(unit)))),
				}
			}
		}

		/// Single-shot sibling with `dyn FnOnce`-bounded continuation trait objects.
		#[document_type_parameters(
			"The lifetime of the continuations and any references they capture.",
			"The pointer brand used for the continuations.",
			"The key type.",
			"The value type.",
			"The result type produced by running the effect."
		)]
		pub enum BoxKVStore<'a, P, K, V, A>
		where
			P: ToDynFnOnce,
			K: 'a,
			V: 'a,
			A: 'a, {
			/// Look up the current value for a key.
			Lookup(K, <P as Pointer>::Of<'a, dyn 'a + FnOnce(Option<V>) -> A>),
			/// Insert, replace, or delete a key.
			Update(K, Option<V>, <P as Pointer>::Of<'a, dyn 'a + FnOnce(()) -> A>),
		}

		impl_kind! {
			impl<P: ToDynFnOnce, K: 'static, V: 'static> for BoxKVStoreBrand<P, K, V> {
				type Of<'a, A: 'a>: 'a = BoxKVStore<'a, P, K, V, A>;
			}
		}

		#[document_type_parameters("The key type.", "The value type.")]
		impl<K, V> Functor for BoxKVStoreBrand<BoxBrand, K, V>
		where
			K: 'static,
			V: 'static,
		{
			/// Maps `f` over the result type of this single-shot key-value-store effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the continuations.",
				"The original result type.",
				"The new result type after applying `f`."
			)]
			#[document_parameters("The function to compose with each continuation.", "The effect to map over.")]
			#[document_returns("A new effect with `f` composed onto each continuation.")]
			#examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					BoxKVStore::Lookup(key, k) =>
						BoxKVStore::Lookup(key, <BoxBrand as ToDynFnOnce>::new(move |value: Option<V>| f(k(value)))),
					BoxKVStore::Update(key, value, k) =>
						BoxKVStore::Update(key, value, <BoxBrand as ToDynFnOnce>::new(move |unit: ()| f(k(unit)))),
				}
			}
		}
	}
}

fn direct_payload_effect_items_tokens(names: DirectPayloadEffectNames) -> TokenStream {
	let cell = format_ident!("{}", names.cell);
	let brand = format_ident!("{}", names.brand);
	let operation = format_ident!("{}", names.operation);
	let payload_parameter = format_ident!("{}", names.payload_parameter);

	let examples = generated_examples(
		"Generated direct-payload effect trait impl examples are smoke examples; wrapper-level helper tests exercise direct operation use.",
	);

	quote! {
		/// Direct-payload first-order effect type.
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The payload value type.",
			"The result type produced by running the effect."
		)]
		pub enum #cell<'a, #payload_parameter, A: 'a> {
			/// Carry one payload value and continue with the next program value.
			#operation(#payload_parameter, A, core::marker::PhantomData<&'a ()>),
		}

		impl_kind! {
			impl<#payload_parameter: 'static> for #brand<#payload_parameter> {
				type Of<'a, A: 'a>: 'a = #cell<'a, #payload_parameter, A>;
			}
		}

		#[document_type_parameters("The lifetime of the effect.", "The payload value type.", "The result type.")]
		#[document_parameters("The effect to clone.")]
		impl<'a, #payload_parameter, A> Clone for #cell<'a, #payload_parameter, A>
		where
			#payload_parameter: Clone,
			A: Clone + 'a,
		{
			/// Clones the effect by cloning the payload and next program value.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new effect carrying cloned values.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					#cell::#operation(payload, next, _) =>
						#cell::#operation(payload.clone(), next.clone(), core::marker::PhantomData),
				}
			}
		}

		#[document_type_parameters("The payload value type.")]
		impl<#payload_parameter> Functor for #brand<#payload_parameter>
		where
			#payload_parameter: 'static,
		{
			/// Maps `f` over the next-program value of this effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the effect.",
				"The original next-program type.",
				"The new next-program type after applying `f`."
			)]
			#[document_parameters("The function to apply to the next-program value.", "The effect to map over.")]
			#[document_returns("A new effect with the same payload value and mapped next value.")]
			#examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#cell::#operation(payload, next, _) =>
						#cell::#operation(payload, f(next), core::marker::PhantomData),
				}
			}
		}

		#[document_type_parameters("The payload value type.")]
		impl<#payload_parameter> SendFunctor for #brand<#payload_parameter>
		where
			#payload_parameter: Send + Sync + 'static,
		{
			/// Maps `f` over the next-program value of this effect in thread-safe contexts.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the effect.",
				"The original next-program type.",
				"The new next-program type after applying `f`."
			)]
			#[document_parameters("The function to apply to the next-program value.", "The effect to map over.")]
			#[document_returns("A new effect with the same payload value and mapped next value.")]
			#examples
			fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
				f: impl Fn(A) -> B + Send + Sync + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#cell::#operation(payload, next, _) =>
						#cell::#operation(payload, f(next), core::marker::PhantomData),
				}
			}
		}
	}
}

pub(super) fn output_effect_items_tokens() -> TokenStream {
	direct_payload_effect_items_tokens(DirectPayloadEffectNames {
		cell: "Output",
		brand: "OutputBrand",
		operation: "Output",
		payload_parameter: "Out",
	})
}

pub(super) fn log_effect_items_tokens() -> TokenStream {
	direct_payload_effect_items_tokens(DirectPayloadEffectNames {
		cell: "Log",
		brand: "LogBrand",
		operation: "Log",
		payload_parameter: "Message",
	})
}

fn fixed_message_abort_effect_items_tokens(names: FixedMessageAbortEffectNames) -> TokenStream {
	let cell = format_ident!("{}", names.cell);
	let brand = format_ident!("{}", names.brand);
	let operation = format_ident!("{}", names.operation);

	let examples = generated_examples(
		"Generated fixed-message abort effect trait impl examples are smoke examples; wrapper-level helper tests exercise direct operation use.",
	);

	quote! {
		/// Fixed-message aborting first-order effect type.
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The phantom result type."
		)]
		pub enum #cell<'a, A: 'a> {
			/// Abort with a message. The program does not continue after this operation.
			#operation(std::string::String, core::marker::PhantomData<&'a A>),
		}

		impl_kind! {
			impl for #brand {
				type Of<'a, A: 'a>: 'a = #cell<'a, A>;
			}
		}

		#[document_type_parameters("The lifetime of the effect.", "The phantom result type.")]
		#[document_parameters("The effect to clone.")]
		impl<'a, A> Clone for #cell<'a, A>
		where
			A: 'a,
		{
			/// Clones the effect by cloning the message.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new effect carrying a clone of the message.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					#cell::#operation(message, _) =>
						#cell::#operation(message.clone(), core::marker::PhantomData),
				}
			}
		}

		impl Functor for #brand {
			/// Maps `f` over the phantom result type of this aborting effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the effect.",
				"The original phantom result type.",
				"The new phantom result type after applying `f`."
			)]
			#[document_parameters(
				"The function to map over the phantom result type.",
				"The effect to map over."
			)]
			#[document_returns("A new effect with the same message and new phantom result type.")]
			#examples
			fn map<'a, A: 'a, B: 'a>(
				_f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#cell::#operation(message, _) =>
						#cell::#operation(message, core::marker::PhantomData),
				}
			}
		}

		impl SendFunctor for #brand {
			/// Maps `f` over the phantom result type of this aborting effect in thread-safe contexts.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the effect.",
				"The original phantom result type.",
				"The new phantom result type after applying `f`."
			)]
			#[document_parameters(
				"The function to map over the phantom result type.",
				"The effect to map over."
			)]
			#[document_returns("A new effect with the same message and new phantom result type.")]
			#examples
			fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
				_f: impl Fn(A) -> B + Send + Sync + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					#cell::#operation(message, _) =>
						#cell::#operation(message, core::marker::PhantomData),
				}
			}
		}
	}
}

pub(super) fn fail_effect_items_tokens() -> TokenStream {
	fixed_message_abort_effect_items_tokens(FixedMessageAbortEffectNames {
		cell: "Fail",
		brand: "FailBrand",
		operation: "Fail",
	})
}
