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

macro_rules! run_constructor_tokens {
	($method:ident, $run_vec:ident, $module:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = constructor_examples(
			"run",
			"Run",
			stringify!($method),
			stringify!($run_vec),
			stringify!($brand),
			"CoyonedaBrand",
		);
		quote! {
			/// Lifts a direct-payload effect into the `Run` program.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness (typically inferred)."
			)]
			#[document_parameters("The payload value to emit.")]
			#[document_returns("A `Run` program suspended at the lifted direct-payload effect.")]
			#examples
			#[inline]
			pub fn $method<Out: 'static, Idx>(out: Out) -> Self
			where
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
					crate::types::effects::member::Member<
						crate::types::Coyoneda<'static, crate::brands::$brand<Out>, ()>,
						Idx,
					>, {
				let effect: crate::types::effects::$module::$cell<'static, Out, ()> =
					crate::types::effects::$module::$cell::$variant(
						out,
						(),
						core::marker::PhantomData,
					);
				Self::lift::<crate::brands::$brand<Out>, Idx>(effect)
			}
		}
	}};
}

macro_rules! rcrun_constructor_tokens {
	($method:ident, $run_vec:ident, $module:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = constructor_examples(
			"rc_run",
			"RcRun",
			stringify!($method),
			stringify!($run_vec),
			stringify!($brand),
			"RcCoyonedaBrand",
		);
		quote! {
			/// Lifts a direct-payload effect into the `RcRun` program.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness (typically inferred)."
			)]
			#[document_parameters("The payload value to emit.")]
			#[document_returns("An `RcRun` program suspended at the lifted direct-payload effect.")]
			#examples
			#[inline]
			pub fn $method<Out: Clone + 'static, Idx>(out: Out) -> Self
			where
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
					Member<RcCoyoneda<'static, crate::brands::$brand<Out>, ()>, Idx>,
				Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, ScopedRow>, crate::types::rc_free::RcTypeErasedValue>,
				>): Clone, {
				let effect: crate::types::effects::$module::$cell<'static, Out, ()> =
					crate::types::effects::$module::$cell::$variant(
						out,
						(),
						core::marker::PhantomData,
					);
				Self::lift::<crate::brands::$brand<Out>, Idx>(effect)
			}
		}
	}};
}

macro_rules! arcrun_constructor_tokens {
	($method:ident, $run_vec:ident, $module:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = constructor_examples(
			"arc_run",
			"ArcRun",
			stringify!($method),
			stringify!($run_vec),
			stringify!($brand),
			"ArcCoyonedaBrand",
		);
		quote! {
			/// Lifts a direct-payload effect into the `ArcRun` program.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness (typically inferred)."
			)]
			#[document_parameters("The payload value to emit.")]
			#[document_returns("An `ArcRun` program suspended at the lifted direct-payload effect.")]
			#examples
			#[inline]
			pub fn $method<Out: Clone + Send + Sync + 'static, Idx>(out: Out) -> Self
			where
				NodeBrand<R, ScopedRow>: SendFunctor,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ()>):
					Member<ArcCoyoneda<'static, crate::brands::$brand<Out>, ()>, Idx>,
				Apply!(<NodeBrand<R, ScopedRow> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, ScopedRow>, ArcTypeErasedValue>,
				>): Clone, {
				let effect: crate::types::effects::$module::$cell<'static, Out, ()> =
					crate::types::effects::$module::$cell::$variant(
						out,
						(),
						core::marker::PhantomData,
					);
				Self::lift::<crate::brands::$brand<Out>, Idx>(effect)
			}
		}
	}};
}

macro_rules! run_explicit_constructor_tokens {
	($method:ident, $run_vec:ident, $module:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = constructor_examples(
			"run_explicit",
			"RunExplicit",
			stringify!($method),
			stringify!($run_vec),
			stringify!($brand),
			"CoyonedaBrand",
		);
		quote! {
			/// Lifts a direct-payload effect into the `RunExplicit` program.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness (typically inferred)."
			)]
			#[document_parameters("The payload value to emit.")]
			#[document_returns("A `RunExplicit` program suspended at the lifted direct-payload effect.")]
			#examples
			#[inline]
			pub fn $method<Out: 'static + 'a, Idx>(out: Out) -> Self
			where
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
					Member<Coyoneda<'a, crate::brands::$brand<Out>, ()>, Idx>, {
				let effect: crate::types::effects::$module::$cell<'a, Out, ()> =
					crate::types::effects::$module::$cell::$variant(
						out,
						(),
						core::marker::PhantomData,
					);
				Self::lift::<crate::brands::$brand<Out>, Idx>(effect)
			}
		}
	}};
}

macro_rules! rcrun_explicit_constructor_tokens {
	($method:ident, $run_vec:ident, $module:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = constructor_examples(
			"rc_run_explicit",
			"RcRunExplicit",
			stringify!($method),
			stringify!($run_vec),
			stringify!($brand),
			"RcCoyonedaBrand",
		);
		quote! {
			/// Lifts a direct-payload effect into the `RcRunExplicit` program.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness (typically inferred)."
			)]
			#[document_parameters("The payload value to emit.")]
			#[document_returns(
				"An `RcRunExplicit` program suspended at the lifted direct-payload effect."
			)]
			#examples
			#[inline]
			pub fn $method<Out: Clone + 'static + 'a, Idx>(out: Out) -> Self
			where
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
					Member<RcCoyoneda<'a, crate::brands::$brand<Out>, ()>, Idx>, {
				let effect: crate::types::effects::$module::$cell<'a, Out, ()> =
					crate::types::effects::$module::$cell::$variant(
						out,
						(),
						core::marker::PhantomData,
					);
				Self::lift::<crate::brands::$brand<Out>, Idx>(effect)
			}
		}
	}};
}

macro_rules! arcrun_explicit_constructor_tokens {
	($method:ident, $run_vec:ident, $module:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = constructor_examples(
			"arc_run_explicit",
			"ArcRunExplicit",
			stringify!($method),
			stringify!($run_vec),
			stringify!($brand),
			"ArcCoyonedaBrand",
		);
		quote! {
			/// Lifts a direct-payload effect into the `ArcRunExplicit` program.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness (typically inferred)."
			)]
			#[document_parameters("The payload value to emit.")]
			#[document_returns(
				"An `ArcRunExplicit` program suspended at the lifted direct-payload effect."
			)]
			#examples
			#[inline]
			pub fn $method<Out: Clone + Send + Sync + 'static + 'a, Idx>(out: Out) -> Self
			where
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, ()>):
					Member<ArcCoyoneda<'a, crate::brands::$brand<Out>, ()>, Idx>,
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
				let effect: crate::types::effects::$module::$cell<'a, Out, ()> =
					crate::types::effects::$module::$cell::$variant(
						out,
						(),
						core::marker::PhantomData,
					);
				Self::lift::<crate::brands::$brand<Out>, Idx>(effect)
			}
		}
	}};
}

macro_rules! run_vec_tokens {
	($method:ident, $run_monoid:ident, $constructor:ident, $brand:ident) => {{
		let examples = vec_runner_examples(
			"run",
			"Run",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"CoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by collecting payloads into a vector.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_returns("A first-order-only `Run` program returning `(result, payloads)`.")]
			#examples
			#[inline]
			pub fn $method<Out, Idx, RMinusEffect>(
				self
			) -> Run<RMinusEffect, CNilBrand, (A, Vec<Out>)>
			where
				Out: Clone + 'static,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Run<R, CNilBrand, A>,
				>): Member<
					Coyoneda<'static, $brand<Out>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							Run<R, CNilBrand, A>,
						>
					),
				>, {
				self.$run_monoid::<Out, Vec<Out>, Idx, RMinusEffect>(|out| vec![out])
			}
		}
	}};
}

macro_rules! run_monoid_tokens {
	($method:ident, $constructor:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = monoid_runner_examples(
			"run",
			"Run",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"CoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by mapping each payload into a monoidal accumulator.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The accumulated payload type.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_parameters("The function that maps one payload into an accumulator chunk.")]
			#[document_returns(
				"A first-order-only `Run` program returning `(result, accumulated_payload)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Acc, Idx, RMinusEffect>(
				self,
				mapper: impl Fn(Out) -> Acc + 'static,
			) -> Run<RMinusEffect, CNilBrand, (A, Acc)>
			where
				Out: Clone + 'static,
				Acc: crate::classes::Monoid + Clone + 'static,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					Run<R, CNilBrand, A>,
				>): Member<
					Coyoneda<'static, $brand<Out>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							Run<R, CNilBrand, A>,
						>
					),
				>, {
				let mapper = std::rc::Rc::new(mapper);
				let accumulator:
					std::rc::Rc<std::cell::RefCell<std::rc::Rc<dyn Fn(Acc) -> Acc>>> =
					std::rc::Rc::new(std::cell::RefCell::new(std::rc::Rc::new(|acc| acc)));
				let handler_accumulator = std::rc::Rc::clone(&accumulator);
				let handled = self.handle_with::<$brand<Out>, Idx, RMinusEffect>(
					move |op: $cell<'static, Out, Run<RMinusEffect, CNilBrand, A>>| match op {
						$cell::$variant(out, next, _) => {
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
	}};
}

macro_rules! rcrun_vec_tokens {
	($method:ident, $run_monoid:ident, $constructor:ident, $brand:ident) => {{
		let examples = vec_runner_examples(
			"rc_run",
			"RcRun",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"RcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by collecting payloads into a vector.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_returns("A first-order-only `RcRun` program returning `(result, payloads)`.")]
			#examples
			#[inline]
			pub fn $method<Out, Idx, RMinusEffect>(
				self
			) -> RcRun<RMinusEffect, CNilBrand, (A, Vec<Out>)>
			where
				A: Clone,
				Out: Clone + 'static,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<RMinusEffect, CNilBrand>, RcTypeErasedValue>,
				>): Clone,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcRun<R, CNilBrand, A>,
				>): Member<
					RcCoyoneda<'static, $brand<Out>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							RcRun<R, CNilBrand, A>,
						>
					),
				>, {
				self.$run_monoid::<Out, Vec<Out>, Idx, RMinusEffect>(|out| vec![out])
			}
		}
	}};
}

macro_rules! rcrun_monoid_tokens {
	($method:ident, $constructor:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = monoid_runner_examples(
			"rc_run",
			"RcRun",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"RcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by mapping each payload into a monoidal accumulator.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The accumulated payload type.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_parameters("The function that maps one payload into an accumulator chunk.")]
			#[document_returns(
				"A first-order-only `RcRun` program returning `(result, accumulated_payload)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Acc, Idx, RMinusEffect>(
				self,
				mapper: impl Fn(Out) -> Acc + 'static,
			) -> RcRun<RMinusEffect, CNilBrand, (A, Acc)>
			where
				A: Clone,
				Out: Clone + 'static,
				Acc: crate::classes::Monoid + Clone + 'static,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcFree<NodeBrand<RMinusEffect, CNilBrand>, RcTypeErasedValue>,
				>): Clone,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					RcRun<R, CNilBrand, A>,
				>): Member<
					RcCoyoneda<'static, $brand<Out>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							RcRun<R, CNilBrand, A>,
						>
					),
				>, {
				let mapper = std::rc::Rc::new(mapper);
				let accumulator:
					std::rc::Rc<std::cell::RefCell<std::rc::Rc<dyn Fn(Acc) -> Acc>>> =
					std::rc::Rc::new(std::cell::RefCell::new(std::rc::Rc::new(|acc| acc)));
				let handler_accumulator = std::rc::Rc::clone(&accumulator);
				let handled = self.handle_with::<$brand<Out>, Idx, RMinusEffect>(
					move |op: $cell<'static, Out, RcRun<RMinusEffect, CNilBrand, A>>| match op {
						$cell::$variant(out, next, _) => {
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
	}};
}

macro_rules! arcrun_vec_tokens {
	($method:ident, $run_monoid:ident, $constructor:ident, $brand:ident) => {{
		let examples = vec_runner_examples(
			"arc_run",
			"ArcRun",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"ArcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by collecting payloads into a vector.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_returns("A first-order-only `ArcRun` program returning `(result, payloads)`.")]
			#examples
			#[inline]
			pub fn $method<Out, Idx, RMinusEffect>(
				self
			) -> ArcRun<RMinusEffect, CNilBrand, (A, Vec<Out>)>
			where
				A: Clone + Send + Sync,
				Out: Clone + Send + Sync + 'static,
				R: Kind_cdc7cd43dac7585f + 'static,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
				NodeBrand<R, CNilBrand>: SendFunctor,
				NodeBrand<RMinusEffect, CNilBrand>: WrapDrop
					+ Kind_cdc7cd43dac7585f<
						Of<'static, ArcFree<NodeBrand<RMinusEffect, CNilBrand>, ArcTypeErasedValue>>:
							Send + Sync,
					> + SendFunctor,
				Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<RMinusEffect, CNilBrand>, ArcTypeErasedValue>,
				>): Clone,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcRun<R, CNilBrand, A>,
				>): Member<
					ArcCoyoneda<'static, $brand<Out>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							ArcRun<R, CNilBrand, A>,
						>
					),
				>, {
				self.$run_monoid::<Out, Vec<Out>, Idx, RMinusEffect>(|out| vec![out])
			}
		}
	}};
}

macro_rules! arcrun_monoid_tokens {
	($method:ident, $constructor:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = monoid_runner_examples(
			"arc_run",
			"ArcRun",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"ArcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by mapping each payload into a monoidal accumulator.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The accumulated payload type.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_parameters("The function that maps one payload into an accumulator chunk.")]
			#[document_returns(
				"A first-order-only `ArcRun` program returning `(result, accumulated_payload)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Acc, Idx, RMinusEffect>(
				self,
				mapper: impl Fn(Out) -> Acc + Send + Sync + 'static,
			) -> ArcRun<RMinusEffect, CNilBrand, (A, Acc)>
			where
				A: Clone + Send + Sync,
				Out: Clone + Send + Sync + 'static,
				Acc: crate::classes::Monoid + Clone + Send + Sync + 'static,
				R: Kind_cdc7cd43dac7585f + 'static,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
				NodeBrand<R, CNilBrand>: SendFunctor,
				NodeBrand<RMinusEffect, CNilBrand>: WrapDrop
					+ Kind_cdc7cd43dac7585f<
						Of<'static, ArcFree<NodeBrand<RMinusEffect, CNilBrand>, ArcTypeErasedValue>>:
							Send + Sync,
					> + SendFunctor,
				Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcFree<NodeBrand<RMinusEffect, CNilBrand>, ArcTypeErasedValue>,
				>): Clone,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'static,
					ArcRun<R, CNilBrand, A>,
				>): Member<
					ArcCoyoneda<'static, $brand<Out>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
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
				let handled = self.handle_with::<$brand<Out>, Idx, RMinusEffect>(
					move |op: $cell<'static, Out, ArcRun<RMinusEffect, CNilBrand, A>>| match op {
						$cell::$variant(out, next, _) => {
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
	}};
}

macro_rules! run_explicit_vec_tokens {
	($method:ident, $run_monoid:ident, $constructor:ident, $brand:ident) => {{
		let examples = vec_runner_examples(
			"run_explicit",
			"RunExplicit",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"CoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by collecting payloads into a vector.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_returns(
				"A first-order-only `RunExplicit` program returning `(result, payloads)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Idx, RMinusEffect>(
				self
			) -> RunExplicit<'a, RMinusEffect, CNilBrand, (A, Vec<Out>)>
			where
				Out: Clone + 'static + 'a,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RunExplicit<'a, R, CNilBrand, A>,
				>): Member<
					Coyoneda<'a, $brand<Out>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RunExplicit<'a, R, CNilBrand, A>,
						>
					),
				>, {
				self.$run_monoid::<Out, Vec<Out>, Idx, RMinusEffect>(|out| vec![out])
			}
		}
	}};
}

macro_rules! run_explicit_monoid_tokens {
	($method:ident, $constructor:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = monoid_runner_examples(
			"run_explicit",
			"RunExplicit",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"CoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by mapping each payload into a monoidal accumulator.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The accumulated payload type.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_parameters("The function that maps one payload into an accumulator chunk.")]
			#[document_returns(
				"A first-order-only `RunExplicit` program returning `(result, accumulated_payload)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Acc, Idx, RMinusEffect>(
				self,
				mapper: impl Fn(Out) -> Acc + 'a,
			) -> RunExplicit<'a, RMinusEffect, CNilBrand, (A, Acc)>
			where
				Out: Clone + 'static + 'a,
				Acc: crate::classes::Monoid + Clone + 'a,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RunExplicit<'a, R, CNilBrand, A>,
				>): Member<
					Coyoneda<'a, $brand<Out>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
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
				let handled = self.handle_with::<$brand<Out>, Idx, RMinusEffect>(
					move |op: $cell<'a, Out, RunExplicit<'a, RMinusEffect, CNilBrand, A>>| match op {
						$cell::$variant(out, next, _) => {
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
	}};
}

macro_rules! rcrun_explicit_vec_tokens {
	($method:ident, $run_monoid:ident, $constructor:ident, $brand:ident) => {{
		let examples = vec_runner_examples(
			"rc_run_explicit",
			"RcRunExplicit",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"RcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by collecting payloads into a vector.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_returns(
				"A first-order-only `RcRunExplicit` program returning `(result, payloads)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Idx, RMinusEffect>(
				self
			) -> RcRunExplicit<'a, RMinusEffect, CNilBrand, (A, Vec<Out>)>
			where
				A: Clone,
				Out: Clone + 'static + 'a,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Vec<Out>)>,
				>): Clone,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcRunExplicit<'a, R, CNilBrand, A>,
				>): Member<
					RcCoyoneda<'a, $brand<Out>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RcRunExplicit<'a, R, CNilBrand, A>,
						>
					),
				>, {
				self.$run_monoid::<Out, Vec<Out>, Idx, RMinusEffect>(|out| vec![out])
			}
		}
	}};
}

macro_rules! rcrun_explicit_monoid_tokens {
	($method:ident, $constructor:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = monoid_runner_examples(
			"rc_run_explicit",
			"RcRunExplicit",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"RcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by mapping each payload into a monoidal accumulator.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The accumulated payload type.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_parameters("The function that maps one payload into an accumulator chunk.")]
			#[document_returns(
				"A first-order-only `RcRunExplicit` program returning `(result, accumulated_payload)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Acc, Idx, RMinusEffect>(
				self,
				mapper: impl Fn(Out) -> Acc + 'a,
			) -> RcRunExplicit<'a, RMinusEffect, CNilBrand, (A, Acc)>
			where
				A: Clone,
				Out: Clone + 'static + 'a,
				Acc: crate::classes::Monoid + Clone + 'a,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
				Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Clone,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Acc)>,
				>): Clone,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					RcRunExplicit<'a, R, CNilBrand, A>,
				>): Member<
					RcCoyoneda<'a, $brand<Out>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
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
				let handled = self.handle_with::<$brand<Out>, Idx, RMinusEffect>(
					move |op: $cell<'a, Out, RcRunExplicit<'a, RMinusEffect, CNilBrand, A>>| {
						match op {
							$cell::$variant(out, next, _) => {
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
	}};
}

macro_rules! arcrun_explicit_vec_tokens {
	($method:ident, $run_monoid:ident, $constructor:ident, $brand:ident) => {{
		let examples = vec_runner_examples(
			"arc_run_explicit",
			"ArcRunExplicit",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"ArcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by collecting payloads into a vector.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_returns(
				"A first-order-only `ArcRunExplicit` program returning `(result, payloads)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Idx, RMinusEffect>(
				self
			) -> ArcRunExplicit<'a, RMinusEffect, CNilBrand, (A, Vec<Out>)>
			where
				A: Clone + Send + Sync,
				Out: Clone + Send + Sync + 'static + 'a,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
				NodeBrand<R, CNilBrand>: SendFunctor,
				NodeBrand<RMinusEffect, CNilBrand>: SendFunctor,
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
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Clone + Send + Sync,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Vec<Out>)>,
				>): Clone + Send + Sync,
				Apply!(<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Send + Sync,
				Apply!(<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Vec<Out>)>,
				>): Send + Sync,
				Apply!(<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcRunExplicit<'a, RMinusEffect, CNilBrand, A>,
				>): Send + Sync,
				Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Send + Sync,
				Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Vec<Out>)>,
				>): Send + Sync,
				Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcRunExplicit<'a, RMinusEffect, CNilBrand, A>,
				>): Send + Sync,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcRunExplicit<'a, R, CNilBrand, A>,
				>): Member<
					ArcCoyoneda<'a, $brand<Out>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, CNilBrand, A>,
						>
					),
				>, {
				self.$run_monoid::<Out, Vec<Out>, Idx, RMinusEffect>(|out| vec![out])
			}
		}
	}};
}

macro_rules! arcrun_explicit_monoid_tokens {
	($method:ident, $constructor:ident, $brand:ident, $cell:ident, $variant:ident) => {{
		let examples = monoid_runner_examples(
			"arc_run_explicit",
			"ArcRunExplicit",
			stringify!($method),
			stringify!($constructor),
			stringify!($brand),
			"ArcCoyonedaBrand",
		);
		quote! {
			/// Interprets a direct-payload effect by mapping each payload into a monoidal accumulator.
			#[__document_module_generated]
			#[document_signature]
			#[document_type_parameters(
				"The payload type carried by the effect.",
				"The accumulated payload type.",
				"The type-level Member-position witness for the effect.",
				"The first-order row brand with the effect removed."
			)]
			#[document_parameters("The function that maps one payload into an accumulator chunk.")]
			#[document_returns(
				"A first-order-only `ArcRunExplicit` program returning `(result, accumulated_payload)`."
			)]
			#examples
			#[inline]
			pub fn $method<Out, Acc, Idx, RMinusEffect>(
				self,
				mapper: impl Fn(Out) -> Acc + Send + Sync + 'a,
			) -> ArcRunExplicit<'a, RMinusEffect, CNilBrand, (A, Acc)>
			where
				A: Clone + Send + Sync,
				Out: Clone + Send + Sync + 'static + 'a,
				Acc: crate::classes::Monoid + Clone + Send + Sync + 'a,
				RMinusEffect: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
				NodeBrand<R, CNilBrand>: SendFunctor,
				NodeBrand<RMinusEffect, CNilBrand>: SendFunctor,
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
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Clone + Send + Sync,
				Apply!(<NodeBrand<RMinusEffect, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Acc)>,
				>): Clone + Send + Sync,
				Apply!(<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Send + Sync,
				Apply!(<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Acc)>,
				>): Send + Sync,
				Apply!(<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcRunExplicit<'a, RMinusEffect, CNilBrand, A>,
				>): Send + Sync,
				Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, A>,
				>): Send + Sync,
				Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcFreeExplicit<'a, NodeBrand<RMinusEffect, CNilBrand>, (A, Acc)>,
				>): Send + Sync,
				Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcRunExplicit<'a, RMinusEffect, CNilBrand, A>,
				>): Send + Sync,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
					'a,
					ArcRunExplicit<'a, R, CNilBrand, A>,
				>): Member<
					ArcCoyoneda<'a, $brand<Out>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
						<RMinusEffect as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
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
				let handled = self.handle_with::<$brand<Out>, Idx, RMinusEffect>(
					move |op: $cell<'a, Out, ArcRunExplicit<'a, RMinusEffect, CNilBrand, A>>| {
						match op {
							$cell::$variant(out, next, _) => {
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
	}};
}

pub(super) fn direct_payload_wrapper_impl_items_from_descriptor(
	wrapper: WrapperName,
	effect: EffectName,
	method: RunWrapperMethod,
) -> Option<syn::Result<Vec<ImplItem>>> {
	generator_descriptors::method_spec(effect, method)?;
	generator_descriptors::wrapper_spec(wrapper)?;

	let tokens = match (wrapper, effect, method) {
		(WrapperName::Run, EffectName::Output, RunWrapperMethod::Output) =>
			run_constructor_tokens!(output, run_output_vec, output, OutputBrand, Output, Output),
		(WrapperName::Run, EffectName::Log, RunWrapperMethod::Log) =>
			run_constructor_tokens!(log, run_log_vec, log, LogBrand, Log, Log),
		(WrapperName::RcRun, EffectName::Output, RunWrapperMethod::Output) =>
			rcrun_constructor_tokens!(output, run_output_vec, output, OutputBrand, Output, Output),
		(WrapperName::RcRun, EffectName::Log, RunWrapperMethod::Log) =>
			rcrun_constructor_tokens!(log, run_log_vec, log, LogBrand, Log, Log),
		(WrapperName::ArcRun, EffectName::Output, RunWrapperMethod::Output) =>
			arcrun_constructor_tokens!(output, run_output_vec, output, OutputBrand, Output, Output),
		(WrapperName::ArcRun, EffectName::Log, RunWrapperMethod::Log) =>
			arcrun_constructor_tokens!(log, run_log_vec, log, LogBrand, Log, Log),
		(WrapperName::RunExplicit, EffectName::Output, RunWrapperMethod::Output) =>
			run_explicit_constructor_tokens!(
				output,
				run_output_vec,
				output,
				OutputBrand,
				Output,
				Output
			),
		(WrapperName::RunExplicit, EffectName::Log, RunWrapperMethod::Log) =>
			run_explicit_constructor_tokens!(log, run_log_vec, log, LogBrand, Log, Log),
		(WrapperName::RcRunExplicit, EffectName::Output, RunWrapperMethod::Output) =>
			rcrun_explicit_constructor_tokens!(
				output,
				run_output_vec,
				output,
				OutputBrand,
				Output,
				Output
			),
		(WrapperName::RcRunExplicit, EffectName::Log, RunWrapperMethod::Log) =>
			rcrun_explicit_constructor_tokens!(log, run_log_vec, log, LogBrand, Log, Log),
		(WrapperName::ArcRunExplicit, EffectName::Output, RunWrapperMethod::Output) =>
			arcrun_explicit_constructor_tokens!(
				output,
				run_output_vec,
				output,
				OutputBrand,
				Output,
				Output
			),
		(WrapperName::ArcRunExplicit, EffectName::Log, RunWrapperMethod::Log) =>
			arcrun_explicit_constructor_tokens!(log, run_log_vec, log, LogBrand, Log, Log),

		(WrapperName::Run, EffectName::Output, RunWrapperMethod::RunOutputVec) =>
			run_vec_tokens!(run_output_vec, run_output_monoid, output, OutputBrand),
		(WrapperName::Run, EffectName::Log, RunWrapperMethod::RunLogVec) =>
			run_vec_tokens!(run_log_vec, run_log_monoid, log, LogBrand),
		(WrapperName::RcRun, EffectName::Output, RunWrapperMethod::RunOutputVec) =>
			rcrun_vec_tokens!(run_output_vec, run_output_monoid, output, OutputBrand),
		(WrapperName::RcRun, EffectName::Log, RunWrapperMethod::RunLogVec) =>
			rcrun_vec_tokens!(run_log_vec, run_log_monoid, log, LogBrand),
		(WrapperName::ArcRun, EffectName::Output, RunWrapperMethod::RunOutputVec) =>
			arcrun_vec_tokens!(run_output_vec, run_output_monoid, output, OutputBrand),
		(WrapperName::ArcRun, EffectName::Log, RunWrapperMethod::RunLogVec) =>
			arcrun_vec_tokens!(run_log_vec, run_log_monoid, log, LogBrand),
		(WrapperName::RunExplicit, EffectName::Output, RunWrapperMethod::RunOutputVec) =>
			run_explicit_vec_tokens!(run_output_vec, run_output_monoid, output, OutputBrand),
		(WrapperName::RunExplicit, EffectName::Log, RunWrapperMethod::RunLogVec) =>
			run_explicit_vec_tokens!(run_log_vec, run_log_monoid, log, LogBrand),
		(WrapperName::RcRunExplicit, EffectName::Output, RunWrapperMethod::RunOutputVec) =>
			rcrun_explicit_vec_tokens!(run_output_vec, run_output_monoid, output, OutputBrand),
		(WrapperName::RcRunExplicit, EffectName::Log, RunWrapperMethod::RunLogVec) =>
			rcrun_explicit_vec_tokens!(run_log_vec, run_log_monoid, log, LogBrand),
		(WrapperName::ArcRunExplicit, EffectName::Output, RunWrapperMethod::RunOutputVec) =>
			arcrun_explicit_vec_tokens!(run_output_vec, run_output_monoid, output, OutputBrand),
		(WrapperName::ArcRunExplicit, EffectName::Log, RunWrapperMethod::RunLogVec) =>
			arcrun_explicit_vec_tokens!(run_log_vec, run_log_monoid, log, LogBrand),

		(WrapperName::Run, EffectName::Output, RunWrapperMethod::RunOutputMonoid) =>
			run_monoid_tokens!(run_output_monoid, output, OutputBrand, Output, Output),
		(WrapperName::Run, EffectName::Log, RunWrapperMethod::RunLogMonoid) =>
			run_monoid_tokens!(run_log_monoid, log, LogBrand, Log, Log),
		(WrapperName::RcRun, EffectName::Output, RunWrapperMethod::RunOutputMonoid) =>
			rcrun_monoid_tokens!(run_output_monoid, output, OutputBrand, Output, Output),
		(WrapperName::RcRun, EffectName::Log, RunWrapperMethod::RunLogMonoid) =>
			rcrun_monoid_tokens!(run_log_monoid, log, LogBrand, Log, Log),
		(WrapperName::ArcRun, EffectName::Output, RunWrapperMethod::RunOutputMonoid) =>
			arcrun_monoid_tokens!(run_output_monoid, output, OutputBrand, Output, Output),
		(WrapperName::ArcRun, EffectName::Log, RunWrapperMethod::RunLogMonoid) =>
			arcrun_monoid_tokens!(run_log_monoid, log, LogBrand, Log, Log),
		(WrapperName::RunExplicit, EffectName::Output, RunWrapperMethod::RunOutputMonoid) =>
			run_explicit_monoid_tokens!(run_output_monoid, output, OutputBrand, Output, Output),
		(WrapperName::RunExplicit, EffectName::Log, RunWrapperMethod::RunLogMonoid) =>
			run_explicit_monoid_tokens!(run_log_monoid, log, LogBrand, Log, Log),
		(WrapperName::RcRunExplicit, EffectName::Output, RunWrapperMethod::RunOutputMonoid) =>
			rcrun_explicit_monoid_tokens!(run_output_monoid, output, OutputBrand, Output, Output),
		(WrapperName::RcRunExplicit, EffectName::Log, RunWrapperMethod::RunLogMonoid) =>
			rcrun_explicit_monoid_tokens!(run_log_monoid, log, LogBrand, Log, Log),
		(WrapperName::ArcRunExplicit, EffectName::Output, RunWrapperMethod::RunOutputMonoid) =>
			arcrun_explicit_monoid_tokens!(run_output_monoid, output, OutputBrand, Output, Output),
		(WrapperName::ArcRunExplicit, EffectName::Log, RunWrapperMethod::RunLogMonoid) =>
			arcrun_explicit_monoid_tokens!(run_log_monoid, log, LogBrand, Log, Log),
		_ => return None,
	};

	Some(impl_items_from_tokens(tokens))
}

fn constructor_examples(
	wrapper_module: &str,
	wrapper_type: &str,
	constructor: &str,
	vec_runner: &str,
	brand: &str,
	row_functor: &str,
) -> TokenStream {
	let row = row_alias(brand, row_functor);
	let source_unit = wrapper_source_type(wrapper_type, "Row", "()");
	let handled_vec = wrapper_source_type(wrapper_type, "CNilBrand", "((), Vec<&'static str>)");
	let constructor_call =
		wrapper_constructor_call(wrapper_type, "Row", "()", constructor, "message");

	doc_example_attrs(vec![
		"".to_owned(),
		"```".to_owned(),
		format!("use fp_library::{{brands::*, types::effects::{wrapper_module}::{wrapper_type}}};"),
		"".to_owned(),
		format!("type Row = {row};"),
		"".to_owned(),
		format!("let program: {source_unit} = {constructor_call};"),
		format!(
			"let handled: {handled_vec} = program.{vec_runner}::<&'static str, _, CNilBrand>();"
		),
		"assert_eq!(handled.extract(), ((), vec![\"message\"]));".to_owned(),
		"```".to_owned(),
	])
}

fn vec_runner_examples(
	wrapper_module: &str,
	wrapper_type: &str,
	vec_runner: &str,
	constructor: &str,
	brand: &str,
	row_functor: &str,
) -> TokenStream {
	let row = row_alias(brand, row_functor);
	let source_i32 = wrapper_source_type(wrapper_type, "Row", "i32");
	let handled_vec = wrapper_source_type(wrapper_type, "CNilBrand", "(i32, Vec<&'static str>)");
	let first_call = wrapper_constructor_call(wrapper_type, "Row", "()", constructor, "first");
	let second_call = wrapper_constructor_call(wrapper_type, "Row", "()", constructor, "second");
	let pure_call = wrapper_pure_call(wrapper_type, "Row", "i32", "7");

	doc_example_attrs(vec![
		"".to_owned(),
		"```".to_owned(),
		format!("use fp_library::{{brands::*, types::effects::{wrapper_module}::{wrapper_type}}};"),
		"".to_owned(),
		format!("type Row = {row};"),
		"".to_owned(),
		format!("let program: {source_i32} = {first_call}"),
		format!("\t.bind(|()| {second_call})"),
		format!("\t.bind(|()| {pure_call});"),
		format!(
			"let handled: {handled_vec} = program.{vec_runner}::<&'static str, _, CNilBrand>();"
		),
		"assert_eq!(handled.extract(), (7, vec![\"first\", \"second\"]));".to_owned(),
		"```".to_owned(),
	])
}

fn monoid_runner_examples(
	wrapper_module: &str,
	wrapper_type: &str,
	monoid_runner: &str,
	constructor: &str,
	brand: &str,
	row_functor: &str,
) -> TokenStream {
	let row = row_alias(brand, row_functor);
	let source_i32 = wrapper_source_type(wrapper_type, "Row", "i32");
	let handled_string = wrapper_source_type(wrapper_type, "CNilBrand", "(i32, String)");
	let first_call = wrapper_constructor_call(wrapper_type, "Row", "()", constructor, "first");
	let second_call = wrapper_constructor_call(wrapper_type, "Row", "()", constructor, "second");
	let pure_call = wrapper_pure_call(wrapper_type, "Row", "i32", "7");

	doc_example_attrs(vec![
		"".to_owned(),
		"```".to_owned(),
		format!("use fp_library::{{brands::*, types::effects::{wrapper_module}::{wrapper_type}}};"),
		"".to_owned(),
		format!("type Row = {row};"),
		"".to_owned(),
		format!("let program: {source_i32} = {first_call}"),
		format!("\t.bind(|()| {second_call})"),
		format!("\t.bind(|()| {pure_call});"),
		format!(
			"let handled: {handled_string} = program.{monoid_runner}::<&'static str, String, _, CNilBrand>(str::to_string);"
		),
		"assert_eq!(handled.extract(), (7, \"firstsecond\".to_string()));".to_owned(),
		"```".to_owned(),
	])
}

fn row_alias(
	brand: &str,
	row_functor: &str,
) -> String {
	format!("CoproductBrand<{row_functor}<{brand}<&'static str>>, CNilBrand>")
}

fn wrapper_source_type(
	wrapper_type: &str,
	row: &str,
	value: &str,
) -> String {
	if wrapper_type.ends_with("Explicit") {
		format!("{wrapper_type}<'static, {row}, CNilBrand, {value}>")
	} else {
		format!("{wrapper_type}<{row}, CNilBrand, {value}>")
	}
}

fn wrapper_constructor_call(
	wrapper_type: &str,
	row: &str,
	value: &str,
	constructor: &str,
	message: &str,
) -> String {
	if wrapper_type.ends_with("Explicit") {
		format!(
			"{wrapper_type}::<'static, {row}, CNilBrand, {value}>::{constructor}::<&'static str, _>(\"{message}\")"
		)
	} else {
		format!(
			"{wrapper_type}::<{row}, CNilBrand, {value}>::{constructor}::<&'static str, _>(\"{message}\")"
		)
	}
}

fn wrapper_pure_call(
	wrapper_type: &str,
	row: &str,
	value: &str,
	expression: &str,
) -> String {
	if wrapper_type.ends_with("Explicit") {
		format!("{wrapper_type}::<'static, {row}, CNilBrand, {value}>::pure({expression})")
	} else {
		format!("{wrapper_type}::<{row}, CNilBrand, {value}>::pure({expression})")
	}
}

fn doc_example_attrs(lines: Vec<String>) -> TokenStream {
	let mut tokens = quote! {
		#[document_examples]
	};
	for line in lines {
		let literal = Literal::string(&line);
		tokens.extend(quote! {
			#[doc = #literal]
		});
	}
	tokens
}
