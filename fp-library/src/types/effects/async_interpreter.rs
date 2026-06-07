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
//! - `handle_async_with_await` is the genuinely asynchronous driver, exposed
//!   publicly as the
//!   [`Run::run_async`](crate::types::effects::run::Run::run_async) method.
//!   For a program whose first-order row head is the
//!   [`Await`](crate::types::effects::await_future::Await) future base-lift
//!   effect, it lowers that effect to a future of the next program and
//!   `.await`s it, and dispatches every other first-order effect to the user
//!   handlers. The returned future is runtime-agnostic and completes only
//!   once every embedded future has.
//!
//! Remaining work extends this across the wrapper family with the
//! local-versus-`Send` split (local futures for the Box and Rc families,
//! `Send` futures for the Arc family), to an arbitrary await position in the
//! row, and to scoped layers via the in-crate dispatch paths.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::{
				AwaitBrand,
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
			},
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::effects::{
				coproduct::Coproduct,
				interpreter::DispatchHandlers,
				node::Node,
				run::Run,
			},
		},
		fp_macros::*,
	};

	/// First-order row whose head is the [`AwaitBrand`] future base-lift
	/// effect, with the remaining effects in `Rest`. The await-aware driver
	/// interprets the head itself and dispatches the tail to user handlers.
	pub(crate) type AwaitRow<Rest> = CoproductBrand<CoyonedaBrand<AwaitBrand>, Rest>;

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
	/// asynchronous driver is `handle_async_with_await`. This is a retained
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
					// await-aware sibling `handle_async_with_await` awaits an
					// embedded future at this point instead.
					Node::First(layer) => program = handlers.dispatch(layer),
					// A first-order-only program has no scoped layers.
					Node::Scoped(empty) => match empty {},
				},
			}
		}
	}

	/// Drives a default `Run` program whose first-order row's head is the
	/// [`AwaitBrand`] future base-lift effect, awaiting each embedded future
	/// and dispatching every other first-order effect to `handlers`.
	///
	/// This is the genuinely asynchronous driver: at an await layer it lowers
	/// the `Coyoneda<AwaitBrand, _>` row entry to a future of the next program
	/// and `.await`s it, so the returned future is only `Ready` once every
	/// embedded future has completed. The continuation stays data throughout.
	/// `handlers` covers only the tail row `Rest`; the await head is handled
	/// here. The program's scoped row is `CNilBrand`, so no scoped layers
	/// occur. See the module documentation; this is a retained foundation, not
	/// dead code.
	#[document_examples(
		skip_call_check,
		reason = "`handle_async_with_await` is crate-internal, so an external doctest cannot call it; the example shows the await-then-advance driver shape with public types."
	)]
	///
	/// ```
	/// use std::{
	/// 	future::Future,
	/// 	pin::{
	/// 		Pin,
	/// 		pin,
	/// 	},
	/// 	task::{
	/// 		Context,
	/// 		Poll,
	/// 		Waker,
	/// 	},
	/// };
	///
	/// // The real driver awaits an embedded future at each await layer and
	/// // dispatches other effects to handlers. The shape, awaiting a sequence
	/// // of futures and summing their results:
	/// async fn drive(futures: Vec<Pin<Box<dyn Future<Output = i32>>>>) -> i32 {
	/// 	let mut total = 0;
	/// 	for future in futures {
	/// 		total += future.await;
	/// 	}
	/// 	total
	/// }
	///
	/// let futures: Vec<Pin<Box<dyn Future<Output = i32>>>> =
	/// 	vec![Box::pin(async { 1 }), Box::pin(async { 2 }), Box::pin(async { 3 })];
	/// let mut future = pin!(drive(futures));
	/// let mut context = Context::from_waker(Waker::noop());
	/// let result = loop {
	/// 	if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
	/// 		break value;
	/// 	}
	/// };
	/// assert_eq!(result, 6);
	/// ```
	pub(crate) async fn handle_async_with_await<Rest, A>(
		program: Run<AwaitRow<Rest>, CNilBrand, A>,
		handlers: impl for<'h> DispatchHandlers<
			'h,
			Apply!(<Rest as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Run<AwaitRow<Rest>, CNilBrand, A>>),
			Run<AwaitRow<Rest>, CNilBrand, A>,
		>,
	) -> A
	where
		Rest: WrapDrop + Functor + 'static,
		A: 'static, {
		let mut program = program;
		loop {
			match program.peel() {
				Ok(value) => return value,
				Err(node) => match node {
					Node::First(layer) => match layer {
						// Await head: lower the Coyoneda to a future of the next
						// program and await it. This is the only genuinely
						// asynchronous suspension point.
						Coproduct::Inl(coyoneda) => program = coyoneda.lower().await,
						// Any other first-order effect: dispatch to its handler.
						Coproduct::Inr(rest) => program = handlers.dispatch(rest),
					},
					// A first-order-only program has no scoped layers.
					Node::Scoped(empty) => match empty {},
				},
			}
		}
	}

	#[document_type_parameters(
		"The tail of the first-order row after the await head: the effects dispatched to handlers.",
		"The result type."
	)]
	#[document_parameters("The async program to run.")]
	impl<Rest, A> Run<CoproductBrand<CoyonedaBrand<AwaitBrand>, Rest>, CNilBrand, A>
	where
		Rest: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Runs this async program to completion, awaiting each embedded
		/// [`Await`](crate::types::effects::await_future::Await) future and
		/// dispatching every other first-order effect to `handlers`.
		///
		/// The await effect must be the head of the program's first-order
		/// row; `handlers` covers the remaining effects in `Rest`. The
		/// returned future is runtime-agnostic and completes only once every
		/// embedded future has, so it can be driven by any executor.
		#[document_signature]
		///
		#[document_parameters(
			"The handler list for the tail row's effects (every first-order effect other than await)."
		)]
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
		pub async fn run_async(
			self,
			handlers: impl for<'h> DispatchHandlers<
				'h,
				Apply!(<Rest as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'h, Self>),
				Self,
			>,
		) -> A {
			handle_async_with_await(self, handlers).await
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::inner::{
			AwaitRow,
			handle_async,
			handle_async_with_await,
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

	// The await-aware driver interleaves awaited futures and ordinary handler
	// dispatch in one program. The program awaits 10, hands the result to an
	// Identity handler that adds 1, then awaits a future computed from that
	// (times 2): 10 -> 11 -> 22. The await head is interpreted by the driver;
	// only the Identity tail effect goes through `handlers`.
	#[test]
	fn handle_async_with_await_interleaves_awaits_and_handlers() {
		type Rest = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
		type Prog<A> = Run<AwaitRow<Rest>, CNilBrand, A>;

		let first: Await<'static, usize> = Box::pin(async { 10 });
		let program: Prog<usize> = Run::lift::<AwaitBrand, _>(first)
			.bind(|awaited| Run::lift::<IdentityBrand, _>(Identity(awaited + 1)))
			.bind(|handled| {
				let second: Await<'static, usize> = Box::pin(async move { handled * 2 });
				Run::lift::<AwaitBrand, _>(second)
			});

		let result = block_on(handle_async_with_await(
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
	async fn handle_async_with_await_awaits_real_pending_futures() {
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

		let result = handle_async_with_await(program, handlers! {}).await;

		assert_eq!(result, 42);
	}
}
