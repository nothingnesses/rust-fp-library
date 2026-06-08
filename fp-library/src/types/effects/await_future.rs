//! Future base-lift effect for the async interpreter.
//!
//! This is the W13 `Future`-embedding (base-lift) effect: the piece that lets
//! a program embed a `Future` so the async interpreter can await it. The
//! effect brand [`AwaitBrand`](crate::brands::AwaitBrand) is a
//! [`Functor`](crate::classes::Functor) over a boxed future, which is what
//! makes the design work: an await effect lifted into the first-order row is
//! a `Coyoneda<AwaitBrand, _>`, and because the brand is a `Functor`, the
//! interpreter lowers it to a future of the next program and awaits that,
//! with no type-erased resume queue.
//!
//! The public surface is the [`Await`] future type and the
//! [`Run::await_future`](crate::types::effects::run::Run::await_future) smart
//! constructor that embeds a `Future` into a program's first-order row. The
//! async driver that interprets the effect currently lives crate-internally,
//! pending its public-surface increment. The boxed future is local
//! (non-`Send`), so this targets the single-shot `Box` `Run` family; a `Send`
//! future shape for the `Arc` family is a later addition.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			brands::AwaitBrand,
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::*,
			types::{
				Coyoneda,
				effects::{
					member::Member,
					run::Run,
				},
			},
		},
		core::{
			future::Future,
			pin::Pin,
		},
		fp_macros::*,
	};

	/// A boxed, pinned future carrying the value an `await` effect produces.
	///
	/// Local (non-`Send`) by design: this is the value shape for the
	/// single-shot Box wrappers. The `Send` / Arc family will use a separate
	/// future shape when that increment lands.
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

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type produced by the embedded future."
	)]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Embeds a [`Future`] into this program's first-order row as an
		/// [`Await`] effect, so an async interpreter can await it and feed the
		/// produced value to the continuation.
		///
		/// The program's row `R` must contain the
		/// [`AwaitBrand`](crate::brands::AwaitBrand) effect. The future is
		/// local (non-`Send`) and boxed, so this targets the single-shot `Box`
		/// `Run` family. The program is interpreted by the crate's async
		/// driver, which awaits the embedded future and advances the program.
		#[document_signature]
		///
		#[document_type_parameters(
			"The type-level Member-position witness for the await effect in the row (typically inferred)."
		)]
		///
		#[document_parameters("The future to embed; its output feeds the continuation.")]
		///
		#[document_returns("A `Run` program suspended at the embedded future.")]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::{
		/// 		AwaitBrand,
		/// 		CNilBrand,
		/// 		CoproductBrand,
		/// 		CoyonedaBrand,
		/// 	},
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<AwaitBrand>, CNilBrand>;
		/// type Prog<A> = Run<Row, CNilBrand, A>;
		///
		/// // Embed a future; the awaited value feeds the continuation.
		/// let program: Prog<i32> = Run::await_future(async { 41 }).bind(|value| Run::pure(value + 1));
		///
		/// // The program is suspended at the await effect until an async
		/// // interpreter drives it.
		/// assert!(program.peel().is_err());
		/// ```
		pub fn await_future<Idx>(future: impl Future<Output = A> + 'static) -> Self
		where
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, A>):
				Member<Coyoneda<'static, AwaitBrand, A>, Idx>, {
			let boxed: Await<'static, A> = Box::pin(future);
			Self::lift::<AwaitBrand, Idx>(boxed)
		}
	}
}

pub use inner::Await;

#[cfg(test)]
mod tests {
	use {
		super::inner::Await,
		crate::{
			brands::{
				AwaitBrand,
				CNilBrand,
				CoproductBrand,
				CoyonedaBrand,
			},
			types::{
				Coyoneda,
				effects::{
					member::Member,
					node::Node,
					run::Run,
				},
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

	fn block_on<F: Future>(future: F) -> F::Output {
		let mut future = pin!(future);
		let mut context = Context::from_waker(Waker::noop());
		loop {
			if let Poll::Ready(value) = future.as_mut().poll(&mut context) {
				return value;
			}
		}
	}

	// Extraction spike: confirms the load-bearing path for the async
	// interpreter. An `await` effect lifted into the row is
	// `Coyoneda<AwaitBrand, _>`; because `AwaitBrand` is a `Functor`, the
	// interpreter can `lower` it back to a future (with the row's mapping
	// applied) and await that future to obtain the next program. Here the
	// mapping is `+ 1`, so awaiting the lowered future yields 6.
	#[test]
	fn coyoneda_await_lowers_to_a_future_and_awaits() {
		let base: Await<'static, i32> = Box::pin(async { 5 });
		let coyoneda: Coyoneda<'static, AwaitBrand, i32> = Coyoneda::lift(base);
		let mapped = coyoneda.map(|value| value + 1);
		let lowered = mapped.lower();
		assert_eq!(block_on(lowered), 6);
	}

	// End-to-end mechanism spike for the async interpreter's await step,
	// exercising everything `handle_async` will do at an await layer except
	// the surrounding driver loop. An `Await` effect is lifted into a row, the
	// program is peeled to its `Node::First` dispatch layer, the `AwaitBrand`
	// entry is projected out of the row, lowered to a future of the next
	// program, awaited to obtain that program, and that program is peeled to
	// its value. The bare lift's next program is `pure(7)`, so the value is 7.
	#[test]
	fn await_effect_projects_lowers_and_awaits_to_next_program() {
		type Row = CoproductBrand<CoyonedaBrand<AwaitBrand>, CNilBrand>;
		type Prog<A> = Run<Row, CNilBrand, A>;

		let base: Await<'static, i32> = Box::pin(async { 7 });
		let program: Prog<i32> = Run::lift::<AwaitBrand, _>(base);

		let result = match program.peel() {
			Ok(value) => Some(value),
			Err(node) => match node {
				Node::First(layer) => {
					let projected: Result<Coyoneda<'static, AwaitBrand, Prog<i32>>, _> =
						layer.project();
					match projected {
						Ok(coyoneda) => {
							let future = coyoneda.lower();
							let next: Prog<i32> = block_on(future);
							next.peel().ok()
						}
						Err(_remainder) => None,
					}
				}
				Node::Scoped(empty) => match empty {},
			},
		};

		assert_eq!(result, Some(7));
	}
}
