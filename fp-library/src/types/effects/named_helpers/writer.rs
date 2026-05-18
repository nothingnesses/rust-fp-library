//! Named Writer helpers layered over the Run wrapper primitives.
//!
//! These helpers expose fold-style and monoidal Writer runners for the
//! existing first-order `tell` operation.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				CNilBrand,
				NodeBrand,
				WriterBrand,
			},
			classes::{
				Functor,
				Monoid,
				SendFunctor,
				WrapDrop,
			},
			kinds::*,
			types::{
				ArcCoyoneda,
				ArcFree,
				ArcFreeExplicit,
				Coyoneda,
				RcCoyoneda,
				RcFree,
				RcFreeExplicit,
				arc_free::ArcTypeErasedValue,
				effects::{
					arc_run::ArcRun,
					arc_run_explicit::ArcRunExplicit,
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::RcRunExplicit,
					run::Run,
					run_explicit::RunExplicit,
					writer::Writer,
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
		std::{
			cell::RefCell,
			rc::Rc as StdRc,
			sync::{
				Arc as StdArc,
				Mutex,
			},
		},
	};

	/// Rc-backed deferred fold function used to restore Writer fold order.
	type SharedWriterFold<'a, Acc> = StdRc<dyn Fn(Acc) -> Acc + 'a>;

	/// Mutable Rc-backed deferred fold chain.
	type SharedWriterFoldCell<'a, Acc> = StdRc<RefCell<SharedWriterFold<'a, Acc>>>;

	/// Arc-backed deferred fold function used to restore Writer fold order.
	type SendSharedWriterFold<'a, Acc> = StdArc<dyn Fn(Acc) -> Acc + Send + Sync + 'a>;

	/// Mutable Arc-backed deferred fold chain.
	type SendSharedWriterFoldCell<'a, Acc> = StdArc<Mutex<SendSharedWriterFold<'a, Acc>>>;

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one Writer effect with an explicit accumulator function.
		///
		/// Each `tell(log)` contributes `folder(current, log)` in program
		/// order. Other first-order effects remain in the narrowed row and
		/// continue to be represented by the returned `Run` program.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `Run` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> =
		/// 	Run::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| Run::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| Run::<Row, CNilBrand, i32>::pure(7));
		/// let handled: Run<CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'static,
		) -> Run<RMinusWriter, CNilBrand, (A, Acc)>
		where
			LogType: Clone + 'static,
			Acc: Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, WriterBrand<LogType>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'static, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'static, LogType, Run<RMinusWriter, CNilBrand, A>>| match op {
					Writer::Tell(log, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let folder = StdRc::clone(&folder);
						*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}

		/// Interprets one Writer effect by appending emitted logs.
		///
		/// `run_writer()` is the monoidal specialization of
		/// [`Run::fold_writer`]. The accumulator starts at
		/// [`Monoid::empty`] and each `tell(log)` appends that log to
		/// the accumulated value.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> =
		/// 	Run::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| Run::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| Run::<Row, CNilBrand, i32>::pure(7));
		/// let handled: Run<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> Run<RMinusWriter, CNilBrand, (A, LogType)>
		where
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, A>,
			>): Member<
					Coyoneda<'static, WriterBrand<LogType>, Run<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `RcRun` program to interpret.")]
	impl<R, A> RcRun<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one Writer effect with an explicit accumulator function.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| RcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| RcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'static,
		) -> RcRun<RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone,
			LogType: Clone + 'static,
			Acc: Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusWriter, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, WriterBrand<LogType>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'static, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'static, LogType, RcRun<RMinusWriter, CNilBrand, A>>| match op {
					Writer::Tell(log, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let folder = StdRc::clone(&folder);
						*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}

		/// Interprets one Writer effect by appending emitted logs.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> =
		/// 	RcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| RcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| RcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRun<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> RcRun<RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone,
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusWriter, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, WriterBrand<LogType>, RcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `ArcRun` program to interpret.")]
	impl<R, A> ArcRun<R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, CNilBrand>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Send + Sync + 'static,
	{
		/// Interprets one Writer effect with an explicit accumulator function.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + Send + Sync + 'static,
		) -> ArcRun<RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone + Send + Sync,
			LogType: Clone + Send + Sync + 'static,
			Acc: Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, WriterBrand<LogType>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
		>,{
			let folder = StdArc::new(folder);
			let accumulator: SendSharedWriterFoldCell<'static, Acc> =
				StdArc::new(Mutex::new(StdArc::new(|acc| acc)));
			let handler_accumulator = StdArc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'static, LogType, ArcRun<RMinusWriter, CNilBrand, A>>| match op {
					Writer::Tell(log, next, _) => {
						let previous = {
							let guard = match handler_accumulator.lock() {
								Ok(guard) => guard,
								Err(poisoned) => poisoned.into_inner(),
							};
							guard.clone()
						};
						let folder = StdArc::clone(&folder);
						let mut guard = match handler_accumulator.lock() {
							Ok(guard) => guard,
							Err(poisoned) => poisoned.into_inner(),
						};
						*guard = StdArc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = {
					let guard = match accumulator.lock() {
						Ok(guard) => guard,
						Err(poisoned) => poisoned.into_inner(),
					};
					guard.clone()
				};
				(result, finish(initial.clone()))
			})
		}

		/// Interprets one Writer effect by appending emitted logs.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> =
		/// 	ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, ()>::tell::<String, _>("second".to_string()))
		/// 		.bind(|()| ArcRun::<Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRun<CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> ArcRun<RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone + Send + Sync,
			LogType: Monoid + Clone + Send + Sync + 'static,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusWriter, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, WriterBrand<LogType>, ArcRun<R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
		>,{
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` program to interpret.")]
	impl<'a, R, A> RunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Interprets one Writer effect with an explicit accumulator function.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("second".to_string())
		/// 		})
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> = program
		/// 	.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'a,
		) -> RunExplicit<'a, RMinusWriter, CNilBrand, (A, Acc)>
		where
			LogType: Clone + 'static,
			Acc: Clone + 'a,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<'a, WriterBrand<LogType>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'a, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'a, LogType, RunExplicit<'a, RMinusWriter, CNilBrand, A>>| match op
				{
					Writer::Tell(log, next, _) => {
						let previous = handler_accumulator.borrow().clone();
						let folder = StdRc::clone(&folder);
						*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
							let after_current = folder(initial, log.clone());
							previous(after_current)
						});
						next
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}

		/// Interprets one Writer effect by appending emitted logs.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `RunExplicit` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("second".to_string())
		/// 		})
		/// 		.bind(|()| RunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> RunExplicit<'a, RMinusWriter, CNilBrand, (A, LogType)>
		where
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					Coyoneda<'a, WriterBrand<LogType>, RunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` program to interpret.")]
	impl<'a, R, A> RcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Interprets one Writer effect with an explicit accumulator function.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + 'a,
		) -> RcRunExplicit<'a, RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone,
			LogType: Clone + 'static,
			Acc: Clone + 'a,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, WriterBrand<LogType>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdRc::new(folder);
			let accumulator: SharedWriterFoldCell<'a, Acc> =
				StdRc::new(RefCell::new(StdRc::new(|acc| acc)));
			let handler_accumulator = StdRc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'a, LogType, RcRunExplicit<'a, RMinusWriter, CNilBrand, A>>| {
					match op {
						Writer::Tell(log, next, _) => {
							let previous = handler_accumulator.borrow().clone();
							let folder = StdRc::clone(&folder);
							*handler_accumulator.borrow_mut() = StdRc::new(move |initial| {
								let after_current = folder(initial, log.clone());
								previous(after_current)
							});
							next
						}
					}
				},
			);
			handled.map(move |result| {
				let finish = accumulator.borrow().clone();
				(result, finish(initial.clone()))
			})
		}

		/// Interprets one Writer effect by appending emitted logs.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns("A first-order-only `RcRunExplicit` program returning `(result, log)`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			RcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> RcRunExplicit<'a, RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone,
			LogType: Monoid + Clone + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, WriterBrand<LogType>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}

	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` program to interpret.")]
	impl<'a, R, A> ArcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'a,
	{
		/// Interprets one Writer effect with an explicit accumulator function.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The accumulated value type.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_parameters(
			"The initial accumulator value.",
			"The function that incorporates one emitted log into the accumulator."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `(result, accumulated_log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, Vec<String>)> =
		/// 	program.fold_writer::<String, Vec<String>, _, CNilBrand>(Vec::new(), |mut logs, log| {
		/// 		logs.push(log);
		/// 		logs
		/// 	});
		/// assert_eq!(handled.extract(), (7, vec!["first".to_string(), "second".to_string()]));
		/// ```
		#[inline]
		pub fn fold_writer<LogType, Acc, Idx, RMinusWriter>(
			self,
			initial: Acc,
			folder: impl Fn(Acc, LogType) -> Acc + Send + Sync + 'a,
		) -> ArcRunExplicit<'a, RMinusWriter, CNilBrand, (A, Acc)>
		where
			A: Clone + Send + Sync,
			LogType: Clone + Send + Sync + 'static,
			Acc: Clone + Send + Sync + 'a,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, Acc)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, WriterBrand<LogType>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			let folder = StdArc::new(folder);
			let accumulator: SendSharedWriterFoldCell<'a, Acc> =
				StdArc::new(Mutex::new(StdArc::new(|acc| acc)));
			let handler_accumulator = StdArc::clone(&accumulator);
			let handled = self.handle_with::<WriterBrand<LogType>, Idx, RMinusWriter>(
				move |op: Writer<'a, LogType, ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>>| {
					match op {
						Writer::Tell(log, next, _) => {
							let previous = {
								let guard = match handler_accumulator.lock() {
									Ok(guard) => guard,
									Err(poisoned) => poisoned.into_inner(),
								};
								guard.clone()
							};
							let folder = StdArc::clone(&folder);
							let mut guard = match handler_accumulator.lock() {
								Ok(guard) => guard,
								Err(poisoned) => poisoned.into_inner(),
							};
							*guard = StdArc::new(move |initial| {
								let after_current = folder(initial, log.clone());
								previous(after_current)
							});
							next
						}
					}
				},
			);
			handled.map(move |result| {
				let finish = {
					let guard = match accumulator.lock() {
						Ok(guard) => guard,
						Err(poisoned) => poisoned.into_inner(),
					};
					guard.clone()
				};
				(result, finish(initial.clone()))
			})
		}

		/// Interprets one Writer effect by appending emitted logs.
		#[document_signature]
		#[document_type_parameters(
			"The log type carried by `WriterBrand`.",
			"The type-level Member-position witness for the Writer effect.",
			"The first-order row brand with the Writer effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `(result, log)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<WriterBrand<String>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>("first".to_string())
		/// 		.bind(|()| {
		/// 			ArcRunExplicit::<'static, Row, CNilBrand, ()>::tell::<String, _>(
		/// 				"second".to_string(),
		/// 			)
		/// 		})
		/// 		.bind(|()| ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(7));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, (i32, String)> =
		/// 	program.run_writer::<String, _, CNilBrand>();
		/// assert_eq!(handled.extract(), (7, "firstsecond".to_string()));
		/// ```
		#[inline]
		pub fn run_writer<LogType, Idx, RMinusWriter>(
			self
		) -> ArcRunExplicit<'a, RMinusWriter, CNilBrand, (A, LogType)>
		where
			A: Clone + Send + Sync,
			LogType: Monoid + Clone + Send + Sync + 'static,
			RMinusWriter: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusWriter, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<RMinusWriter, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Clone + Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Send + Sync,
			Apply!(<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusWriter, CNilBrand>, (A, LogType)>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusWriter, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, WriterBrand<LogType>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusWriter as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.fold_writer::<LogType, LogType, Idx, RMinusWriter>(
				LogType::empty(),
				LogType::append,
			)
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
