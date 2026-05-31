//! Named Reader helpers layered over the Run wrapper primitives.
//!
//! The methods remain inherent methods on the public wrapper types.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				BoxBrand,
				BoxReaderBrand,
				CNilBrand,
				NodeBrand,
				RcBrand,
				ReaderBrand,
				SendReaderBrand,
			},
			classes::{
				Functor,
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
					reader::{
						BoxReader,
						Reader,
						SendReader,
					},
					run::Run,
					run_explicit::RunExplicit,
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
	};

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> Run<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect Reader;
			method asks;
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper Run;
			effect Reader;
			method run_reader;
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> RcRun<R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper RcRun;
			effect Reader;
			method asks;
		}
	}

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `RcRun` program to interpret.")]
	impl<R, A> RcRun<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		define_run_wrapper! {
			wrapper RcRun;
			effect Reader;
			method run_reader;
		}
	}

	#[document_type_parameters(
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<R, S, A> ArcRun<R, S, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + 'static,
		A: Send + Sync + 'static,
	{
		define_run_wrapper! {
			wrapper ArcRun;
			effect Reader;
			method asks;
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
		define_run_wrapper! {
			wrapper ArcRun;
			effect Reader;
			method run_reader;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, S, A> RunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect Reader;
			method asks;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RunExplicit` program to interpret.")]
	impl<'a, R, A> RunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		define_run_wrapper! {
			wrapper RunExplicit;
			effect Reader;
			method run_reader;
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, S, A> RcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Reads the Reader environment and maps it immediately.
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns(
			"An `RcRunExplicit` program that asks for the environment and returns `f(env)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, String> =
		/// 	RcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		/// 	program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl Fn(E) -> A + 'a) -> Self
		where
			E: Clone + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
				Member<RcCoyoneda<'a, ReaderBrand<RcBrand, E>, E>, Idx>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone, {
			RcRunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `RcRunExplicit` program to interpret.")]
	impl<'a, R, A> RcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'a,
	{
		/// Interprets one Reader effect by supplying a fixed environment.
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program with the Reader effect removed."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ReaderBrand<RcBrand, i32>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> RcRunExplicit<'a, RMinusReader, CNilBrand, A>
		where
			A: Clone,
			E: Clone + 'static + 'a,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, ReaderBrand<RcBrand, E>, RcRunExplicit<'a, R, CNilBrand, A>>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<ReaderBrand<RcBrand, E>, Idx, RMinusReader>(
				move |op: Reader<'a, RcBrand, E, RcRunExplicit<'a, RMinusReader, CNilBrand, A>>| {
					match op {
						Reader::Ask(k) => (*k)(env.clone()),
					}
				},
			)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The scoped-effect row brand.",
		"The result type."
	)]
	impl<'a, R, S, A> ArcRunExplicit<'a, R, S, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'a,
	{
		/// Reads the Reader environment and maps it immediately.
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect."
		)]
		#[document_parameters("The projection to apply to the environment.")]
		#[document_returns(
			"An `ArcRunExplicit` program that asks for the environment and returns `f(env)`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, String> =
		/// 	ArcRunExplicit::asks::<i32, _>(|env| format!("env={env}"));
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, String> =
		/// 	program.run_reader::<i32, _, CNilBrand>(7);
		/// assert_eq!(handled.extract(), "env=7");
		/// ```
		#[inline]
		pub fn asks<E, Idx>(f: impl Fn(E) -> A + Send + Sync + 'a) -> Self
		where
			A: Clone,
			E: Clone + Send + Sync + 'static,
			NodeBrand<R, S>: SendFunctor,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, E>):
				Member<ArcCoyoneda<'a, SendReaderBrand<ArcBrand, E>, E>, Idx>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<S as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, E>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, A>,
			>): Clone + Send + Sync, {
			ArcRunExplicit::<'a, R, S, E>::ask::<Idx>().map(f)
		}
	}

	#[document_type_parameters(
		"The lifetime carried by the explicit wrapper.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The `ArcRunExplicit` program to interpret.")]
	impl<'a, R, A> ArcRunExplicit<'a, R, CNilBrand, A>
	where
		R: WrapDrop + SendFunctor + 'static,
		A: Send + Sync + 'a,
	{
		/// Interprets one Reader effect by supplying a fixed environment.
		#[document_signature]
		#[document_type_parameters(
			"The Reader environment type.",
			"The type-level Member-position witness for the Reader effect.",
			"The first-order row brand with the Reader effect removed."
		)]
		#[document_parameters("The environment value supplied to every Reader ask.")]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program with the Reader effect removed."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendReaderBrand<ArcBrand, i32>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, i32>::ask().map(|env| env + 1);
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, i32> =
		/// 	program.run_reader::<i32, _, CNilBrand>(41);
		/// assert_eq!(handled.extract(), 42);
		/// ```
		#[inline]
		pub fn run_reader<E, Idx, RMinusReader>(
			self,
			env: E,
		) -> ArcRunExplicit<'a, RMinusReader, CNilBrand, A>
		where
			A: Clone + Send + Sync,
			E: Clone + Send + Sync + 'static + 'a,
			RMinusReader: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
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
			Apply!(<NodeBrand<RMinusReader, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusReader, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusReader, CNilBrand, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<
						'a,
						SendReaderBrand<ArcBrand, E>,
						ArcRunExplicit<'a, R, CNilBrand, A>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusReader as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			self.handle_with::<SendReaderBrand<ArcBrand, E>, Idx, RMinusReader>(
				move |op: SendReader<
					'a,
					ArcBrand,
					E,
					ArcRunExplicit<'a, RMinusReader, CNilBrand, A>,
				>| {
					match op {
						SendReader::Ask(k) => (*k)(env.clone()),
					}
				},
			)
		}
	}
}
