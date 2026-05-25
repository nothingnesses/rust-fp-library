#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::super::prelude::*;

	/// Raw default-Run Catch replacement adapter.
	pub(crate) struct BoxCatchRawRunReplacer<R, S, E, H> {
		pub(crate) handler: std::cell::RefCell<Option<H>>,
		pub(crate) _row: PhantomData<fn() -> R>,
		pub(crate) _scoped: PhantomData<fn() -> S>,
		pub(crate) _error: PhantomData<fn() -> E>,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The handled error type.",
		"The concrete single-shot recovery handler type."
	)]
	#[document_parameters("The raw default-Run Catch replacement adapter.")]
	impl<R, S, E, H> RunFirstOrderReplacer<ExceptBrand<E>, R, S> for BoxCatchRawRunReplacer<R, S, E, H>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		E: 'static,
		H: FnOnce(E) -> RawRunFree<R, S> + 'static,
	{
		/// Replaces a raw `Throw` operation with the stored recovery
		/// branch.
		#[document_signature]
		#[document_type_parameters("The current raw branch result type.")]
		#[document_parameters("The lowered Except operation selected by raw Catch dispatch.")]
		#[document_returns("The recovery program in the original row.")]
		#[document_examples(
			skip_call_check,
			reason = "Direct-call validation skip predates reason enforcement; audit this example and remove the skip when direct item usage is practical."
		)]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// let run: Run<CNilBrand, CNilBrand, i32> = Run::pure(42);
		/// assert_eq!(run.extract(), 42);
		/// ```
		fn replace<T: 'static>(
			&self,
			effect: Except<'static, E, Run<R, S, T>>,
		) -> Run<R, S, T> {
			match effect {
				Except::Throw(e, _) => {
					#[expect(
						clippy::expect_used,
						reason = "Box-backed Catch handlers are single-shot and the protected action can throw at most once"
					)]
					let handler = self
						.handler
						.borrow_mut()
						.take()
						.expect("BoxCatch handler invoked more than once");
					Run::from_free(Free::continue_from_erased(
						handler(e).erase_type(),
						CatList::empty(),
					))
				}
			}
		}
	}
}

pub(crate) use inner::*;
