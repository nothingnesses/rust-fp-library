//! The `Coroutine` effect: cooperative yielding, emitting an `Out` to the
//! runner and resuming with an `In`, with the yielded-or-done step runner.
//!
//! The effect definition (brand, functor, order marker) and its smart
//! constructor are emitted by `fp_macros::define_effect!` from the operation
//! signature below. [`handle_coroutine`] drives a program to its first yield
//! or its completion and returns a [`Resume`]: either the completed value or
//! the yielded output paired with a continuation that is already folded back
//! through the runner, so a caller can never forget to re-narrow a resumed
//! step. Every other effect re-emits into the residual row, landing with the
//! step that produced it.

fp_macros::define_effect! {
	/// Cooperative yielding: emit an `Out` to the runner, resume with an
	/// `In`.
	#[handler_state(none)]
	#[crate_path(crate)]
	pub effect Coroutine<Out: 'static, In: 'static> {
		/// Yield `output` to the runner; the continuation resumes with the
		/// runner's answer.
		fn yield_value(output: Out) -> In;
	}
}

#[fp_macros::document_module]
mod runner {
	use {
		super::{
			CoroutineBrand,
			CoroutineF,
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
					CoprodUninjector,
					CoproductEmbedder,
				},
			},
		},
		fp_macros::*,
	};

	/// The row's coroutine cell over programs yielding `T`, as selected by
	/// the runner's brand-keyed `uninject`.
	type CoroutineCell<Row, Out, In, T> = Coyoneda<'static, CoroutineBrand<Out, In>, Free<Row, T>>;

	/// A suspended step's continuation: feeding the input resumes the
	/// program to its next step, already folded back through
	/// [`handle_coroutine`].
	#[document_type_parameters(
		"The residual row brand the continuation's steps run over.",
		"The program's result type.",
		"The type the yielded program resumes with.",
		"The yielded output type."
	)]
	pub type ResumeNext<Narrow, A, In, Out> =
		Box<dyn FnOnce(In) -> Free<Narrow, Resume<Narrow, A, In, Out>>>;

	/// The step runner's result: the program completed with its value, or
	/// it yielded an output and waits for an input, with the continuation
	/// already folded back through [`handle_coroutine`].
	#[document_type_parameters(
		"The residual row brand the continuation's steps run over.",
		"The program's result type.",
		"The type the yielded program resumes with.",
		"The yielded output type."
	)]
	pub enum Resume<Narrow, A, In, Out>
	where
		Narrow: WrapDrop + 'static,
		A: 'static,
		In: 'static,
		Out: 'static, {
		/// The program completed with its value.
		Done(A),
		/// The program yielded an output; feeding an input to the
		/// continuation resumes it to its next step.
		Next(Out, ResumeNext<Narrow, A, In, Out>),
	}

	/// Drives a program to its first yield or its completion: the coroutine
	/// cell is eliminated from the row, every other effect re-emits into
	/// the residual row with the step that produced it, and the result is a
	/// [`Resume`] whose continuation re-enters this runner, so consecutive
	/// steps stay over the residual row.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The yielded output type.",
		"The type the yielded program resumes with.",
		"The program's result type.",
		"The coproduct index locating the coroutine cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters("The program to step.")]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the completed value or the yielded output paired with the pre-folded continuation."
	)]
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
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row yielding integers for integers.
	/// 	pub row CoRow {
	/// 		CoroutineBrand<i32, i32>,
	/// 	}
	/// }
	///
	/// let program: Free<CoRow, i32> = yield_value(1).bind(|got: i32| Free::pure(got + 1));
	/// let stepped: Free<CNilBrand, Resume<CNilBrand, i32, i32, i32>> = handle_coroutine(program);
	/// let continuation = match extract(stepped) {
	/// 	Resume::Next(output, continuation) => {
	/// 		assert_eq!(output, 1);
	/// 		continuation
	/// 	}
	/// 	Resume::Done(_) => panic!("the program yields before completing"),
	/// };
	/// match extract(continuation(10)) {
	/// 	Resume::Done(value) => assert_eq!(value, 11),
	/// 	Resume::Next(..) => panic!("the program completes after one yield"),
	/// }
	/// ```
	pub fn handle_coroutine<Row, Narrow, Out, In, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A>
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
		let selected: Result<CoroutineCell<Row, Out, In, A>, _> = layer.uninject();
		match selected {
			Ok(op) => match op.lower() {
				CoroutineF::YieldValue(output, resume) => Free::pure(Resume::Next(
					output,
					Box::new(move |input| handle_coroutine(resume(input))),
				)),
			},
			Err(rest) => {
				let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
					rest.embed();
				let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
				lifted.bind(handle_coroutine)
			}
		}
	}
}

pub use runner::*;
