//! Async interpreter for the effects subsystem (work in progress).
//!
//! This module is the foundation of the asynchronous effect interpreter
//! adopted as the runtime policy for the effects subsystem: a direct async
//! driver loop that peels a program, advances each layer through the existing
//! synchronous dispatch, and keeps the continuation as data (the `Free`
//! tree). Feasibility was established by throwaway spikes; this is the
//! production foundation those spikes pointed at.
//!
//! ## Status: crate-internal and intentionally retained
//!
//! [`handle_async`] is deliberately `pub(crate)` and intentionally not yet
//! wired into a public API or an effect that performs real asynchronous
//! work. It is the integration point the remaining async work plugs into,
//! and it is exercised by this module's own tests. It is NOT dead code:
//!
//! - Revisit and make it public once the requisite pieces are in place,
//!   chiefly a `Future`-embedding (base-lift) effect whose value is obtained
//!   by awaiting an embedded `Future`. At that point the marked dispatch site
//!   in the loop awaits the embedded future before advancing, which is the
//!   only thing that makes the interpreter genuinely asynchronous rather than
//!   a synchronous interpretation returned in a `Future`.
//! - Extend it across the wrapper family with the local-versus-`Send` split
//!   (local futures for the Box and Rc families, `Send` futures for the Arc
//!   family), and to scoped layers via the in-crate dispatch paths.
//!
//! Until those land, do not delete this as unused; it is a checkpoint of the
//! adopted async-interpreter direction with feasibility already proven.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::CNilBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::effects::{
				interpreter::DispatchHandlers,
				node::Node,
				run::Run,
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
	/// This intentionally returns the result in a `Future` even though it does
	/// no asynchronous work yet: the marked dispatch site below is where a
	/// `Future`-embedding effect will `.await` its embedded future once that
	/// effect exists. See the module documentation; this is a retained
	/// foundation, not dead code.
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
					// First-order effect layer. The Future-embedding effect
					// will `.await` its embedded future here before advancing.
					Node::First(layer) => program = handlers.dispatch(layer),
					// A first-order-only program has no scoped layers.
					Node::Scoped(empty) => match empty {},
				},
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::inner::handle_async,
		crate::{
			brands::{
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
				IdentityBrand,
			},
			handlers,
			types::{
				Identity,
				effects::run::Run,
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
}
