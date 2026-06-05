//! W13 async-interpreter feasibility spike (throwaway).
//!
//! Question this answers: can a direct async interpreter driver loop run an
//! fp-library effect program on stable Rust, holding the program as data (the
//! `Run`/`Free` tree) across `.await` points, without a `MonadRec`-over-`Future`
//! impl and without naming a recursive async type?
//!
//! Approach: drive the existing synchronous substrate (`peel` + the real
//! `DispatchHandlers::dispatch`) from inside an `async` block, with a genuine
//! suspension point (`yield_once().await`) at each effect layer. The
//! continuation stays data (a `Run` value); only the driver is async. A
//! std-only `block_on` (no tokio, no `futures`) runs it, which also shows the
//! pattern needs no runtime dependency.
//!
//! Scope is deliberately minimal: default `Run`, first-order-only (Identity
//! effect), single-shot, non-`Send`. No scoped/dual-row, no multi-shot, no
//! public API. See docs/plans/effects/review-1/w13-async-spike.md.
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
