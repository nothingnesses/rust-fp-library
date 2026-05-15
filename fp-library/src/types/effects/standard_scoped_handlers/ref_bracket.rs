#[allow(
	unused_imports,
	reason = "Each scoped-dispatcher child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

#[fp_macros::document_module]
mod inner {
	use super::*;

	/// Dispatcher for the standard `RefBracket` scoped effect.
	///
	/// The dispatcher runs acquire, stores the resource in a refcounted
	/// pointer, passes pointer clones to body and release, runs the
	/// effectful release program on the normal path, and returns the body
	/// result after release completes. During unwinding it relies only on
	/// ordinary Rust `Drop` for the refcounted resource.
	#[derive(Clone, Copy, Debug, Default)]
	pub struct RefBracketHandler;

	/// Constructs a [`RefBracketHandler`].
	#[document_examples]
	///
	/// ```
	/// use {
	/// 	fp_library::{
	/// 		Apply,
	/// 		brands::{
	/// 			CNilBrand,
	/// 			CoproductBrand,
	/// 			NodeBrand,
	/// 			RcBrand,
	/// 			RefBracketBrand,
	/// 		},
	/// 		classes::{
	/// 			Functor,
	/// 			WrapDrop,
	/// 		},
	/// 		handlers,
	/// 		impl_kind,
	/// 		kinds::*,
	/// 		scoped_handlers,
	/// 		types::effects::{
	/// 			rc_run::RcRun,
	/// 			standard_scoped_handlers::ref_bracket_handler,
	/// 		},
	/// 	},
	/// 	std::{
	/// 		cell::Cell,
	/// 		rc::Rc,
	/// 	},
	/// };
	///
	/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
	/// struct ScopedRow;
	///
	/// type FirstRow = CNilBrand;
	/// type UnderlyingRow = CoproductBrand<
	/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
	/// 	CNilBrand,
	/// >;
	/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
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
	/// let observed = Rc::new(Cell::new(0));
	/// let released = Rc::new(Cell::new(false));
	/// let observed_in_body = Rc::clone(&observed);
	/// let released_in_cleanup = Rc::clone(&released);
	/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
	/// 	RcRun::pure(7),
	/// 	move |resource: Rc<i32>| {
	/// 		observed_in_body.set(*resource);
	/// 		RcRun::pure(*resource + 35)
	/// 	},
	/// 	move |resource: Rc<i32>| {
	/// 		released_in_cleanup.set(*resource == 7);
	/// 		RcRun::pure(())
	/// 	},
	/// );
	///
	/// let result = program.interpret(
	/// 	handlers! {},
	/// 	scoped_handlers! {
	/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_handler(),
	/// 	},
	/// );
	///
	/// assert_eq!(result, 42);
	/// assert_eq!(observed.get(), 7);
	/// assert!(released.get());
	/// ```
	pub const fn ref_bracket_handler() -> RefBracketHandler {
		RefBracketHandler
	}

	#[document_parameters("The RefBracket dispatcher receiver.")]
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
		/// use std::rc::Rc;
		///
		/// let resource = Rc::new(7);
		/// let release_resource = Rc::clone(&resource);
		/// assert_eq!(Rc::strong_count(&resource), 2);
		/// let body_result = (|resource: Rc<i32>| *resource + 35)(resource);
		/// assert!((|resource: Rc<i32>| *resource == 7)(release_resource));
		/// assert_eq!(body_result + 1, 43);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "RcRunExplicit RefBracket boundaries are constructed by injecting a RefBracket layer; reaching the non-RefBracket projection branch means a crate-private constructor violated the boundary invariant."
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
			boundary: RcRunExplicitBoundary<'a, R, S, BodyResult, Final, K>,
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
		/// use std::sync::Arc;
		///
		/// let resource = Arc::new(7);
		/// let release_resource = Arc::clone(&resource);
		/// assert_eq!(Arc::strong_count(&resource), 2);
		/// let body_result = (|resource: Arc<i32>| *resource + 35)(resource);
		/// assert!((|resource: Arc<i32>| *resource == 7)(release_resource));
		/// assert_eq!(body_result + 1, 43);
		/// ```
		#[inline]
		#[expect(
			clippy::unreachable,
			reason = "ArcRunExplicit RefBracket boundaries are constructed by injecting a RefBracket layer; reaching the non-RefBracket projection branch means a crate-private constructor violated the boundary invariant."
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
			boundary: ArcRunExplicitBoundary<'a, R, S, BodyResult, Final, K>,
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
		#[document_examples]
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
		#[document_examples]
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
		#[document_examples]
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

	/// Raw scoped dispatch implementation for the Rc-backed RefBracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type after the suspended continuation queue resumes.",
		"The resource type produced by acquire.",
		"The body result type produced before the suspended continuation queue resumes.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Resource, Body, FirstLayer>
		DispatchRcRunRawScopedHandler<
			R,
			S,
			A,
			RefBracketBrand<RcBrand, NodeBrand<R, S>, Resource, Body>,
			FirstLayer,
		> for RefBracketHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		A: Clone + 'static,
		Resource: Clone + 'static,
		Body: Clone + 'static,
		FirstLayer: 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
	{
		#[document_signature]
		#[document_parameters(
			"The raw RefBracket layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::rc_run::RcRun,
		/// };
		///
		/// let program: RcRun<CNilBrand, CNilBrand, i32> = RcRun::pure(42);
		/// assert_eq!(program.extract(), 42);
		/// ```
		fn dispatch_rc_run_raw_scoped_head(
			&self,
			layer: RefBracket<'static, RcBrand, NodeBrand<R, S>, Resource, Body>,
			continuations: RcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, RcRun<R, S, A>>,
		) -> RcRun<R, S, A> {
			match layer {
				RefBracket::Bracket {
					acquire,
					body,
					release,
				} => {
					let bracket =
						RcRun::<R, S, Resource>::from_rc_free(acquire(())).bind(move |resource| {
							let resource = Rc::new(resource);
							let release_resource = Rc::clone(&resource);
							let body = Rc::clone(&body);
							let release = Rc::clone(&release);
							RcRun::<R, S, Body>::from_rc_free(body(resource)).bind(
								move |body_result| {
									RcRun::<R, S, ()>::from_rc_free(release(Rc::clone(
										&release_resource,
									)))
									.map(move |()| body_result.clone())
								},
							)
						});
					RcRun::from_rc_free(RcFree::continue_from_erased(
						bracket.into_rc_free().cast_erased(),
						continuations,
					))
				}
			}
		}
	}

	/// Raw scoped dispatch implementation for the Arc-backed RefBracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type after the suspended continuation queue resumes.",
		"The resource type produced by acquire.",
		"The body result type produced before the suspended continuation queue resumes.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, A, Resource, Body, FirstLayer>
		DispatchArcRunRawScopedHandler<
			R,
			S,
			A,
			SendRefBracketBrand<ArcBrand, NodeBrand<R, S>, Resource, Body>,
			FirstLayer,
		> for RefBracketHandler
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		A: Clone + Send + Sync + 'static,
		Resource: Clone + Send + Sync + 'static,
		Body: Clone + Send + Sync + 'static,
		FirstLayer: 'static,
		NodeBrand<R, S>: WrapDrop
			+ Kind_cdc7cd43dac7585f<
				Of<'static, ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>>: Send + Sync,
			> + SendFunctor
			+ 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		#[document_parameters(
			"The raw RefBracket layer to interpret.",
			"The continuation stack captured before the scoped operation.",
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use fp_library::{
		/// 	brands::*,
		/// 	types::effects::arc_run::ArcRun,
		/// };
		///
		/// let program: ArcRun<CNilBrand, CNilBrand, i32> = ArcRun::pure(42);
		/// assert_eq!(program.extract(), 42);
		/// ```
		fn dispatch_arc_run_raw_scoped_head(
			&self,
			layer: SendRefBracket<'static, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			continuations: ArcRunContinuations<R, S>,
			_fo_handlers: &impl DispatchHandlers<'static, FirstLayer, ArcRun<R, S, A>>,
		) -> ArcRun<R, S, A> {
			match layer {
				SendRefBracket::Bracket {
					acquire,
					body,
					release,
				} => {
					let bracket = ArcRun::<R, S, Resource>::from_arc_free(acquire(())).bind(
						move |resource| {
							let resource = Arc::new(resource);
							let release_resource = Arc::clone(&resource);
							let body = Arc::clone(&body);
							let release = Arc::clone(&release);
							ArcRun::<R, S, Body>::from_arc_free(body(resource)).bind(
								move |body_result| {
									ArcRun::<R, S, ()>::from_arc_free(release(Arc::clone(
										&release_resource,
									)))
									.map(move |()| body_result.clone())
								},
							)
						},
					);
					ArcRun::from_arc_free(ArcFree::continue_from_erased(
						bracket.into_arc_free().cast_erased(),
						continuations,
					))
				}
			}
		}
	}

	/// Dispatch implementation for the Rc-backed RefBracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, Resource, Body>
		DispatchScopedHandler<
			'static,
			RefBracket<'static, RcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, Body>>),
			RcRun<R, S, Body>,
		> for RefBracketHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'static,
		Body: Clone + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			RcFree<NodeBrand<R, S>, RcTypeErasedValue>,
		>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
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
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: RefBracket<'static, RcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, RcRun<R, S, Body>>),
				RcRun<R, S, Body>,
			>,
		) -> RcRun<R, S, Body> {
			match layer {
				RefBracket::Bracket {
					acquire,
					body,
					release,
				} => RcRun::<R, S, Resource>::from_rc_free(acquire(())).bind(move |resource| {
					let resource = Rc::new(resource);
					let release_resource = Rc::clone(&resource);
					let body = Rc::clone(&body);
					let release = Rc::clone(&release);
					RcRun::<R, S, Body>::from_rc_free(body(resource)).bind(move |body_result| {
						RcRun::<R, S, ()>::from_rc_free(release(Rc::clone(&release_resource)))
							.map(move |()| body_result.clone())
					})
				}),
			}
		}
	}

	/// Dispatch implementation for the Arc-backed RefBracket dispatcher.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<R, S, Resource, Body>
		DispatchScopedHandler<
			'static,
			SendRefBracket<'static, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, Body>>),
			ArcRun<R, S, Body>,
		> for RefBracketHandler
	where
		R: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		S: Kind_cdc7cd43dac7585f + WrapDrop + SendFunctor + 'static,
		Resource: Clone + Send + Sync + 'static,
		Body: Clone + Send + Sync + 'static,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<
			'static,
			ArcFree<NodeBrand<R, S>, ArcTypeErasedValue>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
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
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendRefBracket<'static, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'static,
				Apply!(<R as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'static, ArcRun<R, S, Body>>),
				ArcRun<R, S, Body>,
			>,
		) -> ArcRun<R, S, Body> {
			match layer {
				SendRefBracket::Bracket {
					acquire,
					body,
					release,
				} => ArcRun::<R, S, Resource>::from_arc_free(acquire(())).bind(move |resource| {
					let resource = Arc::new(resource);
					let release_resource = Arc::clone(&resource);
					let body = Arc::clone(&body);
					let release = Arc::clone(&release);
					ArcRun::<R, S, Body>::from_arc_free(body(resource)).bind(move |body_result| {
						ArcRun::<R, S, ()>::from_arc_free(release(Arc::clone(&release_resource)))
							.map(move |()| body_result.clone())
					})
				}),
			}
		}
	}

	/// Dispatch implementation for the Rc explicit RefBracket dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, Resource, Body>
		DispatchScopedHandler<
			'a,
			RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				RcRunExplicit<'a, R, S, Body>,
			>),
			RcRunExplicit<'a, R, S, Body>,
		> for RefBracketHandler
	where
		R: WrapDrop + Functor + 'static,
		S: WrapDrop + Functor + 'static,
		Resource: Clone + 'a,
		Body: Clone + 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, Body>,
		>): Clone,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			RcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
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
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: RefBracketExplicit<'a, RcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						RcRunExplicit<'a, R, S, Body>,
					>
				),
				RcRunExplicit<'a, R, S, Body>,
			>,
		) -> RcRunExplicit<'a, R, S, Body> {
			match layer {
				RefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => RcRunExplicit::<R, S, Resource>::from_rc_free_explicit(acquire(())).bind(
					move |resource| {
						let resource = Rc::new(resource);
						let release_resource = Rc::clone(&resource);
						let body = Rc::clone(&body);
						let release = Rc::clone(&release);
						RcRunExplicit::<R, S, Body>::from_rc_free_explicit(body(resource)).bind(
							move |body_result| {
								RcRunExplicit::<R, S, ()>::from_rc_free_explicit(release(
									Rc::clone(&release_resource),
								))
								.map(move |()| body_result.clone())
							},
						)
					},
				),
			}
		}
	}

	/// Dispatch implementation for the Arc explicit RefBracket dispatcher.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The dispatcher receiver.")]
	impl<'a, R, S, Resource, Body>
		DispatchScopedHandler<
			'a,
			SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			Apply!(<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
				'a,
				ArcRunExplicit<'a, R, S, Body>,
			>),
			ArcRunExplicit<'a, R, S, Body>,
		> for RefBracketHandler
	where
		R: WrapDrop + SendFunctor + 'static,
		S: WrapDrop + SendFunctor + 'static,
		Resource: Clone + Send + Sync + 'a,
		Body: Clone + Send + Sync + 'a,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Resource>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, Body>,
		>): Clone + Send + Sync,
		Apply!(<NodeBrand<R, S> as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
			'a,
			ArcFreeExplicit<'a, NodeBrand<R, S>, ()>,
		>): Clone + Send + Sync,
	{
		#[document_signature]
		///
		#[document_parameters(
			"The scoped RefBracket layer to interpret.",
			"The first-order handler list available to the scoped handler."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			Functor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			rc_run::RcRun,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::{
		/// 		cell::Cell,
		/// 		rc::Rc,
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// type FirstRow = CNilBrand;
		/// type UnderlyingRow = CoproductBrand<
		/// 	RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>,
		/// 	CNilBrand,
		/// >;
		/// type Prog = RcRun<FirstRow, ScopedRow, i32>;
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
		/// let released = Rc::new(Cell::new(false));
		/// let released_in_cleanup = Rc::clone(&released);
		/// let program: Prog = RcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	RcRun::pure(7),
		/// 	|resource: Rc<i32>| RcRun::pure(*resource + 35),
		/// 	move |resource: Rc<i32>| {
		/// 		released_in_cleanup.set(*resource == 7);
		/// 		RcRun::pure(())
		/// 	},
		/// );
		/// let result = program.interpret(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.get());
		/// ```
		fn dispatch_scoped_head(
			&self,
			layer: SendRefBracketExplicit<'a, ArcBrand, NodeBrand<R, S>, Resource, Body>,
			_fo_handlers: &impl DispatchHandlers<
				'a,
				Apply!(
					<R as Kind!( type Of<'b, T: 'b>: 'b; )>::Of<
						'a,
						ArcRunExplicit<'a, R, S, Body>,
					>
				),
				ArcRunExplicit<'a, R, S, Body>,
			>,
		) -> ArcRunExplicit<'a, R, S, Body> {
			match layer {
				SendRefBracketExplicit::Bracket {
					acquire,
					body,
					release,
				} => ArcRunExplicit::<R, S, Resource>::from_arc_free_explicit(acquire(())).bind(
					move |resource| {
						let resource = Arc::new(resource);
						let release_resource = Arc::clone(&resource);
						let body = Arc::clone(&body);
						let release = Arc::clone(&release);
						ArcRunExplicit::<R, S, Body>::from_arc_free_explicit(body(resource)).bind(
							move |body_result| {
								ArcRunExplicit::<R, S, ()>::from_arc_free_explicit(release(
									Arc::clone(&release_resource),
								))
								.map(move |()| body_result.clone())
							},
						)
					},
				),
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
	#[document_parameters("The RefBracket dispatcher receiver.")]
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
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `RcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
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
	#[document_parameters("The RefBracket dispatcher receiver.")]
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
			"The first-order handler list retained by the dispatcher contract."
		)]
		#[document_returns(
			"The final `ArcRunExplicit` program produced by the boundary dispatcher."
		)]
		#[document_examples]
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

pub use inner::*;
