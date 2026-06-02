use {
	super::{
		super::generator_descriptors::{
			self,
			EffectName,
			RunWrapperMethod,
			WrapperName,
		},
		impl_items_from_tokens,
	},
	proc_macro2::{
		Literal,
		TokenStream,
	},
	quote::quote,
	syn::ImplItem,
};

pub(super) fn typed_abort_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Except, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Throw) => run_throw_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Throw) => rcrun_throw_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Throw) => arcrun_throw_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Throw) => run_explicit_throw_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Throw) => rcrun_explicit_throw_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Throw) => arcrun_explicit_throw_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn doc_attrs(lines: Vec<String>) -> TokenStream {
	let mut tokens = TokenStream::new();
	for line in lines {
		let line = Literal::string(&line);
		tokens.extend(quote! {
			#[doc = #line]
		});
	}
	tokens
}

fn throw_examples(
	module: &str,
	wrapper: &str,
	coyoneda_brand: &str,
	explicit: bool,
) -> TokenStream {
	let mut lines = vec![
		"".to_string(),
		" ```".to_string(),
		" use fp_library::{".to_string(),
		" \tbrands::*,".to_string(),
		format!(" \ttypes::effects::{module}::{wrapper},"),
		" };".to_string(),
		"".to_string(),
		format!(
			" type FirstRow = CoproductBrand<{coyoneda_brand}<ExceptBrand<&'static str>>, CNilBrand>;"
		),
		" type Scoped = CNilBrand;".to_string(),
		"".to_string(),
	];

	if explicit {
		lines.extend([
			format!(" let prog: {wrapper}<'static, FirstRow, Scoped, i32> ="),
			format!(" \t{wrapper}::throw::<&'static str, _>(\"oops\");"),
			format!(
				" let handled: {wrapper}<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> ="
			),
		]);
	} else {
		lines.extend([
			format!(
				" let prog: {wrapper}<FirstRow, Scoped, i32> = {wrapper}::throw::<&'static str, _>(\"oops\");"
			),
			format!(" let handled: {wrapper}<CNilBrand, CNilBrand, Result<i32, &'static str>> ="),
		]);
	}

	lines.extend([
		" \tprog.run_except::<&'static str, _, CNilBrand>();".to_string(),
		" assert_eq!(handled.extract(), Err(\"oops\"));".to_string(),
		" ```".to_string(),
	]);

	doc_attrs(lines)
}

fn run_throw_tokens() -> TokenStream {
	let examples = throw_examples("run", "Run", "CoyonedaBrand", false);

	quote! {
		/// Lifts a `Throw` except effect into the Run program. Direct
		/// analog of PureScript Run's `throw`. The program raises an
		/// error of type `ErrorType` and never returns to the caller;
		/// the result type `A` is determined by the call-site (any
		/// `A` works because `Throw` doesn't produce one).
		///
		/// `ErrorType` is the error type carried by `ExceptBrand` in
		/// the row. Rust may need a turbofish on `ErrorType` because
		/// the value `e` may not constrain it from the call site
		/// alone.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The error value to throw.")]
		#[doc = ""]
		#[document_returns("A `Run` program suspended at the lifted `Throw` effect.")]
		#[doc = ""]
		#[document_examples]
		#examples
		#[inline]
		pub fn throw<ErrorType: 'static, Idx>(e: ErrorType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<'static, crate::brands::ExceptBrand<ErrorType>, A>,
					Idx,
				>, {
			let effect: crate::types::effects::except::Except<'static, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}
	}
}

fn rcrun_throw_tokens() -> TokenStream {
	let examples = throw_examples("rc_run", "RcRun", "RcCoyonedaBrand", false);

	quote! {
		/// Lifts a `Throw` except effect into the `RcRun` program.
		/// Mirrors [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The error value to throw.")]
		#[doc = ""]
		#[document_returns("An `RcRun` program suspended at the lifted `Throw` effect.")]
		#[doc = ""]
		#[document_examples]
		#examples
		#[inline]
		pub fn throw<ErrorType: Clone + 'static, Idx>(e: ErrorType) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::except::Except<'static, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}
	}
}

fn arcrun_throw_tokens() -> TokenStream {
	let examples = throw_examples("arc_run", "ArcRun", "ArcCoyonedaBrand", false);

	quote! {
		/// Lifts a `Throw` except effect into the `ArcRun` program.
		/// Mirrors [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: requires `ErrorType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade. The same
		/// [`ExceptBrand`](crate::brands::ExceptBrand) serves all six
		/// wrappers because [`Except`](crate::types::effects::except::Except)
		/// has no `dyn Fn` continuation; no parallel `SendExceptBrand`
		/// is needed.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The error value to throw.")]
		#[doc = ""]
		#[document_returns("An `ArcRun` program suspended at the lifted `Throw` effect.")]
		#[doc = ""]
		#[document_examples]
		#examples
		#[inline]
		pub fn throw<ErrorType: Clone + Send + Sync + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, crate::brands::ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::except::Except<'static, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}
	}
}

fn run_explicit_throw_tokens() -> TokenStream {
	let examples = throw_examples("run_explicit", "RunExplicit", "CoyonedaBrand", true);

	quote! {
		/// Lifts a `Throw` except effect into the `RunExplicit`
		/// program. Mirrors
		/// [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The error value to throw.")]
		#[doc = ""]
		#[document_returns("A `RunExplicit` program suspended at the lifted `Throw` effect.")]
		#[doc = ""]
		#[document_examples]
		#examples
		#[inline]
		pub fn throw<ErrorType: 'static, Idx>(e: ErrorType) -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_throw_tokens() -> TokenStream {
	let examples = throw_examples("rc_run_explicit", "RcRunExplicit", "RcCoyonedaBrand", true);

	quote! {
		/// Lifts a `Throw` except effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The error value to throw.")]
		#[doc = ""]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Throw` effect.")]
		#[doc = ""]
		#[document_examples]
		#examples
		#[inline]
		pub fn throw<ErrorType: Clone + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_throw_tokens() -> TokenStream {
	let examples = throw_examples("arc_run_explicit", "ArcRunExplicit", "ArcCoyonedaBrand", true);

	quote! {
		/// Lifts a `Throw` except effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`Run::throw`](crate::types::effects::run::Run::throw);
		/// see that method for cross-wrapper semantics. The same
		/// [`ExceptBrand`](crate::brands::ExceptBrand) serves all six
		/// wrappers because [`Except`](crate::types::effects::except::Except)
		/// has no `dyn Fn` continuation; no parallel `SendExceptBrand`
		/// is needed. Requires `ErrorType: Send + Sync` so the lifted
		/// layer participates in the Arc substrate's thread-safety
		/// cascade.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand` in the row.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[doc = ""]
		#[document_parameters("The error value to throw.")]
		#[doc = ""]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Throw` effect.")]
		#[doc = ""]
		#[document_examples]
		#examples
		#[inline]
		pub fn throw<ErrorType: Clone + Send + Sync + 'static, Idx>(e: ErrorType) -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::except::Except<'a, ErrorType, A> =
				crate::types::effects::except::Except::Throw(e, core::marker::PhantomData);
			Self::lift::<crate::brands::ExceptBrand<ErrorType>, Idx>(effect)
		}
	}
}
