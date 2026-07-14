//! Proof of concept for the streaming vocabulary: pin aliases over the one
//! coroutine functor (a producer yields values and resumes with unit; a
//! consumer yields unit and resumes with a value), and the two fusion
//! primitives over the step runner's `Resume` (`interleave` feeds each
//! side's output to the other's continuation, ending with whichever side
//! completes first; `substitute` replaces each yield with an effect program
//! whose result feeds back).
//!
//! Pinned here: (1) alias cells work as row members; (2) the role-swapping
//! `interleave` recursion monomorphizes; (3) a connect oracle pins the
//! demand-driven interleaving order and the either-side-completes semantics
//! with residual `Writer` effects re-emitted; (4) a deep exchange chain
//! drives without native stack growth.
#![cfg(feature = "effects")]

use fp_library::{
	brands::CNilBrand,
	classes::{
		Functor,
		WrapDrop,
	},
	define_row,
	kinds::LifetimeUnaryKind,
	types::{
		Free,
		effects::{
			coroutine::{
				CoroutineBrand,
				Resume,
				handle_coroutine,
				yield_value,
			},
			handle::extract,
			writer::{
				WriterBrand,
				handle_writer,
				tell,
			},
		},
	},
};

/// A producer cell: yields an `X`, resumes with unit.
type Yield<X> = CoroutineBrand<X, ()>;

/// A consumer cell: yields unit, resumes with an `X`.
type Await<X> = CoroutineBrand<(), X>;

/// Awaits a value from upstream (the await pin's unit payload hidden).
fn await_value<X, R, I>() -> Free<R, X>
where
	X: 'static,
	R: Functor + WrapDrop + 'static,
	<R as LifetimeUnaryKind>::Of<'static, X>: fp_library::types::effects::coproduct::CoprodInjector<
			fp_library::types::Coyoneda<'static, Await<X>, X>,
			I,
		>, {
	yield_value::<(), X, R, I>(())
}

/// Alternately feeds each side's output to the other side's continuation,
/// ending with whichever side completes first.
fn interleave<Narrow, A, In, Out>(
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
		Resume::Next(output, next) => {
			other(output).bind(move |other_step| interleave(next, other_step))
		}
	}
}

/// Replaces each yield with an effect program whose result feeds back in.
fn substitute<Narrow, A, In, Out>(
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

define_row! {
	/// A producing row: integer yields alongside a string log.
	pub row ProduceRow {
		Yield<i32>,
		WriterBrand<String>,
	}
}

define_row! {
	/// A consuming row: integer awaits alongside a string log.
	pub row ConsumeRow {
		Await<i32>,
		WriterBrand<String>,
	}
}

define_row! {
	/// The common residual row both sides narrow to.
	pub row LogRow {
		WriterBrand<String>,
	}
}

/// Connects a producer to a consumer, demand-driven: the consumer runs to
/// its first await, then the producer runs to each yield.
fn connect<A: 'static>(
	producer: Free<ProduceRow, A>,
	consumer: Free<ConsumeRow, A>,
) -> Free<LogRow, A> {
	let consumer_stepped: Free<LogRow, Resume<LogRow, A, i32, ()>> = handle_coroutine(consumer);
	consumer_stepped
		.bind(move |step| interleave(move |()| handle_coroutine(producer), step))
}

#[test]
fn connect_interleaves_demand_driven_and_ends_when_the_consumer_completes() {
	// The consumer demands twice and completes; the producer would yield
	// forever more, but fusion ends with the completing side. The log pins
	// the order: the consumer runs first to its await, then the producer to
	// its yield, alternating.
	fn produce_from(n: i32) -> Free<ProduceRow, i32> {
		tell(format!("p{n}"))
			.bind(move |()| yield_value(n))
			.bind(move |()| produce_from(n + 1))
	}
	let producer = produce_from(1);
	let consumer: Free<ConsumeRow, i32> = tell("c".to_string())
		.bind(|()| await_value())
		.bind(|first: i32| await_value().bind(move |second: i32| Free::pure(first * 10 + second)));
	let narrowed: Free<CNilBrand, (String, i32)> = handle_writer(connect(producer, consumer));
	assert_eq!(extract(narrowed), ("cp1p2".to_string(), 12));
}

define_row! {
	/// A value-for-value exchange alongside a string log.
	pub row EchoRow {
		CoroutineBrand<i32, i32>,
		WriterBrand<String>,
	}
}

#[test]
fn substitute_fulfills_each_yield_with_an_effect_program() {
	// Each yielded value is logged, and its double feeds back in.
	let producer: Free<EchoRow, i32> =
		yield_value(3).bind(|got: i32| yield_value(got + 1).bind(|got2: i32| Free::pure(got2)));
	let stepped: Free<LogRow, Resume<LogRow, i32, i32, i32>> = handle_coroutine(producer);
	let substituted = stepped.bind(|step| {
		substitute(|out: i32| tell(format!("saw {out}")).bind(move |()| Free::pure(out * 2)), step)
	});
	let narrowed: Free<CNilBrand, (String, i32)> = handle_writer(substituted);
	assert_eq!(extract(narrowed), ("saw 3saw 7".to_string(), 14));
}

#[test]
fn a_deep_exchange_chain_drives_without_native_stack_growth() {
	const DEPTH: i32 = 100_000;
	fn produce_from(n: i32) -> Free<ProduceRow, i32> {
		yield_value(n).bind(move |()| produce_from(n + 1))
	}
	fn consume_until(limit: i32) -> Free<ConsumeRow, i32> {
		await_value().bind(move |got: i32| {
			if got >= limit { Free::pure(got) } else { consume_until(limit) }
		})
	}
	let narrowed: Free<CNilBrand, (String, i32)> =
		handle_writer(connect(produce_from(1), consume_until(DEPTH)));
	assert_eq!(extract(narrowed), (String::new(), DEPTH));
}
