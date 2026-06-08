//! Named nondeterminism helpers layered over the Run wrapper primitives.
//!
//! These helpers expose `Empty` as `Option` and `Choose` as `Vec`-backed
//! branch collection.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				CNilBrand,
				ChooseBrand,
				EmptyBrand,
				NodeBrand,
				RcBrand,
				SendChooseBrand,
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
					arc_run::{
						ArcRun,
						make_node_first,
						unwrap_node,
						wrap_first_arc,
					},
					arc_run_explicit::ArcRunExplicit,
					choose::{
						Choose,
						SendChoose,
					},
					empty::Empty,
					member::Member,
					node::Node,
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

	/// Normalizes an `ArcRunExplicit` first-order `Node` projection.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The projected `NodeBrand` value.")]
	#[document_returns("The normalized `Node` value.")]
	#[document_examples(
		skip_call_check,
		reason = "This private helper exists only to move a GAT projection normalization step out of the ArcRunExplicit NonDet helper body."
	)]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	types::effects::arc_run_explicit::ArcRunExplicit,
	/// };
	///
	/// type Program = ArcRunExplicit<'static, CNilBrand, CNilBrand, i32>;
	/// let _shape: Option<Program> = None;
	/// assert!(_shape.is_none());
	/// ```
	fn unwrap_arc_run_explicit_nondet_node<'a, R, A>(
		node: Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, CNilBrand, A>,
		>)
	) -> Node<'a, R, CNilBrand, ArcRunExplicit<'a, R, CNilBrand, A>>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: 'a, {
		node
	}

	/// Builds an `ArcRunExplicit` first-order `Node` projection.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The inner program type carried by the layer."
	)]
	#[document_parameters("The first-order layer payload.")]
	#[document_returns("The projected `NodeBrand` value.")]
	#[document_examples(
		skip_call_check,
		reason = "This private helper exists only to move a GAT projection construction step out of the ArcRunExplicit NonDet helper body."
	)]
	///
	/// ```
	/// use fp_library::brands::*;
	///
	/// type Row = CNilBrand;
	/// let _row: Option<Row> = None;
	/// assert!(_row.is_none());
	/// ```
	fn make_arc_run_explicit_nondet_node_first<'a, R, A>(
		layer: Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>)
	) -> Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>)
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		A: 'a, {
		Node::First(layer)
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
			effect Empty;
			method run_empty;
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
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Choose;
			method run_choose;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Choose;
			method run_nondet;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Choose;
			method run_first_success;
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
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Choose;
			method run_choose;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Choose;
			method run_nondet;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Choose;
			method run_first_success;
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
			effect Empty;
			method run_empty;
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
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Choose;
			method run_choose;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Choose;
			method run_nondet;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Choose;
			method run_first_success;
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
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Choose;
			method run_choose;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Choose;
			method run_nondet;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Choose;
			method run_first_success;
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
