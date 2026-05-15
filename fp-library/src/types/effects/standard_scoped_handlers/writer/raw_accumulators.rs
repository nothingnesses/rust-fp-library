#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::super::prelude::*;

	/// Writer accumulator used by post-applying `censor` and `listen` on
	/// default `Run`.
	#[allow(
		dead_code,
		reason = "Constructed by the WriterPostHandler/listen wiring after the accumulation protocol is proven."
	)]
	pub(crate) struct BoxWriterAccumulator<W>(pub(crate) PhantomData<fn() -> W>);

	/// Writer accumulator used by post-applying `censor` and `listen` on
	/// `RcRun`.
	#[allow(
		dead_code,
		reason = "Constructed by the WriterPostHandler/listen wiring after the accumulation protocol is proven."
	)]
	pub(crate) struct RcWriterAccumulator<W>(pub(crate) PhantomData<fn() -> W>);

	/// Writer accumulator used by post-applying `censor` and `listen` on
	/// `ArcRun`.
	#[allow(
		dead_code,
		reason = "Constructed by the WriterPostHandler/listen wiring after the accumulation protocol is proven."
	)]
	pub(crate) struct ArcWriterAccumulator<W>(pub(crate) PhantomData<fn() -> W>);

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer accumulator.")]
	impl<R, S, W> RunFirstOrderAccumulator<WriterBrand<W>, R, S, W> for BoxWriterAccumulator<W>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		W: Monoid + Clone + 'static,
	{
		/// Produces the neutral Writer log for a selected action with no `Tell`s.
		#[document_signature]
		#[document_returns("The neutral Writer log.")]
		#[document_examples]
		///
		/// ```
		/// let accumulated_log = String::new();
		/// assert_eq!(accumulated_log, "");
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Consumes one default-wrapper Writer `Tell` and prepends its log to the accumulated suffix.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns("The continuation with the Writer log accumulated.")]
		#[document_examples]
		///
		/// ```
		/// let current_log = "first".to_string();
		/// let accumulated_suffix = "second".to_string();
		/// assert_eq!(current_log + &accumulated_suffix, "firstsecond");
		/// ```
		fn accumulate<T: 'static>(
			&self,
			effect: Writer<'static, W, Run<R, S, (T, W)>>,
		) -> Run<R, S, (T, W)> {
			match effect {
				Writer::Tell(log, next, _) =>
					next.map(move |(value, accumulated)| (value, W::append(log, accumulated))),
			}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer accumulator.")]
	impl<R, S, W> RcRunFirstOrderAccumulator<WriterBrand<W>, R, S, W> for RcWriterAccumulator<W>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
		W: Monoid + Clone + 'static,
	{
		/// Produces the neutral Writer log for a selected action with no `Tell`s.
		#[document_signature]
		#[document_returns("The neutral Writer log.")]
		#[document_examples]
		///
		/// ```
		/// let accumulated_log = String::new();
		/// assert_eq!(accumulated_log, "");
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Consumes one Rc-wrapper Writer `Tell` and prepends its log to the accumulated suffix.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns("The continuation with the Writer log accumulated.")]
		#[document_examples]
		///
		/// ```
		/// let current_log = "first".to_string();
		/// let accumulated_suffix = "second".to_string();
		/// assert_eq!(current_log + &accumulated_suffix, "firstsecond");
		/// ```
		fn accumulate<T: Clone + 'static>(
			&self,
			effect: Writer<'static, W, RcRun<R, S, (T, W)>>,
		) -> RcRun<R, S, (T, W)> {
			match effect {
				Writer::Tell(log, next, _) => next
					.map(move |(value, accumulated)| (value, W::append(log.clone(), accumulated))),
			}
		}
	}

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer accumulator.")]
	impl<R, S, W> ArcRunFirstOrderAccumulator<WriterBrand<W>, R, S, W> for ArcWriterAccumulator<W>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone,
		W: Monoid + Clone + Send + Sync + 'static,
	{
		/// Produces the neutral Writer log for a selected action with no `Tell`s.
		#[document_signature]
		#[document_returns("The neutral Writer log.")]
		#[document_examples]
		///
		/// ```
		/// let accumulated_log = String::new();
		/// assert_eq!(accumulated_log, "");
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Consumes one Arc-wrapper Writer `Tell` and prepends its log to the accumulated suffix.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns("The continuation with the Writer log accumulated.")]
		#[document_examples]
		///
		/// ```
		/// let current_log = "first".to_string();
		/// let accumulated_suffix = "second".to_string();
		/// assert_eq!(current_log + &accumulated_suffix, "firstsecond");
		/// ```
		fn accumulate<T: Clone + Send + Sync + 'static>(
			&self,
			effect: Writer<'static, W, ArcRun<R, S, (T, W)>>,
		) -> ArcRun<R, S, (T, W)> {
			match effect {
				Writer::Tell(log, next, _) => next
					.map(move |(value, accumulated)| (value, W::append(log.clone(), accumulated))),
			}
		}
	}
}
