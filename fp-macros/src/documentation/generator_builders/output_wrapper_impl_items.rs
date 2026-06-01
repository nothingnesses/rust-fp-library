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

pub(super) fn output_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(EffectName::Output, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, method) {
		(WrapperName::Run, RunWrapperMethod::Output) => run_output_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunOutputVec) => run_run_output_vec_tokens(),
		(WrapperName::Run, RunWrapperMethod::RunOutputMonoid) => run_run_output_monoid_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::Output) => rcrun_output_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunOutputVec) => rcrun_run_output_vec_tokens(),
		(WrapperName::RcRun, RunWrapperMethod::RunOutputMonoid) => rcrun_run_output_monoid_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::Output) => arcrun_output_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunOutputVec) => arcrun_run_output_vec_tokens(),
		(WrapperName::ArcRun, RunWrapperMethod::RunOutputMonoid) =>
			arcrun_run_output_monoid_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::Output) => run_explicit_output_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunOutputVec) =>
			run_explicit_run_output_vec_tokens(),
		(WrapperName::RunExplicit, RunWrapperMethod::RunOutputMonoid) =>
			run_explicit_run_output_monoid_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::Output) => rcrun_explicit_output_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunOutputVec) =>
			rcrun_explicit_run_output_vec_tokens(),
		(WrapperName::RcRunExplicit, RunWrapperMethod::RunOutputMonoid) =>
			rcrun_explicit_run_output_monoid_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::Output) => arcrun_explicit_output_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunOutputVec) =>
			arcrun_explicit_run_output_vec_tokens(),
		(WrapperName::ArcRunExplicit, RunWrapperMethod::RunOutputMonoid) =>
			arcrun_explicit_run_output_monoid_tokens(),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn run_output_tokens() -> TokenStream {
	quote! {
		/// Lifts an Output effect into the Run program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The output value to emit.")]
		#[document_returns("A `Run` program suspended at the lifted Output effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, ()> = Run::output::<String, _>("logged".to_string());
		/// let handled: Run<CNilBrand, CNilBrand, ((), Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), vec!["logged".to_string()]));
		/// ```
		#[inline]
		pub fn output<Out: 'static, Idx>(out: Out) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				crate::types::effects::member::Member<
					crate::types::Coyoneda<'static, crate::brands::OutputBrand<Out>, ()>,
					Idx,
				>, {
			let effect: crate::types::effects::output::Output<'static, Out, ()> =
				crate::types::effects::output::Output::Output(
					out,
					(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::OutputBrand<Out>, Idx>(effect)
		}
	}
}

fn rcrun_output_tokens() -> TokenStream {
	quote! {
		/// Lifts an Output effect into the `RcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The output value to emit.")]
		#[document_returns("An `RcRun` program suspended at the lifted Output effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, ()> = RcRun::output::<String, _>("logged".to_string());
		/// let handled: RcRun<CNilBrand, CNilBrand, ((), Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), vec!["logged".to_string()]));
		/// ```
		#[inline]
		pub fn output<Out: Clone + 'static, Idx>(out: Out) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<RcCoyoneda<'static, crate::brands::OutputBrand<Out>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::output::Output<'static, Out, ()> =
				crate::types::effects::output::Output::Output(
					out,
					(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::OutputBrand<Out>, Idx>(effect)
		}
	}
}

fn arcrun_output_tokens() -> TokenStream {
	quote! {
		/// Lifts an Output effect into the `ArcRun` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The output value to emit.")]
		#[document_returns("An `ArcRun` program suspended at the lifted Output effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, ()> = ArcRun::output::<String, _>("logged".to_string());
		/// let handled: ArcRun<CNilBrand, CNilBrand, ((), Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), vec!["logged".to_string()]));
		/// ```
		#[inline]
		pub fn output<Out: Clone + Send + Sync + 'static, Idx>(out: Out) -> Self
		where
			NodeBrand<R, ScopedRow>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
				Member<ArcCoyoneda<'static, crate::brands::OutputBrand<Out>, ()>, Idx>,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
			>): Clone, {
			let effect: crate::types::effects::output::Output<'static, Out, ()> =
				crate::types::effects::output::Output::Output(
					out,
					(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::OutputBrand<Out>, Idx>(effect)
		}
	}
}

fn run_explicit_output_tokens() -> TokenStream {
	quote! {
		/// Lifts an Output effect into the `RunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The output value to emit.")]
		#[document_returns("A `RunExplicit` program suspended at the lifted Output effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, ()> =
		/// 	RunExplicit::output::<String, _>("logged".to_string());
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, ((), Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), vec!["logged".to_string()]));
		/// ```
		#[inline]
		pub fn output<Out: 'static + 'a, Idx>(out: Out) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<Coyoneda<'a, crate::brands::OutputBrand<Out>, ()>, Idx>, {
			let effect: crate::types::effects::output::Output<'a, Out, ()> =
				crate::types::effects::output::Output::Output(
					out,
					(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::OutputBrand<Out>, Idx>(effect)
		}
	}
}

fn rcrun_explicit_output_tokens() -> TokenStream {
	quote! {
		/// Lifts an Output effect into the `RcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The output value to emit.")]
		#[document_returns("An `RcRunExplicit` program suspended at the lifted Output effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, ()> =
		/// 	RcRunExplicit::output::<String, _>("logged".to_string());
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, ((), Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), vec!["logged".to_string()]));
		/// ```
		#[inline]
		pub fn output<Out: Clone + 'static + 'a, Idx>(out: Out) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<RcCoyoneda<'a, crate::brands::OutputBrand<Out>, ()>, Idx>, {
			let effect: crate::types::effects::output::Output<'a, Out, ()> =
				crate::types::effects::output::Output::Output(
					out,
					(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::OutputBrand<Out>, Idx>(effect)
		}
	}
}

fn arcrun_explicit_output_tokens() -> TokenStream {
	quote! {
		/// Lifts an Output effect into the `ArcRunExplicit` program.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness (typically inferred)."
		)]
		#[document_parameters("The output value to emit.")]
		#[document_returns("An `ArcRunExplicit` program suspended at the lifted Output effect.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, ()> =
		/// 	ArcRunExplicit::output::<String, _>("logged".to_string());
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, ((), Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), ((), vec!["logged".to_string()]));
		/// ```
		#[inline]
		pub fn output<Out: Clone + Send + Sync + 'static + 'a, Idx>(out: Out) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
				Member<ArcCoyoneda<'a, crate::brands::OutputBrand<Out>, ()>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<ScopedRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, ScopedRow>, ()>,
			>): Clone + Send + Sync, {
			let effect: crate::types::effects::output::Output<'a, Out, ()> =
				crate::types::effects::output::Output::Output(
					out,
					(),
					core::marker::PhantomData,
				);
			Self::lift::<crate::brands::OutputBrand<Out>, Idx>(effect)
		}
	}
}

fn run_run_output_vec_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by collecting emitted values into a vector.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning `(result, outputs)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> =
		/// 	Run::<Row, CNilBrand, ()>::output::<String, _>("first".to_string())
		/// 		.bind(|()| Run::<Row, CNilBrand, ()>::output::<String, _>("second".to_string()))
		/// 		.bind(|()| Run::<Row, CNilBrand, i32>::pure(7));
		/// let handled: Run<CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn run_output_vec<Out, Idx, RMinusOutput>(
			self
		) -> Run<RMinusOutput, CNilBrand, (A, Vec<Out>)>
		where
			Out: Clone + 'static,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
				Coyoneda<'static, OutputBrand<Out>, Run<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, CNilBrand, A>,
					>
				),
			>, {
			self.run_output_monoid::<Out, Vec<Out>, Idx, RMinusOutput>(|out| vec![out])
		}
	}
}

fn run_run_output_monoid_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by mapping each emitted value into a monoidal accumulator.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The accumulated output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_parameters("The function that maps one emitted output into an accumulator chunk.")]
		#[document_returns("A first-order-only `Run` program returning `(result, accumulated_output)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> =
		/// 	Run::<Row, CNilBrand, ()>::output::<&'static str, _>("a")
		/// 		.bind(|()| Run::<Row, CNilBrand, ()>::output::<&'static str, _>("b"))
		/// 		.bind(|()| Run::<Row, CNilBrand, i32>::pure(7));
		/// let handled: Run<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
		/// assert_eq!(handled.extract(), (7, "ab".to_string()));
		/// ```
		#[inline]
		pub fn run_output_monoid<Out, Acc, Idx, RMinusOutput>(
			self,
			mapper: impl Fn(Out) -> Acc + 'static,
		) -> Run<RMinusOutput, CNilBrand, (A, Acc)>
		where
			Out: Clone + 'static,
			Acc: crate::classes::Monoid + Clone + 'static,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
				Coyoneda<'static, OutputBrand<Out>, Run<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						Run<R, CNilBrand, A>,
					>
				),
			>, {
			let mapper = std::rc::Rc::new(mapper);
			let accumulator: std::rc::Rc<std::cell::RefCell<std::rc::Rc<dyn Fn(Acc) -> Acc>>> =
				std::rc::Rc::new(std::cell::RefCell::new(std::rc::Rc::new(|acc| acc)));
			let handler_accumulator = std::rc::Rc::clone(&accumulator);
			let handled = self.handle_with::<OutputBrand<Out>, Idx, RMinusOutput>(
				move |op: Output<'static, Out, Run<RMinusOutput, CNilBrand, A>>| match op {
					Output::Output(out, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let mapper = std::rc::Rc::clone(&mapper);
						*handler_accumulator.borrow_mut() = std::rc::Rc::new(move |initial| {
							let after_current = <Acc as crate::classes::Semigroup>::append(
								initial,
								mapper(out.clone()),
							);
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(<Acc as crate::classes::Monoid>::empty()))
			})
		}
	}
}

fn rcrun_run_output_vec_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by collecting emitted values into a vector.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning `(result, outputs)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, ()>::output::<String, _>("first".to_string())
		/// 		.bind(|()| RcRun::<Row, CNilBrand, ()>::output::<String, _>("second".to_string()))
		/// 		.bind(|()| RcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn run_output_vec<Out, Idx, RMinusOutput>(
			self
		) -> RcRun<RMinusOutput, CNilBrand, (A, Vec<Out>)>
		where
			A: Clone,
			Out: Clone + 'static,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusOutput, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'static, OutputBrand<Out>, RcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<R, CNilBrand, A>,
					>
				),
			>, {
			self.run_output_monoid::<Out, Vec<Out>, Idx, RMinusOutput>(|out| vec![out])
		}
	}
}

fn rcrun_run_output_monoid_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by mapping each emitted value into a monoidal accumulator.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The accumulated output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_parameters("The function that maps one emitted output into an accumulator chunk.")]
		#[document_returns("A first-order-only `RcRun` program returning `(result, accumulated_output)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, ()>::output::<&'static str, _>("a")
		/// 		.bind(|()| RcRun::<Row, CNilBrand, ()>::output::<&'static str, _>("b"))
		/// 		.bind(|()| RcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
		/// assert_eq!(handled.extract(), (7, "ab".to_string()));
		/// ```
		#[inline]
		pub fn run_output_monoid<Out, Acc, Idx, RMinusOutput>(
			self,
			mapper: impl Fn(Out) -> Acc + 'static,
		) -> RcRun<RMinusOutput, CNilBrand, (A, Acc)>
		where
			A: Clone,
			Out: Clone + 'static,
			Acc: crate::classes::Monoid + Clone + 'static,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusOutput, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'static, OutputBrand<Out>, RcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						RcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let mapper = std::rc::Rc::new(mapper);
			let accumulator: std::rc::Rc<std::cell::RefCell<std::rc::Rc<dyn Fn(Acc) -> Acc>>> =
				std::rc::Rc::new(std::cell::RefCell::new(std::rc::Rc::new(|acc| acc)));
			let handler_accumulator = std::rc::Rc::clone(&accumulator);
			let handled = self.handle_with::<OutputBrand<Out>, Idx, RMinusOutput>(
				move |op: Output<'static, Out, RcRun<RMinusOutput, CNilBrand, A>>| match op {
					Output::Output(out, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let mapper = std::rc::Rc::clone(&mapper);
						*handler_accumulator.borrow_mut() = std::rc::Rc::new(move |initial| {
							let after_current = <Acc as crate::classes::Semigroup>::append(
								initial,
								mapper(out.clone()),
							);
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(<Acc as crate::classes::Monoid>::empty()))
			})
		}
	}
}

fn arcrun_run_output_vec_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by collecting emitted values into a vector.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, outputs)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, ()>::output::<String, _>("first".to_string())
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, ()>::output::<String, _>("second".to_string()))
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn run_output_vec<Out, Idx, RMinusOutput>(
			self
		) -> ArcRun<RMinusOutput, CNilBrand, (A, Vec<Out>)>
		where
			A: Clone + Send + Sync,
			Out: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusOutput, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusOutput, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusOutput, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'static, OutputBrand<Out>, ArcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcRun<R, CNilBrand, A>,
					>
				),
			>, {
			self.run_output_monoid::<Out, Vec<Out>, Idx, RMinusOutput>(|out| vec![out])
		}
	}
}

fn arcrun_run_output_monoid_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by mapping each emitted value into a monoidal accumulator.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The accumulated output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_parameters("The function that maps one emitted output into an accumulator chunk.")]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, accumulated_output)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, ()>::output::<&'static str, _>("a")
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, ()>::output::<&'static str, _>("b"))
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
		/// assert_eq!(handled.extract(), (7, "ab".to_string()));
		/// ```
		#[inline]
		pub fn run_output_monoid<Out, Acc, Idx, RMinusOutput>(
			self,
			mapper: impl Fn(Out) -> Acc + Send + Sync + 'static,
		) -> ArcRun<RMinusOutput, CNilBrand, (A, Acc)>
		where
			A: Clone + Send + Sync,
			Out: Clone + Send + Sync + 'static,
			Acc: crate::classes::Monoid + Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusOutput, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusOutput, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusOutput, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'static, OutputBrand<Out>, ArcRun<R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'static,
						ArcRun<R, CNilBrand, A>,
					>
				),
			>, {
			let mapper = std::sync::Arc::new(mapper);
			let accumulator: std::sync::Arc<
				std::sync::Mutex<std::sync::Arc<dyn Fn(Acc) -> Acc + Send + Sync>>,
			> = std::sync::Arc::new(std::sync::Mutex::new(std::sync::Arc::new(|acc| acc)));
			let handler_accumulator = std::sync::Arc::clone(&accumulator);
			let handled = self.handle_with::<OutputBrand<Out>, Idx, RMinusOutput>(
				move |op: Output<'static, Out, ArcRun<RMinusOutput, CNilBrand, A>>| match op {
					Output::Output(out, next, _) => {
						let previous = {
							let guard = match handler_accumulator.lock() {
								Ok(guard) => guard,
								Err(poisoned) => poisoned.into_inner(),
							};
							guard.clone()
						};
						let mapper = std::sync::Arc::clone(&mapper);
						let mut guard = match handler_accumulator.lock() {
							Ok(guard) => guard,
							Err(poisoned) => poisoned.into_inner(),
						};
						*guard = std::sync::Arc::new(move |initial| {
							let after_current = <Acc as crate::classes::Semigroup>::append(
								initial,
								mapper(out.clone()),
							);
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = {
					let guard = match accumulator.lock() {
						Ok(guard) => guard,
						Err(poisoned) => poisoned.into_inner(),
					};
					guard.clone()
				};
				(result, finish(<Acc as crate::classes::Monoid>::empty()))
			})
		}
	}
}

fn run_explicit_run_output_vec_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by collecting emitted values into a vector.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_returns("A first-order-only `RunExplicit` program returning `(result, outputs)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::output::<String, _>("first".to_string())
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, ()>::output::<String, _>("second".to_string()))
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn run_output_vec<Out, Idx, RMinusOutput>(
			self
		) -> RunExplicit<'a, RMinusOutput, CNilBrand, (A, Vec<Out>)>
		where
			Out: Clone + 'static + 'a,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				Coyoneda<'a, OutputBrand<Out>, RunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			self.run_output_monoid::<Out, Vec<Out>, Idx, RMinusOutput>(|out| vec![out])
		}
	}
}

fn run_explicit_run_output_monoid_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by mapping each emitted value into a monoidal accumulator.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The accumulated output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_parameters("The function that maps one emitted output into an accumulator chunk.")]
		#[document_returns("A first-order-only `RunExplicit` program returning `(result, accumulated_output)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::output::<&'static str, _>("a")
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, ()>::output::<&'static str, _>("b"))
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
		/// assert_eq!(handled.extract(), (7, "ab".to_string()));
		/// ```
		#[inline]
		pub fn run_output_monoid<Out, Acc, Idx, RMinusOutput>(
			self,
			mapper: impl Fn(Out) -> Acc + 'a,
		) -> RunExplicit<'a, RMinusOutput, CNilBrand, (A, Acc)>
		where
			Out: Clone + 'static + 'a,
			Acc: crate::classes::Monoid + Clone + 'a,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				Coyoneda<'a, OutputBrand<Out>, RunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let mapper = std::rc::Rc::new(mapper);
			let accumulator: std::rc::Rc<
				std::cell::RefCell<std::rc::Rc<dyn Fn(Acc) -> Acc + 'a>>,
			> = std::rc::Rc::new(std::cell::RefCell::new(std::rc::Rc::new(|acc| acc)));
			let handler_accumulator = std::rc::Rc::clone(&accumulator);
			let handled = self.handle_with::<OutputBrand<Out>, Idx, RMinusOutput>(
				move |op: Output<'a, Out, RunExplicit<'a, RMinusOutput, CNilBrand, A>>| match op {
					Output::Output(out, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let mapper = std::rc::Rc::clone(&mapper);
						*handler_accumulator.borrow_mut() = std::rc::Rc::new(move |initial| {
							let after_current = <Acc as crate::classes::Semigroup>::append(
								initial,
								mapper(out.clone()),
							);
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(<Acc as crate::classes::Monoid>::empty()))
			})
		}
	}
}

fn rcrun_explicit_run_output_vec_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by collecting emitted values into a vector.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_returns("A first-order-only `RcRunExplicit` program returning `(result, outputs)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::output::<String, _>("first".to_string())
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, ()>::output::<String, _>("second".to_string()))
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn run_output_vec<Out, Idx, RMinusOutput>(
			self
		) -> RcRunExplicit<'a, RMinusOutput, CNilBrand, (A, Vec<Out>)>
		where
			A: Clone,
			Out: Clone + 'static + 'a,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Vec<Out>)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'a, OutputBrand<Out>, RcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			self.run_output_monoid::<Out, Vec<Out>, Idx, RMinusOutput>(|out| vec![out])
		}
	}
}

fn rcrun_explicit_run_output_monoid_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by mapping each emitted value into a monoidal accumulator.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The accumulated output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_parameters("The function that maps one emitted output into an accumulator chunk.")]
		#[document_returns("A first-order-only `RcRunExplicit` program returning `(result, accumulated_output)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::output::<&'static str, _>("a")
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, ()>::output::<&'static str, _>("b"))
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
		/// assert_eq!(handled.extract(), (7, "ab".to_string()));
		/// ```
		#[inline]
		pub fn run_output_monoid<Out, Acc, Idx, RMinusOutput>(
			self,
			mapper: impl Fn(Out) -> Acc + 'a,
		) -> RcRunExplicit<'a, RMinusOutput, CNilBrand, (A, Acc)>
		where
			A: Clone,
			Out: Clone + 'static + 'a,
			Acc: crate::classes::Monoid + Clone + 'a,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Acc)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				RcCoyoneda<'a, OutputBrand<Out>, RcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let mapper = std::rc::Rc::new(mapper);
			let accumulator: std::rc::Rc<
				std::cell::RefCell<std::rc::Rc<dyn Fn(Acc) -> Acc + 'a>>,
			> = std::rc::Rc::new(std::cell::RefCell::new(std::rc::Rc::new(|acc| acc)));
			let handler_accumulator = std::rc::Rc::clone(&accumulator);
			let handled = self.handle_with::<OutputBrand<Out>, Idx, RMinusOutput>(
				move |op: Output<'a, Out, RcRunExplicit<'a, RMinusOutput, CNilBrand, A>>| {
					match op {
						Output::Output(out, next, _) => {
							let previous = handler_accumulator.borrow().clone();
							let mapper = std::rc::Rc::clone(&mapper);
							*handler_accumulator.borrow_mut() = std::rc::Rc::new(move |initial| {
								let after_current = <Acc as crate::classes::Semigroup>::append(
									initial,
									mapper(out.clone()),
								);
								previous(after_current)
							});
							next
						}
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(<Acc as crate::classes::Monoid>::empty()))
			})
		}
	}
}

fn arcrun_explicit_run_output_vec_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by collecting emitted values into a vector.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_returns("A first-order-only `ArcRunExplicit` program returning `(result, outputs)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<OutputBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, ()>::output::<String, _>("first".to_string())
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, ()>::output::<String, _>("second".to_string()))
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.run_output_vec::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn run_output_vec<Out, Idx, RMinusOutput>(
			self
		) -> ArcRunExplicit<'a, RMinusOutput, CNilBrand, (A, Vec<Out>)>
		where
			A: Clone + Send + Sync,
			Out: Clone + Send + Sync + 'static + 'a,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusOutput, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Vec<Out>)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Vec<Out>)>,
			>): Send + Sync,
			Apply!(<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusOutput, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Vec<Out>)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusOutput, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'a, OutputBrand<Out>, ArcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			self.run_output_monoid::<Out, Vec<Out>, Idx, RMinusOutput>(|out| vec![out])
		}
	}
}

fn arcrun_explicit_run_output_monoid_tokens() -> TokenStream {
	quote! {
		/// Interprets Output by mapping each emitted value into a monoidal accumulator.
		#[__document_module_generated]
		#[document_signature]
		#[document_type_parameters(
			"The emitted output type.",
			"The accumulated output type.",
			"The type-level Member-position witness for the Output effect.",
			"The first-order row brand with the Output effect removed."
		)]
		#[document_parameters("The function that maps one emitted output into an accumulator chunk.")]
		#[document_returns("A first-order-only `ArcRunExplicit` program returning `(result, accumulated_output)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<OutputBrand<&'static str>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, ()>::output::<&'static str, _>("a")
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, ()>::output::<&'static str, _>("b"))
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_output_monoid::<&'static str, String, _, CNilBrand>(str::to_string);
		/// assert_eq!(handled.extract(), (7, "ab".to_string()));
		/// ```
		#[inline]
		pub fn run_output_monoid<Out, Acc, Idx, RMinusOutput>(
			self,
			mapper: impl Fn(Out) -> Acc + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, RMinusOutput, CNilBrand, (A, Acc)>
		where
			A: Clone + Send + Sync,
			Out: Clone + Send + Sync + 'static + 'a,
			Acc: crate::classes::Monoid + Clone + Send + Sync + 'a,
			RMinusOutput: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusOutput, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusOutput, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Acc)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Acc)>,
			>): Send + Sync,
			Apply!(<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusOutput, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusOutput, CNilBrand>, (A, Acc)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusOutput, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
				ArcCoyoneda<'a, OutputBrand<Out>, ArcRunExplicit<'a, R, CNilBrand, A>>,
				Idx,
				Remainder = Apply!(
					<RMinusOutput as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>
				),
			>, {
			let mapper = std::sync::Arc::new(mapper);
			let accumulator: std::sync::Arc<
				std::sync::Mutex<std::sync::Arc<dyn Fn(Acc) -> Acc + Send + Sync + 'a>>,
			> = std::sync::Arc::new(std::sync::Mutex::new(std::sync::Arc::new(|acc| acc)));
			let handler_accumulator = std::sync::Arc::clone(&accumulator);
			let handled = self.handle_with::<OutputBrand<Out>, Idx, RMinusOutput>(
				move |op: Output<'a, Out, ArcRunExplicit<'a, RMinusOutput, CNilBrand, A>>| {
					match op {
						Output::Output(out, next, _) => {
							let previous = {
								let guard = match handler_accumulator.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.clone()
							};
							let mapper = std::sync::Arc::clone(&mapper);
							let mut guard = match handler_accumulator.lock() {
								Ok(guard) => guard,
								Err(poisoned) => poisoned.into_inner(),
							};
							*guard = std::sync::Arc::new(move |initial| {
								let after_current = <Acc as crate::classes::Semigroup>::append(
									initial,
									mapper(out.clone()),
								);
								previous(after_current)
							});
							next
						}
					}
				},
			);
			handled.map(move |result| {
				let finish = {
					let guard = match accumulator.lock() {
						Ok(guard) => guard,
						Err(poisoned) => poisoned.into_inner(),
					};
					guard.clone()
				};
				(result, finish(<Acc as crate::classes::Monoid>::empty()))
			})
		}
	}
}
