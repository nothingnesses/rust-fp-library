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
	proc_macro2::TokenStream,
	quote::quote,
	syn::ImplItem,
};

pub(super) fn phantom_abort_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Empty, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Empty) => run_empty_constructor_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Empty) => rcrun_empty_constructor_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Empty) => arcrun_empty_constructor_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Empty) =>
			run_explicit_empty_constructor_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Empty) =>
			rcrun_explicit_empty_constructor_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Empty) =>
			arcrun_explicit_empty_constructor_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the Run program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `Run` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: Run<FirstRow, Scoped, i32> = Run::empty();
		/// let handled: Run<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				crate::types::effects::member::Member<
						crate::types::Coyoneda<'static, crate::brands::EmptyBrand, A>,
						Idx,
					>, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn rcrun_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `RcRun` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRun` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::empty();
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<RcCoyoneda<'static, crate::brands::EmptyBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn arcrun_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `ArcRun` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// thread-safe multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRun` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::empty();
		/// let handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> = prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Send + Sync,
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<ArcCoyoneda<'static, crate::brands::EmptyBrand, A>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::empty::Empty<'static, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn run_explicit_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `RunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning a fallback value in a
		/// single-shot interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("A `RunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RunExplicit<'static, FirstRow, Scoped, i32> = RunExplicit::empty();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<Coyoneda<'a, crate::brands::EmptyBrand, A>, Idx>, {
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn rcrun_explicit_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `RcRunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> = RcRunExplicit::empty();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<RcCoyoneda<'a, crate::brands::EmptyBrand, A>, Idx>, {
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}

fn arcrun_explicit_empty_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Empty` effect into the `ArcRunExplicit` program.
		///
		/// `Empty` aborts the current branch without producing the
		/// result type `A`. A handler decides how that absence is
		/// represented, such as returning an empty collection in a
		/// thread-safe multi-shot nondeterministic interpreter.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Empty` effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> = ArcRunExplicit::empty();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	prog.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn empty<Idx>() -> Self
		where
			A: Clone + Send + Sync + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>):
				Member<ArcCoyoneda<'a, crate::brands::EmptyBrand, A>, Idx>,
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
			let effect: crate::types::effects::empty::Empty<'a, A> =
				crate::types::effects::empty::Empty::Empty(core::marker::PhantomData);
			Self::lift::<crate::brands::EmptyBrand, Idx>(effect)
		}
	}
}
