//! The `Writer` effect: an append-only log of `W`, with the folding and
//! collecting narrowing runners.
//!
//! The effect definition (brand, functor, order marker) and its smart
//! constructor are emitted by `fp_macros::define_effect!` from the operation
//! signature below. The accumulator is append-only: a handler never rewrites
//! or truncates earlier writes, which is what makes observed-delta slicing
//! (the tail an action appended) sound for scoped observers. [`fold_writer`]
//! eliminates the effect from a row by folding each written message into an
//! accumulator, and [`handle_writer`] is its monoid instance,
//! purescript-run's `foldWriter` and `runWriter` shapes.

fp_macros::define_effect! {
	/// Writer over an append-only log of `W`. `tell` appends to the log.
	#[handler_state(shared_by_reference)]
	#[crate_path(crate)]
	pub effect Writer<W: 'static> {
		/// Append `message` to the log.
		fn tell(message: W) -> ();
	}
}

#[fp_macros::document_module]
mod runner {
	use {
		super::{
			WriterBrand,
			WriterF,
		},
		crate::{
			classes::{
				Functor,
				Monoid,
				Semigroup,
				WrapDrop,
			},
			kinds::LifetimeUnaryKind,
			types::{
				Coyoneda,
				Free,
				effects::{
					coproduct::{
						CoprodUninjector,
						CoproductEmbedder,
					},
					handle::handle_accum,
				},
			},
		},
		fp_macros::*,
	};

	/// Eliminates the `Writer` effect from a row by folding each written
	/// message into an accumulator in program order; the final accumulator is
	/// paired with the program's result, and every other effect is re-emitted
	/// into the residual row.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The written message type.",
		"The accumulator type.",
		"The program's result type.",
		"The coproduct index locating the writer cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters(
		"The initial accumulator.",
		"The fold applied to the accumulator and each written message.",
		"The program to interpret."
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
	/// 	define_row,
	/// 	types::{
	/// 		Free,
	/// 		effects::{
	/// 			handle::extract,
	/// 			writer::{
	/// 				WriterBrand,
	/// 				fold_writer,
	/// 				tell,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row telling integers.
	/// 	pub row TallyRow {
	/// 		WriterBrand<i32>,
	/// 	}
	/// }
	///
	/// let program: Free<TallyRow, i32> = tell(3).bind(|()| tell(4)).bind(|()| Free::pure(7));
	/// let narrowed: Free<CNilBrand, (i32, i32)> =
	/// 	fold_writer(0, |sum, message| sum + message, program);
	/// assert_eq!(extract(narrowed), (7, 7));
	/// ```
	pub fn fold_writer<Row, Narrow, W, S, A, UninjectIndex, EmbedIndices>(
		initial: S,
		fold: impl Fn(S, W) -> S + 'static,
		program: Free<Row, A>,
	) -> Free<Narrow, (S, A)>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		W: 'static,
		S: 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>:
			CoprodUninjector<Coyoneda<'static, WriterBrand<W>, Free<Row, A>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, WriterBrand<W>, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>, {
		handle_accum::<WriterBrand<W>, _, _, _, _, _, _>(initial, program, move |s, op| match op {
			WriterF::Tell(message, resume) => (fold(s, message), resume(())),
		})
	}

	/// Eliminates the `Writer` effect from a row by appending each written
	/// message into the monoid's accumulating value, starting from its empty
	/// element: [`fold_writer`] at the monoid's own append.
	#[document_signature]
	///
	#[document_type_parameters(
		"The source row brand.",
		"The residual row brand.",
		"The written message type (a monoid).",
		"The program's result type.",
		"The coproduct index locating the writer cell (inferred).",
		"The coproduct indices embedding the remainder (inferred)."
	)]
	///
	#[document_parameters("The program to interpret.")]
	///
	#[document_returns(
		"The residual program over the narrowed row, yielding the accumulated log paired with the program's result."
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
	/// 			handle::extract,
	/// 			writer::{
	/// 				WriterBrand,
	/// 				handle_writer,
	/// 				tell,
	/// 			},
	/// 		},
	/// 	},
	/// };
	///
	/// define_row! {
	/// 	/// A one-cell row telling strings.
	/// 	pub row LogRow {
	/// 		WriterBrand<String>,
	/// 	}
	/// }
	///
	/// let program: Free<LogRow, i32> =
	/// 	tell("Hello, ".to_string()).bind(|()| tell("World!".to_string())).bind(|()| Free::pure(1));
	/// let narrowed: Free<CNilBrand, (String, i32)> = handle_writer(program);
	/// assert_eq!(extract(narrowed), ("Hello, World!".to_string(), 1));
	/// ```
	pub fn handle_writer<Row, Narrow, W, A, UninjectIndex, EmbedIndices>(
		program: Free<Row, A>
	) -> Free<Narrow, (W, A)>
	where
		Row: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		Narrow: LifetimeUnaryKind + Functor + WrapDrop + 'static,
		W: Monoid + 'static,
		A: 'static,
		UninjectIndex: 'static,
		EmbedIndices: 'static,
		<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>>:
			CoprodUninjector<Coyoneda<'static, WriterBrand<W>, Free<Row, A>>, UninjectIndex>,
		<<Row as LifetimeUnaryKind>::Of<'static, Free<Row, A>> as CoprodUninjector<
			Coyoneda<'static, WriterBrand<W>, Free<Row, A>>,
			UninjectIndex,
		>>::Remainder: CoproductEmbedder<
				<Narrow as LifetimeUnaryKind>::Of<'static, Free<Row, A>>,
				EmbedIndices,
			>, {
		fold_writer(<W as Monoid>::empty(), <W as Semigroup>::append, program)
	}
}

pub use runner::*;

#[cfg(test)]
mod tests {
	use crate::types::effects::fs1::{
		Fixture,
		run,
		tell,
	};

	// Behaviour-parity oracle bucket A (single-effect): successive `tell`s
	// accumulate into the `Writer` log in order.
	#[test]
	fn tell_accumulates_the_log_in_order() {
		let program = tell("Hello".to_string()).bind(|()| tell(" world!".to_string()));
		let fx = Fixture::new();
		assert_eq!(run(program, &fx.handlers()), Ok(()));
		assert_eq!(*fx.log.borrow(), "Hello world!");
	}
}
