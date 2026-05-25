#[fp_macros::document_module]
pub(crate) mod inner {
	use super::super::{
		super::prelude::*,
		inner::RefBracketHandler,
	};

	#[document_parameters("The RefBracket handler receiver.")]
	#[allow(
		dead_code,
		reason = "Focused RefBracket carrier methods are introduced before the wrapper interpreter route constructs these private layers."
	)]
	impl RefBracketHandler {
		/// Dispatch an indexed `RcRunExplicit` RefBracket boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Rc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The type-level Member-position witness for the scoped RefBracket layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed RefBracket boundary produced around the lifecycle-generated action.",
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
		/// 		standard_scoped_handlers::ref_bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketExplicitBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
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
		/// let boundary = Prog::ref_bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::rc::Rc<i32>| RcRunExplicit::pure(*resource + 35),
		/// 	|_resource: std::rc::Rc<i32>| RcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let program: Prog = ref_bracket_handler()
		/// 	.dispatch_rc_run_explicit_ref_bracket_boundary(boundary, &handlers! {});
		///
		/// assert!(matches!(program.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RcRunExplicit RefBracket boundaries are constructed by injecting a RefBracket layer; reaching the non-RefBracket projection branch means a crate-private constructor violated the boundary invariant."
		)]
		#[expect(
			clippy::type_complexity,
			reason = "Boundary dispatch methods must name the full boundary value shape, selected row member, and first-order handler row so the carrier-aware scoped dispatch protocol stays private."
		)]
		pub fn dispatch_rc_run_explicit_ref_bracket_boundary<
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
				RefBracketExplicitBrand<RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
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
			>): Member<
					RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
					ScopedIdx,
				>,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
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
				RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(bracket) => bracket,
				Err(_) => unreachable!(
					"RcRunExplicit RefBracket boundary contained a non-RefBracket scoped layer"
				),
			};

			match bracket {
				RefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => continuation.resume_rc_with_supplied_action(fo_handlers, move || {
					RcRunExplicit::from_rc_free_explicit(acquire(())).bind(move |resource| {
						let resource = Rc::new(resource);
						let release_resource = Rc::clone(&resource);
						let body = body.clone();
						let release = release.clone();
						RcRunExplicit::from_rc_free_explicit(body(resource)).bind(
							move |body_result| {
								RcRunExplicit::from_rc_free_explicit(release(Rc::clone(
									&release_resource,
								)))
								.map(move |()| body_result.clone())
							},
						)
					})
				}),
			}
		}

		/// Dispatch an indexed `ArcRunExplicit` RefBracket boundary.
		#[document_signature]
		#[document_type_parameters(
			"The lifetime of values carried by the Arc-backed explicit wrapper.",
			"The first-order row brand.",
			"The scoped row brand.",
			"The acquired resource type.",
			"The body result type returned after release.",
			"The final result type after the outer continuation resumes.",
			"The concrete outer-continuation closure type.",
			"The type-level Member-position witness for the scoped RefBracket layer.",
			"The first-order handler layer type."
		)]
		#[document_parameters(
			"The indexed RefBracket boundary produced around the lifecycle-generated action.",
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
		/// 		standard_scoped_handlers::ref_bracket_handler,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	SendRefBracketExplicitBrand<ArcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
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
		/// let boundary = Prog::ref_bracket::<i32, _>(
		/// 	acquire,
		/// 	|resource: std::sync::Arc<i32>| ArcRunExplicit::pure(*resource + 35),
		/// 	|_resource: std::sync::Arc<i32>| ArcRunExplicit::pure(()),
		/// )
		/// .map(|value| value + 1);
		/// let program: Prog = ref_bracket_handler()
		/// 	.dispatch_arc_run_explicit_ref_bracket_boundary(boundary, &handlers! {});
		///
		/// assert!(matches!(program.peel(), Ok(43)));
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "ArcRunExplicit RefBracket boundaries are constructed by injecting a RefBracket layer; reaching the non-RefBracket projection branch means a crate-private constructor violated the boundary invariant."
		)]
		#[expect(
			clippy::type_complexity,
			reason = "Boundary dispatch methods must name the full boundary value shape, selected row member, and first-order handler row so the carrier-aware scoped dispatch protocol stays private."
		)]
		pub fn dispatch_arc_run_explicit_ref_bracket_boundary<
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
				SendRefBracketExplicitBrand<ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
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
					SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
					ScopedIdx,
				> + Send
				+ Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
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
				SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
				ScopedIdx,
			>>::project(layer)
			{
				Ok(bracket) => bracket,
				Err(_) => unreachable!(
					"ArcRunExplicit RefBracket boundary contained a non-RefBracket scoped layer"
				),
			};

			match bracket {
				SendRefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => continuation.resume_arc_with_supplied_action(fo_handlers, move || {
					ArcRunExplicit::from_arc_free_explicit(acquire(())).bind(move |resource| {
						let resource = Arc::new(resource);
						let release_resource = Arc::clone(&resource);
						let body = body.clone();
						let release = release.clone();
						ArcRunExplicit::from_arc_free_explicit(body(resource)).bind(
							move |body_result| {
								ArcRunExplicit::from_arc_free_explicit(release(Arc::clone(
									&release_resource,
								)))
								.map(move |()| body_result.clone())
							},
						)
					})
				}),
			}
		}

		/// Dispatch a private `RunExplicit` RefBracket carrier-cell layer.
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
		#[document_parameters(
			"The private RefBracket layer carrying acquire, body, release, and the `RunExplicit` action-supplied carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RunExplicit` program produced by the carrier.")]
		#[document_examples(
			skip_call_check,
			reason = "This private RunExplicit carrier helper consumes crate-private action-supplied continuation cells and single-shot lifecycle closures constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the lifecycle behaviour."
		)]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let acquire = || 7;
		/// let body = |resource: Rc<i32>| {
		/// 	assert_eq!(Rc::strong_count(&resource), 2);
		/// 	*resource + 35
		/// };
		/// let release = |resource: Rc<i32>| {
		/// 	assert_eq!(Rc::strong_count(&resource), 1);
		/// 	*resource == 7
		/// };
		/// let resource = Rc::new(acquire());
		/// let release_resource = Rc::clone(&resource);
		/// let body_result = body(resource);
		/// assert!(release(release_resource));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private RefBracket carrier signature must keep the wrapper, lifecycle, pointer, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_run_explicit_ref_bracket_carrier<
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
			layer: RunExplicitRefBracketCarrierLayer<
				'a,
				RcBrand,
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
			Resource: 'a,
			BodyResult: 'a,
			Final: 'a,
			K: Fn(BodyResult) -> RunExplicit<'a, R, S, Final> + 'a,
			Acquire: FnOnce() -> RunExplicit<'a, R, S, Resource> + 'a,
			BodyFn: FnOnce(Rc<Resource>) -> RunExplicit<'a, R, S, BodyResult> + 'a,
			Release: FnOnce(Rc<Resource>) -> RunExplicit<'a, R, S, ()> + 'a,
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
					let resource = Rc::new(resource);
					let release_resource = Rc::clone(&resource);
					let release_resource = std::cell::RefCell::new(Some(release_resource));
					#[expect(
						clippy::expect_used,
						reason = "Box-backed RefBracket carrier body is single-shot; RunExplicit invokes this continuation once"
					)]
					let body = body
						.borrow_mut()
						.take()
						.expect("RunExplicit RefBracket carrier body invoked more than once");
					let release = Rc::clone(&release);
					body(resource).bind(move |body_result| {
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefBracket carrier release is single-shot; RunExplicit invokes this continuation once"
						)]
						let release = release
							.borrow_mut()
							.take()
							.expect("RunExplicit RefBracket carrier release invoked more than once");
						#[expect(
							clippy::expect_used,
							reason = "Box-backed RefBracket carrier release pointer is single-shot; RunExplicit invokes this continuation once"
						)]
						let release_resource = release_resource
							.borrow_mut()
							.take()
							.expect("RunExplicit RefBracket carrier release pointer used more than once");
						let body_result = std::cell::RefCell::new(Some(body_result));
						release(release_resource).bind(move |()| {
							#[expect(
								clippy::expect_used,
								reason = "Box-backed RefBracket carrier result is single-shot; RunExplicit invokes this continuation once"
							)]
							RunExplicit::pure(body_result.borrow_mut().take().expect(
								"RunExplicit RefBracket carrier result returned more than once",
							))
						})
					})
				})
			})
		}

		/// Dispatch a private `RcRunExplicit` RefBracket carrier-cell layer.
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
			"The private RefBracket layer carrying acquire, body, release, and the `RcRunExplicit` action-supplied carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the carrier.")]
		#[document_examples(
			skip_call_check,
			reason = "This private RcRunExplicit carrier helper consumes crate-private action-supplied continuation cells constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the shared-resource lifecycle behaviour."
		)]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let resource = Rc::new(7);
		/// let release_resource = Rc::clone(&resource);
		/// assert_eq!(Rc::strong_count(&resource), 2);
		/// let body_result = (|resource: Rc<i32>| *resource + 35)(resource);
		/// assert!((|resource: Rc<i32>| *resource == 7)(release_resource));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private RefBracket carrier signature must keep the wrapper, lifecycle, pointer, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_rc_run_explicit_ref_bracket_carrier<
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
			layer: RunExplicitRefBracketCarrierLayer<
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
			BodyFn: Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, BodyResult> + Clone + 'a,
			Release: Fn(Rc<Resource>) -> RcRunExplicit<'a, R, S, ()> + Clone + 'a,
			FirstLayer: 'a,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
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
					let resource = Rc::new(resource);
					let release_resource = Rc::clone(&resource);
					let body = body.clone();
					let release = release.clone();
					body(resource).bind(move |body_result| {
						release(Rc::clone(&release_resource)).map(move |()| body_result.clone())
					})
				})
			})
		}

		/// Dispatch a private `ArcRunExplicit` RefBracket carrier-cell layer.
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
			"The private RefBracket layer carrying acquire, body, release, and the `ArcRunExplicit` action-supplied carrier cell.",
			"The first-order handler list available while resuming the generated action."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the carrier.")]
		#[document_examples(
			skip_call_check,
			reason = "This private ArcRunExplicit carrier helper consumes crate-private action-supplied continuation cells constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the thread-safe shared-resource lifecycle behaviour."
		)]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let resource = Arc::new(7);
		/// let release_resource = Arc::clone(&resource);
		/// assert_eq!(Arc::strong_count(&resource), 2);
		/// let body_result = (|resource: Arc<i32>| *resource + 35)(resource);
		/// assert!((|resource: Arc<i32>| *resource == 7)(release_resource));
		/// assert_eq!(body_result, 42);
		/// ```
		#[inline]
		#[expect(
			clippy::type_complexity,
			reason = "The private RefBracket carrier signature must keep the wrapper, lifecycle, pointer, resource, and continuation types explicit."
		)]
		pub(crate) fn dispatch_arc_run_explicit_ref_bracket_carrier<
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
			layer: RunExplicitRefBracketCarrierLayer<
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
			BodyFn: Fn(Arc<Resource>) -> ArcRunExplicit<'a, R, S, BodyResult>
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
				ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
			>): Clone + Send + Sync,
			Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
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
					let resource = Arc::new(resource);
					let release_resource = Arc::clone(&resource);
					let body = body.clone();
					let release = release.clone();
					body(resource).bind(move |body_result| {
						release(Arc::clone(&release_resource)).map(move |()| body_result.clone())
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
	#[document_parameters("The RefBracket handler receiver.")]
	impl<'a, R, S, Resource, BodyResult, Final, K, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			FirstLayer,
			RcRunExplicit<'a, R, S, Final>,
			RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
		> for RefBracketHandler
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
			RcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone,
	{
		/// Run the Rc RefBracket lifecycle and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The RefBracket layer carrying acquire, body, and release programs.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The final `RcRunExplicit` program produced by the boundary handler.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the lifecycle and outer-continuation semantics."
		)]
		///
		/// ```
		/// use std::rc::Rc;
		///
		/// let resource = Rc::new(7);
		/// let release_resource = Rc::clone(&resource);
		/// let body_result = *resource + 34;
		/// let released = *release_resource == 7;
		/// let outer = |value| value + 1;
		/// assert!(released);
		/// assert_eq!(outer(body_result), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				RcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, RcRunExplicit<'a, R, S, Final>>,
		) -> RcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				RefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => RcRunExplicit::from_rc_free_explicit(acquire(())).bind(move |resource| {
					let resource = Rc::new(resource);
					let release_resource = Rc::clone(&resource);
					let body = body.clone();
					let release = release.clone();
					let outer = outer.clone();
					RcRunExplicit::from_rc_free_explicit(body(resource)).bind(move |body_result| {
						let outer = outer.clone();
						let release_resource = Rc::clone(&release_resource);
						RcRunExplicit::from_rc_free_explicit(release(release_resource))
							.bind(move |()| outer(body_result.clone()))
					})
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
	#[document_parameters("The RefBracket handler receiver.")]
	impl<'a, R, S, Resource, BodyResult, Final, K, FirstLayer>
		DispatchScopedCarrierHandler<
			'a,
			SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			FirstLayer,
			ArcRunExplicit<'a, R, S, Final>,
			ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
		> for RefBracketHandler
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
			ArcFreeExplicit<'a, NodeBrand<R, S>, BodyResult>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Final>,
		>): Clone + Send + Sync,
	{
		/// Run the Arc RefBracket lifecycle and then apply the stored outer continuation.
		#[document_signature]
		#[document_parameters(
			"The RefBracket layer carrying acquire, body, and release programs.",
			"The wrapper-owned continuation carrier.",
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The final `ArcRunExplicit` program produced by the boundary handler.")]
		#[document_examples(
			skip_call_check,
			reason = "This scoped-carrier protocol hook receives a crate-private thread-safe ScopedContinuation produced by the interpreter; external examples cannot construct that continuation directly, so the example documents the lifecycle and outer-continuation semantics."
		)]
		///
		/// ```
		/// use std::sync::Arc;
		///
		/// let resource = Arc::new(7);
		/// let release_resource = Arc::clone(&resource);
		/// let body_result = *resource + 34;
		/// let released = *release_resource == 7;
		/// let outer = |value| value + 1;
		/// assert!(released);
		/// assert_eq!(outer(body_result), 42);
		/// ```
		#[inline]
		fn dispatch_scoped_carrier_head(
			&self,
			layer: SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, BodyResult>,
			continuation: crate::types::effects::interpreter::ScopedContinuation<
				ArcRunExplicitActionSuppliedScopedContinuation<'a, R, S, BodyResult, Final, K>,
			>,
			_fo_handlers: &impl DispatchHandlers<'a, FirstLayer, ArcRunExplicit<'a, R, S, Final>>,
		) -> ArcRunExplicit<'a, R, S, Final> {
			let outer = continuation.into_inner().outer.clone();
			match layer {
				SendRefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => ArcRunExplicit::from_arc_free_explicit(acquire(())).bind(move |resource| {
					let resource = Arc::new(resource);
					let release_resource = Arc::clone(&resource);
					let body = body.clone();
					let release = release.clone();
					let outer = outer.clone();
					ArcRunExplicit::from_arc_free_explicit(body(resource)).bind(
						move |body_result| {
							let outer = outer.clone();
							let release_resource = Arc::clone(&release_resource);
							ArcRunExplicit::from_arc_free_explicit(release(release_resource))
								.bind(move |()| outer(body_result.clone()))
						},
					)
				}),
			}
		}
	}
}
