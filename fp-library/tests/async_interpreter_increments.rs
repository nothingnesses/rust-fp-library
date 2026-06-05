//! Async-interpreter feasibility POC for the effects system (throwaway).
//!
//! Motivation. A direct async interpreter driver loop (an `async` block whose
//! loop peels the program, awaits at each layer, and advances via the real
//! dispatch, keeping the program as data rather than capturing the async call
//! stack) is known to run a first-order effect program on stable Rust. This
//! POC settles the three follow-on feasibility questions, each checked on
//! stable Rust over the real substrate:
//!
//!   1. Async-producing handlers: obtaining the next program's input from
//!      awaited IO, with the awaited value flowing into the final result.
//!   2. The scoped (around-action) effect path: driving a scoped effect under
//!      the async loop via the public `peel` plus `dispatch_scoped` path.
//!   3. The Send / Arc family: the async driver on `ArcRun`, run on a spawned
//!      thread to prove the program, handlers, and driver future are all
//!      `Send + 'static`.
//!
//! Method and harness. Each test is an `async` block whose loop peels the
//! program and advances via the real dispatch, awaiting at each layer, driven
//! by a std-only `block_on` (no `tokio`, no `futures`). The continuation
//! stays data; only the driver is async. Each test below documents its own
//! construction and what its assertions prove.
//!
//! Finding. All three pass. Design note for case 1: the await happens in the
//! driver, which then advances with an ordinary synchronous handler built
//! from the awaited value. This keeps handlers synchronous and confines async
//! to the driver, which avoids the async-closure lending problem (a handler
//! returning a future that borrows captured handler state) and preserves the
//! existing synchronous handler ecosystem. So the feasible and preferable
//! shape is "await in the driver, dispatch synchronously," not "handlers
//! return futures."
#![cfg(feature = "effects")]
#![expect(
	clippy::expect_used,
	reason = "POC test uses panicking operations for brevity and clarity."
)]

use {
	fp_library::{
		brands::{
			ArcCoyonedaBrand,
			BoxBrand,
			BoxReaderBrand,
			BoxSpanBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			IdentityBrand,
		},
		handlers,
		scoped_handlers,
		types::{
			Identity,
			effects::{
				arc_run::ArcRun,
				interpreter::{
					DispatchHandlers,
					DispatchScopedHandlers,
				},
				node::Node,
				reader::BoxReader,
				run::Run,
				standard_scoped_handlers::span_handler,
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
		thread,
	},
};

// -- std-only async harness (shared with the first spike's approach) --

fn block_on<F: Future>(future: F) -> F::Output {
	let mut future = pin!(future);
	let mut context = Context::from_waker(Waker::noop());
	loop {
		if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
			return value;
		}
	}
}

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

/// Stand-in for real async IO: yields once, then returns the value. Used to
/// prove an awaited result flows into the program.
async fn fetch(value: i32) -> i32 {
	YieldOnce(false).await;
	value
}

// -- 1. Async-producing handlers --------------------------------------------
//
// The handler awaits `fetch(..)` to obtain the Reader environment, then feeds
// it to the suspended continuation via `BoxReader::Ask(k) => k(env)`. The
// awaited values (10, then 20) flow into the program's result (30).

type ReaderRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type ReaderProg = Run<ReaderRow, CNilBrand, i32>;

#[test]
fn async_producing_handler_threads_awaited_values() {
	let program: ReaderProg = Run::<ReaderRow, CNilBrand, i32>::ask().bind(|first: i32| {
		Run::<ReaderRow, CNilBrand, i32>::ask().bind(move |second: i32| Run::pure(first + second))
	});

	let mut fetched = Vec::new();
	let mut next_value = 0;
	let result = block_on(async {
		let mut program = program;
		loop {
			match program.peel() {
				Ok(value) => break value,
				Err(node) => match node {
					Node::First(layer) => {
						next_value += 10;
						// The awaited value is produced asynchronously here.
						let environment = fetch(next_value).await;
						fetched.push(environment);
						let handlers = handlers! {
							BoxReaderBrand<BoxBrand, i32>:
								move |operation: BoxReader<'_, BoxBrand, i32, ReaderProg>| match operation {
									BoxReader::Ask(continuation) => continuation(environment),
								},
						};
						program = handlers.dispatch(layer);
					}
					Node::Scoped(empty) => match empty {},
				},
			}
		}
	});

	assert_eq!(fetched, vec![10, 20], "both asks resolved via awaited IO");
	assert_eq!(result, 30, "awaited values threaded into the result");
}

// -- 2. Scoped / dual-row path ----------------------------------------------
//
// Drives a scoped Span effect under the async loop via the public
// `peel` + `DispatchScopedHandlers::dispatch_scoped` path, awaiting at the
// scoped layer. Proves a scoped layer is held across `.await` and dispatched.

#[derive(Debug, PartialEq, Eq)]
struct NonCloneTag(&'static str);

type SpanScopedRow = CoproductBrand<BoxSpanBrand<BoxBrand, NonCloneTag>, CNilBrand>;
type SpanProg = Run<CNilBrand, SpanScopedRow, i32>;

#[test]
fn async_driver_runs_scoped_span() {
	let action: SpanProg = Run::pure(42);
	let program: SpanProg = Run::span::<NonCloneTag, _>(NonCloneTag("request"), action);

	let first_order = handlers! {};
	let scoped = scoped_handlers! {
		BoxSpanBrand<BoxBrand, NonCloneTag>: span_handler(),
	};

	let mut scoped_suspensions = 0usize;
	let result = block_on(async {
		let mut program = program;
		loop {
			match program.peel() {
				Ok(value) => break value,
				Err(node) => match node {
					Node::First(empty) => match empty {},
					Node::Scoped(layer) => {
						YieldOnce(false).await;
						scoped_suspensions += 1;
						program = scoped.dispatch_scoped(layer, &first_order);
					}
				},
			}
		}
	});

	assert_eq!(result, 42, "span resumes its action unchanged");
	assert!(scoped_suspensions >= 1, "driver suspended at the scoped layer");
}

// -- 3. Send / Arc family ---------------------------------------------------
//
// The async driver on ArcRun, run on a spawned thread. If this compiles and
// runs, the program, handlers, and the driver future are all `Send + 'static`.

type ArcRow = CoproductBrand<ArcCoyonedaBrand<IdentityBrand>, CNilBrand>;
type ArcProg = ArcRun<ArcRow, CNilBrand, usize>;

fn arc_deep_program(count: usize) -> ArcProg {
	let mut program: ArcProg = ArcRun::pure(0);
	for _ in 0 .. count {
		program =
			program.bind(|accumulator| ArcRun::lift::<IdentityBrand, _>(Identity(accumulator + 1)));
	}
	program
}

async fn run_arc_async(program: ArcProg) -> usize {
	let handlers = handlers! {
		IdentityBrand: |operation: Identity<ArcProg>| operation.0,
	};
	let mut program = program;
	loop {
		match program.peel() {
			Ok(value) => break value,
			Err(node) => match node {
				Node::First(layer) => {
					YieldOnce(false).await;
					program = handlers.dispatch(layer);
				}
				Node::Scoped(empty) => match empty {},
			},
		}
	}
}

#[test]
fn async_driver_arc_family_is_send() {
	let program = arc_deep_program(100);
	let handle = thread::spawn(move || block_on(run_arc_async(program)));
	let result = handle.join().expect("worker thread should not panic");
	assert_eq!(result, 100, "Arc-family async driver ran across a thread boundary");
}
