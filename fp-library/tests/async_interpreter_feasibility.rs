//! Async-interpreter feasibility spike for the effects system (throwaway).
//!
//! Motivation. The effects interpreter is synchronous and mono-in-`A`:
//! handler closures return the next program directly, not a `Future`. The
//! library targets stable Rust, where a recursive async type cannot be named
//! and there is no stack-safe recursion combinator over `Future`. That cast
//! doubt on whether an async interpreter is even expressible here. This test
//! answers the question directly: can a direct async interpreter driver loop
//! run an effect program on stable Rust, holding the program as data (the
//! `Run` / `Free` tree) across `.await` points, with no recursion combinator
//! over `Future` and without naming a recursive async type?
//!
//! Method. Drive the existing synchronous substrate (`peel` plus the real
//! `DispatchHandlers::dispatch`) from inside an `async` block, with a genuine
//! suspension point (`yield_once().await`) at each effect layer. The
//! continuation stays data (a `Run` value); only the driver is async. A
//! std-only `block_on` (no `tokio`, no `futures`) runs it, which also shows
//! the pattern needs no async-runtime dependency.
//!
//! Finding. It works (this test passes). The driver needs no recursion
//! combinator over `Future`, no named recursive async type, and not even a
//! `Pin<Box<dyn Future>>` around a recursive tail, because the loop is flat
//! and the continuation is data rather than a captured async call stack. It
//! is stack-safe at depth (the loop is iterative; peel and dispatch are O(1)
//! per step) and runtime-neutral (a std-only executor suffices).
//!
//! Scope is deliberately minimal: default `Run`, first-order only (the
//! Identity effect), single-shot, non-`Send`, no scoped effects. A companion
//! POC test exercises async-producing handlers, the scoped effect path, and
//! the `Send` / Arc family; this file stands alone and does not depend on it.
#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::{
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
		},
		handlers,
		types::{
			Identity,
			effects::{
				interpreter::DispatchHandlers,
				node::Node,
				run::Run,
			},
		},
	},
	std::{
		future::Future,
		pin::{
			Pin,
			pin,
		},
		task::{
			Context,
			Poll,
			Waker,
		},
	},
};

type FirstRow = CoproductBrand<CoyonedaBrand<IdentityBrand>, CNilBrand>;
type Scoped = CNilBrand;
type Prog<A> = Run<FirstRow, Scoped, A>;

/// Minimal std-only executor: busy-polls until ready. Sufficient because the
/// only pending source here is `YieldOnce`, which is ready on the second poll.
fn block_on<F: Future>(future: F) -> F::Output {
	let mut future = pin!(future);
	let mut context = Context::from_waker(Waker::noop());
	loop {
		if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
			return value;
		}
	}
}

/// A future that suspends exactly once, then completes. Proves the driver loop
/// genuinely yields control (returns `Poll::Pending`) at each effect layer.
struct YieldOnce(bool);

impl Future for YieldOnce {
	type Output = ();

	fn poll(
		mut self: Pin<&mut Self>,
		context: &mut Context<'_>,
	) -> Poll<()> {
		if self.0 {
			Poll::Ready(())
		} else {
			self.0 = true;
			context.waker().wake_by_ref();
			Poll::Pending
		}
	}
}

fn yield_once() -> YieldOnce {
	YieldOnce(false)
}

/// Builds a deep first-order program: `count` chained Identity effects that
/// thread a running total. Depth stresses the driver's stack safety.
fn deep_program(count: usize) -> Prog<usize> {
	let mut program: Prog<usize> = Run::pure(0);
	for _ in 0 .. count {
		program =
			program.bind(|accumulator| Run::lift::<IdentityBrand, _>(Identity(accumulator + 1)));
	}
	program
}

// The async interpreter driver: an async block whose loop peels the program,
// awaits at each first-order layer, and advances via the real dispatch. The
// program (`Run` value) is held across the `.await` the whole time.
#[test]
fn async_driver_runs_first_order_program_on_stable() {
	let depth = 2000;
	let program = deep_program(depth);
	let handlers = handlers! {
		IdentityBrand: |operation: Identity<Prog<usize>>| operation.0,
	};

	let mut suspensions = 0usize;
	let result = block_on(async {
		let mut program = program;
		loop {
			match program.peel() {
				Ok(value) => break value,
				Err(node) => match node {
					Node::First(layer) => {
						yield_once().await;
						suspensions += 1;
						program = handlers.dispatch(layer);
					}
					Node::Scoped(empty) => match empty {},
				},
			}
		}
	});

	assert_eq!(result, depth, "deep program should thread to its depth");
	assert_eq!(suspensions, depth, "driver should suspend once per effect layer",);
}
