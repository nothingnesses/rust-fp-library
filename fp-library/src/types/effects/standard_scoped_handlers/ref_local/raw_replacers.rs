#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::super::prelude::*;

	/// Raw default-Run RefLocal replacement adapter.
	pub(crate) struct BoxRefLocalRawRunReplacer<E> {
		pub(crate) local_env: E,
	}

	/// Raw RcRun RefLocal replacement adapter.
	pub(crate) struct RcRefLocalRawRunReplacer<E> {
		pub(crate) local_env: E,
	}

	/// Raw ArcRun RefLocal replacement adapter.
	pub(crate) struct ArcRefLocalRawRunReplacer<E> {
		pub(crate) local_env: E,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Reader environment type."
	)]
	#[document_parameters("The raw default-Run RefLocal replacement adapter.")]
	impl<R, S, E> RunFirstOrderReplacer<BoxReaderBrand<BoxBrand, E>, R, S>
		for BoxRefLocalRawRunReplacer<E>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		E: Clone + 'static,
	{
		/// Answers a raw Reader ask with the borrowed-local
		/// environment value.
		#[document_signature]
		#[document_type_parameters("The current raw branch result type.")]
		#[document_parameters("The lowered Reader operation selected by raw RefLocal dispatch.")]
		#[document_returns("The action program resumed with the borrowed-local environment.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw RefLocal replacement adapter is crate-private and its fields are crate-private; external doctests cannot construct the adapter to call the trait method directly, so the example documents the borrowed-local ask-answering semantics."
		)]
		///
		/// ```
		/// let inherited_env = 20;
		/// let local_env = inherited_env + 1;
		/// let ask_continuation = |env| env * 2;
		///
		/// assert_eq!(ask_continuation(local_env), 42);
		/// ```
		fn replace<T: 'static>(
			&self,
			effect: BoxReader<'static, BoxBrand, E, Run<R, S, T>>,
		) -> Run<R, S, T> {
			match effect {
				BoxReader::Ask(k) => k(self.local_env.clone()),
			}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Reader environment type."
	)]
	#[document_parameters("The raw RcRun RefLocal replacement adapter.")]
	impl<R, S, E> RcRunFirstOrderReplacer<ReaderBrand<RcBrand, E>, R, S> for RcRefLocalRawRunReplacer<E>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		E: Clone + 'static,
	{
		/// Answers a raw Reader ask with the borrowed-local
		/// environment value.
		#[document_signature]
		#[document_type_parameters("The current raw branch result type.")]
		#[document_parameters("The lowered Reader operation selected by raw RefLocal dispatch.")]
		#[document_returns("The action program resumed with the borrowed-local environment.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw RefLocal replacement adapter is crate-private and its fields are crate-private; external doctests cannot construct the adapter to call the trait method directly, so the example documents the borrowed-local ask-answering semantics."
		)]
		///
		/// ```
		/// let inherited_env = String::from("parent");
		/// let local_env = inherited_env.replace("parent", "local");
		/// let ask_continuation = |env: String| env.len();
		///
		/// assert_eq!(ask_continuation(local_env), 5);
		/// ```
		fn replace<T: Clone + 'static>(
			&self,
			effect: Reader<'static, RcBrand, E, RcRun<R, S, T>>,
		) -> RcRun<R, S, T> {
			match effect {
				Reader::Ask(k) => k(self.local_env.clone()),
			}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Reader environment type."
	)]
	#[document_parameters("The raw ArcRun RefLocal replacement adapter.")]
	impl<R, S, E> ArcRunFirstOrderReplacer<SendReaderBrand<ArcBrand, E>, R, S>
		for ArcRefLocalRawRunReplacer<E>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		E: Clone + Send + Sync + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
	{
		/// Answers a raw SendReader ask with the borrowed-local
		/// environment value.
		#[document_signature]
		#[document_type_parameters("The current raw branch result type.")]
		#[document_parameters(
			"The lowered SendReader operation selected by raw RefLocal dispatch."
		)]
		#[document_returns("The action program resumed with the borrowed-local environment.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw RefLocal replacement adapter is crate-private and its fields are crate-private; external doctests cannot construct the adapter to call the trait method directly, so the example documents the SendReader borrowed-local ask-answering semantics."
		)]
		///
		/// ```
		/// let inherited_env = 7;
		/// let local_env = inherited_env + 35;
		/// let ask_continuation = |env| env;
		///
		/// assert_eq!(ask_continuation(local_env), 42);
		/// ```
		fn replace<T: Clone + Send + Sync + 'static>(
			&self,
			effect: SendReader<'static, ArcBrand, E, ArcRun<R, S, T>>,
		) -> ArcRun<R, S, T> {
			match effect {
				SendReader::Ask(k) => k(self.local_env.clone()),
			}
		}
	}
}

pub(crate) use inner::*;
