//! Future base-lift effect and its terminal async driver.
//!
//! This is the `Future`-embedding (base-lift) effect: the piece that lets a
//! program embed a `Future` so an async driver can await it. The effect brand
//! [`AwaitBrand`](crate::brands::AwaitBrand) is a
//! [`Functor`](crate::classes::Functor) over a boxed future, which is what
//! makes the design work: an await effect lifted into a first-order row is a
//! `Coyoneda<AwaitBrand, _>`, and because the brand is a `Functor`, an async
//! driver lowers it to a future of the next program and awaits that, with no
//! type-erased resume queue.
//!
//! [`await_future`] embeds a future into any row holding the `Await` cell,
//! and [`run_async`] is the terminal driver, the async sibling of
//! [`extract`](crate::types::effects::handle::extract): it drives a program
//! whose row's only cell is `Await`, awaiting each lowered future with the
//! continuation held as data. Mixed rows compose through the narrowing tier
//! with no async context: a narrowing runner re-emits unmatched `Await`
//! cells lazily (the rest of its fold rides inside the re-emitted
//! continuation), so stacking runners over a mixed row leaves the
//! `Await`-only residual this driver finishes. The boxed future is local
//! (non-`Send`), so this targets single-shot programs; a `Send` future shape
//! for a thread-safe family is a later addition.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::AwaitBrand,
			classes::Functor,
			kinds::*,
		},
		core::{
			future::Future,
			pin::Pin,
		},
		fp_macros::*,
	};

	/// A boxed, pinned future carrying the value an `await` effect produces.
	///
	/// Local (non-`Send`) by design: this is the value shape for single-shot
	/// programs. A `Send` future shape will be added when a thread-safe family
	/// lands.
	pub type Await<'a, A> = Pin<Box<dyn Future<Output = A> + 'a>>;

	impl_kind! {
		for AwaitBrand {
			type Of<'a, A: 'a>: 'a = Await<'a, A>;
		}
	}

	impl Functor for AwaitBrand {
		/// Maps the eventual output of the boxed future, returning a future
		/// that awaits the original and applies the function to its result.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime bounding the future and the mapping function.",
			"The original future's output type.",
			"The mapped output type."
		)]
		///
		#[document_parameters(
			"The function to apply to the future's eventual output.",
			"The boxed future to map over."
		)]
		///
		#[document_returns("A boxed future that yields the mapped output.")]
		///
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		brands::AwaitBrand,
		/// 		classes::Functor,
		/// 		types::effects::await_future::Await,
		/// 	},
		/// 	std::{
		/// 		pin::pin,
		/// 		task::{
		/// 			Context,
		/// 			Poll,
		/// 			Waker,
		/// 		},
		/// 	},
		/// };
		///
		/// // Mapping a future's output awaits it and applies the function.
		/// let base: Await<'static, i32> = Box::pin(async { 5 });
		/// let mapped: Await<'static, i32> = <AwaitBrand as Functor>::map(|value| value + 1, base);
		///
		/// let mut future = pin!(mapped);
		/// let mut context = Context::from_waker(Waker::noop());
		/// let result = loop {
		/// 	if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
		/// 		break value;
		/// 	}
		/// };
		/// assert_eq!(result, 6);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			func: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			Box::pin(async move { func(fa.await) })
		}
	}
}

pub use inner::Await;

#[fp_macros::document_module]
mod runner {
	use {
		super::Await,
		crate::{
			brands::AwaitBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::LifetimeUnaryKind,
			types::{
				Coyoneda,
				Free,
				effects::coproduct::{
					CNil,
					CoprodInjector,
					CoprodUninjector,
				},
			},
		},
		fp_macros::*,
		std::future::Future,
	};

	/// Embeds a future into a program's row as an `Await` cell: the
	/// program suspends at the future, and an async driver awaits it to
	/// produce the value the continuation resumes with.
	#[document_signature]
	///
	#[document_type_parameters(
		"The future's output type.",
		"The row brand the program runs over.",
		"The coproduct index locating the `Await` cell (inferred)."
	)]
	///
	#[document_parameters("The future to embed.")]
	///
	#[document_returns("The one-operation program suspending at the embedded future.")]
	///
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::AwaitBrand,
	/// 		define_row,
	/// 		types::{
	/// 			Free,
	/// 			effects::await_future::{
	/// 				await_future,
	/// 				run_async,
	/// 			},
	/// 		},
	/// 	},
	/// 	std::{
	/// 		pin::pin,
	/// 		task::{
	/// 			Context,
	/// 			Poll,
	/// 			Waker,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row holding the future base-lift effect.
	/// 	pub row AwaitRow {
	/// 		AwaitBrand,
	/// 	}
	/// }
	///
	/// let program: Free<AwaitRow, i32> =
	/// 	await_future::<i32, _, _>(async { 41 }).bind(|value: i32| Free::pure(value + 1));
	///
	/// let mut future = pin!(run_async(program));
	/// let mut context = Context::from_waker(Waker::noop());
	/// let result = loop {
	/// 	if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
	/// 		break value;
	/// 	}
	/// };
	/// assert_eq!(result, 42);
	/// ```
	pub fn await_future<A, R, I>(future: impl Future<Output = A> + 'static) -> Free<R, A>
	where
		A: 'static,
		R: Functor + WrapDrop + 'static,
		<R as LifetimeUnaryKind>::Of<'static, A>:
			CoprodInjector<Coyoneda<'static, AwaitBrand, A>, I>, {
		let boxed: Await<'static, A> = Box::pin(future);
		let coyo: Coyoneda<'static, AwaitBrand, A> = Coyoneda::lift(boxed);
		let node: <R as LifetimeUnaryKind>::Of<'static, A> = CoprodInjector::inject(coyo);
		Free::lift_f(node)
	}

	/// Drives a program whose row's only cell is `Await` to completion:
	/// each layer is lowered to a future of the next program and awaited,
	/// with the continuation held as data across the suspension. The
	/// terminal driver of an async stack, the async sibling of
	/// [`extract`](crate::types::effects::handle::extract): narrowing
	/// runners eliminate every other effect first (they re-emit unmatched
	/// `Await` cells lazily, so they need no async context), and this
	/// driver finishes the `Await`-only residual. The returned future is
	/// runtime-agnostic; any executor drives it.
	#[document_signature]
	///
	#[document_type_parameters(
		"The row brand, whose only cell is the `Await` effect.",
		"The program's result type.",
		"The coproduct index locating the `Await` cell (inferred)."
	)]
	///
	#[document_parameters("The program to drive.")]
	///
	#[document_returns("The program's final value, once every embedded future has completed.")]
	///
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		brands::AwaitBrand,
	/// 		define_row,
	/// 		types::{
	/// 			Free,
	/// 			effects::await_future::{
	/// 				await_future,
	/// 				run_async,
	/// 			},
	/// 		},
	/// 	},
	/// 	std::{
	/// 		pin::pin,
	/// 		task::{
	/// 			Context,
	/// 			Poll,
	/// 			Waker,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row holding the future base-lift effect.
	/// 	pub row AwaitRow {
	/// 		AwaitBrand,
	/// 	}
	/// }
	///
	/// // Two suspensions threaded through one continuation chain.
	/// let program: Free<AwaitRow, i32> = await_future::<i32, _, _>(async { 20 })
	/// 	.bind(|first: i32| await_future::<i32, _, _>(async move { first * 2 }))
	/// 	.bind(|second: i32| Free::pure(second + 2));
	///
	/// let mut future = pin!(run_async(program));
	/// let mut context = Context::from_waker(Waker::noop());
	/// let result = loop {
	/// 	if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
	/// 		break value;
	/// 	}
	/// };
	/// assert_eq!(result, 42);
	/// ```
	pub async fn run_async<Row, A, UninjectIndex>(program: Free<Row, A>) -> A
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
}

pub use runner::*;

#[cfg(test)]
mod tests {
	use {
		super::inner::Await,
		crate::{
			brands::AwaitBrand,
			types::Coyoneda,
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

	fn block_on<F: Future>(future: F) -> F::Output {
		let mut future = pin!(future);
		let mut context = Context::from_waker(Waker::noop());
		loop {
			if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
				return value;
			}
		}
	}

	// An `await` effect lifted into the row is `Coyoneda<AwaitBrand, _>`;
	// because `AwaitBrand` is a `Functor`, a driver can `lower` it back to a
	// future (with the row's mapping applied) and await that future to obtain
	// the next program. Here the mapping is `+ 1`, so awaiting the lowered
	// future yields 6.
	#[test]
	fn coyoneda_await_lowers_to_a_future_and_awaits() {
		let base: Await<'static, i32> = Box::pin(async { 5 });
		let coyoneda: Coyoneda<'static, AwaitBrand, i32> = Coyoneda::lift(base);
		let mapped = coyoneda.map(|value| value + 1);
		let lowered = mapped.lower();
		assert_eq!(block_on(lowered), 6);
	}
}
