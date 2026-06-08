//! Named Writer helpers layered over the Run wrapper primitives.
//!
//! These helpers expose fold-style and monoidal Writer runners for the
//! existing first-order `tell` operation.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				NodeBrand,
				WriterBrand,
			},
			classes::{
				Functor,
				Monoid,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				Coyoneda,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
					arc_run_explicit::ArcRunExplicit,
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::RcRunExplicit,
					run::Run,
					run_explicit::RunExplicit,
					writer::Writer,
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
		std::{
			cell::RefCell,
			rc::Rc as StdRc,
			sync::{
				Arc as StdArc,
				Mutex,
			},
		},
	};

	/// Rc-backed deferred fold function used to restore Writer fold order.
	type SharedWriterFold<'a, Acc> = StdRc<dyn Fn(Acc) -> Acc + 'a>;

	/// Mutable Rc-backed deferred fold chain.
	type SharedWriterFoldCell<'a, Acc> = StdRc<RefCell<SharedWriterFold<'a, Acc>>>;

	/// Arc-backed deferred fold function used to restore Writer fold order.
	type SendSharedWriterFold<'a, Acc> = StdArc<dyn Fn(Acc) -> Acc + Send + Sync + 'a>;

	/// Mutable Arc-backed deferred fold chain.
	type SendSharedWriterFoldCell<'a, Acc> = StdArc<Mutex<SendSharedWriterFold<'a, Acc>>>;

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect Writer;
			method fold_writer;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Writer;
			method run_writer;
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `RcRun` program to interpret.")]
	impl<R, A> RcRun<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper RcRun;
			effect Writer;
			method fold_writer;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Writer;
			method run_writer;
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `ArcRun` program to interpret.")]
	impl<R, A> ArcRun<R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, CNilBrand>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Send + Sync + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRun;
			effect Writer;
			method fold_writer;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Writer;
			method run_writer;
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` program to interpret.")]
	impl<'a, R, A> RunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect Writer;
			method fold_writer;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect Writer;
			method run_writer;
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` program to interpret.")]
	impl<'a, R, A> RcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Writer;
			method fold_writer;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Writer;
			method run_writer;
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` program to interpret.")]
	impl<'a, R, A> ArcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'a,
	{
		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Writer;
			method fold_writer;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Writer;
			method run_writer;
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
