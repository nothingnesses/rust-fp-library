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
		define_run_wrapper! {
			wrapper Run;
			effect Empty;
			method run_empty;
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
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper RcRun;
			effect Choose;
			method run_choose;
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
		define_run_wrapper! {
			wrapper ArcRun;
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper ArcRun;
			effect Choose;
			method run_choose;
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
		define_run_wrapper! {
			wrapper RunExplicit;
			effect Empty;
			method run_empty;
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
		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper RcRunExplicit;
			effect Choose;
			method run_choose;
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
		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Empty;
			method run_empty;
		}

		define_run_wrapper! {
			wrapper ArcRunExplicit;
			effect Choose;
			method run_choose;
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
