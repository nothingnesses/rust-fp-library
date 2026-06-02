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
		(WrapperName::Run, RunWrapperMethod::RunExcept) => run_run_except_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Throw) => rcrun_throw_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunExcept) => rcrun_run_except_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Throw) => arcrun_throw_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunExcept) => arcrun_run_except_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Throw) => run_explicit_throw_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunExcept) => run_explicit_run_except_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Throw) => rcrun_explicit_throw_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunExcept) =>
			rcrun_explicit_run_except_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Throw) => arcrun_explicit_throw_tokens(),
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
