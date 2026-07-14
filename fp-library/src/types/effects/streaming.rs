//! The streaming vocabulary over the `Coroutine` effect: pin aliases naming
//! the producer and consumer roles, and the fusion combinators that connect
//! them, the purescript-run-streaming shape.
//!
//! One functor underlies everything: a [`Yield`] cell emits a value and
//! resumes with unit, an [`Await`] cell emits unit and resumes with a value,
//! and both are pins of [`Coroutine`](super::coroutine). Row shapes follow
//! from the cells: a row holding a `Yield<X>` cell is a producer row, one
//! holding an `Await<X>` cell is a consumer row, and one holding both is a
//! transformer row (the two pins are distinct brand types, so they share a
//! row without collision). The fusion primitives work on the step runner's
//! [`Resume`](super::coroutine::Resume): [`interleave`] alternately feeds
//! each side's output to the
//! other side's continuation, ending with whichever side completes first,
//! and [`substitute`] replaces each yield with an effect program whose
//! result feeds back in (which needs a value-for-value pin: a `Yield<X>`
//! cell resumes with unit, so there is nothing to feed back). [`connect`]
//! and [`for_each`] are the named conveniences over them.
//!
//! The [`Await`] alias here is streaming vocabulary (a coroutine pin) and is
//! unrelated to the future base-lift effect
//! [`await_future::Await`](super::await_future::Await), which lifts a boxed
//! future into a row for an async driver.

#[fp_macros::document_module]
mod vocabulary {
	use {
		super::super::coroutine::{
			CoroutineBrand,
			Resume,
			handle_coroutine,
			yield_value,
		},
		crate::{
			classes::{
				Functor,
				WrapDrop,
			},
			kinds::LifetimeUnaryKind,
			types::{
				Coyoneda,
				Free,
				effects::coproduct::{
					CoprodInjector,
					CoprodUninjector,
					CoproductEmbedder,
				},
			},
		},
		fp_macros::*,
	};

	/// A producer cell: yields an `X` downstream, resumes with unit.
	#[document_type_parameters("The yielded value type.")]
	pub type Yield<X> = CoroutineBrand<X, ()>;

	/// A consumer cell: yields unit, resumes with an `X` from upstream.
	#[document_type_parameters("The awaited value type.")]
	pub type Await<X> = CoroutineBrand<(), X>;

	/// Awaits a value from upstream: the [`Await`] pin's constructor, with
	/// the pin's unit payload hidden.
	#[document_signature]
	///
	#[document_type_parameters(
		"The awaited value type.",
		"The row brand the program runs over.",
		"The coproduct index locating the await cell (inferred)."
	)]
	///
	#[document_returns("The one-operation program awaiting a value.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			handle::extract,
	/// 			streaming::{
	/// 				Await,
	/// 				Yield,
	/// 				await_value,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell consumer row.
	/// 	pub row TakeRow {
	/// 		Await<i32>,
	/// 	}
	/// }
	///
	/// // The consumer suspends on its await until fused with a producer.
	/// let consumer: Free<TakeRow, i32> = await_value().bind(|got: i32| Free::pure(got + 1));
	/// assert!(consumer.resume().is_err());
	/// ```
	pub fn await_value<X, R, I>() -> Free<R, X>
	where
		X: 'static,
		R: Functor + WrapDrop + 'static,
		<R as LifetimeUnaryKind>::Of<'static, X>: CoprodInjector<Coyoneda<'static, Await<X>, X>, I>, {
		yield_value::<(), X, R, I>(())
	}

	/// Alternately feeds each side's output to the other side's
	/// continuation, ending with whichever side completes first. The two
	/// sides swap roles at every exchange: `step` yields `Out` and resumes
	/// with `In`, while `other` consumes that `Out` and yields the next
	/// `In`.
	#[document_signature]
	///
	#[document_type_parameters(
		"The residual row brand both sides' steps run over.",
		"The fused result type.",
		"What `step` resumes with (and `other` yields).",
		"What `step` yields (and `other` consumes)."
	)]
	///
	#[document_parameters(
		"The other side, entered with this side's first output.",
		"This side's current step."
	)]
	///
	#[document_returns("The fused program over the residual row.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			coroutine::{
	/// 				Resume,
	/// 				handle_coroutine,
	/// 				yield_value,
	/// 			},
	/// 			handle::extract,
	/// 			streaming::{
	/// 				Await,
	/// 				Yield,
	/// 				await_value,
	/// 				interleave,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell producer row.
	/// 	pub row GiveRow {
	/// 		Yield<i32>,
	/// 	}
	/// }
	///
	/// define_row! {
	/// 	/// A one-cell consumer row.
	/// 	pub row TakeRow {
	/// 		Await<i32>,
	/// 	}
	/// }
	///
	/// let producer: Free<GiveRow, i32> = yield_value(7).bind(|()| Free::pure(0));
	/// let consumer: Free<TakeRow, i32> = await_value().bind(|got: i32| Free::pure(got));
	/// let stepped: Free<CNilBrand, Resume<CNilBrand, i32, i32, ()>> = handle_coroutine(consumer);
	/// let fused = stepped.bind(move |step| interleave(move |()| handle_coroutine(producer), step));
	/// assert_eq!(extract(fused), 7);
	/// ```
	pub fn interleave<Narrow, A, In, Out>(
		other: impl FnOnce(Out) -> Free<Narrow, Resume<Narrow, A, Out, In>> + 'static,
		step: Resume<Narrow, A, In, Out>,
	) -> Free<Narrow, A>
	where
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		A: 'static,
		In: 'static,
		Out: 'static, {
		match step {
			Resume::Done(value) => Free::pure(value),
			Resume::Next(output, next) =>
				other(output).bind(move |other_step| interleave(next, other_step)),
		}
	}

	/// Replaces each yield with an effect program whose result feeds back
	/// in. The pin must be value-for-value: a [`Yield`] pin resumes with
	/// unit, so it has nothing to feed back.
	#[document_signature]
	///
	#[document_type_parameters(
		"The residual row brand the steps run over.",
		"The program's result type.",
		"The type fed back in for each yield.",
		"The yielded output type."
	)]
	///
	#[document_parameters(
		"The effect program run for each yielded output, its result fed back in.",
		"The current step."
	)]
	///
	#[document_returns("The substituted program over the residual row.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			coroutine::{
	/// 				CoroutineBrand,
	/// 				Resume,
	/// 				handle_coroutine,
	/// 				yield_value,
	/// 			},
	/// 			handle::extract,
	/// 			streaming::substitute,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell value-for-value exchange row.
	/// 	pub row EchoRow {
	/// 		CoroutineBrand<i32, i32>,
	/// 	}
	/// }
	///
	/// let program: Free<EchoRow, i32> = yield_value(3).bind(|got: i32| Free::pure(got));
	/// let stepped: Free<CNilBrand, Resume<CNilBrand, i32, i32, i32>> = handle_coroutine(program);
	/// let substituted = stepped.bind(|step| substitute(|out: i32| Free::pure(out * 2), step));
	/// assert_eq!(extract(substituted), 6);
	/// ```
	pub fn substitute<Narrow, A, In, Out>(
		fulfill: impl Fn(Out) -> Free<Narrow, In> + Clone + 'static,
		step: Resume<Narrow, A, In, Out>,
	) -> Free<Narrow, A>
	where
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		A: 'static,
		In: 'static,
		Out: 'static, {
		match step {
			Resume::Done(value) => Free::pure(value),
			Resume::Next(output, next) => {
				let again = fulfill.clone();
				fulfill(output)
					.bind(move |input| next(input).bind(move |step| substitute(again, step)))
			}
		}
	}

	/// Connects a producer to a consumer, demand-driven: the consumer runs
	/// to its first await, then the producer runs to each yield, the two
	/// alternating until whichever side completes first supplies the fused
	/// result. Every other effect on either side re-emits into the shared
	/// residual row, in the order the interleaving reaches it.
	#[document_signature]
	///
	#[document_type_parameters(
		"The producer's row brand.",
		"The consumer's row brand.",
		"The shared residual row brand.",
		"The streamed value type.",
		"The fused result type.",
		"The coproduct index locating the producer's yield cell (inferred).",
		"The coproduct indices embedding the producer's remainder (inferred).",
		"The coproduct index locating the consumer's await cell (inferred).",
		"The coproduct indices embedding the consumer's remainder (inferred)."
	)]
	///
	#[document_parameters("The producer.", "The consumer.")]
	///
	#[document_returns("The fused program over the shared residual row.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			coroutine::yield_value,
	/// 			handle::extract,
	/// 			streaming::{
	/// 				Await,
	/// 				Yield,
	/// 				await_value,
	/// 				connect,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell producer row.
	/// 	pub row GiveRow {
	/// 		Yield<i32>,
	/// 	}
	/// }
	///
	/// define_row! {
	/// 	/// A one-cell consumer row.
	/// 	pub row TakeRow {
	/// 		Await<i32>,
	/// 	}
	/// }
	///
	/// let producer: Free<GiveRow, i32> =
	/// 	yield_value(4).bind(|()| yield_value(5)).bind(|()| Free::pure(0));
	/// let consumer: Free<TakeRow, i32> = await_value()
	/// 	.bind(|first: i32| await_value().bind(move |second: i32| Free::pure(first * 10 + second)));
	/// let fused: Free<CNilBrand, i32> = connect(producer, consumer);
	/// assert_eq!(extract(fused), 45);
	/// ```
	pub fn connect<RowP, RowC, Narrow, X, A, UninjectP, EmbedP, UninjectC, EmbedC>(
		producer: Free<RowP, A>,
		consumer: Free<RowC, A>,
	) -> Free<Narrow, A>
	where
		RowP: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		RowC: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		X: 'static,
		A: 'static,
		UninjectP: 'static,
		EmbedP: 'static,
		UninjectC: 'static,
		EmbedC: 'static,
		<RowP as LifetimeUnaryKind>::Of<'static, Free<RowP, A>>:
			CoprodUninjector<Coyoneda<'static, Yield<X>, Free<RowP, A>>, UninjectP>,
		<<RowP as LifetimeUnaryKind>::Of<'static, Free<RowP, A>> as CoprodUninjector<
			Coyoneda<'static, Yield<X>, Free<RowP, A>>,
			UninjectP,
		>>::Remainder:
			CoproductEmbedder<<Narrow as LifetimeUnaryKind>::Of<'static, Free<RowP, A>>, EmbedP>,
		<RowC as LifetimeUnaryKind>::Of<'static, Free<RowC, A>>:
			CoprodUninjector<Coyoneda<'static, Await<X>, Free<RowC, A>>, UninjectC>,
		<<RowC as LifetimeUnaryKind>::Of<'static, Free<RowC, A>> as CoprodUninjector<
			Coyoneda<'static, Await<X>, Free<RowC, A>>,
			UninjectC,
		>>::Remainder:
			CoproductEmbedder<<Narrow as LifetimeUnaryKind>::Of<'static, Free<RowC, A>>, EmbedC>, {
		let stepped: Free<Narrow, Resume<Narrow, A, X, ()>> = handle_coroutine(consumer);
		stepped.bind(move |step| interleave(move |()| handle_coroutine(producer), step))
	}

	/// Runs a program, fulfilling each yielded output with an effect
	/// program whose result feeds back in: [`substitute`] over the step
	/// runner, the for-substitution convenience.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The yielded output type.",
		"The type fed back in for each yield.",
		"The program's result type.",
		"The coproduct index locating the coroutine cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The program whose yields are fulfilled.",
		"The effect program run for each yielded output, its result fed back in."
	)]
	///
	#[document_returns("The fulfilled program over the residual row.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			coroutine::{
	/// 				CoroutineBrand,
	/// 				yield_value,
	/// 			},
	/// 			handle::extract,
	/// 			streaming::for_each,
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell value-for-value exchange row.
	/// 	pub row EchoRow {
	/// 		CoroutineBrand<i32, i32>,
	/// 	}
	/// }
	///
	/// let program: Free<EchoRow, i32> =
	/// 	yield_value(3).bind(|got: i32| yield_value(got + 1)).bind(|got2: i32| Free::pure(got2));
	/// let fulfilled: Free<CNilBrand, i32> = for_each(program, |out: i32| Free::pure(out * 2));
	/// assert_eq!(extract(fulfilled), 14);
	/// ```
	pub fn for_each<Row, Narrow, Out, In, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A>,
		fulfill: impl Fn(Out) -> Free<Narrow, In> + Clone + 'static,
	) -> Free<Narrow, A>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Out: 'static,
		In: 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>: CoprodUninjector<
				Coyoneda<'static, CoroutineBrand<Out, In>, Free<Row, A>>,
				UninjectIndex,
			>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, CoroutineBrand<Out, In>, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>, {
		handle_coroutine(program).bind(move |step| substitute(fulfill, step))
	}
}

pub use vocabulary::*;
