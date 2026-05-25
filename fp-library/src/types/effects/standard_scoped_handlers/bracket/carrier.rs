#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::{
		super::prelude::*,
		inner::BracketHandler,
	};

	/// Carrier-aware Bracket dispatch for `RunExplicitBoundary`.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The acquired resource type.",
		"The selected Bracket body result type.",
		"The final program result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Bracket handler receiver.")]
	impl<'a, R, S, Resource, BodyResult, Final, K, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BoxBracketExplicit<'a, BoxBrand, NodeBrand<R, S>, Resource, BodyResult>,
			FirstLayer,
			RunExplicit<'a, R, S, Final>,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
		> for BracketHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'a,
		BodyResult: 'a,
		Final: 'a,
		K: Fn(BodyResult) -> RunExplicit<'a, R, S, Final> + 'a,
		FirstLayer: 'a,
		RunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>:
			ScopedResumeTypes<
					'a,
					ActionValue = BodyResult,
					ActionProgram = RunExplicit<'a, R, S, BodyResult>,
					OperationValue = BodyResult,
					OperationProgram = RunExplicit<'a, R, S, BodyResult>,
				>,
	{
		/// Run acquire, body, release, and then the boundary's outer
		/// continuation.
		#[document_signature]
		#[document_parameters(
			"The Bracket scoped layer carrying lifecycle programs.",
			"The wrapper-owned continuation carrier for the body result.",
			"The first-order handler list available while resuming the selected action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the Bracket boundary.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the lifecycle and outer-continuation semantics."
		)]
		///
		/// ```
		/// let resource = 7;
		/// let body_result = resource + 34;
		/// let released = resource == 7;
		/// let outer = |value| value + 1;
		/// assert!(released);
		/// assert_eq!(outer(body_result), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BoxBracketExplicit<'a, BoxBrand, NodeBrand<R, S>, Resource, BodyResult>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				BoxBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => {
					let body = std::cell::RefCell::new(Some(body));
					let release = Rc::new(std::cell::RefCell::new(Some(release)));
					RunExplicit::from_free_explicit(*acquire(())).bind(move |resource| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed Bracket boundary body is single-shot; RunExplicit invokes this continuation once"
						)]
						let body = body
							.borrow_mut()
							.take()
							.expect("RunExplicit Bracket boundary body invoked more than once");
						let release = Rc::clone(&release);
						let outer = outer.clone();
						RunExplicit::from_free_explicit(*body(Box::new(resource))).bind(
							move |(resource, body_result)| {
								#[expect(
									clippy::expect_used,
									reason = "Box-backed Bracket boundary release is single-shot; RunExplicit invokes this continuation once"
								)]
								let release = release.borrow_mut().take().expect(
									"RunExplicit Bracket boundary release invoked more than once",
								);
								let body_result = std::cell::RefCell::new(Some(body_result));
								let outer = outer.clone();
								RunExplicit::from_free_explicit(*release(Box::new(resource))).bind(
									move |()| {
										#[expect(
											clippy::expect_used,
											reason = "Box-backed Bracket boundary result is single-shot; RunExplicit invokes this continuation once"
										)]
										let body_result = body_result.borrow_mut().take().expect(
											"RunExplicit Bracket boundary result returned more than once",
										);
										(*outer)(body_result)
									},
								)
							},
						)
					})
				}
			}
		}
	}

	#[document_parameters("The Bracket handler receiver.")]
	#[allow(
		dead_code,
		reason = "Focused Bracket carrier methods are introduced before the wrapper interpreter route constructs these private layers."
	)]
	impl BracketHandler {
		/// Dispatch an indexed `RunExplicit` Bracket boundary.
		///
		/// The Bracket layer stores the lifecycle cells while the
		/// boundary continuation owns the typed outer resume. The
		/// handler generates the selected body action from acquire,
		/// body, and release, then resumes the outer continuation after
		/// release has completed.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The type-level Member-position witness for the scoped Bracket layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Bracket boundary produced around the lifecycle-generated action.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		Functor,
		/// 		WrapDrop,
		/// 	},
		/// 	handlers,
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::{
		/// 		run_explicit::RunExplicit,
		/// 		standard_scoped_handlers::bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	BoxBracketExplicitBrand<BoxBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let acquire: Prog = RunExplicit::pure(7);
		/// let boundary = Prog::bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: Box<i32>| RunExplicit::pure((*resource, 42)),
		/// 	|_resource: Box<i32>| RunExplicit::pure(()),
		/// );
		/// let program: Prog =
		/// 	bracket_handler().dispatch_run_explicit_bracket_boundary(boundary, &handlers! {});
		///
		/// assert!(matches!(program.peel(), Ok(42)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RunExplicit Bracket boundaries are constructed by injecting a Bracket layer; reaching the non-Bracket projection branch means a crate-private constructor violated the boundary invariant."
		)]
		#[expect(
			clippy::type_complexity,
			reason = "Boundary dispatch methods must name the full boundary value shape, selected row member, and first-order handler row so the carrier-aware scoped dispatch protocol stays private."
		)]
		pub fn dispatch_run_explicit_bracket_boundary<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			ScopedIdx,
			FirstLayer,
		>(
			&self,
			boundary: RunExplicitBoundary<
				'a,
				R,
				S,
				BoxBracketExplicitBrand<BoxBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
				BodyResult,
				Final,
				K,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: Clone + 'a,
			BodyResult: 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RunExplicit<'a, R, S, Final> + 'a,
			FirstLayer: 'a,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RunExplicit<'a, R, S, BodyResult>,
			>): Member<
					BoxBracketExplicit<'a, BoxBrand, NodeBrand<R, S>, Resource, BodyResult>,
					ScopedIdx,
				>, {
			let (layer, continuation) = boundary.into_parts();
			let bracket = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RunExplicit<'a, R, S, BodyResult>,
				>) as Member<
				BoxBracketExplicit<'a, BoxBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(bracket) => bracket,
				Err(_) => unreachable!(
					"RunExplicit Bracket boundary contained a non-Bracket scoped layer"
				),
			};

			match bracket {
				BoxBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => {
					let body = std::cell::RefCell::new(Some(body));
					let release = Rc::new(std::cell::RefCell::new(Some(release)));

					continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
						RunExplicit::from_free_explicit(*acquire(())).bind(move |resource| {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed Bracket boundary body is single-shot; RunExplicit invokes this continuation once"
							)]
							let body = body
								.borrow_mut()
								.take()
								.expect("RunExplicit Bracket boundary body invoked more than once");
							let release = Rc::clone(&release);
							RunExplicit::from_free_explicit(*body(Box::new(resource))).bind(
								move |(resource, body_result)| {
									#[expect(
										clippy::expect_used,
										reason = "Box-backed Bracket boundary release is single-shot; RunExplicit invokes this continuation once"
									)]
									let release = release.borrow_mut().take().expect(
										"RunExplicit Bracket boundary release invoked more than once",
									);
									let body_result = std::cell::RefCell::new(Some(body_result));
									RunExplicit::from_free_explicit(*release(Box::new(resource)))
										.bind(move |()| {
											#[expect(
												clippy::expect_used,
												reason = "Box-backed Bracket boundary result is single-shot; RunExplicit invokes this continuation once"
											)]
											RunExplicit::pure(body_result.borrow_mut().take().expect(
												"RunExplicit Bracket boundary result returned more than once",
											))
										})
								},
							)
						})
					})
				}
			}
		}

		/// Dispatch an indexed `RcRunExplicit` Bracket boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The type-level Member-position witness for the scoped Bracket layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Bracket boundary produced around the lifecycle-generated action.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		Functor,
		/// 		WrapDrop,
		/// 	},
		/// 	handlers,
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::{
		/// 		rc_run_explicit::RcRunExplicit,
		/// 		standard_scoped_handlers::bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	BracketExplicitBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl Functor for ScopedRow {
		/// 	fn map<'a, A: 'a, B: 'a>(
		/// 		f: impl Fn(A) -> B + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as Functor>::map(f, fa)
		/// 	}
		/// }
		///
		/// let acquire: Prog = RcRunExplicit::pure(7);
		/// let boundary = Prog::bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::rc::Rc<i32>| RcRunExplicit::pure((*resource, 42)),
		/// 	|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let program: Prog =
		/// 	bracket_handler().dispatch_rc_run_explicit_bracket_boundary(boundary, &handlers! {});
		///
		/// assert!(matches!(program.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RcRunExplicit Bracket boundaries are constructed by injecting a Bracket layer; reaching the non-Bracket projection branch means a crate-private constructor violated the boundary invariant."
		)]
		#[expect(
			clippy::type_complexity,
			reason = "Boundary dispatch methods must name the full boundary value shape, selected row member, and first-order handler row so the carrier-aware scoped dispatch protocol stays private."
		)]
		pub fn dispatch_rc_run_explicit_bracket_boundary<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			ScopedIdx,
			FirstLayer,
		>(
			&self,
			boundary: RcRunExplicitBoundary<
				'a,
				R,
				S,
				BracketExplicitBrand<RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
				BodyResult,
				Final,
				K,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>> + 'a
			    ),
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: Clone + 'a,
			BodyResult: Clone + 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RcRunExplicit<'a, R, S, Final> + 'a,
			FirstLayer: 'a,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, BodyResult>,
			>): Member<BracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>, ScopedIdx>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone, {
			let (layer, continuation) = boundary.into_parts();
			let bracket = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					RcRunExplicit<'a, R, S, BodyResult>,
				>) as Member<
				BracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(bracket) => bracket,
				Err(_) => unreachable!(
					"RcRunExplicit Bracket boundary contained a non-Bracket scoped layer"
				),
			};

			match bracket {
				BracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					RcRunExplicit::from_rc_free_explicit(acquire(())).bind(move |resource| {
						let body = body.clone();
						let release = release.clone();
						RcRunExplicit::from_rc_free_explicit(body(Rc::new(resource))).bind(
							move |(resource, body_result)| {
								RcRunExplicit::from_rc_free_explicit(release(Rc::new(resource)))
									.map(move |()| body_result.clone())
							},
						)
					})
				}),
			}
		}

		/// Dispatch an indexed `ArcRunExplicit` Bracket boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The type-level Member-position witness for the scoped Bracket layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed Bracket boundary produced around the lifecycle-generated action.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the boundary.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	Apply,
		/// 	brands::*,
		/// 	classes::{
		/// 		SendFunctor,
		/// 		WrapDrop,
		/// 	},
		/// 	handlers,
		/// 	impl_kind,
		/// 	kinds::*,
		/// 	types::effects::{
		/// 		arc_run_explicit::ArcRunExplicit,
		/// 		standard_scoped_handlers::bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	SendBracketExplicitBrand<ArcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = ArcRunExplicit<'static, FirstRow, ScopedRow, i32>;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Apply!(<UnderlyingRow as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>);
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		<UnderlyingRow as WrapDrop>::drop(fa)
		/// 	}
		/// }
		///
		/// impl SendFunctor for ScopedRow {
		/// 	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		/// 		f: impl Fn(A) -> B + Send + Sync + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		<UnderlyingRow as SendFunctor>::send_map(f, fa)
		/// 	}
		/// }
		///
		/// let acquire: Prog = ArcRunExplicit::pure(7);
		/// let boundary = Prog::bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::sync::Arc<i32>| ArcRunExplicit::pure((*resource, 42)),
		/// 	|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let program: Prog =
		/// 	bracket_handler().dispatch_arc_run_explicit_bracket_boundary(boundary, &handlers! {});
		///
		/// assert!(matches!(program.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "ArcRunExplicit Bracket boundaries are constructed by injecting a Bracket layer; reaching the non-Bracket projection branch means a crate-private constructor violated the boundary invariant."
		)]
		#[expect(
			clippy::type_complexity,
			reason = "Boundary dispatch methods must name the full boundary value shape, selected row member, and first-order handler row so the carrier-aware scoped dispatch protocol stays private."
		)]
		pub fn dispatch_arc_run_explicit_bracket_boundary<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			ScopedIdx,
			FirstLayer,
		>(
			&self,
			boundary: ArcRunExplicitBoundary<
				'a,
				R,
				S,
				SendBracketExplicitBrand<ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
				BodyResult,
				Final,
				K,
			>,
			fo_handlers: &'a (
			        impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>
			        + Send
			        + Sync
			        + 'a
			    ),
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + SendFunctor + 'static,
			S: WrapDrop + SendFunctor + 'static,
			Resource: Clone + Send + Sync + 'a,
			BodyResult: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(BodyResult) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			FirstLayer: 'a,
			Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, BodyResult>,
			>): Member<
					SendBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
					ScopedIdx,
				> + Send
				+ Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync, {
			let (layer, continuation) = boundary.into_parts();
			let bracket = match <Apply!(<S as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
					'a,
					ArcRunExplicit<'a, R, S, BodyResult>,
				>) as Member<
				SendBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(bracket) => bracket,
				Err(_) => unreachable!(
					"ArcRunExplicit Bracket boundary contained a non-Bracket scoped layer"
				),
			};

			match bracket {
				SendBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					ArcRunExplicit::from_arc_free_explicit(acquire(())).bind(move |resource| {
						let body = body.clone();
						let release = release.clone();
						ArcRunExplicit::from_arc_free_explicit(body(Arc::new(resource))).bind(
							move |(resource, body_result)| {
								ArcRunExplicit::from_arc_free_explicit(release(Arc::new(resource)))
									.map(move |()| body_result.clone())
							},
						)
					})
				}),
			}
		}

		/// Dispatch a private `RunExplicit` Bracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		///
		#[document_parameters(
			"The private Bracket layer carrying acquire, body, release, and the `RunExplicit` action-supplied carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		#[document_examples(
			skip_call_check,
			reason = "This private RunExplicit carrier helper consumes crate-private action-supplied continuation cells and single-shot lifecycle closures constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the lifecycle behaviour."
		)]
		///
		/// ```
		/// let acquire = || 7;
		/// let body = |resource: Box<i32>| (*resource, *resource + 35);
		/// let release = |resource: Box<i32>| *resource == 7;
		/// let resource = acquire();
		/// let (resource, body_result) = body(Box::new(resource));
		/// assert!(release(Box::new(resource)));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private Bracket carrier signature must keep the wrapper, lifecycle, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_run_explicit_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitBracketCarrierLayer<
				'a,
				BoxBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				RunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RunExplicit<'a, R, S, Final>>,
		) -> RunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: Clone + 'a,
			BodyResult: 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RunExplicit<'a, R, S, Final> + 'a,
			Acquire: FnOnce() -> RunExplicit<'a, R, S, Resource> + 'a,
			BodyFn: FnOnce(Box<Resource>) -> RunExplicit<'a, R, S, (Resource, BodyResult)> + 'a,
			Release: FnOnce(Box<Resource>) -> RunExplicit<'a, R, S, ()> + 'a,
			FirstLayer: 'a,
			RunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = RunExplicit<'a, R, S, BodyResult>,
						OperationValue = BodyResult,
						OperationProgram = RunExplicit<'a, R, S, BodyResult>,
					> + ExplicitActionSuppliedScopedResume<'a, FirstLayer, RunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();
			let body = std::cell::RefCell::new(Some(body));
			let release = Rc::new(std::cell::RefCell::new(Some(release)));

			continuation.resume_explicit_with_supplied_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					#[expect(
						clippy::expect_used,
						reason = "Box-backed Bracket carrier body is single-shot; RunExplicit invokes this continuation once"
					)]
					let body = body
						.borrow_mut()
						.take()
						.expect("RunExplicit Bracket carrier body invoked more than once");
					let release = Rc::clone(&release);
					body(Box::new(resource)).bind(move |(resource, body_result)| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed Bracket carrier release is single-shot; RunExplicit invokes this continuation once"
						)]
						let release = release
							.borrow_mut()
							.take()
							.expect("RunExplicit Bracket carrier release invoked more than once");
						let body_result = std::cell::RefCell::new(Some(body_result));
						release(Box::new(resource)).bind(move |()| {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed Bracket carrier result is single-shot; RunExplicit invokes this continuation once"
							)]
							RunExplicit::pure(body_result.borrow_mut().take().expect(
								"RunExplicit Bracket carrier result returned more than once",
							))
						})
					})
				})
			})
		}

		/// Dispatch a private `RcRunExplicit` Bracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private Bracket layer carrying acquire, body, release, and the `RcRunExplicit` action-supplied carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		#[document_examples(
			skip_call_check,
			reason = "This private RcRunExplicit carrier helper consumes crate-private action-supplied continuation cells constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the Rc-backed lifecycle behaviour."
		)]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let resource = 7;
		/// let (resource, body_result) =
		/// 	(|resource: Rc<i32>| (*resource, *resource + 35))(Rc::new(resource));
		/// assert!((|resource: Rc<i32>| *resource == 7)(Rc::new(resource)));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private Bracket carrier signature must keep the wrapper, lifecycle, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_rc_run_explicit_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitBracketCarrierLayer<
				'a,
				RcBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + Functor + 'static,
			S: WrapDrop + Functor + 'static,
			Resource: Clone + 'a,
			BodyResult: Clone + 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RcRunExplicit<'a, R, S, Final> + 'a,
			Acquire: Fn() -> RcRunExplicit<'a, R, S, Resource> + 'a,
			BodyFn:
				Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, (Resource, BodyResult)> + Clone + 'a,
			Release: Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, ()> + Clone + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = RcRunExplicit<'a, R, S, BodyResult>,
						OperationValue = BodyResult,
						OperationProgram = RcRunExplicit<'a, R, S, BodyResult>,
					> + RcActionSuppliedScopedResume<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();

			continuation.resume_rc_with_supplied_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					let body = body.clone();
					let release = release.clone();
					body(Rc::new(resource)).bind(move |(resource, body_result)| {
						release(Rc::new(resource)).map(move |()| body_result.clone())
					})
				})
			})
		}

		/// Dispatch a private `ArcRunExplicit` Bracket carrier-cell layer.
		#[document_signature]
		///
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final program result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The acquire lifecycle cell type.",
			"The body lifecycle cell type.",
			"The release lifecycle cell type.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The private Bracket layer carrying acquire, body, release, and the `ArcRunExplicit` action-supplied carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		#[document_examples(
			skip_call_check,
			reason = "This private ArcRunExplicit carrier helper consumes crate-private action-supplied continuation cells constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the thread-safe lifecycle behaviour."
		)]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let resource = 7;
		/// let (resource, body_result) =
		/// 	(|resource: Arc<i32>| (*resource, *resource + 35))(Arc::new(resource));
		/// assert!((|resource: Arc<i32>| *resource == 7)(Arc::new(resource)));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private Bracket carrier signature must keep the wrapper, lifecycle, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_arc_run_explicit_bracket_carrier<
			'a,
			R,
			S,
			Resource,
			BodyResult,
			Final,
			K,
			Acquire,
			BodyFn,
			Release,
			FirstLayer,
		>(
			&self,
			layer: RunExplicitBracketCarrierLayer<
				'a,
				ArcBrand,
				Resource,
				BodyResult,
				Acquire,
				BodyFn,
				Release,
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final>
		where
			R: WrapDrop + SendFunctor + 'static,
			S: WrapDrop + SendFunctor + 'static,
			Resource: Clone + Send + Sync + 'a,
			BodyResult: Clone + Send + Sync + 'a,
			Final: Send + Sync + 'a,
			K: Fn(BodyResult) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
			Acquire: Fn() -> ArcRunExplicit<'a, R, S, Resource> + Send + Sync + 'a,
			BodyFn: Fn(Arc<Resource>) -> ArcRunExplicit<'a, R, S, (Resource, BodyResult)>
				+ Clone
				+ Send
				+ Sync
				+ 'a,
			Release: Fn(Arc<Resource>) -> ArcRunExplicit<'a, R, S, ()> + Clone + Send + Sync + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
			>): Clone + Send + Sync,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>:
				ScopedResumeTypes<
						'a,
						ActionValue = BodyResult,
						ActionProgram = ArcRunExplicit<'a, R, S, BodyResult>,
						OperationValue = BodyResult,
						OperationProgram = ArcRunExplicit<'a, R, S, BodyResult>,
					> + ArcActionSuppliedScopedResume<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>, {
			let (acquire, body, release, continuation) = layer.into_parts();

			continuation.resume_arc_with_supplied_action(fo_handlers, move || {
				acquire().bind(move |resource| {
					let body = body.clone();
					let release = release.clone();
					body(Arc::new(resource)).bind(move |(resource, body_result)| {
						release(Arc::new(resource)).map(move |()| body_result.clone())
					})
				})
			})
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the shared Explicit boundary.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The acquired resource type.",
		"The selected body result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Bracket handler receiver.")]
	impl<'a, R, S, Resource, BodyResult, Final, K, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			BracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
		> for BracketHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'a,
		BodyResult: Clone + 'a,
		Final: 'a,
		K: Fn(BodyResult) -> RcRunExplicit<'a, R, S, Final> + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
	{
		/// Run the Rc Bracket lifecycle and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Bracket layer carrying acquire, body, and release programs.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the boundary handler.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private RcRunExplicit ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the lifecycle and outer-continuation semantics."
		)]
		///
		/// ```
		/// let resource = 7;
		/// let body_result = resource + 34;
		/// let released = resource == 7;
		/// let outer = |value| value + 1;
		/// assert!(released);
		/// assert_eq!(outer(body_result), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: BracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				BracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => RcRunExplicit::from_rc_free_explicit(acquire(())).bind(move |resource| {
					let body = body.clone();
					let release = release.clone();
					let outer = outer.clone();
					RcRunExplicit::from_rc_free_explicit(body(Rc::new(resource))).bind(
						move |(resource, body_result)| {
							let outer = outer.clone();
							RcRunExplicit::from_rc_free_explicit(release(Rc::new(resource)))
								.bind(move |()| outer(body_result.clone()))
						},
					)
				}),
			}
		}
	}

	#[document_type_parameters(
		"The lifetime of values carried by the shared Explicit boundary.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The acquired resource type.",
		"The selected body result type.",
		"The final result type after the outer continuation resumes.",
		"The concrete outer-continuation closure type.",
		"The first-order handler layer type."
	)]
	#[document_parameters("The Bracket handler receiver.")]
	impl<'a, R, S, Resource, BodyResult, Final, K, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
		> for BracketHandler
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Resource: Clone + Send + Sync + 'a,
		BodyResult: Clone + Send + Sync + 'a,
		Final: Send + Sync + 'a,
		K: Fn(BodyResult) -> ArcRunExplicit<'a, R, S, Final> + Send + Sync + 'a,
		FirstLayer: 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, (Resource, BodyResult)>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
	{
		/// Run the Arc Bracket lifecycle and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The Bracket layer carrying acquire, body, and release programs.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the boundary handler.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private ArcRunExplicit ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the thread-safe lifecycle and outer-continuation semantics."
		)]
		///
		/// ```
		/// let resource = 7;
		/// let body_result = resource + 34;
		/// let released = resource == 7;
		/// let outer = |value| value + 1;
		/// assert!(released);
		/// assert_eq!(outer(body_result), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				SendBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => ArcRunExplicit::from_arc_free_explicit(acquire(())).bind(move |resource| {
					let body = body.clone();
					let release = release.clone();
					let outer = outer.clone();
					ArcRunExplicit::from_arc_free_explicit(body(Arc::new(resource))).bind(
						move |(resource, body_result)| {
							let outer = outer.clone();
							ArcRunExplicit::from_arc_free_explicit(release(Arc::new(resource)))
								.bind(move |()| outer(body_result.clone()))
						},
					)
				}),
			}
		}
	}
}
