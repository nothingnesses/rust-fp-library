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
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
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
	#[document_parameters("The Writer preserving accumulator.")]
	impl<R, S, W> RunFirstOrderPreservingAccumulator<WriterBrand<W>, R, S, W>
		for BoxWriterAccumulator<W>
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
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Preserves one default-wrapper Writer `Tell` and prepends its log to the accumulated suffix.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns("The Writer operation with its original log preserved and accumulated.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	types::effects::{
		/// 		run::Run,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// let next = Run::<CNilBrand, CNilBrand, (i32, String)>::pure((7, "second".into()));
		/// let effect = Writer::Tell("first".to_string(), next, core::marker::PhantomData);
		/// let Writer::Tell(log, next, marker) = effect;
		/// let observed_log = log.clone();
		/// let preserved =
		/// 	Writer::Tell(log, next.map(move |(value, suffix)| (value, observed_log + &suffix)), marker);
		/// let Writer::Tell(emitted_log, next, _) = preserved;
		/// assert_eq!(emitted_log, "first");
		/// assert_eq!(next.extract(), (7, "firstsecond".to_string()));
		/// ```
		fn accumulate_preserving<T: 'static>(
			&self,
			effect: Writer<'static, W, Run<R, S, (T, W)>>,
		) -> Writer<'static, W, Run<R, S, (T, W)>> {
			match effect {
				Writer::Tell(log, next, marker) => {
					let accumulated_log = log.clone();
					Writer::Tell(
						log,
						next.map(move |(value, accumulated)| {
							(value, W::append(accumulated_log, accumulated))
						}),
						marker,
					)
				}
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
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
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
	#[document_parameters("The Writer preserving accumulator.")]
	impl<R, S, W> RcRunFirstOrderPreservingAccumulator<WriterBrand<W>, R, S, W>
		for RcWriterAccumulator<W>
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
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Preserves one Rc-wrapper Writer `Tell` and prepends its log to the accumulated suffix.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns("The Writer operation with its original log preserved and accumulated.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	types::effects::{
		/// 		rc_run::RcRun,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// let next = RcRun::<CNilBrand, CNilBrand, (i32, String)>::pure((7, "second".into()));
		/// let effect = Writer::Tell("first".to_string(), next, core::marker::PhantomData);
		/// let Writer::Tell(log, next, marker) = effect;
		/// let observed_log = log.clone();
		/// let preserved = Writer::Tell(
		/// 	log,
		/// 	next.map(move |(value, suffix)| (value, observed_log.clone() + &suffix)),
		/// 	marker,
		/// );
		/// let Writer::Tell(emitted_log, next, _) = preserved;
		/// assert_eq!(emitted_log, "first");
		/// assert_eq!(next.extract(), (7, "firstsecond".to_string()));
		/// ```
		fn accumulate_preserving<T: Clone + 'static>(
			&self,
			effect: Writer<'static, W, RcRun<R, S, (T, W)>>,
		) -> Writer<'static, W, RcRun<R, S, (T, W)>> {
			match effect {
				Writer::Tell(log, next, marker) => {
					let accumulated_log = log.clone();
					Writer::Tell(
						log,
						next.map(move |(value, accumulated)| {
							(value, W::append(accumulated_log.clone(), accumulated))
						}),
						marker,
					)
				}
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
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
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

	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer preserving accumulator.")]
	impl<R, S, W> ArcRunFirstOrderPreservingAccumulator<WriterBrand<W>, R, S, W>
		for ArcWriterAccumulator<W>
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
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Preserves one Arc-wrapper Writer `Tell` and prepends its log to the accumulated suffix.
		#[document_signature]
		#[document_type_parameters("The current branch result type.")]
		#[document_parameters("The lowered Writer operation.")]
		#[document_returns("The Writer operation with its original log preserved and accumulated.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::CNilBrand,
		/// 	types::effects::{
		/// 		arc_run::ArcRun,
		/// 		writer::Writer,
		/// 	},
		/// };
		///
		/// let next = ArcRun::<CNilBrand, CNilBrand, (i32, String)>::pure((7, "second".into()));
		/// let effect = Writer::Tell("first".to_string(), next, core::marker::PhantomData);
		/// let Writer::Tell(log, next, marker) = effect;
		/// let observed_log = log.clone();
		/// let preserved = Writer::Tell(
		/// 	log,
		/// 	next.map(move |(value, suffix)| (value, observed_log.clone() + &suffix)),
		/// 	marker,
		/// );
		/// let Writer::Tell(emitted_log, next, _) = preserved;
		/// assert_eq!(emitted_log, "first");
		/// assert_eq!(next.extract(), (7, "firstsecond".to_string()));
		/// ```
		fn accumulate_preserving<T: Clone + Send + Sync + 'static>(
			&self,
			effect: Writer<'static, W, ArcRun<R, S, (T, W)>>,
		) -> Writer<'static, W, ArcRun<R, S, (T, W)>> {
			match effect {
				Writer::Tell(log, next, marker) => {
					let accumulated_log = log.clone();
					Writer::Tell(
						log,
						next.map(move |(value, accumulated)| {
							(value, W::append(accumulated_log.clone(), accumulated))
						}),
						marker,
					)
				}
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer accumulator.")]
	impl<'a, R, S, W> RunExplicitFirstOrderAccumulator<'a, WriterBrand<W>, R, S, W>
		for BoxWriterAccumulator<W>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		W: Monoid + Clone + 'static,
	{
		/// Produces the neutral Writer log for a selected action with no `Tell`s.
		#[document_signature]
		#[document_returns("The neutral Writer log.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Consumes one single-shot Explicit Writer `Tell` and prepends its log to the accumulated suffix.
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
		fn accumulate<T: 'a>(
			&self,
			effect: Writer<'a, W, RunExplicit<'a, R, S, (T, W)>>,
		) -> RunExplicit<'a, R, S, (T, W)> {
			match effect {
				Writer::Tell(log, next, _) => next
					.map(move |(value, accumulated)| (value, W::append(log.clone(), accumulated))),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer accumulator.")]
	impl<'a, R, S, W> RcRunExplicitFirstOrderAccumulator<'a, WriterBrand<W>, R, S, W>
		for RcWriterAccumulator<W>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		W: Monoid + Clone + 'static,
	{
		/// Produces the neutral Writer log for a selected action with no `Tell`s.
		#[document_signature]
		#[document_returns("The neutral Writer log.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Consumes one Rc Explicit Writer `Tell` and prepends its log to the accumulated suffix.
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
		fn accumulate<T: Clone + 'a>(
			&self,
			effect: Writer<'a, W, RcRunExplicit<'a, R, S, (T, W)>>,
		) -> RcRunExplicit<'a, R, S, (T, W)>
		where
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, W)>,
			>): Clone,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, W)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (T, W)>,
			>): Clone, {
			match effect {
				Writer::Tell(log, next, _) => next
					.map(move |(value, accumulated)| (value, W::append(log.clone(), accumulated))),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The Writer log type."
	)]
	#[document_parameters("The Writer accumulator.")]
	impl<'a, R, S, W> ArcRunExplicitFirstOrderAccumulator<'a, WriterBrand<W>, R, S, W>
		for ArcWriterAccumulator<W>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		W: Monoid + Clone + Send + Sync + 'static,
	{
		/// Produces the neutral Writer log for a selected action with no `Tell`s.
		#[document_signature]
		#[document_returns("The neutral Writer log.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::classes::Monoid;
		///
		/// fn selected_action_without_tells<W: Monoid>(value: i32) -> (i32, W) {
		/// 	(value, W::empty())
		/// }
		///
		/// assert_eq!(selected_action_without_tells::<String>(7), (7, String::new()));
		/// ```
		fn empty(&self) -> W {
			W::empty()
		}

		/// Consumes one Arc Explicit Writer `Tell` and prepends its log to the accumulated suffix.
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
		fn accumulate<T: Clone + Send + Sync + 'a>(
			&self,
			effect: Writer<'a, W, ArcRunExplicit<'a, R, S, (T, W)>>,
		) -> ArcRunExplicit<'a, R, S, (T, W)>
		where
			ArcFreeExplicit<'a, NodeBrand<R, S>, (T, W)>: Send + Sync,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, (T, W)>,
			>): Clone + Send + Sync,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, (T, W)>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, (T, W)>,
			>): Clone + Send + Sync, {
			match effect {
				Writer::Tell(log, next, _) => next
					.map(move |(value, accumulated)| (value, W::append(log.clone(), accumulated))),
			}
		}
	}
}

pub(crate) use inner::*;
