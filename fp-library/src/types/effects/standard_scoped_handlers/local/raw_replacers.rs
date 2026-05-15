#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::super::prelude::*;

	/// Raw default-Run Local replacement adapter.
	pub(crate) struct BoxLocalRawRunReplacer<E> {
		pub(crate) local_env: E,
	}

	/// Raw RcRun Local replacement adapter.
	pub(crate) struct RcLocalRawRunReplacer<E> {
		pub(crate) local_env: E,
	}

	/// Raw ArcRun Local replacement adapter.
	pub(crate) struct ArcLocalRawRunReplacer<E> {
		pub(crate) local_env: E,
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Reader environment type."
	)]
	#[document_parameters("The raw default-Run Local replacement adapter.")]
	impl<R, S, E> RunFirstOrderReplacer<BoxReaderBrand<BoxBrand, E>, R, S> for BoxLocalRawRunReplacer<E>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		E: Clone + 'static,
	{
		/// Answers a raw Reader ask with the local environment value.
		#[document_signature]
		#[document_type_parameters("The current raw branch result type.")]
		#[document_parameters("The lowered Reader operation selected by raw Local dispatch.")]
		#[document_returns("The action program resumed with the local environment.")]
		#[document_examples]
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
	#[document_parameters("The raw RcRun Local replacement adapter.")]
	impl<R, S, E> RcRunFirstOrderReplacer<ReaderBrand<RcBrand, E>, R, S> for RcLocalRawRunReplacer<E>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		E: Clone + 'static,
	{
		/// Answers a raw Reader ask with the local environment value.
		#[document_signature]
		#[document_type_parameters("The current raw branch result type.")]
		#[document_parameters("The lowered Reader operation selected by raw Local dispatch.")]
		#[document_returns("The action program resumed with the local environment.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Prog = RcRun<CNilBrand, CNilBrand, i32>;
		/// let run: Prog = RcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
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
	#[document_parameters("The raw ArcRun Local replacement adapter.")]
	impl<R, S, E> ArcRunFirstOrderReplacer<SendReaderBrand<ArcBrand, E>, R, S>
		for ArcLocalRawRunReplacer<E>
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
		/// Answers a raw SendReader ask with the local environment value.
		#[document_signature]
		#[document_type_parameters("The current raw branch result type.")]
		#[document_parameters("The lowered SendReader operation selected by raw Local dispatch.")]
		#[document_returns("The action program resumed with the local environment.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Prog = ArcRun<CNilBrand, CNilBrand, i32>;
		/// let run: Prog = ArcRun::pure(42);
		/// assert_eq!(run.extract(), 42);
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
