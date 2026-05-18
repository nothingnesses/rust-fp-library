//! Named nondeterminism helpers layered over the Run wrapper primitives.
//!
//! These helpers expose `Empty` as `Option` and `Choose` as `Vec`-backed
//! branch collection.

#[fp_macros::document_module]
pub(crate) mod inner {
	use {
		crate::{
			Apply,
			brands::{
				ArcBrand,
				CNilBrand,
				ChooseBrand,
				EmptyBrand,
				NodeBrand,
				RcBrand,
				SendChooseBrand,
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
					choose::{
						Choose,
						SendChoose,
					},
					empty::Empty,
					member::Member,
					rc_run::RcRun,
					rc_run_explicit::RcRunExplicit,
					run::Run,
					run_explicit::RunExplicit,
				},
				rc_free::RcTypeErasedValue,
			},
		},
		fp_macros::*,
	};

	#[document_type_parameters("The first-order effect row brand.", "The result type.")]
	#[document_parameters("The `Run` program to interpret.")]
	impl<R, A> Run<R, CNilBrand, A>
	where
		R: WrapDrop + Functor + 'static,
		A: 'static,
	{
		/// Interprets one Empty effect into `Option`.
		///
		/// Normal completion becomes `Some(value)`. An `Empty` operation
		/// aborts the current branch and becomes `None`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns("A first-order-only `Run` program returning `Some(result)` or `None`.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run::Run,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: Run<Row, CNilBrand, i32> = Run::empty();
		/// let handled: Run<CNilBrand, CNilBrand, Option<i32>> = program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(self) -> Run<RMinusEmpty, CNilBrand, Option<A>>
		where
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				Run<R, CNilBrand, Option<A>>,
			>): Member<
					Coyoneda<'static, EmptyBrand, Run<R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										Run<R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'static, Run<RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => Run::pure(None),
				},
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
		/// Interprets one Empty effect into `Option`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::empty();
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> = program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(self) -> RcRun<RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusEmpty, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, Option<A>>,
			>): Member<
					RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'static, RcRun<RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => RcRun::pure(None),
				},
			)
		}

		/// Interprets one Choose effect into a `Vec`.
		///
		/// Pure results become singleton vectors. Each `choose()` branches
		/// into the `true` path followed by the `false` path and concatenates
		/// the branch results in that order.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning all branch results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| RcRun::<Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> = program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(self) -> RcRun<RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusChoose, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, Vec<A>>,
			>): Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, Vec<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, Vec<A>>,
									>
								),
				>, {
			self.map(|value| vec![value]).handle_with::<ChooseBrand<RcBrand>, Idx, RMinusChoose>(
				|op: Choose<'static, RcBrand, RcRun<RMinusChoose, CNilBrand, Vec<A>>>| match op {
					Choose::Alt(k) => {
						let true_branch = (*k)(true);
						let false_branch = (*k)(false);
						true_branch.bind(move |true_values| {
							let false_branch = false_branch.clone();
							false_branch.map(move |false_values| {
								let mut values = true_values.clone();
								values.extend(false_values);
								values
							})
						})
					}
				},
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
		/// Interprets one Empty effect into `Option`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::empty();
		/// let handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> = program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> ArcRun<RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusEmpty, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusEmpty, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusEmpty, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, Option<A>>,
			>): Member<
					ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, Option<A>>,
									>
								),
		>,{
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'static, ArcRun<RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => ArcRun::pure(None),
				},
			)
		}

		/// Interprets one Choose effect into a `Vec`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning all branch results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| ArcRun::<Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 }));
		/// let handled: ArcRun<CNilBrand, CNilBrand, Vec<i32>> = program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(
			self
		) -> ArcRun<RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusChoose, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusChoose, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusChoose, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, Vec<A>>,
			>): Member<
					ArcCoyoneda<
						'static,
						SendChooseBrand<ArcBrand>,
						ArcRun<R, CNilBrand, Vec<A>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, Vec<A>>,
									>
								),
		>,{
			self.map(|value| vec![value])
				.handle_with::<SendChooseBrand<ArcBrand>, Idx, RMinusChoose>(
					|op: SendChoose<'static, ArcBrand, ArcRun<RMinusChoose, CNilBrand, Vec<A>>>| {
						match op {
							SendChoose::Alt(k) => {
								let true_branch = (*k)(true);
								let false_branch = (*k)(false);
								true_branch.bind(move |true_values| {
									let false_branch = false_branch.clone();
									false_branch.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
								})
							}
						}
					},
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
		/// Interprets one Empty effect into `Option`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `RunExplicit` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::run_explicit::RunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<CoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: RunExplicit<'static, Row, CNilBrand, i32> = RunExplicit::empty();
		/// let handled: RunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> RunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>
		where
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Member<
					Coyoneda<'a, EmptyBrand, RunExplicit<'a, R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RunExplicit<'a, R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'a, RunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => RunExplicit::pure(None),
				},
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
		/// Interprets one Empty effect into `Option`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> = RcRunExplicit::empty();
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> RcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Clone,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Member<
					RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'a, RcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => RcRunExplicit::pure(None),
				},
			)
		}

		/// Interprets one Choose effect into a `Vec`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning all branch results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, CNilBrand>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, bool>::choose().bind(|branch| {
		/// 		RcRunExplicit::<'static, Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(
			self
		) -> RcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Clone,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, Vec<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, Vec<A>>,
									>
								),
				>, {
			self.map(|value| vec![value]).handle_with::<ChooseBrand<RcBrand>, Idx, RMinusChoose>(
				|op: Choose<'a, RcBrand, RcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>>| {
					match op {
						Choose::Alt(k) => {
							let true_branch = (*k)(true);
							let false_branch = (*k)(false);
							true_branch.bind(move |true_values| {
								let false_branch = false_branch.clone();
								false_branch.map(move |false_values| {
									let mut values = true_values.clone();
									values.extend(false_values);
									values
								})
							})
						}
					}
				},
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
		/// Interprets one Empty effect into `Option`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Empty effect.",
			"The first-order row brand with the Empty effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning `Some(result)` or `None`."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> = ArcRunExplicit::empty();
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_empty::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), None);
		/// ```
		#[inline]
		pub fn run_empty<Idx, RMinusEmpty>(
			self
		) -> ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			RMinusEmpty: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusEmpty, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusEmpty, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusEmpty, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
			>): Member<
					ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, Option<A>>>,
					Idx,
					Remainder = Apply!(
									<RMinusEmpty as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, Option<A>>,
									>
								),
				>, {
			self.map(Some).handle_with::<EmptyBrand, Idx, RMinusEmpty>(
				|op: Empty<'a, ArcRunExplicit<'a, RMinusEmpty, CNilBrand, Option<A>>>| match op {
					Empty::Empty(_) => ArcRunExplicit::pure(None),
				},
			)
		}

		/// Interprets one Choose effect into a `Vec`.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The first-order row brand with the Choose effect removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning all branch results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, CNilBrand>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, bool>::choose().bind(|branch| {
		/// 		ArcRunExplicit::<'static, Row, CNilBrand, i32>::pure(if branch { 1 } else { 0 })
		/// 	});
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_choose::<_, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1, 0]);
		/// ```
		#[inline]
		pub fn run_choose<Idx, RMinusChoose>(
			self
		) -> ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusChoose, CNilBrand>: SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Clone + Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<NodeBrand<RMinusChoose, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusChoose, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
			>): Member<
					ArcCoyoneda<
						'a,
						SendChooseBrand<ArcBrand>,
						ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
					>,
					Idx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, Vec<A>>,
									>
								),
				>, {
			self.map(|value| vec![value])
				.handle_with::<SendChooseBrand<ArcBrand>, Idx, RMinusChoose>(
					|op: SendChoose<
						'a,
						ArcBrand,
						ArcRunExplicit<'a, RMinusChoose, CNilBrand, Vec<A>>,
					>| {
						match op {
							SendChoose::Alt(k) => {
								let true_branch = (*k)(true);
								let false_branch = (*k)(false);
								true_branch.bind(move |true_values| {
									let false_branch = false_branch.clone();
									false_branch.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
								})
							}
						}
					},
				)
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
