//! Async-interpreter feasibility: finer remainders (throwaway).
//!
//! Motivation. Earlier feasibility tests showed a direct async interpreter
//! driver loop (an `async` block that peels the program, awaits at each
//! layer, and advances via the real dispatch, keeping the program as data)
//! runs first-order programs, async-producing handlers, a simple scoped
//! effect, and the `Send` / Arc family on stable Rust. This file closes two
//! remaining feasibility questions reachable through the public API:
//!
//!   1. Combined scoped plus Arc. The Arc family carries `Send + Sync`
//!      bounds; this drives a scoped `Span` program on `ArcRun` on a spawned
//!      thread, which compiles only if the scoped carriers, program, and
//!      driver future are all `Send + 'static`.
//!   2. Real runtime integration. The earlier tests used a std-only
//!      busy-polling `block_on` with a yield-once future. This drives the
//!      interpreter on the Tokio runtime with real async IO
//!      (`tokio::time::sleep`) at the await point, confirming the pattern
//!      works with a concrete ecosystem runtime, not just a hand-written
//!      executor.
//!
//! A third remainder, carrier-based scoped effects (`Local`, `Catch`,
//! `Bracket`) under async, is covered by an in-crate test rather than here:
//! on the non-explicit wrappers those effects dispatch through a
//! crate-private raw scoped path, not the public `dispatch_scoped` used for
//! the witness-free `Span`, so an external integration test cannot reach
//! them. The async-ness is identical either way (it lives in the driver,
//! around a synchronous dispatch call).
//!
//! Method. Each test is an `async` driver loop that peels the program and
//! advances via the real dispatch (`DispatchHandlers::dispatch` for
//! first-order layers, `DispatchScopedHandlers::dispatch_scoped` for
//! witness-free scoped layers), awaiting at each layer. The continuation
//! stays data; only the driver is async. Test 1 uses a std-only `block_on`;
//! test 2 uses Tokio.
#![cfg(feature = "effects")]
#![expect(
	clippy::expect_used,
	reason = "POC test uses panicking operations for brevity and clarity."
)]

use {
	fp_library::{
		brands::{
			ArcBrand,
			BoxBrand,
			BoxReaderBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
			SendSpanBrand,
		},
		handlers,
		scoped_handlers,
		types::effects::{
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
		time::Duration,
	},
};

// -- std-only async harness for test 1 --

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

// -- 1. Combined scoped plus Arc --------------------------------------------
//
// A scoped Span program on the Arc family, driven on a spawned thread. The
// `thread::spawn` only compiles if the program, scoped handlers, and driver
// future are all `Send + 'static`, so success proves the scoped carriers are
// thread-safe under async.

type ArcSpanScopedRow = CoproductBrand<SendSpanBrand<ArcBrand, String>, CNilBrand>;
type ArcSpanProg = ArcRun<CNilBrand, ArcSpanScopedRow, i32>;

#[test]
fn scoped_arc_family_under_async_crosses_thread() {
	let action: ArcSpanProg = ArcRun::pure(42);
	let program: ArcSpanProg = ArcRun::span::<String, _>("request".to_owned(), action);

	let result = thread::spawn(move || {
		let first_order = handlers! {};
		let scoped = scoped_handlers! {
			SendSpanBrand<ArcBrand, String>: span_handler(),
		};
		block_on(async {
			let mut program = program;
			loop {
				match program.peel() {
					Ok(value) => break value,
					Err(node) => match node {
						Node::First(empty) => match empty {},
						Node::Scoped(layer) => {
							YieldOnce(false).await;
							program = scoped.dispatch_scoped(layer, &first_order);
						}
					},
				}
			}
		})
	})
	.join()
	.expect("worker thread should not panic");

	assert_eq!(result, 42, "Arc-family scoped Span ran across a thread boundary");
}

// -- 2. Real runtime integration (Tokio) ------------------------------------
//
// The async driver on the Tokio runtime, with a real async IO await
// (`tokio::time::sleep`) at each layer producing the Reader environment. The
// awaited values (10, then 20) thread into the result (30). Uses the
// current-thread flavour because the default `Run` program is not `Send`.

type ReaderRow = CoproductBrand<CoyonedaBrand<BoxReaderBrand<BoxBrand, i32>>, CNilBrand>;
type ReaderProg = Run<ReaderRow, CNilBrand, i32>;

#[tokio::test(flavor = "current_thread")]
async fn real_runtime_tokio_drives_async_interpreter() {
	let program: ReaderProg = Run::<ReaderRow, CNilBrand, i32>::ask().bind(|first: i32| {
		Run::<ReaderRow, CNilBrand, i32>::ask().bind(move |second: i32| Run::pure(first + second))
	});

	let mut next_value = 0;
	let mut program = program;
	let result = loop {
		match program.peel() {
			Ok(value) => break value,
			Err(node) => match node {
				Node::First(layer) => {
					next_value += 10;
					// Real async IO on a concrete runtime.
					tokio::time::sleep(Duration::from_millis(1)).await;
					let environment = next_value;
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
	};

	assert_eq!(result, 30, "interpreter ran on Tokio with real async IO");
}
