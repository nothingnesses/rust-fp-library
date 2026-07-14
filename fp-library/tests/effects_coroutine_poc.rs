//! Proof of concept for the coroutine step runner: `Coroutine<Out, In>`
//! yields an `Out` to the runner and resumes with an `In`, and the runner
//! drives a program to its first yield or its completion, returning a
//! yielded-or-done `Resume` whose continuation is pre-folded through the
//! runner itself, so a caller can never forget to re-narrow. Every other
//! effect re-emits into the residual row.
//!
//! Pinned here: (1) the recursive `Resume` enum (a boxed continuation
//! returning `Free` of `Resume`) and the pre-folded runner compile on the
//! public surface; (2) a yielded-or-done oracle passes over a row with a
//! residual `Writer`, the residual writes landing with the step that made
//! them; (3) a program with no yield is `Done` in one step.
#![cfg(feature = "effects")]

use {
	fp_library::{
		brands::CNilBrand,
		classes::{
			Functor,
			WrapDrop,
		},
		define_effect,
		define_row,
		kinds::LifetimeUnaryKind,
		types::{
			Coyoneda,
			Free,
			effects::{
				coproduct::{
					CoprodUninjector,
					CoproductEmbedder,
				},
				handle::extract,
				writer::{
					WriterBrand,
					handle_writer,
					tell,
				},
			},
		},
	},
	std::boxed::Box,
};

define_effect! {
	/// Cooperative yielding: emit an `Out` to the runner, resume with an
	/// `In`.
	#[handler_state(none)]
	pub effect Coroutine<Out: 'static, In: 'static> {
		/// Yield `output` to the runner; the continuation resumes with the
		/// runner's answer.
		fn yield_value(output: Out) -> In;
	}
}

/// The step runner's result: the program completed with its value, or it
/// yielded an output and waits for an input, the continuation already
/// folded back through the runner.
pub enum Resume<Narrow, A, In, Out>
where
	Narrow: WrapDrop + 'static,
	A: 'static,
	In: 'static,
	Out: 'static, {
	/// The program completed.
	Done(A),
	/// The program yielded `Out`; feeding an `In` resumes it to its next
	/// step.
	Next(Out, Box<dyn FnOnce(In) -> Free<Narrow, Resume<Narrow, A, In, Out>>>),
}

/// Drives a program to its first yield or its completion, re-emitting every
/// other effect into the residual row.
fn handle_coroutine<Row, Narrow, Out, In, A, UninjectIndex, EmbedIndices>(
	program: Free<Row, A>,
) -> Free<Narrow, Resume<Narrow, A, In, Out>>
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
	let layer = match program.resume() {
		Ok(value) => return Free::pure(Resume::Done(value)),
		Err(layer) => layer,
	};
	let selected: Result<Coyoneda<'static, CoroutineBrand<Out, In>, Free<Row, A>>, _> =
		layer.uninject();
	match selected {
		Ok(op) => match op.lower() {
			CoroutineF::YieldValue(output, resume) => Free::pure(Resume::Next(
				output,
				Box::new(move |input| handle_coroutine(resume(input))),
			)),
		},
		Err(rest) => {
			let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> = rest.embed();
			let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
			lifted.bind(handle_coroutine)
		}
	}
}

define_row! {
	/// Integer-for-integer yielding alongside a string log.
	pub row CoLogRow {
		CoroutineBrand<i32, i32>,
		WriterBrand<String>,
	}
}

define_row! {
	/// The residual after the coroutine cell is eliminated.
	pub row LogOnlyRow {
		WriterBrand<String>,
	}
}

#[test]
fn the_runner_steps_to_each_yield_and_residual_effects_land_with_their_step() {
	// Step one: the program logs, yields 1, and suspends; the log written
	// before the yield comes out with the first step.
	let program: Free<CoLogRow, i32> = tell("start".to_string())
		.bind(|()| yield_value(1))
		.bind(|got: i32| tell(format!("got {got}")).bind(move |()| Free::pure(got + 1)));
	let stepped: Free<LogOnlyRow, Resume<LogOnlyRow, i32, i32, i32>> = handle_coroutine(program);
	let narrowed: Free<CNilBrand, (String, Resume<LogOnlyRow, i32, i32, i32>)> =
		handle_writer(stepped);
	let (first_log, resume) = extract(narrowed);
	assert_eq!(first_log, "start");
	let (output, continuation) = match resume {
		Resume::Next(output, continuation) => (output, continuation),
		Resume::Done(_) => panic!("the program yields before completing"),
	};
	assert_eq!(output, 1);

	// Step two: feeding 10 resumes the continuation, whose log and result
	// come out with the second step.
	let narrowed: Free<CNilBrand, (String, Resume<LogOnlyRow, i32, i32, i32>)> =
		handle_writer(continuation(10));
	let (second_log, resume) = extract(narrowed);
	assert_eq!(second_log, "got 10");
	match resume {
		Resume::Done(value) => assert_eq!(value, 11),
		Resume::Next(..) => panic!("the program completes after one yield"),
	}
}

#[test]
fn a_program_with_no_yield_is_done_in_one_step() {
	let program: Free<CoLogRow, i32> = tell("only".to_string()).bind(|()| Free::pure(5));
	let stepped: Free<LogOnlyRow, Resume<LogOnlyRow, i32, i32, i32>> = handle_coroutine(program);
	let narrowed: Free<CNilBrand, (String, Resume<LogOnlyRow, i32, i32, i32>)> =
		handle_writer(stepped);
	let (log, resume) = extract(narrowed);
	assert_eq!(log, "only");
	match resume {
		Resume::Done(value) => assert_eq!(value, 5),
		Resume::Next(..) => panic!("no yield exists to suspend on"),
	}
}
