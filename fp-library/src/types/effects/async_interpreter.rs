//! Async interpreter for the effects subsystem.
//!
//! A direct async driver loop, per the adopted runtime policy: it peels a
//! program, advances each layer, and keeps the continuation as data (the
//! `Free` tree) across every `.await`, with no `MonadRec`-over-`Future` and
//! no named recursive async type.
//!
//! Two drivers live here:
//!
//! - [`handle_async`] is a crate-internal first-order driver that advances
//!   every layer through the ordinary synchronous handler dispatch and
//!   returns the result in a `Future`. It performs no asynchronous work
//!   itself; it is a foundation exercised by this module's tests.
//! - `handle_async_anywhere` is the genuinely asynchronous driver, exposed
//!   publicly as the
//!   [`Run::run_async`](crate::types::effects::run::Run::run_async) method.
//!   For a program whose first-order row contains the
//!   [`Await`](crate::types::effects::await_future::Await) future base-lift
//!   effect at any position, it projects that effect out of the row, lowers it
//!   to a future of the next program and `.await`s it, and dispatches every
//!   other first-order effect to the user handlers. The returned future is
//!   runtime-agnostic and completes only once every embedded future has.
//!
//! Remaining work extends this across the wrapper family with the
//! local-versus-`Send` split (local futures for the Box and Rc families,
//! `Send` futures for the Arc family) and to scoped layers via the in-crate
//! dispatch paths.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				AwaitBrand,
				CNilBrand,
			},
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Coyoneda,
				effects::{
					interpreter::DispatchHandlers,
					member::Member,
					node::Node,
					run::Run,
				},
			},
		},
		fp_macros::*,
	};

	/// Drives a first-order default `Run` program to completion
	/// asynchronously, returning a runtime-agnostic future.
	///
	/// The loop peels the program and, at each first-order layer, advances
	/// via the ordinary synchronous handler dispatch; the continuation stays
	/// data the whole time. The program's scoped row is `CNilBrand`, so no
	/// scoped layers occur.
	///
	/// This returns the result in a `Future` but does no asynchronous work
	/// itself; it is the synchronous-dispatch foundation. The genuinely
	/// asynchronous driver is `handle_async_anywhere`. This is a retained
	/// foundation exercised by this module's tests, not dead code.
	#[document_examples(
		skip_call_check,
		reason = "`handle_async` is crate-internal, so an external doctest cannot call it; the example shows the async driver-loop shape (advance a program-as-data to a result) with public types."
	)]
	///
	/// ```
	/// use std::{
	/// 	future::Future,
	/// 	pin::pin,
	/// 	task::{
	/// 		Context,
	/// 		Poll,
	/// 		Waker,
	/// 	},
	/// };
	///
	/// // The real driver peels a `Run` and dispatches handlers, awaiting an
	/// // embedded future at each layer once that effect lands. The shape,
	/// // with a toy program represented as data:
	/// async fn drive(steps: Vec<i32>) -> i32 {
	/// 	steps.into_iter().sum()
	/// }
	///
	/// let mut future = pin!(drive(vec![1, 2, 3]));
	/// let mut context = Context::from_waker(Waker::noop());
	/// let result = loop {
	/// 	if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
	/// 		break value;
	/// 	}
	/// };
	/// assert_eq!(result, 6);
	/// ```
	#[cfg_attr(
		not(test),
		expect(
			dead_code,
			reason = "W13 async-interpreter foundation, retained until it is wired in or made public; exercised by this module's tests. See the module docs."
		)
	)]
	pub(crate) async fn handle_async<R, A>(
		program: Run<R, CNilBrand, A>,
		handlers: impl for<'h> DispatchHandlers<
			'h,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<R, CNilBrand, A>>),
			Run<R, CNilBrand, A>,
		>,
	) -> A
	where
		R: WrapDrop + Functor + 'static,
		A: 'static, {
		let mut program = program;
		loop {
			match program.peel() {
				Ok(value) => return value,
				Err(node) => match node {
					// First-order effect layer, dispatched synchronously. The
					// await-aware sibling `handle_async_anywhere` awaits an
					// embedded future at this point instead.
					Node::First(layer) => program = handlers.dispatch(layer),
					// A first-order-only program has no scoped layers.
					Node::Scoped(empty) => match empty {},
				},
			}
		}
	}

	/// Drives a default `Run` program in which the
	/// [`AwaitBrand`] future base-lift effect appears at any position `Idx` in
	/// the first-order row, awaiting each embedded future and dispatching every
	/// other first-order effect to `handlers`. This is the driver behind the
	/// public [`Run::run_async`](crate::types::effects::run::Run::run_async)
	/// method.
	///
	/// At each first-order layer it projects the await effect out of the row by
	/// its `Member` position; on a hit it lowers the `Coyoneda<AwaitBrand, _>`
	/// to a future of the next program and `.await`s it (the only asynchronous
	/// suspension point, so the returned future is `Ready` only once every
	/// embedded future has completed), and on a miss it dispatches the row
	/// remainder (every effect other than await) to `handlers`. The
	/// continuation stays data throughout. The program's scoped row is
	/// `CNilBrand`, so no scoped layers occur.
	#[document_examples(
		skip_call_check,
		reason = "`handle_async_anywhere` is crate-internal, so an external doctest cannot call it; the example shows the project-or-dispatch driver shape with public types."
	)]
	///
	/// ```
	/// use std::{
	/// 	future::Future,
	/// 	pin::pin,
	/// 	task::{
	/// 		Context,
	/// 		Poll,
	/// 		Waker,
	/// 	},
	/// };
	///
	/// // At each layer the driver either awaits an embedded future or
	/// // dispatches a non-await effect. The shape, awaiting a future and then
	/// // applying a synchronous step:
	/// async fn drive(future: std::pin::Pin<Box<dyn Future<Output = i32>>>) -> i32 {
	/// 	let awaited = future.await;
	/// 	awaited + 1
	/// }
	///
	/// let mut future = pin!(drive(Box::pin(async { 41 })));
	/// let mut context = Context::from_waker(Waker::noop());
	/// let result = loop {
	/// 	if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
	/// 		break value;
	/// 	}
	/// };
	/// assert_eq!(result, 42);
	/// ```
	pub(crate) async fn handle_async_anywhere<R, A, Idx>(
		program: Run<R, CNilBrand, A>,
		handlers: impl DispatchHandlers<
			'static,
			<Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>) as Member<
				Coyoneda<'static, AwaitBrand, Run<R, CNilBrand, A>>,
				Idx,
			>>::Remainder,
			Run<R, CNilBrand, A>,
		>,
	) -> A
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
		Idx: 'static,
		Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
			Member<Coyoneda<'static, AwaitBrand, Run<R, CNilBrand, A>>, Idx>,
		<Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>) as Member<
			Coyoneda<'static, AwaitBrand, Run<R, CNilBrand, A>>,
			Idx,
	>>::Remainder: 'static,{
		let mut program = program;
		loop {
			match program.peel() {
				Ok(value) => return value,
				Err(node) => match node {
					Node::First(layer) => match Member::<
						Coyoneda<'static, AwaitBrand, Run<R, CNilBrand, A>>,
						Idx,
					>::project(layer)
					{
						// Await effect: lower to a future of the next program
						// and await it. The only asynchronous suspension point.
						Ok(coyoneda) => program = coyoneda.lower().await,
						// Every other first-order effect: dispatch the remainder.
						Err(remainder) => program = handlers.dispatch(remainder),
					},
					// A first-order-only program has no scoped layers.
					Node::Scoped(empty) => match empty {},
				},
			}
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand, which must contain the await effect.",
		"The result type."
	)]
	#[document_parameters("The async program to run.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Runs this async program to completion, awaiting each embedded
		/// [`Await`](crate::types::effects::await_future::Await) future and
		/// dispatching every other first-order effect to `handlers`.
		///
		/// The program's first-order row `R` must contain the await effect at
		/// some position; `handlers` covers every other first-order effect.
		/// The returned future is runtime-agnostic and completes only once
		/// every embedded future has, so it can be driven by any executor.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type-level Member-position witness for the await effect in the row (typically inferred)."
		)]
		///
		#[document_parameters("The handler list for every first-order effect other than await.")]
		///
		#[document_returns("A runtime-agnostic future of the program's result.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::{
		/// 			AwaitBrand,
		/// 			CNilBrand,
		/// 			CoproductBrand,
		/// 			CoyonedaBrand,
		/// 		},
		/// 		handlers,
		/// 		types::effects::run::Run,
		/// 	},
		/// 	std::{
		/// 		future::Future,
		/// 		pin::pin,
		/// 		task::{
		/// 			Context,
		/// 			Poll,
		/// 			Waker,
		/// 		},
		/// 	},
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<AwaitBrand>, CNilBrand>;
		/// type Prog<A> = Run<Row, CNilBrand, A>;
		///
		/// let program: Prog<i32> = Run::await_future(async { 41 }).bind(|value| Run::pure(value + 1));
		///
		/// // Drive the runtime-agnostic future on a trivial executor.
		/// let mut future = pin!(program.run_async(handlers! {}));
		/// let mut context = Context::from_waker(Waker::noop());
		/// let result = loop {
		/// 	if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
		/// 		break value;
		/// 	}
		/// };
		/// assert_eq!(result, 42);
		/// ```
		pub async fn run_async<Idx>(
			self,
			handlers: impl DispatchHandlers<
				'static,
				<Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>) as Member<
					Coyoneda<'static, AwaitBrand, Run<R, CNilBrand, A>>,
					Idx,
				>>::Remainder,
				Run<R, CNilBrand, A>,
			>,
		) -> A
		where
			Idx: 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>):
				Member<Coyoneda<'static, AwaitBrand, Run<R, CNilBrand, A>>, Idx>,
			<Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, Run<R, CNilBrand, A>>) as Member<
				Coyoneda<'static, AwaitBrand, Run<R, CNilBrand, A>>,
				Idx,
		>>::Remainder: 'static,{
			handle_async_anywhere(self, handlers).await
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::inner::{
			handle_async,
			handle_async_anywhere,
		},
		crate::{
			brands::{
				AwaitBrand,
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
				IdentityBrand,
			},
			handlers,
			types::{
				Identity,
				effects::{
					await_future::Await,
					run::Run,
				},
			},
		},
		std::{
			future::Future,
			pin::pin,
			task::{
				Context,
				Poll,
				Waker,
			},
		},
	};

	type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
	type Prog<A> = Run<FirstRow, CNilBrand, A>;

	// First-order row whose head is the await effect, with the rest in `Rest`.
	type AwaitRow<Rest> = CoproductBrand<CoyonedaBrand<AwaitBrand>, Rest>;

	// Minimal std-only executor: the interpreter does no asynchronous work
	// yet, so a single poll completes it.
	fn block_on<F: Future>(future: F) -> F::Output {
		let mut future = pin!(future);
		let mut context = Context::from_waker(Waker::noop());
		loop {
			if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
				return value;
			}
		}
	}

	#[test]
	fn handle_async_drives_a_first_order_program() {
		let mut program: Prog<usize> = Run::pure(0);
		for _ in 0 .. 500 {
			program = program
				.bind(|accumulator| Run::lift::<IdentityBrand, _>(Identity(accumulator + 1)));
		}

		let result = block_on(handle_async(
			program,
			handlers! {
				IdentityBrand: |operation: Identity<Prog<usize>>| operation.0,
			},
		));

		assert_eq!(result, 500);
	}

	// Resume-path spike for the Future base-lift effect (representation A).
	// Confirms that a value obtained asynchronously (here a plain value
	// standing in for an awaited future's output), type-erased exactly as the
	// substrate stores resumed values, resumes correctly through
	// `continue_from_erased` into the program's pending continuation queue.
	// The Identity effect is only a vehicle to create a suspension with a
	// pending continuation; the spike bypasses its handler and supplies the
	// value directly, as the real await effect's interpreter will after
	// awaiting an embedded future.
	#[test]
	fn future_value_resumes_via_continue_from_erased() {
		use crate::{
			brands::NodeBrand,
			types::{
				Free,
				free::{
					FreeRawStep,
					TypeErasedValue,
				},
			},
		};

		let program: Prog<usize> = Run::lift::<IdentityBrand, _>(Identity(0))
			.bind(|received: usize| Run::pure(received + 1));

		let result = match program.into_free().into_raw_step() {
			FreeRawStep::Suspended {
				continuations, ..
			} => {
				let awaited: TypeErasedValue = Box::new(5usize);
				let resumed = Free::<NodeBrand<FirstRow, CNilBrand>, usize>::continue_from_erased(
					Free::from_erased_value(awaited),
					continuations,
				);
				Run::from_free(resumed).peel().ok()
			}
			FreeRawStep::Done(_) => None,
		};

		// The awaited value (5) threads through the pending continuation
		// (`received + 1`), so the program completes with 6.
		assert_eq!(result, Some(6));
	}

	// The driver interleaves awaited futures and ordinary handler dispatch in
	// one program. The program awaits 10, hands the result to an Identity
	// handler that adds 1, then awaits a future computed from that (times 2):
	// 10 -> 11 -> 22. The await effect (at the row head here) is interpreted by
	// the driver; only the Identity tail effect goes through `handlers`.
	#[test]
	fn handle_async_anywhere_interleaves_awaits_and_handlers() {
		type Rest = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		type Prog<A> = Run<AwaitRow<Rest>, CNilBrand, A>;

		let first: Await<'static, usize> = Box::pin(async { 10 });
		let program: Prog<usize> = Run::lift::<AwaitBrand, _>(first)
			.bind(|awaited| Run::lift::<IdentityBrand, _>(Identity(awaited + 1)))
			.bind(|handled| {
				let second: Await<'static, usize> = Box::pin(async move { handled * 2 });
				Run::lift::<AwaitBrand, _>(second)
			});

		let result = block_on(handle_async_anywhere(
			program,
			handlers! {
				IdentityBrand: |operation: Identity<Prog<usize>>| operation.0,
			},
		));

		assert_eq!(result, 22);
	}

	// The driver awaits genuinely pending futures on a real runtime: each
	// embedded future yields to the Tokio scheduler before producing its
	// value, so completion requires real rescheduling, not a single poll. The
	// row has no effects other than await, so the handler list is empty. The
	// program awaits 5 (after a yield), then awaits 5 + 37 (after another
	// yield): the result is 42.
	#[tokio::test(flavor = "current_thread")]
	async fn handle_async_anywhere_awaits_real_pending_futures() {
		type Prog<A> = Run<AwaitRow<CNilBrand>, CNilBrand, A>;

		let first: Await<'static, usize> = Box::pin(async {
			tokio::task::yield_now().await;
			5
		});
		let program: Prog<usize> = Run::lift::<AwaitBrand, _>(first).bind(|awaited| {
			let second: Await<'static, usize> = Box::pin(async move {
				tokio::task::yield_now().await;
				awaited + 37
			});
			Run::lift::<AwaitBrand, _>(second)
		});

		let result = handle_async_anywhere(program, handlers! {}).await;

		assert_eq!(result, 42);
	}

	// The arbitrary-position driver awaits an await effect that is not at the
	// row head. Here the row is Identity then await: an Identity effect
	// produces 10, and the continuation awaits a future computed from it
	// (times 2). The driver projects the await effect by its Member position,
	// awaits it, and dispatches the Identity effect (the row remainder) to the
	// handler list. The result is 20.
	#[test]
	fn handle_async_anywhere_awaits_effect_at_non_head_position() {
		type Row = CoproductBrand<
			CoyonedaBrand<IdentityBrand>,
			CoproductBrand<CoyonedaBrand<AwaitBrand>, CNilBrand>,
		>;
		type Prog<A> = Run<Row, CNilBrand, A>;

		let program: Prog<usize> = Run::lift::<IdentityBrand, _>(Identity(10usize))
			.bind(|value| Run::await_future(async move { value * 2 }));

		let result = block_on(handle_async_anywhere(
			program,
			handlers! {
				IdentityBrand: |operation: Identity<Prog<usize>>| operation.0,
			},
		));

		assert_eq!(result, 20);
	}
}
