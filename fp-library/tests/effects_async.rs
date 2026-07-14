//! The async terminal driver, driven end to end: `await_future` embeds a
//! future into a program's row as an `Await` cell, and `run_async` drives a
//! program whose row's only cell is `Await`, awaiting each lowered future
//! with the continuation held as data.
//!
//! Pinned here: (1) the driver is generic over any one-cell `Await` row
//! (the uninject remainder is the empty coproduct, so the no-match arm is
//! unreachable by construction); (2) mixed rows compose through the
//! narrowing tier with no async context: a narrowing runner re-emits
//! unmatched `Await` cells lazily, the rest of its fold riding inside the
//! re-emitted continuation, so state threads across suspensions; (3) a
//! future that suspends before completing is re-polled to completion, so
//! the driver genuinely awaits rather than assuming readiness.
#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::AwaitBrand,
		classes::{
			Functor,
			WrapDrop,
		},
		define_row,
		kinds::LifetimeUnaryKind,
		types::{
			Coyoneda,
			Free,
			effects::{
				await_future::Await,
				coproduct::{
					CNil,
					CoprodInjector,
					CoprodUninjector,
				},
				state::{
					StateBrand,
					get,
					handle_state,
					put,
				},
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

// -- The proof-of-concept generic surface (the shape to graduate to src) --

/// Embeds a future into a program's row as an `Await` cell.
fn await_future<A, R, I>(future: impl Future<Output = A> + 'static) -> Free<R, A>
where
	A: 'static,
	R: Functor + WrapDrop + 'static,
	<R as LifetimeUnaryKind>::Of<'static, A>: CoprodInjector<Coyoneda<'static, AwaitBrand, A>, I>, {
	let boxed: Await<'static, A> = Box::pin(future);
	let coyo: Coyoneda<'static, AwaitBrand, A> = Coyoneda::lift(boxed);
	let node: <R as LifetimeUnaryKind>::Of<'static, A> = CoprodInjector::inject(coyo);
	Free::lift_f(node)
}

/// Drives a program whose row's only cell is `Await`: peel a layer, lower
/// the awaited future, await it, and continue with the produced program.
async fn run_async<Row, A, UninjectIndex>(program: Free<Row, A>) -> A
where
	Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
	A: 'static,
	UninjectIndex: 'static,
	<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>: CoprodUninjector<
			Coyoneda<'static, AwaitBrand, Free<Row, A>>,
			UninjectIndex,
			Remainder = CNil,
		>, {
	let mut program = program;
	loop {
		let layer = match program.resume() {
			Ok(value) => return value,
			Err(layer) => layer,
		};
		match layer.uninject() {
			Ok(coyo) => {
				let future: Await<'static, Free<Row, A>> = coyo.lower();
				program = future.await;
			}
			Err(remainder) => match remainder {},
		}
	}
}

// -- Rows and the trivial executor --

define_row! {
	/// A future base-lift cell alongside integer state.
	pub row AppRow {
		AwaitBrand,
		StateBrand<i32>,
	}
}

define_row! {
	/// The residual after the state cell is eliminated.
	pub row AwaitRow {
		AwaitBrand,
	}
}

/// Polls a runtime-agnostic future to completion on a busy-poll executor.
fn block_on<F: Future>(future: F) -> F::Output {
	let mut future = pin!(future);
	let mut context = Context::from_waker(Waker::noop());
	loop {
		if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
			break value;
		}
	}
}

/// A future that suspends once before completing, so driving it to a value
/// proves the driver re-polls rather than assuming readiness.
struct YieldOnce {
	polled: bool,
	value: i32,
}

impl Future for YieldOnce {
	type Output = i32;

	fn poll(
		mut self: Pin<&mut Self>,
		context: &mut Context<'_>,
	) -> Poll<i32> {
		if self.polled {
			Poll::Ready(self.value)
		} else {
			self.polled = true;
			context.waker().wake_by_ref();
			Poll::Pending
		}
	}
}

// -- The oracles --

#[test]
fn a_mixed_row_narrows_through_state_and_drives_to_completion() {
	// The program interleaves awaiting with state operations: await 20,
	// write 21, read it back, then await the doubling. The narrowing runner
	// re-emits the `Await` cells lazily, so the state fold resumes across
	// each suspension.
	let program: Free<AppRow, i32> = await_future::<i32, _, _>(async { 20 })
		.bind(|first: i32| put(first + 1))
		.bind(|()| get())
		.bind(|value: i32| await_future::<i32, _, _>(async move { value * 2 }));
	let narrowed: Free<AwaitRow, (i32, i32)> = handle_state(0, program);
	let (final_state, result) = block_on(run_async(narrowed));
	assert_eq!(result, 42);
	assert_eq!(final_state, 21);
}

#[test]
fn a_suspending_future_is_repolled_to_completion() {
	let program: Free<AwaitRow, i32> = await_future::<i32, _, _>(YieldOnce {
		polled: false,
		value: 41,
	})
	.bind(|value: i32| Free::pure(value + 1));
	assert_eq!(block_on(run_async(program)), 42);
}
