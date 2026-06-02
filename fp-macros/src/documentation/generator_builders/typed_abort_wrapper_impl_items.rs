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
		(WrapperName::Run, RunWrapperMethod::ThrowUnit) => run_throw_unit_tokens(),
		(WrapperName::Run, RunWrapperMethod::Rethrow) => run_rethrow_tokens(),
		(WrapperName::Run, RunWrapperMethod::Note) => run_note_tokens(),
		(WrapperName::Run, RunWrapperMethod::FromOption) => run_from_option_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunExcept) => run_run_except_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Throw) => rcrun_throw_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::ThrowUnit) => rcrun_throw_unit_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Rethrow) => rcrun_rethrow_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Note) => rcrun_note_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::FromOption) => rcrun_from_option_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunExcept) => rcrun_run_except_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Throw) => arcrun_throw_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::ThrowUnit) => arcrun_throw_unit_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Rethrow) => arcrun_rethrow_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Note) => arcrun_note_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::FromOption) => arcrun_from_option_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunExcept) => arcrun_run_except_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Throw) => run_explicit_throw_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::ThrowUnit) => run_explicit_throw_unit_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Rethrow) => run_explicit_rethrow_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Note) => run_explicit_note_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::FromOption) =>
			run_explicit_from_option_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunExcept) => run_explicit_run_except_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Throw) => rcrun_explicit_throw_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::ThrowUnit) =>
			rcrun_explicit_throw_unit_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Rethrow) => rcrun_explicit_rethrow_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Note) => rcrun_explicit_note_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::FromOption) =>
			rcrun_explicit_from_option_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunExcept) =>
			rcrun_explicit_run_except_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Throw) => arcrun_explicit_throw_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::ThrowUnit) =>
			arcrun_explicit_throw_unit_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Rethrow) =>
			arcrun_explicit_rethrow_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Note) => arcrun_explicit_note_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::FromOption) =>
			arcrun_explicit_from_option_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunExcept) =>
			arcrun_explicit_run_except_tokens(),
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

fn run_throw_unit_tokens() -> TokenStream {
	quote! {
		/// Throws unit in an Except row.
		///
		/// `throw_unit()` is the unit-error variant of [`Run::throw`]. It mirrors
		/// PureScript Run's
		/// [`fail`](https://github.com/natefaubion/purescript-run/blob/abec7c343e92154d44b9dafd52b91ee82d32a870/src/Run/Except.purs)
		/// helper while using Rust's unit type as the error payload.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("A `Run` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::throw_unit();
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn throw_unit<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<()>, A>, Idx>, {
			Self::throw::<(), Idx>(())
		}
	}
}

fn run_rethrow_tokens() -> TokenStream {
	quote! {
		/// Converts a Rust `Result` into an Except program.
		///
		/// `Ok(value)` becomes a pure program. `Err(error)` becomes
		/// a thrown `ExceptBrand<ErrorType>` effect.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters("The result to convert.")]
		#[document_returns("A pure program for `Ok`, or a thrown Except program for `Err`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn run_note_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program with a supplied error.
		///
		/// `Some(value)` becomes a pure program. `None` throws the supplied
		/// error. This is Rust's `Option`-shaped analogue of PureScript Run's
		/// [`note`](https://github.com/natefaubion/purescript-run/blob/abec7c343e92154d44b9dafd52b91ee82d32a870/src/Run/Except.purs)
		/// helper.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters(
			"The error to throw when `value` is `None`.",
			"The option to convert."
		)]
		#[document_returns("A pure program for `Some`, or a thrown Except program for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::note::<&'static str, _>("missing", None);
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn run_from_option_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program that throws unit for `None`.
		///
		/// This is the Rust-named equivalent of PureScript Run's
		/// [`fromJust`](https://github.com/natefaubion/purescript-run/blob/abec7c343e92154d44b9dafd52b91ee82d32a870/src/Run/Except.purs)
		/// helper. The name uses `Option` because that is the Rust type being
		/// converted.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::from_option(None);
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, ExceptBrand<()>, A>, Idx>, {
			Self::note::<(), Idx>((), value)
		}
	}
}

fn run_explicit_throw_unit_tokens() -> TokenStream {
	quote! {
		/// Throws unit in an Except row.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("A `RunExplicit` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::throw_unit();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn throw_unit<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<()>, A>, Idx>, {
			Self::throw::<(), Idx>(())
		}
	}
}

fn run_explicit_rethrow_tokens() -> TokenStream {
	quote! {
		/// Converts a Rust `Result` into an Except program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters("The result to convert.")]
		#[document_returns("A pure program for `Ok`, or a thrown Except program for `Err`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn run_explicit_note_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program with a supplied error.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters(
			"The error to throw when `value` is `None`.",
			"The option to convert."
		)]
		#[document_returns("A pure program for `Some`, or a thrown Except program for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::note::<&'static str, _>("missing", None);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn run_explicit_from_option_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::from_option(None);
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, ExceptBrand<()>, A>, Idx>, {
			Self::note::<(), Idx>((), value)
		}
	}
}

fn rcrun_throw_unit_tokens() -> TokenStream {
	quote! {
		/// Throws unit in an Except row.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `RcRun` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::throw_unit();
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn throw_unit<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			Self::throw::<(), Idx>(())
		}
	}
}

fn rcrun_rethrow_tokens() -> TokenStream {
	quote! {
		/// Converts a Rust `Result` into an Except program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters("The result to convert.")]
		#[document_returns("A pure program for `Ok`, or a thrown Except program for `Err`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn rcrun_note_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program with a supplied error.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters(
			"The error to throw when `value` is `None`.",
			"The option to convert."
		)]
		#[document_returns("A pure program for `Some`, or a thrown Except program for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::note::<&'static str, _>("missing", None);
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn rcrun_from_option_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::from_option(None);
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, RcTypeErasedValue>,
			>): Clone, {
			Self::note::<(), Idx>((), value)
		}
	}
}

fn arcrun_throw_unit_tokens() -> TokenStream {
	quote! {
		/// Throws unit in an Except row.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `ArcRun` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::throw_unit();
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn throw_unit<Idx>() -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			Self::throw::<(), Idx>(())
		}
	}
}

fn arcrun_rethrow_tokens() -> TokenStream {
	quote! {
		/// Converts a Rust `Result` into an Except program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters("The result to convert.")]
		#[document_returns("A pure program for `Ok`, or a thrown Except program for `Err`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + Send + Sync + 'static, Idx>(
			result: Result<A, ErrorType>
		) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn arcrun_note_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program with a supplied error.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters(
			"The error to throw when `value` is `None`.",
			"The option to convert."
		)]
		#[document_returns("A pure program for `Some`, or a thrown Except program for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::note::<&'static str, _>("missing", None);
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + Send + Sync + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn arcrun_from_option_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::from_option(None);
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			Self::note::<(), Idx>((), value)
		}
	}
}

fn rcrun_explicit_throw_unit_tokens() -> TokenStream {
	quote! {
		/// Throws unit in an Except row.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `RcRunExplicit` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> = RcRunExplicit::throw_unit();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn throw_unit<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			Self::throw::<(), Idx>(())
		}
	}
}

fn rcrun_explicit_rethrow_tokens() -> TokenStream {
	quote! {
		/// Converts a Rust `Result` into an Except program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters("The result to convert.")]
		#[document_returns("A pure program for `Ok`, or a thrown Except program for `Err`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + 'static, Idx>(result: Result<A, ErrorType>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn rcrun_explicit_note_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program with a supplied error.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters(
			"The error to throw when `value` is `None`.",
			"The option to convert."
		)]
		#[document_returns("A pure program for `Some`, or a thrown Except program for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::note::<&'static str, _>("missing", None);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn rcrun_explicit_from_option_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> = RcRunExplicit::from_option(None);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, ScopedRow>, A>,
			>): Clone, {
			Self::note::<(), Idx>((), value)
		}
	}
}

fn arcrun_explicit_throw_unit_tokens() -> TokenStream {
	quote! {
		/// Throws unit in an Except row.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_returns("An `ArcRunExplicit` program suspended at `ExceptBrand<()>`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> = ArcRunExplicit::throw_unit();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn throw_unit<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
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
			Self::throw::<(), Idx>(())
		}
	}
}

fn arcrun_explicit_rethrow_tokens() -> TokenStream {
	quote! {
		/// Converts a Rust `Result` into an Except program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters("The result to convert.")]
		#[document_returns("A pure program for `Ok`, or a thrown Except program for `Err`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::rethrow::<&'static str, _>(Err("missing"));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn rethrow<ErrorType: Clone + Send + Sync + 'static, Idx>(
			result: Result<A, ErrorType>
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
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
			match result {
				Ok(value) => Self::pure(value),
				Err(error) => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn arcrun_explicit_note_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program with a supplied error.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect."
		)]
		#[document_parameters(
			"The error to throw when `value` is `None`.",
			"The option to convert."
		)]
		#[document_returns("A pure program for `Some`, or a thrown Except program for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::note::<&'static str, _>("missing", None);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn note<ErrorType: Clone + Send + Sync + 'static, Idx>(
			error: ErrorType,
			value: Option<A>,
		) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<ErrorType>, A>, Idx>,
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
			match value {
				Some(value) => Self::pure(value),
				None => Self::throw::<ErrorType, Idx>(error),
			}
		}
	}
}

fn arcrun_explicit_from_option_tokens() -> TokenStream {
	quote! {
		/// Converts an `Option` into an Except program that throws unit for `None`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness for the Except effect.")]
		#[document_parameters("The option to convert.")]
		#[document_returns("A pure program for `Some`, or a thrown `ExceptBrand<()>` for `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<()>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> = ArcRunExplicit::from_option(None);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, ()>> =
		/// 	program.run_except::<(), _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err(()));
		/// ```
		#[inline]
		pub fn from_option<Idx>(value: Option<A>) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, ExceptBrand<()>, A>, Idx>,
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
			Self::note::<(), Idx>((), value)
		}
	}
}

fn run_run_except_tokens() -> TokenStream {
	quote! {
		/// Interprets one Except effect into a Rust `Result`.
		///
		/// A pure result becomes `Ok(result)`. A thrown error becomes
		/// `Err(error)`. Other first-order effects remain in the narrowed
		/// row and continue to be represented by the returned `Run` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `Run` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::throw::<&'static str, _>("missing");
		/// let handled: Run<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> Run<RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					Coyoneda<
						'static,
						ExceptBrand<ErrorType>,
						Run<R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'static,
					ErrorType,
					Run<RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => Run::pure(Err(error)),
					}
				},
			)
		}
	}
}

fn rcrun_run_except_tokens() -> TokenStream {
	quote! {
		/// Interprets one Except effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::throw::<&'static str, _>("missing");
		/// let handled: RcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> RcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusExcept, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					RcCoyoneda<
						'static,
						ExceptBrand<ErrorType>,
						RcRun<R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'static,
					ErrorType,
					RcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => RcRun::pure(Err(error)),
					}
				},
			)
		}
	}
}

fn arcrun_run_except_tokens() -> TokenStream {
	quote! {
		/// Interprets one Except effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::throw::<&'static str, _>("missing");
		/// let handled: ArcRun<CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> ArcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusExcept, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<
						'static,
						ArcFree<NodeBrand<RMinusExcept, CNilBrand>, ArcTypeErasedValue>,
					>: Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusExcept, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					ArcCoyoneda<
						'static,
						ExceptBrand<ErrorType>,
						ArcRun<R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'static,
					ErrorType,
					ArcRun<RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => ArcRun::pure(Err(error)),
					}
				},
			)
		}
	}
}

fn run_explicit_run_except_tokens() -> TokenStream {
	quote! {
		/// Interprets one Except effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::throw::<&'static str, _>("missing");
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> RunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					Coyoneda<
						'a,
						ExceptBrand<ErrorType>,
						RunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'a,
					ErrorType,
					RunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => RunExplicit::pure(Err(error)),
					}
				},
			)
		}
	}
}

fn rcrun_explicit_run_except_tokens() -> TokenStream {
	quote! {
		/// Interprets one Except effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::throw::<&'static str, _>("missing");
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> RcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Clone,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					RcCoyoneda<
						'a,
						ExceptBrand<ErrorType>,
						RcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'a,
					ErrorType,
					RcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => RcRunExplicit::pure(Err(error)),
					}
				},
			)
		}
	}
}

fn arcrun_explicit_run_except_tokens() -> TokenStream {
	quote! {
		/// Interprets one Except effect into a Rust `Result`.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The error type carried by `ExceptBrand`.",
			"The type-level Member-position witness for the Except effect.",
			"The first-order row brand with the Except effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `Ok(result)` or `Err(error)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<ExceptBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::throw::<&'static str, _>("missing");
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Result<i32, &'static str>> =
		/// 	program.run_except::<&'static str, _, CNilBrand>();
		/// assert_eq!(handled.extract(), Err("missing"));
		/// ```
		#[inline]
		pub fn run_except<ErrorType, Idx, RMinusExcept>(
			self
		) -> ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>
		where
			ErrorType: Clone + Send + Sync + 'static,
			RMinusExcept: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusExcept, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusExcept, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusExcept, CNilBrand>, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
			>): Member<
					ArcCoyoneda<
						'a,
						ExceptBrand<ErrorType>,
						ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusExcept as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, Result<A, ErrorType>>,
									>
								),
				>, {
			self.map(Ok).handle_with::<ExceptBrand<ErrorType>, Idx, RMinusExcept>(
				|op: Except<
					'a,
					ErrorType,
					ArcRunExplicit<'a, RMinusExcept, CNilBrand, Result<A, ErrorType>>,
				>| {
					match op {
						Except::Throw(error, _) => ArcRunExplicit::pure(Err(error)),
					}
				},
			)
		}
	}
}
