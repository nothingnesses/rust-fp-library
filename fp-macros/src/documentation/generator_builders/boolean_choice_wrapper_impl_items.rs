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

pub(super) fn boolean_choice_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Choose, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::RcRun, RunWrapperMethod::Choose) => rcrun_choose_constructor_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Choose) => arcrun_choose_constructor_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Choose) =>
			rcrun_explicit_choose_constructor_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Choose) =>
			arcrun_explicit_choose_constructor_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn rcrun_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `RcRun` program.
		/// Direct analog of PureScript Run's `choose` /
		/// `runChoose`. The program nondeterministically returns
		/// `true` or `false`; the handler runs the continuation
		/// twice (once per branch) to capture both outcomes.
		///
		/// `Choose` ships only on the four multi-shot wrappers
		/// because the handler must clone the continuation to invoke
		/// it twice. Threads
		/// [`RcBrand`](crate::brands::RcBrand) as the pointer kind.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `RcRun` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRun<FirstRow, Scoped, i32> = RcRun::<FirstRow, Scoped, bool>::choose()
		/// 	.bind(|branch| RcRun::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> = prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, bool>):
				Member<RcCoyoneda<'static, crate::brands::ChooseBrand<RcBrand>, bool>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::choose::Choose<'static, RcBrand, bool> =
				crate::types::effects::choose::Choose::Alt(
					<RcBrand as crate::classes::ToDynCloneFn>::new(|b: bool| b),
				);
			Self::lift::<crate::brands::ChooseBrand<RcBrand>, Idx>(effect)
		}
	}
}

fn arcrun_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `ArcRun` program.
		/// Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics. Differences for
		/// `ArcRun`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendChooseBrand`](crate::brands::SendChooseBrand) (rather
		/// than `ChooseBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `ArcRun` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRun<FirstRow, Scoped, i32> = ArcRun::<FirstRow, Scoped, bool>::choose()
		/// 	.bind(|branch| ArcRun::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: ArcRun<CNilBrand, CNilBrand, Vec<i32>> = prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, bool>): Member<
					ArcCoyoneda<
						'static,
						crate::brands::SendChooseBrand<crate::brands::ArcBrand>,
						bool,
					>,
					Idx,
				>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::choose::SendChoose<
				'static,
				crate::brands::ArcBrand,
				bool,
			> = crate::types::effects::choose::SendChoose::Alt(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|b: bool| b),
			);
			Self::lift::<crate::brands::SendChooseBrand<crate::brands::ArcBrand>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `RcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: RcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	RcRunExplicit::<FirstRow, Scoped, bool>::choose().bind(|branch| {
		/// 		RcRunExplicit::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					RcCoyoneda<'a, crate::brands::ChooseBrand<crate::brands::RcBrand>, bool>,
					Idx,
				>, {
			let effect: crate::types::effects::choose::Choose<'a, crate::brands::RcBrand, bool> =
				crate::types::effects::choose::Choose::Alt(
					<crate::brands::RcBrand as crate::classes::ToDynCloneFn>::new(|b: bool| b),
				);
			Self::lift::<crate::brands::ChooseBrand<crate::brands::RcBrand>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_choose_constructor_tokens() -> TokenStream {
	quote! {
		/// Lifts an `Alt` choose effect into the `ArcRunExplicit`
		/// program. Mirrors
		/// [`RcRun::choose`](crate::types::effects::rc_run::RcRun::choose);
		/// see that method for cross-wrapper semantics. Differences
		/// for `ArcRunExplicit`: threads
		/// [`ArcBrand`](crate::brands::ArcBrand) as the pointer kind
		/// and uses
		/// [`SendChooseBrand`](crate::brands::SendChooseBrand) (rather
		/// than `ChooseBrand`) so the continuation projection is
		/// structurally `Send + Sync`.
		#[__document_module_generated]
		#[document_signature]
		#[doc = ""]
		#[document_type_parameters("The type-level Member-position witness (typically inferred).")]
		#[doc = ""]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted `Alt` effect.")]
		#[doc = ""]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type FirstRow = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		/// type Scoped = CNilBrand;
		///
		/// let prog: ArcRunExplicit<'static, FirstRow, Scoped, i32> =
		/// 	ArcRunExplicit::<FirstRow, Scoped, bool>::choose().bind(|branch| {
		/// 		ArcRunExplicit::<FirstRow, Scoped, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	prog.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn choose<Idx>() -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, bool>): Member<
					ArcCoyoneda<'a, crate::brands::SendChooseBrand<crate::brands::ArcBrand>, bool>,
					Idx,
				>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, bool>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::choose::SendChoose<
				'a,
				crate::brands::ArcBrand,
				bool,
			> = crate::types::effects::choose::SendChoose::Alt(
				<crate::brands::ArcBrand as crate::classes::ToDynSendFn>::new(|b: bool| b),
			);
			Self::lift::<crate::brands::SendChooseBrand<crate::brands::ArcBrand>, Idx>(effect)
		}
	}
}
