//! Descriptor-backed builders for wrapper-wide Run methods.

use {
	super::impl_items_from_tokens,
	crate::documentation::generator_descriptors::{
		RunWrapperCoreMethod,
		WrapperName,
	},
	proc_macro2::TokenStream,
	quote::quote,
	syn::ImplItem,
};

pub(super) fn run_wrapper_method_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperCoreMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	match (wrapper, method) {
		(WrapperName::Run, RunWrapperCoreMethod::Expand) =>
			Some(impl_items_from_tokens(run_expand_tokens())),
		(WrapperName::RcRun, RunWrapperCoreMethod::Expand) =>
			Some(impl_items_from_tokens(rcrun_expand_tokens())),
		(WrapperName::ArcRun, RunWrapperCoreMethod::Expand) =>
			Some(impl_items_from_tokens(arcrun_expand_tokens())),
		(WrapperName::RunExplicit, RunWrapperCoreMethod::Expand) =>
			Some(impl_items_from_tokens(run_explicit_expand_tokens())),
		(WrapperName::RcRunExplicit, RunWrapperCoreMethod::Expand) =>
			Some(impl_items_from_tokens(rcrun_explicit_expand_tokens())),
		(WrapperName::ArcRunExplicit, RunWrapperCoreMethod::Expand) =>
			Some(impl_items_from_tokens(arcrun_explicit_expand_tokens())),
		(WrapperName::Run, RunWrapperCoreMethod::Weaken) =>
			Some(impl_items_from_tokens(run_weaken_tokens())),
		(WrapperName::RcRun, RunWrapperCoreMethod::Weaken) =>
			Some(impl_items_from_tokens(rcrun_weaken_tokens())),
		(WrapperName::ArcRun, RunWrapperCoreMethod::Weaken) =>
			Some(impl_items_from_tokens(arcrun_weaken_tokens())),
		(WrapperName::RunExplicit, RunWrapperCoreMethod::Weaken) =>
			Some(impl_items_from_tokens(run_explicit_weaken_tokens())),
		(WrapperName::RcRunExplicit, RunWrapperCoreMethod::Weaken) =>
			Some(impl_items_from_tokens(rcrun_explicit_weaken_tokens())),
		(WrapperName::ArcRunExplicit, RunWrapperCoreMethod::Weaken) =>
			Some(impl_items_from_tokens(arcrun_explicit_weaken_tokens())),
	}
}

fn run_expand_tokens() -> TokenStream {
	quote! {
		/// Widens both effect rows of a `Run` program.
		///
		/// `expand` is the general row-subsumption operation: it
		/// structurally rewrites the program from rows `R` and
		/// `ScopedRow` into compatible wider rows `R2` and `S2`.
		/// Unlike PureScript Run's representation cast, this operation
		/// walks the Rust row representation because `Coproduct` row
		/// shapes can have different layouts.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The target first-order row brand.",
			"The target scoped row brand.",
			"The first-order row embedding witness.",
			"The scoped row embedding witness."
		)]
		#[document_returns("A `Run` program with the same behavior over the target rows.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: Run<SmallRow, CNilBrand, i32> = Run::pure(42);
		/// let widened: Run<WideRow, CNilBrand, i32> = run.expand();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn expand<R2, S2, REmbedIdx, SEmbedIdx>(self) -> Run<R2, S2, A>
		where
			R2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			S2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			SEmbedIdx: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<
					crate::brands::NodeBrand<R2, S2>,
					crate::types::free::TypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::Free<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::free::TypeErasedValue,
						>,
					>),
					REmbedIdx,
				>,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<
					crate::brands::NodeBrand<R2, S2>,
					crate::types::free::TypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::Free<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::free::TypeErasedValue,
						>,
					>),
					SEmbedIdx,
				>, {
			Run(self.0.expand::<R2, S2, REmbedIdx, SEmbedIdx>())
		}
	}
}

fn rcrun_expand_tokens() -> TokenStream {
	quote! {
		/// Widens both effect rows of an `RcRun` program.
		///
		/// `expand` structurally rewrites the program from rows `R`
		/// and `ScopedRow` into compatible wider rows `R2` and `S2`.
		/// The traversal preserves the shared continuation queue in the
		/// underlying `RcFree` substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The target first-order row brand.",
			"The target scoped row brand.",
			"The first-order row embedding witness.",
			"The scoped row embedding witness."
		)]
		#[document_returns("An `RcRun` program with the same behavior over the target rows.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: RcRun<SmallRow, CNilBrand, i32> = RcRun::pure(42);
		/// let widened: RcRun<WideRow, CNilBrand, i32> = run.expand();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn expand<R2, S2, REmbedIdx, SEmbedIdx>(self) -> RcRun<R2, S2, A>
		where
			R2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			S2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			SEmbedIdx: 'static,
			Apply!(<crate::brands::NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::RcFree<
					crate::brands::NodeBrand<R, ScopedRow>,
					crate::types::rc_free::RcTypeErasedValue,
				>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::RcFree<
					crate::brands::NodeBrand<R2, S2>,
					crate::types::rc_free::RcTypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::RcFree<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::rc_free::RcTypeErasedValue,
						>,
					>),
					REmbedIdx,
				>,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::RcFree<
					crate::brands::NodeBrand<R2, S2>,
					crate::types::rc_free::RcTypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::RcFree<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::rc_free::RcTypeErasedValue,
						>,
					>),
					SEmbedIdx,
				>, {
			RcRun::from_rc_free(
				crate::types::effects::row_embed::embed_rc_free_node::<
					R,
					ScopedRow,
					R2,
					S2,
					A,
					REmbedIdx,
					SEmbedIdx,
				>(self.into_rc_free()),
			)
		}
	}
}

fn arcrun_expand_tokens() -> TokenStream {
	quote! {
		/// Widens both effect rows of an `ArcRun` program.
		///
		/// `expand` structurally rewrites the program from rows `R`
		/// and `ScopedRow` into compatible wider rows `R2` and `S2`.
		/// The traversal preserves the shared thread-safe continuation
		/// queue in the underlying `ArcFree` substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The target first-order row brand.",
			"The target scoped row brand.",
			"The first-order row embedding witness.",
			"The scoped row embedding witness."
		)]
		#[document_returns("An `ArcRun` program with the same behavior over the target rows.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: ArcRun<SmallRow, CNilBrand, i32> = ArcRun::pure(42);
		/// let widened: ArcRun<WideRow, CNilBrand, i32> = run.expand();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn expand<R2, S2, REmbedIdx, SEmbedIdx>(self) -> ArcRun<R2, S2, A>
		where
			crate::brands::NodeBrand<R, ScopedRow>: crate::classes::WrapDrop
				+ crate::kinds::Kind_cdc7cd43dac7585f<
					Of<
						'static,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R, ScopedRow>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					> = crate::types::effects::node::Node<
						'static,
						R,
						ScopedRow,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R, ScopedRow>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					>,
				> + 'static,
			crate::brands::NodeBrand<R2, S2>: crate::classes::WrapDrop
				+ crate::kinds::Kind_cdc7cd43dac7585f<
					Of<
						'static,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					> = crate::types::effects::node::Node<
						'static,
						R2,
						S2,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					>,
				> + 'static,
			R: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			ScopedRow: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			R2: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			S2: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			REmbedIdx: 'static,
			SEmbedIdx: 'static,
			crate::types::effects::node::Node<
				'static,
				R,
				ScopedRow,
				crate::types::ArcFree<
					crate::brands::NodeBrand<R, ScopedRow>,
					crate::types::arc_free::ArcTypeErasedValue,
				>,
			>: Clone + Send + Sync,
			crate::types::effects::node::Node<
				'static,
				R2,
				S2,
				crate::types::ArcFree<
					crate::brands::NodeBrand<R2, S2>,
					crate::types::arc_free::ArcTypeErasedValue,
				>,
			>: Send + Sync,
			crate::types::ArcFree<
				crate::brands::NodeBrand<R, ScopedRow>,
				crate::types::arc_free::ArcTypeErasedValue,
			>: Send + Sync,
			crate::types::ArcFree<
				crate::brands::NodeBrand<R2, S2>,
				crate::types::arc_free::ArcTypeErasedValue,
			>: Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::ArcFree<
					crate::brands::NodeBrand<R2, S2>,
					crate::types::arc_free::ArcTypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					>),
					REmbedIdx,
				> + Send
				+ Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::ArcFree<
					crate::brands::NodeBrand<R2, S2>,
					crate::types::arc_free::ArcTypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R2, S2>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					>),
					SEmbedIdx,
				> + Send
				+ Sync, {
			ArcRun::from_arc_free(
				crate::types::effects::row_embed::embed_arc_free_node::<
					R,
					ScopedRow,
					R2,
					S2,
					A,
					REmbedIdx,
					SEmbedIdx,
				>(self.into_arc_free()),
			)
		}
	}
}

fn run_explicit_expand_tokens() -> TokenStream {
	quote! {
		/// Widens both effect rows of a `RunExplicit` program.
		///
		/// `expand` structurally rewrites the program from rows `R`
		/// and `ScopedRow` into compatible wider rows `R2` and `S2`,
		/// preserving the explicit single-shot substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The target first-order row brand.",
			"The target scoped row brand.",
			"The first-order row embedding witness.",
			"The scoped row embedding witness."
		)]
		#[document_returns("A `RunExplicit` program with the same behavior over the target rows.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: RunExplicit<'static, SmallRow, CNilBrand, i32> = RunExplicit::pure(42);
		/// let widened: RunExplicit<'static, WideRow, CNilBrand, i32> = run.expand();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn expand<R2, S2, REmbedIdx, SEmbedIdx>(self) -> RunExplicit<'a, R2, S2, A>
		where
			R2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			S2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			SEmbedIdx: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Box<crate::types::FreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						Box<crate::types::FreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>>,
					>),
					REmbedIdx,
				>,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Box<crate::types::FreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						Box<crate::types::FreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>>,
					>),
					SEmbedIdx,
				>, {
			RunExplicit::from_free_explicit(
				crate::types::effects::row_embed::embed_free_explicit_node::<
					'a,
					R,
					ScopedRow,
					R2,
					S2,
					A,
					REmbedIdx,
					SEmbedIdx,
				>(self.into_free_explicit()),
			)
		}
	}
}

fn rcrun_explicit_expand_tokens() -> TokenStream {
	quote! {
		/// Widens both effect rows of an `RcRunExplicit` program.
		///
		/// `expand` structurally rewrites the program from rows `R`
		/// and `ScopedRow` into compatible wider rows `R2` and `S2`,
		/// preserving the shared explicit substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The target first-order row brand.",
			"The target scoped row brand.",
			"The first-order row embedding witness.",
			"The scoped row embedding witness."
		)]
		#[document_returns("An `RcRunExplicit` program with the same behavior over the target rows.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: RcRunExplicit<'static, SmallRow, CNilBrand, i32> = RcRunExplicit::pure(42);
		/// let widened: RcRunExplicit<'static, WideRow, CNilBrand, i32> = run.expand();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn expand<R2, S2, REmbedIdx, SEmbedIdx>(self) -> RcRunExplicit<'a, R2, S2, A>
		where
			A: Clone,
			R2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			S2: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			SEmbedIdx: 'static,
			Apply!(<crate::brands::NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::RcFreeExplicit<'a, crate::brands::NodeBrand<R, ScopedRow>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::RcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						crate::types::RcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
					>),
					REmbedIdx,
				>,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::RcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						crate::types::RcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
					>),
					SEmbedIdx,
				>, {
			RcRunExplicit::from_rc_free_explicit(
				crate::types::effects::row_embed::embed_rc_free_explicit_node::<
					'a,
					R,
					ScopedRow,
					R2,
					S2,
					A,
					REmbedIdx,
					SEmbedIdx,
				>(self.into_rc_free_explicit()),
			)
		}
	}
}

fn arcrun_explicit_expand_tokens() -> TokenStream {
	quote! {
		/// Widens both effect rows of an `ArcRunExplicit` program.
		///
		/// `expand` structurally rewrites the program from rows `R`
		/// and `ScopedRow` into compatible wider rows `R2` and `S2`,
		/// preserving the shared thread-safe explicit substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The target first-order row brand.",
			"The target scoped row brand.",
			"The first-order row embedding witness.",
			"The scoped row embedding witness."
		)]
		#[document_returns("An `ArcRunExplicit` program with the same behavior over the target rows.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: ArcRunExplicit<'static, SmallRow, CNilBrand, i32> = ArcRunExplicit::pure(42);
		/// let widened: ArcRunExplicit<'static, WideRow, CNilBrand, i32> = run.expand();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn expand<R2, S2, REmbedIdx, SEmbedIdx>(self) -> ArcRunExplicit<'a, R2, S2, A>
		where
			A: Clone + Send + Sync,
			R2: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			S2: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			REmbedIdx: 'static,
			SEmbedIdx: 'static,
			crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R, ScopedRow>, A>:
				Send + Sync,
			crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>:
				Send + Sync,
			Apply!(<crate::brands::NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R, ScopedRow>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<R2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
					>),
					REmbedIdx,
				> + Send
				+ Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<S2 as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R2, S2>, A>,
					>),
					SEmbedIdx,
				> + Send
				+ Sync, {
			ArcRunExplicit::from_arc_free_explicit(
				crate::types::effects::row_embed::embed_arc_free_explicit_node::<
					'a,
					R,
					ScopedRow,
					R2,
					S2,
					A,
					REmbedIdx,
					SEmbedIdx,
				>(self.into_arc_free_explicit()),
			)
		}
	}
}

fn run_weaken_tokens() -> TokenStream {
	quote! {
		/// Prepends one unused first-order effect row cell.
		///
		/// `weaken` is a Heftia / OpenUnion-inspired convenience for
		/// widening only the first-order row. It leaves the scoped row
		/// unchanged and is equivalent to adding an unused effect at the
		/// head of the first-order row.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The first-order effect brand to add at the head of the row.",
			"The first-order row embedding witness."
		)]
		#[document_returns("A `Run` program with the same behavior and one additional first-order row cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: Run<SmallRow, CNilBrand, i32> = Run::pure(42);
		/// let widened: Run<WideRow, CNilBrand, i32> = run.weaken();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn weaken<E, REmbedIdx>(self) -> Run<crate::brands::CoproductBrand<E, R>, ScopedRow, A>
		where
			E: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::Free<
					crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
					crate::types::free::TypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<crate::brands::CoproductBrand<E, R> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::Free<
							crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
							crate::types::free::TypeErasedValue,
						>,
					>),
					REmbedIdx,
				>, {
			Run(self.0.weaken::<E, REmbedIdx>())
		}
	}
}

fn rcrun_weaken_tokens() -> TokenStream {
	quote! {
		/// Prepends one unused first-order effect row cell.
		///
		/// `weaken` widens only the first-order row of an `RcRun`
		/// program. It leaves the scoped row unchanged and preserves
		/// the shared continuation queue in the underlying `RcFree`
		/// substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The first-order effect brand to add at the head of the row.",
			"The first-order row embedding witness."
		)]
		#[document_returns("An `RcRun` program with the same behavior and one additional first-order row cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: RcRun<SmallRow, CNilBrand, i32> = RcRun::pure(42);
		/// let widened: RcRun<WideRow, CNilBrand, i32> = run.weaken();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn weaken<E, REmbedIdx>(
			self
		) -> RcRun<crate::brands::CoproductBrand<E, R>, ScopedRow, A>
		where
			E: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			Apply!(<crate::brands::NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::RcFree<
					crate::brands::NodeBrand<R, ScopedRow>,
					crate::types::rc_free::RcTypeErasedValue,
				>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::RcFree<
					crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
					crate::types::rc_free::RcTypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<crate::brands::CoproductBrand<E, R> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::RcFree<
							crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
							crate::types::rc_free::RcTypeErasedValue,
						>,
					>),
					REmbedIdx,
				>, {
			RcRun::from_rc_free(
				crate::types::effects::row_embed::embed_rc_free_node_first_order::<
					R,
					ScopedRow,
					crate::brands::CoproductBrand<E, R>,
					A,
					REmbedIdx,
				>(self.into_rc_free()),
			)
		}
	}
}

fn arcrun_weaken_tokens() -> TokenStream {
	quote! {
		/// Prepends one unused first-order effect row cell.
		///
		/// `weaken` widens only the first-order row of an `ArcRun`
		/// program. It leaves the scoped row unchanged and preserves
		/// the shared thread-safe continuation queue in the underlying
		/// `ArcFree` substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The first-order effect brand to add at the head of the row.",
			"The first-order row embedding witness."
		)]
		#[document_returns("An `ArcRun` program with the same behavior and one additional first-order row cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: ArcRun<SmallRow, CNilBrand, i32> = ArcRun::pure(42);
		/// let widened: ArcRun<WideRow, CNilBrand, i32> = run.weaken();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn weaken<E, REmbedIdx>(
			self
		) -> ArcRun<crate::brands::CoproductBrand<E, R>, ScopedRow, A>
		where
			crate::brands::NodeBrand<R, ScopedRow>: crate::classes::WrapDrop
				+ crate::kinds::Kind_cdc7cd43dac7585f<
					Of<
						'static,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R, ScopedRow>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					> = crate::types::effects::node::Node<
						'static,
						R,
						ScopedRow,
						crate::types::ArcFree<
							crate::brands::NodeBrand<R, ScopedRow>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					>,
				> + 'static,
			crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>:
				crate::classes::WrapDrop
					+ crate::kinds::Kind_cdc7cd43dac7585f<
						Of<
							'static,
							crate::types::ArcFree<
								crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
								crate::types::arc_free::ArcTypeErasedValue,
							>,
						> = crate::types::effects::node::Node<
							'static,
							crate::brands::CoproductBrand<E, R>,
							ScopedRow,
							crate::types::ArcFree<
								crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
								crate::types::arc_free::ArcTypeErasedValue,
							>,
						>,
					> + 'static,
			R: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			ScopedRow: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			E: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			REmbedIdx: 'static,
			crate::types::effects::node::Node<
				'static,
				R,
				ScopedRow,
				crate::types::ArcFree<
					crate::brands::NodeBrand<R, ScopedRow>,
					crate::types::arc_free::ArcTypeErasedValue,
				>,
			>: Clone + Send + Sync,
			crate::types::effects::node::Node<
				'static,
				crate::brands::CoproductBrand<E, R>,
				ScopedRow,
				crate::types::ArcFree<
					crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
					crate::types::arc_free::ArcTypeErasedValue,
				>,
			>: Send + Sync,
			crate::types::ArcFree<
				crate::brands::NodeBrand<R, ScopedRow>,
				crate::types::arc_free::ArcTypeErasedValue,
			>: Send + Sync,
			crate::types::ArcFree<
				crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
				crate::types::arc_free::ArcTypeErasedValue,
			>: Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				crate::types::ArcFree<
					crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
					crate::types::arc_free::ArcTypeErasedValue,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<crate::brands::CoproductBrand<E, R> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						crate::types::ArcFree<
							crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
							crate::types::arc_free::ArcTypeErasedValue,
						>,
					>),
					REmbedIdx,
				> + Send
				+ Sync, {
			ArcRun::from_arc_free(
				crate::types::effects::row_embed::embed_arc_free_node_first_order::<
					R,
					ScopedRow,
					crate::brands::CoproductBrand<E, R>,
					A,
					REmbedIdx,
				>(self.into_arc_free()),
			)
		}
	}
}

fn run_explicit_weaken_tokens() -> TokenStream {
	quote! {
		/// Prepends one unused first-order effect row cell.
		///
		/// `weaken` widens only the first-order row of a `RunExplicit`
		/// program. It leaves the scoped row unchanged and preserves
		/// the explicit single-shot substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The first-order effect brand to add at the head of the row.",
			"The first-order row embedding witness."
		)]
		#[document_returns("A `RunExplicit` program with the same behavior and one additional first-order row cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: RunExplicit<'static, SmallRow, CNilBrand, i32> = RunExplicit::pure(42);
		/// let widened: RunExplicit<'static, WideRow, CNilBrand, i32> = run.weaken();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn weaken<E, REmbedIdx>(
			self
		) -> RunExplicit<'a, crate::brands::CoproductBrand<E, R>, ScopedRow, A>
		where
			E: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				Box<
					crate::types::FreeExplicit<
						'a,
						crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
						A,
					>,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<crate::brands::CoproductBrand<E, R> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						Box<
							crate::types::FreeExplicit<
								'a,
								crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
								A,
							>,
						>,
					>),
					REmbedIdx,
				>, {
			RunExplicit::from_free_explicit(
				crate::types::effects::row_embed::embed_free_explicit_node_first_order::<
					'a,
					R,
					ScopedRow,
					crate::brands::CoproductBrand<E, R>,
					A,
					REmbedIdx,
				>(self.into_free_explicit()),
			)
		}
	}
}

fn rcrun_explicit_weaken_tokens() -> TokenStream {
	quote! {
		/// Prepends one unused first-order effect row cell.
		///
		/// `weaken` widens only the first-order row of an
		/// `RcRunExplicit` program. It leaves the scoped row unchanged
		/// and preserves the shared explicit substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The first-order effect brand to add at the head of the row.",
			"The first-order row embedding witness."
		)]
		#[document_returns("An `RcRunExplicit` program with the same behavior and one additional first-order row cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<RcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: RcRunExplicit<'static, SmallRow, CNilBrand, i32> = RcRunExplicit::pure(42);
		/// let widened: RcRunExplicit<'static, WideRow, CNilBrand, i32> = run.weaken();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn weaken<E, REmbedIdx>(
			self
		) -> RcRunExplicit<'a, crate::brands::CoproductBrand<E, R>, ScopedRow, A>
		where
			A: Clone,
			E: crate::classes::WrapDrop + crate::classes::Functor + 'static,
			REmbedIdx: 'static,
			Apply!(<crate::brands::NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::RcFreeExplicit<'a, crate::brands::NodeBrand<R, ScopedRow>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::RcFreeExplicit<
					'a,
					crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
					A,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<crate::brands::CoproductBrand<E, R> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						crate::types::RcFreeExplicit<
							'a,
							crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
							A,
						>,
					>),
					REmbedIdx,
				>, {
			RcRunExplicit::from_rc_free_explicit(
				crate::types::effects::row_embed::embed_rc_free_explicit_node_first_order::<
					'a,
					R,
					ScopedRow,
					crate::brands::CoproductBrand<E, R>,
					A,
					REmbedIdx,
				>(self.into_rc_free_explicit()),
			)
		}
	}
}

fn arcrun_explicit_weaken_tokens() -> TokenStream {
	quote! {
		/// Prepends one unused first-order effect row cell.
		///
		/// `weaken` widens only the first-order row of an
		/// `ArcRunExplicit` program. It leaves the scoped row unchanged
		/// and preserves the shared thread-safe explicit substrate.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The first-order effect brand to add at the head of the row.",
			"The first-order row embedding witness."
		)]
		#[document_returns("An `ArcRunExplicit` program with the same behavior and one additional first-order row cell.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type SmallRow = CNilBrand;
		/// type WideRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, SmallRow>;
		///
		/// let run: ArcRunExplicit<'static, SmallRow, CNilBrand, i32> = ArcRunExplicit::pure(42);
		/// let widened: ArcRunExplicit<'static, WideRow, CNilBrand, i32> = run.weaken();
		/// assert!(matches!(widened.peel(), Ok(42)));
		/// ```
		#[inline]
		pub fn weaken<E, REmbedIdx>(
			self
		) -> ArcRunExplicit<'a, crate::brands::CoproductBrand<E, R>, ScopedRow, A>
		where
			A: Clone + Send + Sync,
			E: crate::classes::WrapDrop + crate::classes::SendFunctor + 'static,
			REmbedIdx: 'static,
			crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R, ScopedRow>, A>:
				Send + Sync,
			crate::types::ArcFreeExplicit<
				'a,
				crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
				A,
			>: Send + Sync,
			Apply!(<crate::brands::NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::ArcFreeExplicit<'a, crate::brands::NodeBrand<R, ScopedRow>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				crate::types::ArcFreeExplicit<
					'a,
					crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
					A,
				>,
			>): crate::types::effects::coproduct::CoproductEmbedder<
					Apply!(<crate::brands::CoproductBrand<E, R> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						crate::types::ArcFreeExplicit<
							'a,
							crate::brands::NodeBrand<crate::brands::CoproductBrand<E, R>, ScopedRow>,
							A,
						>,
					>),
					REmbedIdx,
				> + Send
				+ Sync, {
			ArcRunExplicit::from_arc_free_explicit(
				crate::types::effects::row_embed::embed_arc_free_explicit_node_first_order::<
					'a,
					R,
					ScopedRow,
					crate::brands::CoproductBrand<E, R>,
					A,
					REmbedIdx,
				>(self.into_arc_free_explicit()),
			)
		}
	}
}
