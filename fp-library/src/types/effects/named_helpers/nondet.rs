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
					arc_run::{
						ArcRun,
						make_node_first,
						unwrap_node,
						wrap_first_arc,
					},
					arc_run_explicit::ArcRunExplicit,
					choose::{
						Choose,
						SendChoose,
					},
					empty::Empty,
					member::Member,
					node::Node,
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

	/// Normalizes an `ArcRunExplicit` first-order `Node` projection.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The result type."
	)]
	#[document_parameters("The projected `NodeBrand` value.")]
	#[document_returns("The normalized `Node` value.")]
	#[document_examples(
		skip_call_check,
		reason = "This private helper exists only to move a GAT projection normalization step out of the ArcRunExplicit NonDet helper body."
	)]
	///
	/// ```
	/// use fp_library::{
	/// 	brands::*,
	/// 	types::effects::arc_run_explicit::ArcRunExplicit,
	/// };
	///
	/// type Program = ArcRunExplicit<'static, CNilBrand, CNilBrand, i32>;
	/// let _shape: Option<Program> = None;
	/// assert!(_shape.is_none());
	/// ```
	fn unwrap_arc_run_explicit_nondet_node<'a, R, A>(
		node: Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcRunExplicit<'a, R, CNilBrand, A>,
		>)
	) -> Node<'a, R, CNilBrand, ArcRunExplicit<'a, R, CNilBrand, A>>
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		A: 'a, {
		node
	}

	/// Builds an `ArcRunExplicit` first-order `Node` projection.
	#[document_signature]
	#[document_type_parameters(
		"The lifetime of the program and its captures.",
		"The first-order effect row brand.",
		"The inner program type carried by the layer."
	)]
	#[document_parameters("The first-order layer payload.")]
	#[document_returns("The projected `NodeBrand` value.")]
	#[document_examples(
		skip_call_check,
		reason = "This private helper exists only to move a GAT projection construction step out of the ArcRunExplicit NonDet helper body."
	)]
	///
	/// ```
	/// use fp_library::brands::*;
	///
	/// type Row = CNilBrand;
	/// let _row: Option<Row> = None;
	/// assert!(_row.is_none());
	/// ```
	fn make_arc_run_explicit_nondet_node_first<'a, R, A>(
		layer: Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>)
	) -> Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<'a, A>)
	where
		R: Kind_cdc7cd43dac7585f + 'static,
		A: 'a, {
		Node::First(layer)
	}

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

		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns("A first-order-only `RcRun` program returning all successful results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { RcRun::pure(1) } else { RcRun::empty() });
		/// let handled: RcRun<CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRun<RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusNonDet, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRun::pure(vec![a]),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>
				) as Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |true_values| {
								let false_branch = (*k_for_false)(false);
								false_branch
									.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
									.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							RcRun<R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRun::pure(Vec::new())
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRun<R, CNilBrand, A>| {
									inner
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free()
								},
								rest,
							);
							RcRun::from_rc_free(
								RcFree::<NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>::wrap(
									Node::First(mapped_free),
								),
							)
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
		}

		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `RcRun` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRun<Row, CNilBrand, i32> = RcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { RcRun::empty() } else { RcRun::pure(2) });
		/// let handled: RcRun<CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRun<RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<R, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcFree<NodeBrand<RMinusNonDet, CNilBrand>, RcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				RcRun<R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										RcRun<R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRun::pure(Some(a)),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, CNilBrand, A>>
				) as Member<
					RcCoyoneda<'static, ChooseBrand<RcBrand>, RcRun<R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |first| match first {
								Some(value) => RcRun::pure(Some(value)),
								None => (*k_for_false)(false)
									.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
									),
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'static,
							RcRun<R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'static, EmptyBrand, RcRun<R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRun::pure(None)
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRun<R, CNilBrand, A>| {
									inner
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free()
								},
								rest,
							);
							RcRun::from_rc_free(RcFree::<
								NodeBrand<RMinusNonDet, CNilBrand>,
								Option<A>,
							>::wrap(Node::First(mapped_free)))
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
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

		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns("A first-order-only `ArcRun` program returning all successful results.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { ArcRun::pure(1) } else { ArcRun::empty() });
		/// let handled: ArcRun<CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRun<RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<
						'static,
						SendChooseBrand<ArcBrand>,
						ArcRun<R, CNilBrand, A>,
					>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
		>,{
			match self.peel() {
				Ok(a) => ArcRun::pure(vec![a]),
				Err(node) => match unwrap_node::<R, CNilBrand, ArcRun<R, CNilBrand, A>>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>
					) as Member<
						ArcCoyoneda<'static, SendChooseBrand<ArcBrand>, ArcRun<R, CNilBrand, A>>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
								.bind(move |true_values| {
									let false_branch = (*k_for_false)(false);
									false_branch
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.map(move |false_values| {
											let mut values = true_values.clone();
											values.extend(false_values);
											values
										})
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								ArcRun<R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRun::pure(Vec::new())
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRun<R, CNilBrand, A>| {
										inner
											.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free()
									},
									rest,
								);
								let node_first = make_node_first::<
									RMinusNonDet,
									CNilBrand,
									ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
								>(mapped_free);
								ArcRun::from_arc_free(wrap_first_arc::<
									RMinusNonDet,
									CNilBrand,
									Vec<A>,
								>(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
		}

		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRun` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRun<Row, CNilBrand, i32> = ArcRun::<Row, CNilBrand, bool>::choose()
		/// 	.bind(|branch| if branch { ArcRun::empty() } else { ArcRun::pure(2) });
		/// let handled: ArcRun<CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRun<RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			R: Kind_cdc7cd43dac7585f + 'static,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: WrapDrop
				+ Kind_cdc7cd43dac7585f<
					Of<'static, ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>>:
						Send + Sync,
				> + SendFunctor,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<R, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, ArcTypeErasedValue>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<
						'static,
						SendChooseBrand<ArcBrand>,
						ArcRun<R, CNilBrand, A>,
					>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'static,
				ArcRun<R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'static,
										ArcRun<R, CNilBrand, A>,
									>
								),
		>,{
			match self.peel() {
				Ok(a) => ArcRun::pure(Some(a)),
				Err(node) => match unwrap_node::<R, CNilBrand, ArcRun<R, CNilBrand, A>>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, CNilBrand, A>>
					) as Member<
						ArcCoyoneda<'static, SendChooseBrand<ArcBrand>, ArcRun<R, CNilBrand, A>>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
								)
								.bind(move |first| match first {
									Some(value) => ArcRun::pure(Some(value)),
									None => (*k_for_false)(false)
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										),
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'static,
								ArcRun<R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'static, EmptyBrand, ArcRun<R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRun::pure(None)
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRun<R, CNilBrand, A>| {
										inner
											.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free()
									},
									rest,
								);
								let node_first = make_node_first::<
									RMinusNonDet,
									CNilBrand,
									ArcFree<NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
								>(mapped_free);
								ArcRun::from_arc_free(wrap_first_arc::<
									RMinusNonDet,
									CNilBrand,
									Option<A>,
								>(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
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

		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning all successful results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { RcRunExplicit::pure(1) } else { RcRunExplicit::empty() });
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(vec![a]),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>
					>
				) as Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |true_values| {
								let false_branch = (*k_for_false)(false);
								false_branch
									.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
									.map(move |false_values| {
										let mut values = true_values.clone();
										values.extend(false_values);
										values
									})
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRunExplicit::pure(Vec::new())
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRunExplicit<'a, R, CNilBrand, A>| {
									inner
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free_explicit()
								},
								rest,
							);
							RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
								'a,
								NodeBrand<RMinusNonDet, CNilBrand>,
								Vec<A>,
							>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
		}

		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `RcRunExplicit` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run_explicit::RcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<RcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<RcCoyonedaBrand<ChooseBrand<RcBrand>>, EmptyRow>;
		///
		/// let program: RcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	RcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { RcRunExplicit::empty() } else { RcRunExplicit::pure(2) });
		/// let handled: RcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> RcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + Functor + 'static,
			Apply!(<NodeBrand<R, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, CNilBrand>, A>,
			>): Clone,
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Clone,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				RcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										RcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => RcRunExplicit::pure(Some(a)),
				Err(Node::First(layer)) => match <Apply!(
					<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
						'a,
						RcRunExplicit<'a, R, CNilBrand, A>
					>
				) as Member<
					RcCoyoneda<'a, ChooseBrand<RcBrand>, RcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
				>>::project(layer)
				{
					Ok(coyo) => {
						let Choose::Alt(k) = coyo.lower_ref();
						let true_branch = (*k)(true);
						let k_for_false = k.clone();
						true_branch
							.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
							.bind(move |first| match first {
								Some(value) => RcRunExplicit::pure(Some(value)),
								None => (*k_for_false)(false)
									.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
									),
							})
					}
					Err(rest) => match <Apply!(
						<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							RcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						RcCoyoneda<'a, EmptyBrand, RcRunExplicit<'a, R, CNilBrand, A>>,
						EmptyIdx,
					>>::project(rest)
					{
						Ok(coyo) => {
							let Empty::Empty(_) = coyo.lower_ref();
							RcRunExplicit::pure(None)
						}
						Err(rest) => {
							let mapped_free = <RMinusNonDet as Functor>::map(
								move |inner: RcRunExplicit<'a, R, CNilBrand, A>| {
									inner
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.into_rc_free_explicit()
								},
								rest,
							);
							RcRunExplicit::from_rc_free_explicit(RcFreeExplicit::<
								'a,
								NodeBrand<RMinusNonDet, CNilBrand>,
								Option<A>,
							>::wrap(Node::First(
								mapped_free,
							)))
						}
					},
				},
				Err(Node::Scoped(layer)) => match layer {},
			}
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

		/// Interprets `Choose` and `Empty` together into a `Vec`.
		///
		/// Pure results become singleton vectors, `Empty` contributes
		/// no results, and `Choose` explores the `true` branch before
		/// the `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning all successful results."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { ArcRunExplicit::pure(1) } else { ArcRunExplicit::empty() });
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Vec<i32>> =
		/// 	program.run_nondet::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), vec![1]);
		/// ```
		#[inline]
		pub fn run_nondet<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>
		where
			A: Clone + Send + Sync,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Vec<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, SendChooseBrand<ArcBrand>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(vec![a]),
				Err(node) => match unwrap_arc_run_explicit_nondet_node::<R, A>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						ArcCoyoneda<
							'a,
							SendChooseBrand<ArcBrand>,
							ArcRunExplicit<'a, R, CNilBrand, A>,
						>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>()
								.bind(move |true_values| {
									let false_branch = (*k_for_false)(false);
									false_branch
										.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										)
										.map(move |false_values| {
											let mut values = true_values.clone();
											values.extend(false_values);
											values
										})
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								ArcRunExplicit<'a, R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRunExplicit::pure(Vec::new())
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
										inner
											.run_nondet::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free_explicit()
									},
									rest,
								);
								let node_first = make_arc_run_explicit_nondet_node_first::<
									RMinusNonDet,
									ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Vec<A>>,
								>(mapped_free);
								ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
									'a,
									NodeBrand<RMinusNonDet, CNilBrand>,
									Vec<A>,
								>::wrap(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
		}

		/// Interprets `Choose` and `Empty` into the first successful result.
		///
		/// Pure results become `Some(value)`, `Empty` becomes `None`,
		/// and `Choose` tries the `true` branch before evaluating the
		/// `false` branch.
		#[document_signature]
		#[document_type_parameters(
			"The type-level Member-position witness for the Choose effect.",
			"The type-level Member-position witness for the Empty effect after Choose is removed.",
			"The row brand with Choose removed.",
			"The first-order row brand with both NonDet effects removed."
		)]
		#[document_returns(
			"A first-order-only `ArcRunExplicit` program returning the first successful result."
		)]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run_explicit::ArcRunExplicit,
		/// };
		///
		/// type EmptyRow = CoproductBrand<ArcCoyonedaBrand<EmptyBrand>, CNilBrand>;
		/// type Row = CoproductBrand<ArcCoyonedaBrand<SendChooseBrand<ArcBrand>>, EmptyRow>;
		///
		/// let program: ArcRunExplicit<'static, Row, CNilBrand, i32> =
		/// 	ArcRunExplicit::<'static, Row, CNilBrand, bool>::choose()
		/// 		.bind(|branch| if branch { ArcRunExplicit::empty() } else { ArcRunExplicit::pure(2) });
		/// let handled: ArcRunExplicit<'static, CNilBrand, CNilBrand, Option<i32>> =
		/// 	program.run_first_success::<_, _, EmptyRow, CNilBrand>();
		/// assert_eq!(handled.extract(), Some(2));
		/// ```
		#[inline]
		pub fn run_first_success<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
			self
		) -> ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>
		where
			A: Clone + Send + Sync,
			RMinusChoose: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			RMinusNonDet: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
			NodeBrand<R, CNilBrand>: SendFunctor,
			NodeBrand<RMinusNonDet, CNilBrand>: SendFunctor,
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
			Apply!(<NodeBrand<RMinusNonDet, CNilBrand> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Clone + Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<RMinusNonDet, CNilBrand>, Option<A>>,
			>): Send + Sync,
			Apply!(<CNilBrand as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, RMinusNonDet, CNilBrand, Option<A>>,
			>): Send + Sync,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, SendChooseBrand<ArcBrand>, ArcRunExplicit<'a, R, CNilBrand, A>>,
					ChooseIdx,
					Remainder = Apply!(
									<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>,
			Apply!(<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
				'a,
				ArcRunExplicit<'a, R, CNilBrand, A>,
			>): Member<
					ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
					EmptyIdx,
					Remainder = Apply!(
									<RMinusNonDet as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
										'a,
										ArcRunExplicit<'a, R, CNilBrand, A>,
									>
								),
				>, {
			match self.peel() {
				Ok(a) => ArcRunExplicit::pure(Some(a)),
				Err(node) => match unwrap_arc_run_explicit_nondet_node::<R, A>(node) {
					Node::First(layer) => match <Apply!(
						<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
							'a,
							ArcRunExplicit<'a, R, CNilBrand, A>
						>
					) as Member<
						ArcCoyoneda<
							'a,
							SendChooseBrand<ArcBrand>,
							ArcRunExplicit<'a, R, CNilBrand, A>,
						>,
						ChooseIdx,
					>>::project(layer)
					{
						Ok(coyo) => {
							let SendChoose::Alt(k) = coyo.lower_ref();
							let true_branch = (*k)(true);
							let k_for_false = k.clone();
							true_branch
								.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
								)
								.bind(move |first| match first {
									Some(value) => ArcRunExplicit::pure(Some(value)),
									None => (*k_for_false)(false)
										.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
										),
								})
						}
						Err(rest) => match <Apply!(
							<RMinusChoose as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
								'a,
								ArcRunExplicit<'a, R, CNilBrand, A>
							>
						) as Member<
							ArcCoyoneda<'a, EmptyBrand, ArcRunExplicit<'a, R, CNilBrand, A>>,
							EmptyIdx,
						>>::project(rest)
						{
							Ok(coyo) => {
								let Empty::Empty(_) = coyo.lower_ref();
								ArcRunExplicit::pure(None)
							}
							Err(rest) => {
								let mapped_free = <RMinusNonDet as SendFunctor>::send_map(
									move |inner: ArcRunExplicit<'a, R, CNilBrand, A>| {
										inner
											.run_first_success::<ChooseIdx, EmptyIdx, RMinusChoose, RMinusNonDet>(
											)
											.into_arc_free_explicit()
									},
									rest,
								);
								let node_first = make_arc_run_explicit_nondet_node_first::<
									RMinusNonDet,
									ArcFreeExplicit<
										'a,
										NodeBrand<RMinusNonDet, CNilBrand>,
										Option<A>,
									>,
								>(mapped_free);
								ArcRunExplicit::from_arc_free_explicit(ArcFreeExplicit::<
									'a,
									NodeBrand<RMinusNonDet, CNilBrand>,
									Option<A>,
								>::wrap(node_first))
							}
						},
					},
					Node::Scoped(layer) => match layer {},
				},
			}
		}
	}
}

#[expect(
	unused_imports,
	reason = "inherent impl modules follow the document_module re-export boundary"
)]
pub use inner::*;
