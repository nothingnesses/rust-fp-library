//! One-shot delimited continuations, driven end to end: `shift` captures
//! the continuation from the operation site to the delimiter as a
//! first-class one-shot value, and `run_shift` is the delimiter, a
//! narrowing fold whose return clause maps the program's result into the
//! prompt's answer program.
//!
//! Pinned here: (1) resuming the captured continuation runs the rest of
//! the source program to the delimiter; (2) dropping the continuation is
//! abort, structurally (the unresumed tail never runs); (3) the shift body
//! post-processes the resumed answer; (4) the prompt interleaves with
//! another effect through the re-emission path, the body reading state the
//! source program wrote.
#![cfg(feature = "effects")]

use {
	fp_library::{
		Apply,
		brands::CNilBrand,
		classes::{
			Functor,
			WrapDrop,
		},
		define_row,
		impl_kind,
		kinds::*,
		types::{
			Coyoneda,
			Free,
			effects::{
				coproduct::{
					CoprodInjector,
					CoprodUninjector,
					CoproductEmbedder,
				},
				handle::extract,
				state::{
					StateBrand,
					get,
					handle_state,
					put,
				},
			},
		},
	},
	std::marker::PhantomData,
};

// -- The proof-of-concept encoding (the shape to graduate to src) --

/// The reified one-shot continuation from the capture point to the
/// delimiter: resuming it runs the rest of the source program and yields
/// the prompt's answer program over the residual row.
pub type ShiftExit<'a, Narrow, Ans, V> = Box<dyn FnOnce(V) -> Free<Narrow, Ans> + 'a>;

/// Brand for the one-shot `Shift` effect: `Narrow` is the residual row the
/// prompt delimits to, `Ans` the prompt's answer type, `V` the captured
/// value type.
pub struct ShiftBrand<Narrow, Ans, V>(PhantomData<(Narrow, Ans, V)>);

/// The operations of `ShiftBrand`, one variant: the capture.
pub enum ShiftF<'a, Narrow, Ans, V, A>
where
	Narrow: WrapDrop + 'static,
	Ans: 'static, {
	/// Capture: the body receives the reified continuation and produces the
	/// prompt's answer program; the operation resumes with `V` when the body
	/// invokes the continuation.
	Shift(
		Box<dyn FnOnce(ShiftExit<'static, Narrow, Ans, V>) -> Free<Narrow, Ans> + 'a>,
		Box<dyn FnOnce(V) -> A + 'a>,
	),
}

impl_kind! {
	impl<Narrow, Ans, V> for ShiftBrand<Narrow, Ans, V>
	where
		Narrow: WrapDrop + 'static,
		Ans: 'static,
		V: 'static,
	{
		type Of<'a, A: 'a>: 'a = ShiftF<'a, Narrow, Ans, V, A>;
	}
}

impl<Narrow: WrapDrop + 'static, Ans: 'static, V: 'static> Functor for ShiftBrand<Narrow, Ans, V> {
	fn map<'a, A: 'a, B: 'a>(
		f: impl Fn(A) -> B + 'a,
		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		match fa {
			ShiftF::Shift(body, k) => ShiftF::Shift(body, Box::new(move |v| f(k(v)))),
		}
	}
}

/// Captures the continuation from here to the enclosing `run_shift`: the
/// body receives it as a one-shot value and produces the prompt's answer
/// program over the residual row; invoking the continuation resumes this
/// operation with the passed value.
fn shift<Narrow, Ans, V, R, I>(
	body: impl FnOnce(ShiftExit<'static, Narrow, Ans, V>) -> Free<Narrow, Ans> + 'static
) -> Free<R, V>
where
	Narrow: WrapDrop + 'static,
	Ans: 'static,
	V: 'static,
	R: Functor + WrapDrop + 'static,
	<R as LifetimeUnaryKind>::Of<'static, V>:
		CoprodInjector<Coyoneda<'static, ShiftBrand<Narrow, Ans, V>, V>, I>, {
	let cell: ShiftF<'static, Narrow, Ans, V, V> =
		ShiftF::Shift(Box::new(body), Box::new(|x| x));
	let coyo: Coyoneda<'static, ShiftBrand<Narrow, Ans, V>, V> = Coyoneda::lift(cell);
	let node: <R as LifetimeUnaryKind>::Of<'static, V> = CoprodInjector::inject(coyo);
	Free::lift_f(node)
}

/// The delimiter: folds the program at its own result type, handing each
/// `Shift` cell's delimited continuation to its body, re-emitting every
/// other cell into the residual row, and mapping a normal completion
/// through the return clause.
fn run_shift<Row, Narrow, Ans, V, A, UninjectIndex, EmbedIndices>(
	program: Free<Row, A>,
	on_pure: impl FnOnce(A) -> Free<Narrow, Ans> + 'static,
) -> Free<Narrow, Ans>
where
	Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
	Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
	Ans: 'static,
	V: 'static,
	A: 'static,
	UninjectIndex: 'static,
	EmbedIndices: 'static,
	<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>: CoprodUninjector<
			Coyoneda<'static, ShiftBrand<Narrow, Ans, V>, Free<Row, A>>,
			UninjectIndex,
		>,
	<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
		Coyoneda<'static, ShiftBrand<Narrow, Ans, V>, Free<Row, A>>,
		UninjectIndex,
	>>::Remainder: CoproductEmbedder<
			<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
			EmbedIndices,
		>, {
	let mut program = program;
	loop {
		let layer = match program.resume() {
			Ok(value) => return on_pure(value),
			Err(layer) => layer,
		};
		let selected: Result<Coyoneda<'static, ShiftBrand<Narrow, Ans, V>, Free<Row, A>>, _> =
			layer.uninject();
		match selected {
			Ok(coyo) => {
				let ShiftF::Shift(body, k) = coyo.lower();
				let exit: ShiftExit<'static, Narrow, Ans, V> =
					Box::new(move |v| run_shift(k(v), on_pure));
				return body(exit);
			}
			Err(rest) => {
				let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
					rest.embed();
				let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
				return lifted.bind(move |rest_program| run_shift(rest_program, on_pure));
			}
		}
	}
}

// -- Rows --

define_row! {
	/// A one-cell prompt delimiting straight to the empty row.
	pub row PureShiftRow {
		ShiftBrand<CNilBrand, i32, i32>,
	}
}

define_row! {
	/// The residual row a stateful prompt delimits to.
	pub row StateRow {
		StateBrand<i32>,
	}
}

define_row! {
	/// A prompt alongside integer state, delimiting to the state-only row.
	pub row PromptRow {
		ShiftBrand<StateRow, i32, i32>,
		StateBrand<i32>,
	}
}

// -- The oracles --

#[test]
fn resuming_the_continuation_runs_the_rest_of_the_program() {
	let program: Free<PureShiftRow, i32> =
		shift::<_, _, i32, _, _>(|exit| exit(5)).bind(|v: i32| Free::pure(v + 1));
	let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 6);
}

#[test]
fn dropping_the_continuation_aborts_the_unresumed_tail() {
	// The body never invokes the continuation, so the source program's tail
	// (the +1) is discarded with it; the prompt answers 99 directly.
	let program: Free<PureShiftRow, i32> =
		shift::<_, _, i32, _, _>(|_exit| Free::pure(99)).bind(|v: i32| Free::pure(v + 1));
	let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 99);
}

#[test]
fn the_body_post_processes_the_resumed_answer() {
	// exit(5) runs the tail (+1) to the delimiter yielding 6; the body then
	// doubles the completed answer: shift's defining composition.
	let program: Free<PureShiftRow, i32> =
		shift::<_, _, i32, _, _>(|exit| exit(5).bind(|ans: i32| Free::pure(ans * 2)))
			.bind(|v: i32| Free::pure(v + 1));
	let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	assert_eq!(extract(narrowed), 12);
}

#[test]
fn the_prompt_interleaves_with_state_through_the_re_emission_path() {
	// The source program writes state before capturing; the body, running
	// over the residual row, reads that state and resumes with it; the
	// resumed tail doubles. 1 written, body reads 1, resumes 11, tail 22.
	let program: Free<PromptRow, i32> = put(1)
		.bind(|()| shift::<_, _, i32, _, _>(|exit| get().bind(move |s: i32| exit(s + 10))))
		.bind(|v: i32| Free::pure(v * 2));
	let narrowed: Free<StateRow, i32> = run_shift(program, Free::pure);
	let handled: Free<CNilBrand, (i32, i32)> = handle_state(0, narrowed);
	assert_eq!(extract(handled), (1, 22));
}
