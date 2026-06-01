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

pub(super) fn output_effect_items_tokens() -> TokenStream {
	let examples = generated_examples(
		"Generated Output trait impl examples are smoke examples; wrapper-level helper tests exercise direct operation use.",
	);

	quote! {
		/// Output-emitting first-order effect type.
		#[document_type_parameters(
			"The lifetime of the effect.",
			"The output value type.",
			"The result type produced by running the effect."
		)]
		pub enum Output<'a, Out, A: 'a> {
			/// Emit one output value and continue with the next program value.
			Output(Out, A, core::marker::PhantomData<&'a ()>),
		}

		impl_kind! {
			impl<Out: 'static> for OutputBrand<Out> {
				type Of<'a, A: 'a>: 'a = Output<'a, Out, A>;
			}
		}

		#[document_type_parameters("The lifetime of the effect.", "The output value type.", "The result type.")]
		#[document_parameters("The output effect to clone.")]
		impl<'a, Out, A> Clone for Output<'a, Out, A>
		where
			Out: Clone,
			A: Clone + 'a,
		{
			/// Clones the output effect by cloning the output and next program value.
			#[__document_module_generated]
			#[document_signature]
			#[document_returns("A new output effect carrying cloned values.")]
			#examples
			fn clone(&self) -> Self {
				match self {
					Output::Output(out, next, _) => Output::Output(out.clone(), next.clone(), core::marker::PhantomData),
				}
			}
		}

		#[document_type_parameters("The output value type.")]
		impl<Out> Functor for OutputBrand<Out>
		where
			Out: 'static,
		{
			/// Maps `f` over the next-program value of this output effect.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the effect.",
				"The original next-program type.",
				"The new next-program type after applying `f`."
			)]
			#[document_parameters("The function to apply to the next-program value.", "The output effect to map over.")]
			#[document_returns("A new output effect with the same output value and mapped next value.")]
			#examples
			fn map<'a, A: 'a, B: 'a>(
				f: impl Fn(A) -> B + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					Output::Output(out, next, _) => Output::Output(out, f(next), core::marker::PhantomData),
				}
			}
		}

		#[document_type_parameters("The output value type.")]
		impl<Out> SendFunctor for OutputBrand<Out>
		where
			Out: Send + Sync + 'static,
		{
			/// Maps `f` over the next-program value of this output effect in thread-safe contexts.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The lifetime of the effect.",
				"The original next-program type.",
				"The new next-program type after applying `f`."
			)]
			#[document_parameters("The function to apply to the next-program value.", "The output effect to map over.")]
			#[document_returns("A new output effect with the same output value and mapped next value.")]
			#examples
			fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
				f: impl Fn(A) -> B + Send + Sync + 'a,
				fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
			) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
				match fa {
					Output::Output(out, next, _) => Output::Output(out, f(next), core::marker::PhantomData),
				}
			}
		}
	}
}
