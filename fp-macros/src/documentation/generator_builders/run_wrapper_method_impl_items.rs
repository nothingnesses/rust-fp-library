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
		_ => Some(Ok(Vec::new())),
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
