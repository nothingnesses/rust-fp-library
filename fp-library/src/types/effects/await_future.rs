//! Future base-lift effect.
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
//! The brand and its `Functor` are substrate-agnostic: the [`Await`] future
//! type and this `Functor` are the reusable piece an async driver builds on.
//! The boxed future is local (non-`Send`), so this targets single-shot
//! programs; a `Send` future shape for a thread-safe family is a later addition.

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
		#[document_examples(
			skip_call_check,
			reason = "`AwaitBrand` is crate-internal, so an external doctest cannot name it; the example shows the same map-the-future-output shape with public types."
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
		/// // Mapping a future's output is just awaiting it and applying the
		/// // function, the shape AwaitBrand's Functor impl uses:
		/// let base: Pin<Box<dyn Future<Output = i32>>> = Box::pin(async { 5 });
		/// let mapped: Pin<Box<dyn Future<Output = i32>>> = Box::pin(async move { base.await + 1 });
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
