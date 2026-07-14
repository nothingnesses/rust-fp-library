//! One-shot delimited continuations: the `Shift` effect and its
//! `run_shift` delimiter.
//!
//! [`shift`] captures the continuation from the operation site to the
//! enclosing delimiter as a first-class one-shot value: the shift body
//! receives it and produces the prompt's answer program over the residual
//! row, so resuming, post-processing the resumed answer, and aborting (by
//! dropping the continuation) are all ordinary data flow. [`run_shift`] is
//! the delimiter, a narrowing fold in the same family as the accumulator
//! runners: it hands each capture its delimited continuation, re-emits
//! every other effect into the residual row lazily, and maps a normal
//! completion through its return clause.
//!
//! The cell is hand-written rather than `define_effect!`-emitted: the
//! body payload's type receives the reified continuation, a shape outside
//! the macro's operation grammar, which makes this effect the catalog's
//! exemplar of the hand-written path the custom-effects guide documents.
//! On the single-shot `Box` store the captured continuation is `FnOnce`,
//! so one capture resumes at most once by type; the multi-shot form
//! (re-callable continuations on the `Rc`/`Arc` stores, heftia's fork
//! primitive) belongs to the multi-shot interpretation round.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			Apply,
			classes::{
				Functor,
				WrapDrop,
			},
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
					order::{
						FirstOrder,
						OrderOf,
					},
				},
			},
		},
		fp_macros::*,
		std::marker::PhantomData,
	};

	/// The reified one-shot continuation from a capture point to its
	/// delimiter: resuming it runs the rest of the source program to the
	/// delimiter and yields the prompt's answer program over the residual
	/// row. Dropping it aborts the unresumed tail, structurally.
	#[document_type_parameters(
		"The lifetime bounding the capture.",
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type (what resuming passes in)."
	)]
	pub type ShiftExit<'a, Narrow, Ans, V> = Box<dyn FnOnce(V) -> Free<Narrow, Ans> + 'a>;

	/// A capture body: from the reified continuation to the prompt's answer
	/// program over the residual row.
	#[document_type_parameters(
		"The lifetime bounding the body.",
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	pub type ShiftBody<'a, Narrow, Ans, V> =
		Box<dyn FnOnce(ShiftExit<'static, Narrow, Ans, V>) -> Free<Narrow, Ans> + 'a>;

	/// The capture cell as the delimiter selects it from a program layer.
	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The program the cell's continuation resumes into."
	)]
	pub type CapturedCell<Narrow, Ans, V, P> = Coyoneda<'static, ShiftBrand<Narrow, Ans, V>, P>;

	/// Brand for the one-shot `Shift` effect: delimited-continuation
	/// capture, pinned per prompt.
	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	pub struct ShiftBrand<Narrow, Ans, V>(PhantomData<(Narrow, Ans, V)>);

	/// The operations of [`ShiftBrand`]: one variant, the capture, with the
	/// continuation hole `A`.
	#[document_type_parameters(
		"The lifetime bounding the stored callables.",
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The continuation hole."
	)]
	pub enum ShiftF<'a, Narrow, Ans, V, A>
	where
		Narrow: WrapDrop + 'static,
		Ans: 'static, {
		/// Capture: the body receives the reified continuation and produces
		/// the prompt's answer program; the operation resumes with `V` when
		/// the body invokes the continuation.
		Shift(ShiftBody<'a, Narrow, Ans, V>, Box<dyn FnOnce(V) -> A + 'a>),
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

	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	impl<Narrow: WrapDrop + 'static, Ans: 'static, V: 'static> Functor for ShiftBrand<Narrow, Ans, V> {
		/// Composes the mapped function into the capture's continuation; the
		/// body is untouched.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of the values.",
			"The type of the value(s) inside the functor.",
			"The type of the result(s) of applying the function."
		)]
		///
		#[document_parameters(
			"The function to apply to the value(s) inside the functor.",
			"The functor instance containing the value(s)."
		)]
		///
		#[document_returns(
			"A new functor instance containing the result(s) of applying the function."
		)]
		///
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	classes::Functor,
		/// 	types::effects::shift::{
		/// 		ShiftBrand,
		/// 		ShiftF,
		/// 	},
		/// };
		///
		/// let op: ShiftF<'static, CNilBrand, i32, i32, i32> =
		/// 	ShiftF::Shift(Box::new(|exit| exit(5)), Box::new(|v| v));
		/// let mapped = <ShiftBrand<CNilBrand, i32, i32> as Functor>::map(|v: i32| v + 1, op);
		/// let ShiftF::Shift(_body, k) = mapped;
		/// assert_eq!(k(5), 6);
		/// ```
		fn map<'a, A: 'a, B: 'a>(
			f: impl Fn(A) -> B + 'a,
			fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
			match fa {
				ShiftF::Shift(body, k) => ShiftF::Shift(body, Box::new(move |v| f(k(v)))),
			}
		}
	}

	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type."
	)]
	impl<Narrow, Ans, V> OrderOf for ShiftBrand<Narrow, Ans, V> {
		type Order = FirstOrder;
	}

	/// Captures the continuation from here to the enclosing [`run_shift`]:
	/// the body receives it as a one-shot value and produces the prompt's
	/// answer program over the residual row. Invoking the continuation
	/// resumes this operation with the passed value and yields the source
	/// program's completed answer, which the body may return directly or
	/// post-process; dropping the continuation aborts the unresumed tail,
	/// structurally.
	///
	/// The body runs over the residual row, so it cannot capture at this
	/// same prompt again, while every other effect of the row (and any
	/// other prompt the residual row holds) remains available to it.
	#[document_signature]
	///
	#[document_type_parameters(
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The row brand the program runs over.",
		"The coproduct index locating the `Shift` cell (inferred)."
	)]
	///
	#[document_parameters("The capture body, from the reified continuation to the answer program.")]
	///
	#[document_returns("The one-operation program suspending at the capture.")]
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
	/// 			shift::{
	/// 				ShiftBrand,
	/// 				run_shift,
	/// 				shift,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell prompt delimiting straight to the empty row.
	/// 	pub row PromptRow {
	/// 		ShiftBrand<CNilBrand, i32, i32>,
	/// 	}
	/// }
	///
	/// // The body resumes with 5; the source tail adds 1.
	/// let program: Free<PromptRow, i32> =
	/// 	shift::<_, _, i32, _, _>(|exit| exit(5)).bind(|v: i32| Free::pure(v + 1));
	/// let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	/// assert_eq!(extract(narrowed), 6);
	/// ```
	pub fn shift<Narrow, Ans, V, R, I>(
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

	/// The delimiter: folds the program at its own result type, handing
	/// each `Shift` cell's delimited one-shot continuation to its body,
	/// re-emitting every other effect into the residual row lazily, and
	/// mapping a normal completion through the return clause. The return
	/// clause is consumed exactly once, along whichever of the three paths
	/// runs: normal completion applies it, a capture moves it into the
	/// reified continuation (so dropping the continuation drops the tail
	/// with it), and a re-emitted layer carries it forward.
	///
	/// Native stack use grows with the number of captures resumed in one
	/// synchronous chain, because each resumed continuation re-enters the
	/// fold inside the caller's frame, matching
	/// [`run_cont`](crate::types::effects::handle::run_cont)'s documented
	/// trade-off.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand the prompt delimits to.",
		"The prompt's answer type.",
		"The captured value type.",
		"The program's result type.",
		"The coproduct index locating the `Shift` cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The program to delimit.",
		"The return clause, mapping the program's result into the answer program."
	)]
	///
	#[document_returns("The prompt's answer program over the residual row.")]
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
	/// 			shift::{
	/// 				ShiftBrand,
	/// 				run_shift,
	/// 				shift,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell prompt delimiting straight to the empty row.
	/// 	pub row PromptRow {
	/// 		ShiftBrand<CNilBrand, i32, i32>,
	/// 	}
	/// }
	///
	/// // exit(5) runs the tail (+1) to the delimiter yielding 6; the body
	/// // then doubles the completed answer: shift's defining composition.
	/// let program: Free<PromptRow, i32> =
	/// 	shift::<_, _, i32, _, _>(|exit| exit(5).bind(|ans: i32| Free::pure(ans * 2)))
	/// 		.bind(|v: i32| Free::pure(v + 1));
	/// let narrowed: Free<CNilBrand, i32> = run_shift(program, Free::pure);
	/// assert_eq!(extract(narrowed), 12);
	/// ```
	pub fn run_shift<Row, Narrow, Ans, V, A, UninjectIndex, EmbedIndices>(
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
		let layer = match program.resume() {
			Ok(value) => return on_pure(value),
			Err(layer) => layer,
		};
		let selected: Result<CapturedCell<Narrow, Ans, V, Free<Row, A>>, _> = layer.uninject();
		match selected {
			Ok(coyo) => {
				let ShiftF::Shift(body, k) = coyo.lower();
				let exit: ShiftExit<'static, Narrow, Ans, V> =
					Box::new(move |v| run_shift(k(v), on_pure));
				body(exit)
			}
			Err(rest) => {
				let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
					rest.embed();
				let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
				lifted.bind(move |rest_program| run_shift(rest_program, on_pure))
			}
		}
	}
}

pub use inner::*;
