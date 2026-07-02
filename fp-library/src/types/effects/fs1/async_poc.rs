//! FS-1 slice: the async re-point proof-of-concept.
//!
//! This module is a build-and-run proof that the continuation-as-data
//! async driver re-points onto the FS-1 `Free<Row, A>` substrate. The deleted
//! dual-row async driver stepped its program one layer at a time, projected
//! the `Await` future-lift effect out of its row, lowered the matched cell to
//! a future of the next program and `.await`ed it, and dispatched every other
//! effect to the handlers, keeping the continuation as data across each
//! suspension. The substrate-specific couplings are exactly three: step one
//! layer, project the await brand, and lower the suspended future. The FS-1
//! substrate already provides all three as `Free::resume`, the row's `uninject`,
//! and `Coyoneda::lower`, and because the whole program is an owned `Free` tree,
//! holding it across an `.await` is sound (no borrow spans the suspension).
//!
//! The POC reuses the real `AwaitBrand` (its `Functor` over a boxed future is
//! substrate-agnostic; only the deleted driver machinery was dual-row) over
//! a one-effect FS-1 row, and drives a program that embeds a future and then
//! binds a pure continuation, on a trivial poll-to-completion executor. A green
//! test here is the evidence that async carries forward onto FS-1 unchanged in
//! mechanism; it also seeds the eventual async reintroduction.

use {
	crate::{
		Apply,
		brands::{
			AwaitBrand,
			CNilBrand,
			CoproductBrand,
			CoyonedaBrand,
		},
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::{
				await_future::Await,
				coproduct::Coproduct,
			},
		},
	},
	std::future::Future,
};

/// A one-effect FS-1 row carrying only the future-lift effect, enough to drive
/// the await mechanism over the kept `Free` substrate.
type AwaitRow = CoproductBrand<CoyonedaBrand<AwaitBrand>, CNilBrand>;

/// The row cell over a result `A`, as handed to `Free::lift_f`.
type AwaitNode<A> = Apply!(<AwaitRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>);

/// Embed a future into a program's row as an `Await` cell, mirroring the
/// deleted dual row's future-embedding constructor but producing an FS-1
/// `Free<AwaitRow, A>`.
fn await_future<A: 'static>(future: impl Future<Output = A> + 'static) -> Free<AwaitRow, A> {
	let boxed: Await<'static, A> = Box::pin(future);
	let coyo: Coyoneda<'static, AwaitBrand, A> = Coyoneda::lift(boxed);
	let node: AwaitNode<A> = Coproduct::inject(coyo);
	Free::lift_f(node)
}

/// The async driver, re-pointed onto FS-1: peel a layer with `resume`, project
/// the await cell with `uninject`, lower it to a future of the next program and
/// `.await` it, keeping the continuation `Free<AwaitRow, A>` as data across the
/// suspension. The one-effect row's remainder is the uninhabited terminal row,
/// so the no-match arm is unreachable; a fuller row would dispatch it.
async fn drive<A: 'static>(mut program: Free<AwaitRow, A>) -> A {
	loop {
		let layer = match program.resume() {
			Ok(value) => return value,
			Err(layer) => layer,
		};
		let selected: Result<Coyoneda<'static, AwaitBrand, Free<AwaitRow, A>>, _> =
			layer.uninject();
		match selected {
			Ok(coyo) => {
				let future: Await<'static, Free<AwaitRow, A>> = coyo.lower();
				program = future.await;
			}
			Err(remainder) => match remainder {},
		}
	}
}

#[cfg(test)]
mod tests {
	use {
		super::{
			AwaitRow,
			await_future,
			drive,
		},
		crate::types::Free,
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

	// Poll a runtime-agnostic future to completion on a trivial executor.
	fn block_on<F: Future<Output = i32>>(future: F) -> i32 {
		let mut future = pin!(future);
		let mut context = Context::from_waker(Waker::noop());
		loop {
			if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
				break value;
			}
		}
	}

	// A program that embeds a future and binds a pure continuation drives to
	// completion over the FS-1 `Free` substrate, awaiting the embedded future and
	// threading the continuation as data: 41 awaited, then +1 -> 42.
	#[test]
	fn await_repoints_onto_the_fs1_free_substrate() {
		let program: Free<AwaitRow, i32> =
			await_future(async { 41 }).bind(|value| Free::<AwaitRow, i32>::pure(value + 1));
		assert_eq!(block_on(drive(program)), 42);
	}
}
