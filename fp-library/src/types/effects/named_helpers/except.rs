//! Named Except helpers layered over the Run wrapper primitives.
//!
//! These helpers expose Rust `Result` and `Option` shaped conveniences for the
//! existing `throw` and `handle_with` machinery.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				ExceptBrand,
				NodeBrand,
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
					except::Except,
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

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> Run<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect Except;
			method throw_unit;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Except;
			method rethrow;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Except;
			method note;
		}

		define_run_wrapper! {
			wrapper Run;
			effect Except;
			method from_option;
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect Except;
			method run_except;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> RunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect Except;
			method throw_unit;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect Except;
			method rethrow;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect Except;
			method note;
		}

		define_run_wrapper! {
			wrapper RunExplicit;
			effect Except;
			method from_option;
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
		A: 'static,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect Except;
			method run_except;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> RcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Except;
			method throw_unit;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Except;
			method rethrow;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Except;
			method note;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Except;
			method from_option;
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
		A: Clone + 'static,
	{
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Except;
			method run_except;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, ScopedRow, A> ArcRunExplicit<'a, R, ScopedRow, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Except;
			method throw_unit;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Except;
			method rethrow;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Except;
			method note;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Except;
			method from_option;
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
		A: Clone + Send + Sync + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Except;
			method run_except;
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> ArcRun<R, ScopedRow, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		ScopedRow: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, ScopedRow>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Clone + Send + Sync + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRun;
			effect Except;
			method throw_unit;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Except;
			method rethrow;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Except;
			method note;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Except;
			method from_option;
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
		A: Clone + Send + Sync + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRun;
			effect Except;
			method run_except;
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, ScopedRow, A> RcRun<R, ScopedRow, A>
	where
		R: WrapDrop + Functor + 'static,
		ScopedRow: WrapDrop + Functor + 'static,
		A: Clone + 'static,
	{
		define_run_wrapper! {
			wrapper RcRun;
			effect Except;
			method throw_unit;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Except;
			method rethrow;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Except;
			method note;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Except;
			method from_option;
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
			effect Except;
			method run_except;
		}
	}
}
