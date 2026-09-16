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
//! the driver genuinely awaits rather than assuming readiness; (4) the
//! returned future is runtime-agnostic, driven here both by a busy-poll
//! executor and by the Tokio scheduler over real timers.
#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::AwaitBrand,
		define_row,
		types::{
			Free,
			effects::{
				await_future::{
					await_future,
					run_async,
				},
				state::{
					StateBrand,
					get,
					handle_state,
					put,
					transact_state,
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
		time::Duration,
	},
};

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

/// Interleaves awaiting with state operations: await the first value,
/// write its successor, read it back, then await the doubling. The
/// narrowing runner re-emits the `Await` cells lazily, so the state fold
/// resumes across each suspension. Threads 20 -> 21 -> 42.
fn interleaved(
	first: impl Future<Output = i32> + 'static,
	double: impl Fn(i32) -> Pin<Box<dyn Future<Output = i32>>> + 'static,
) -> Free<AppRow, i32> {
	await_future::<i32, _, _>(first)
		.bind(|first: i32| put(first + 1))
		.bind(|()| get())
		.bind(move |value: i32| await_future::<i32, _, _>(double(value)))
}

#[test]
fn a_mixed_row_narrows_through_state_and_drives_to_completion() {
	let program = interleaved(async { 20 }, |value| Box::pin(async move { value * 2 }));
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

#[test]
fn a_transactional_scope_spans_a_suspension() {
	// The transaction opens before the suspension and commits after it: the
	// local write of 10 is read back on the far side of the await (the
	// scope's accumulator rides the re-emitted continuation), and the
	// commit lands only once the driver resumes past the suspension.
	let program: Free<AppRow, i32> = put(1).bind(|()| {
		transact_state::<_, i32, _, _, _, _, _>(put(10).bind(|()| {
			await_future::<i32, _, _>(async { 32 })
				.bind(|awaited: i32| get().bind(move |local: i32| Free::pure(awaited + local)))
		}))
	});
	let narrowed: Free<AwaitRow, (i32, i32)> = handle_state(0, program);
	let (final_state, result) = block_on(run_async(narrowed));
	assert_eq!(result, 42);
	assert_eq!(final_state, 10);
}

#[tokio::test(flavor = "current_thread")]
async fn the_narrowed_composition_runs_on_tokio() {
	// The embedded futures are real timers, so completion requires the
	// Tokio scheduler to drive them, not a single poll; the same program
	// shape the busy-poll test uses runs unchanged on a real runtime.
	let program = interleaved(
		async {
			tokio::time::sleep(Duration::from_millis(1)).await;
			20
		},
		|value| {
			Box::pin(async move {
				tokio::time::sleep(Duration::from_millis(1)).await;
				value * 2
			})
		},
	);
	let narrowed: Free<AwaitRow, (i32, i32)> = handle_state(0, program);
	let (final_state, result) = run_async(narrowed).await;
	assert_eq!(result, 42);
	assert_eq!(final_state, 21);
}
