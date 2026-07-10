//! The generic interpretation surface: the narrowing accumulator runner and
//! the terminal extractor.
//!
//! A narrowing runner eliminates one effect brand from a row and returns the
//! residual program over the remaining cells, so runners stack in any order
//! and handler-ordering semantics (which effect scopes which) fall out of the
//! stacking order. [`handle_accum`] is the generic core: it threads an
//! accumulator through the eliminated effect's operations iteratively, and
//! re-emits every unmatched layer into the residual row with the recursive
//! continuation deferred into `bind`, so the walk is constant-stack per step
//! regardless of program length. [`extract`] closes a fully narrowed
//! pipeline: once every cell is eliminated the residual row is
//! [`CNilBrand`](crate::brands::CNilBrand) and the program is necessarily a
//! pure value.

#[fp_macros::document_module]
mod inner {
	use {
		crate::{
			brands::CNilBrand,
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

	/// Eliminates one effect brand from a row by threading an accumulator
	/// through its operations; every other layer is re-emitted into the
	/// residual row with the continuation deferred, so unmatched effects run
	/// under whatever interpreter later drives the residual program.
	///
	/// The step function receives the current accumulator and one lowered
	/// operation of the eliminated effect, and returns the new accumulator
	/// and the continuation program (usually by invoking the operation's
	/// resume function). Matched operations are driven iteratively; an
	/// unmatched layer is embedded into the residual row with its holes as
	/// results, and the runner re-enters through `bind`, so native stack use
	/// per step is constant. Sequential threading moves the accumulator, so
	/// no `Clone` bound is needed here; runners that fork interpretation
	/// (nondeterministic choice) clone the accumulator per branch instead.
	///
	/// The two index parameters position the eliminated brand inside the
	/// row's coproduct and the remainder inside the residual row; both are
	/// inferred at every call site (turbofish as
	/// `handle_accum::<EffectBrand, _, _, _, _, _, _>`); the eliminated brand
	/// always needs naming, because it appears in the signature only through
	/// its kind projection, which inference cannot invert.
	#[document_signature]
	///
	#[document_type_parameters(
		"The eliminated effect brand.",
		"The source row brand.",
		"The residual row brand.",
		"The accumulator type.",
		"The program's result type.",
		"The coproduct index locating the eliminated brand (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The initial accumulator.",
		"The program to interpret.",
		"The step function applied to each matched operation."
	)]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the final accumulator paired with the program's result."
	)]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	define_effect,
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::handle::{
	/// 			extract,
	/// 			handle_accum,
	/// 		},
	/// 	},
	/// };
	///
	/// define_effect! {
	/// 	/// A running total.
	/// 	#[handler_state(threaded_by_value)]
	/// 	pub effect Counter {
	/// 		/// Add `amount` to the total, resuming with the new total.
	/// 		fn add(amount: i32) -> i32;
	/// 	}
	/// }
	///
	/// define_row! {
	/// 	/// The one-cell row.
	/// 	pub row CounterRow {
	/// 		CounterBrand,
	/// 	}
	/// }
	///
	/// let program: Free<CounterRow, i32> = add(2).bind(|_| add(3));
	/// let narrowed: Free<CNilBrand, (i32, i32)> =
	/// 	handle_accum::<CounterBrand, _, _, _, _, _, _>(0, program, |s, op| match op {
	/// 		CounterF::Add(amount, resume) => (s + amount, resume(s + amount)),
	/// 	});
	/// assert_eq!(extract(narrowed), (5, 5));
	/// ```
	pub fn handle_accum<EBrand, Row, Narrow, S, A, UninjectIndex, EmbedIndices>(
		mut s: S,
		mut program: Free<Row, A>,
		step: impl Fn(S, <EBrand as LifetimeUnaryKind>::Of<'static, Free<Row, A>>) -> (S, Free<Row, A>)
		+ 'static,
	) -> Free<Narrow, (S, A)>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		EBrand: LifetimeUnaryKind + Functor + 'static,
		S: 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>:
			CoprodUninjector<Coyoneda<'static, EBrand, Free<Row, A>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, EBrand, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>, {
		loop {
			let layer = match program.resume() {
				Ok(value) => return Free::pure((s, value)),
				Err(layer) => layer,
			};
			let selected: Result<Coyoneda<'static, EBrand, Free<Row, A>>, _> = layer.uninject();
			match selected {
				Ok(op) => {
					let (next_s, next_program) = step(s, op.lower());
					s = next_s;
					program = next_program;
				}
				Err(rest) => {
					let residual: <Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>> =
						rest.embed();
					// The Box-store `bind` arm must be selected explicitly:
					// `bind` is defined per store, and in generic position the
					// method call is ambiguous until the receiver's store is
					// pinned.
					let lifted: Free<Narrow, Free<Row, A>> = Free::lift_f(residual);
					return lifted.bind(move |rest_program| handle_accum(s, rest_program, step));
				}
			}
		}
	}

	/// Extracts the value from a fully narrowed program: over the empty row
	/// no operation can be suspended, so the program is necessarily a pure
	/// value and the suspended case is uninhabited.
	#[document_signature]
	///
	#[document_type_parameters("The program's result type.")]
	///
	#[document_parameters("The fully narrowed program.")]
	///
	#[document_returns("The program's value.")]
	///
	#[document_examples]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::CNilBrand,
	/// 	types::{
	/// 		Free,
	/// 		effects::handle::extract,
	/// 	},
	/// };
	///
	/// let program: Free<CNilBrand, i32> = Free::pure(7);
	/// assert_eq!(extract(program), 7);
	/// ```
	pub fn extract<A: 'static>(program: Free<CNilBrand, A>) -> A {
		match program.resume() {
			Ok(value) => value,
			Err(layer) => match layer {},
		}
	}
}

pub use inner::*;
