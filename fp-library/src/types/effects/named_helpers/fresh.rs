//! Named Fresh helpers layered over the Run wrapper primitives.
//!
//! The generic runner threads an owned generated-value counter. Each
//! Fresh operation receives the current value and the stored counter is
//! advanced with the caller-supplied successor function.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxFreshBrand,
				CNilBrand,
				FreshBrand,
				NodeBrand,
				RcBrand,
				SendFreshBrand,
			},
			classes::{
				Functor,
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
					fresh::{
						BoxFresh,
						Fresh,
						SendFresh,
					},
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::RcRunExplicit,
					run::Run,
					run_explicit::RunExplicit,
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
	};

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect Fresh;
			method run_fresh_with;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Fresh;
			method run_fresh;
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `RcRun` program to interpret.")]
	impl<R, A> RcRun<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		define_run_wrapper! {
			wrapper RcRun;
			effect Fresh;
			method run_fresh_with;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Fresh;
			method run_fresh;
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
			> + SendFunctor
			+ 'static,
		A: Clone + Send + Sync + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRun;
			effect Fresh;
			method run_fresh_with;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Fresh;
			method run_fresh;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
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
			effect Fresh;
			method run_fresh_with;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect Fresh;
			method run_fresh;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` program to interpret.")]
	impl<'a, R, A> RcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: Clone + 'a,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Fresh;
			method run_fresh_with;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Fresh;
			method run_fresh;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` program to interpret.")]
	impl<'a, R, A> ArcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, CNilBrand>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'a, ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		A: Clone + Send + Sync + 'a,
	{
		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Fresh;
			method run_fresh_with;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Fresh;
			method run_fresh;
		}
	}
}
