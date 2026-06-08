#[allow(
	unused_imports,
	reason = "Each scoped-handler child module consumes a different subset of the shared parent prelude."
)]
use super::prelude::*;

mod carrier;

#[fp_macros::document_module]
mod inner {
	use super::*;

	/// Handler for the standard `RefBracket` scoped effect.
	///
	/// The handler runs acquire, stores the resource in a refcounted
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
	/// let handler = ref_bracket_handler();
	///
	/// let result = program.handle(
	/// 	handlers! {},
	/// 	scoped_handlers! {
	/// 		RefBracketBrand<RcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>: handler,
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

	/// Raw scoped dispatch implementation for the Rc-backed RefBracket handler.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type after the suspended continuation queue resumes.",
		"The resource type produced by acquire.",
		"The body result type produced before the suspended continuation queue resumes.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The handler receiver.")]
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
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw scoped-handler protocol hook receives type-erased RcRun action carriers and continuation stacks constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the supported public handler path."
		)]
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
		/// let result = program.handle(
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

	/// Raw scoped dispatch implementation for the Arc-backed RefBracket handler.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The final result type after the suspended continuation queue resumes.",
		"The resource type produced by acquire.",
		"The body result type produced before the suspended continuation queue resumes.",
		"The first first-order handler layer type."
	)]
	#[document_parameters("The handler receiver.")]
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
			"The first-order handler list retained by the handler contract."
		)]
		#[document_returns("The program produced after interpreting the scoped operation.")]
		#[document_examples(
			skip_call_check,
			reason = "This raw scoped-handler protocol hook receives type-erased ArcRun action carriers and continuation stacks constructed by the interpreter; external examples cannot construct those protocol inputs directly, so the example documents the supported public handler path."
		)]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			SendFunctor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			arc_run::ArcRun,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			ref_bracket::SendRefBracket,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		atomic::{
		/// 			AtomicBool,
		/// 			AtomicI32,
		/// 			Ordering,
		/// 		},
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Coproduct<SendRefBracket<'a, ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>, CNil>;
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		None
		/// 	}
		/// }
		///
		/// impl SendFunctor for ScopedRow {
		/// 	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		/// 		_f: impl Fn(A) -> B + Send + Sync + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		match fa {
		/// 			Coproduct::Inl(layer) => Coproduct::Inl(layer),
		/// 			Coproduct::Inr(remainder) => match remainder {},
		/// 		}
		/// 	}
		/// }
		///
		/// type FirstRow = CNilBrand;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let observed = Arc::new(AtomicI32::new(0));
		/// let released = Arc::new(AtomicBool::new(false));
		/// let observed_in_body = Arc::clone(&observed);
		/// let released_in_cleanup = Arc::clone(&released);
		/// let program: Prog = ArcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	ArcRun::pure(7),
		/// 	move |resource: Arc<i32>| {
		/// 		observed_in_body.store(*resource, Ordering::SeqCst);
		/// 		ArcRun::pure(*resource + 35)
		/// 	},
		/// 	move |resource: Arc<i32>| {
		/// 		released_in_cleanup.store(*resource == 7, Ordering::SeqCst);
		/// 		ArcRun::pure(())
		/// 	},
		/// );
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendRefBracketBrand<ArcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>:
		/// 			ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert_eq!(observed.load(Ordering::SeqCst), 7);
		/// assert!(released.load(Ordering::SeqCst));
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

	/// Dispatch implementation for the Rc-backed RefBracket handler.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The handler receiver.")]
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
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public API is installing the handler with scoped_handlers! and running handle, so the example documents the supported handler path instead of direct protocol invocation."
		)]
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
		/// let result = program.handle(
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

	/// Dispatch implementation for the Arc-backed RefBracket handler.
	#[document_type_parameters(
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The handler receiver.")]
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
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public API is installing the handler with scoped_handlers! and running handle, so the example documents the supported handler path instead of direct protocol invocation."
		)]
		///
		/// ```
		/// use {
		/// 	fp_library::{
		/// 		Apply,
		/// 		brands::*,
		/// 		classes::{
		/// 			SendFunctor,
		/// 			WrapDrop,
		/// 		},
		/// 		handlers,
		/// 		impl_kind,
		/// 		kinds::*,
		/// 		scoped_handlers,
		/// 		types::effects::{
		/// 			arc_run::ArcRun,
		/// 			coproduct::{
		/// 				CNil,
		/// 				Coproduct,
		/// 			},
		/// 			ref_bracket::SendRefBracket,
		/// 			standard_scoped_handlers::ref_bracket_handler,
		/// 		},
		/// 	},
		/// 	std::sync::{
		/// 		Arc,
		/// 		atomic::{
		/// 			AtomicBool,
		/// 			Ordering,
		/// 		},
		/// 	},
		/// };
		///
		/// #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
		/// struct ScopedRow;
		///
		/// impl_kind! {
		/// 	impl for ScopedRow {
		/// 		type Of<'a, A: 'a>: 'a =
		/// 			Coproduct<SendRefBracket<'a, ArcBrand, NodeBrand<CNilBrand, ScopedRow>, i32, i32>, CNil>;
		/// 	}
		/// }
		///
		/// impl WrapDrop for ScopedRow {
		/// 	fn drop<'a, X: 'a>(
		/// 		_fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, X>)
		/// 	) -> Option<X> {
		/// 		None
		/// 	}
		/// }
		///
		/// impl SendFunctor for ScopedRow {
		/// 	fn send_map<'a, A: Send + Sync + 'a, B: Send + Sync + 'a>(
		/// 		_f: impl Fn(A) -> B + Send + Sync + 'a,
		/// 		fa: Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, A>),
		/// 	) -> Apply!(<Self as Kind!( type Of<'a, T: 'a>: 'a; )>::Of<'a, B>) {
		/// 		match fa {
		/// 			Coproduct::Inl(layer) => Coproduct::Inl(layer),
		/// 			Coproduct::Inr(remainder) => match remainder {},
		/// 		}
		/// 	}
		/// }
		///
		/// type FirstRow = CNilBrand;
		/// type Prog = ArcRun<FirstRow, ScopedRow, i32>;
		///
		/// let released = Arc::new(AtomicBool::new(false));
		/// let released_in_cleanup = Arc::clone(&released);
		/// let program: Prog = ArcRun::<FirstRow, ScopedRow, i32>::ref_bracket::<i32, _>(
		/// 	ArcRun::pure(7),
		/// 	|resource: Arc<i32>| ArcRun::pure(*resource + 35),
		/// 	move |resource: Arc<i32>| {
		/// 		released_in_cleanup.store(*resource == 7, Ordering::SeqCst);
		/// 		ArcRun::pure(())
		/// 	},
		/// );
		/// let result = program.handle(
		/// 	handlers! {},
		/// 	scoped_handlers! {
		/// 		SendRefBracketBrand<ArcBrand, NodeBrand<FirstRow, ScopedRow>, i32, i32>:
		/// 			ref_bracket_handler(),
		/// 	},
		/// );
		///
		/// assert_eq!(result, 42);
		/// assert!(released.load(Ordering::SeqCst));
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

	/// Dispatch implementation for the Rc explicit RefBracket handler.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The handler receiver.")]
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
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public explicit-wrapper API dispatches the RefBracket boundary through ref_bracket_handler before continuing interpretation, so the example documents that supported handler path instead of direct protocol invocation."
		)]
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

	/// Dispatch implementation for the Arc explicit RefBracket handler.
	#[document_type_parameters(
		"The lifetime of values carried by the explicit wrapper.",
		"The first-order row brand.",
		"The scoped row brand.",
		"The resource type produced by acquire.",
		"The body result type returned after release."
	)]
	#[document_parameters("The handler receiver.")]
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
		#[document_examples(
			skip_call_check,
			reason = "This trait impl method is a scoped-handler protocol hook; the public explicit-wrapper API dispatches the RefBracket boundary through ref_bracket_handler before continuing interpretation, so the example documents that supported handler path instead of direct protocol invocation."
		)]
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
}

pub use inner::*;
